#requires -Version 7.0
<#
.SYNOPSIS
Installs, updates, or removes the separately distributed riAuth device host.

.DESCRIPTION
Run from an elevated 64-bit PowerShell 7 process on Windows. Sign this script
and RiAuth.DeviceHost.exe with a trusted Authenticode code-signing certificate
before use. The first install requires the operator to supply that certificate's
SHA-1 thumbprint. Subsequent updates and uninstall require the exact same signer.
An optional x64 credential provider DLL may be installed and registered with
the host. The provider is never installed unless -CredentialProviderPath is
supplied. The host and provider must have the same pinned signer.

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Install -PackagePath .\RiAuth.DeviceHost.exe -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Install -PackagePath .\RiAuth.DeviceHost.exe -CredentialProviderPath .\RiAuth.CredentialProvider.dll -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Update -PackagePath .\RiAuth.DeviceHost.exe

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Uninstall
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Install', 'Update', 'Uninstall')]
    [string] $Action,

    [string] $PackagePath,

    [string] $CredentialProviderPath,

    [string] $SignerThumbprint
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Normalize-Thumbprint {
    param([Parameter(Mandatory = $true)][string] $Value)
    $normalized = $Value.Replace(' ', '').Replace(':', '').ToUpperInvariant()
    if ($normalized -cnotmatch '^[0-9A-F]{40}$') {
        throw 'SignerThumbprint must be a 40-digit certificate thumbprint.'
    }
    return $normalized
}

function Assert-SignedBy {
    param(
        [Parameter(Mandatory = $true)][string] $LiteralPath,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )

    if (-not (Test-Path -LiteralPath $LiteralPath -PathType Leaf)) {
        throw "Signed file is missing: $LiteralPath"
    }
    $signature = Get-AuthenticodeSignature -LiteralPath $LiteralPath
    if ($signature.Status -ne 'Valid' -or $null -eq $signature.SignerCertificate) {
        throw "Authenticode signature is not valid: $LiteralPath ($($signature.Status))"
    }
    $actual = Normalize-Thumbprint $signature.SignerCertificate.Thumbprint
    if ($actual -cne $ExpectedThumbprint) {
        throw "Authenticode signer does not match the pinned certificate: $LiteralPath"
    }
}

function Get-PinnedSigner {
    param([Parameter(Mandatory = $true)][string] $RegistryKey)
    if (-not (Test-Path -LiteralPath $RegistryKey)) {
        throw 'No installed device host signer pin exists. Install before updating or uninstalling.'
    }
    $value = (Get-ItemProperty -LiteralPath $RegistryKey -Name 'SignerThumbprint').SignerThumbprint
    return Normalize-Thumbprint $value
}

function Assert-InstalledPayload {
    param(
        [Parameter(Mandatory = $true)][string] $Executable,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )
    Assert-SignedBy -LiteralPath $Executable -ExpectedThumbprint $ExpectedThumbprint
    $item = Get-Item -LiteralPath $Executable
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'The installed executable must not be a reparse point.'
    }
}

function Assert-X64NativeDll {
    param([Parameter(Mandatory = $true)][string] $LiteralPath)
    # Credential providers run in LogonUI. An ARM64 or x86 DLL must not be
    # registered into a native x64 LogonUI process.
    if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne
        [Runtime.InteropServices.Architecture]::X64) {
        throw 'The credential provider package requires x64 Windows.'
    }
    $stream = [IO.File]::Open($LiteralPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    $reader = [IO.BinaryReader]::new($stream)
    try {
        if ($stream.Length -lt 0x100 -or $reader.ReadUInt16() -ne 0x5A4D) {
            throw "Credential provider is not a PE DLL: $LiteralPath"
        }
        $stream.Position = 0x3C
        $offset = $reader.ReadInt32()
        if ($offset -lt 0x40 -or $offset -gt ($stream.Length - 0x110)) {
            throw "Credential provider has an invalid PE header: $LiteralPath"
        }
        $stream.Position = $offset
        if ($reader.ReadUInt32() -ne 0x00004550 -or $reader.ReadUInt16() -ne 0x8664) {
            throw "Credential provider must be an x64 PE DLL: $LiteralPath"
        }
        $stream.Position = $offset + 22
        if (($reader.ReadUInt16() -band 0x2000) -eq 0) {
            throw "Credential provider is not a PE DLL: $LiteralPath"
        }
        # PE32+ data-directory 14 is the CLR runtime header. The provider
        # must be a native COM in-process server, not an IL-only assembly.
        $stream.Position = $offset + 24
        if ($reader.ReadUInt16() -ne 0x20B) {
            throw "Credential provider must use the PE32+ format: $LiteralPath"
        }
        $stream.Position = $offset + 24 + 112
        if ($reader.ReadUInt32() -eq 0) {
            throw "Credential provider DLL must export its COM class factory: $LiteralPath"
        }
        $stream.Position = $offset + 24 + 112 + (14 * 8)
        if ($reader.ReadUInt32() -ne 0) {
            throw "Credential provider must be a native COM DLL: $LiteralPath"
        }
    } finally {
        $reader.Dispose()
        $stream.Dispose()
    }
}

function Assert-ProviderRegistration {
    param(
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey,
        [Parameter(Mandatory = $true)][string] $DllPath
    )
    if (-not (Test-Path -LiteralPath $ClassKey) -or
        -not (Test-Path -LiteralPath $InprocKey) -or
        -not (Test-Path -LiteralPath $ProviderKey)) {
        throw 'Credential provider registry entries are incomplete.'
    }
    $class = Get-Item -LiteralPath $ClassKey
    $inproc = Get-Item -LiteralPath $InprocKey
    $provider = Get-Item -LiteralPath $ProviderKey
    if ($class.GetValue('') -cne 'RiAuth Credential Provider' -or
        $inproc.GetValue('') -cne $DllPath -or
        $inproc.GetValue('ThreadingModel') -cne 'Apartment' -or
        $provider.GetValue('') -cne 'RiAuth') {
        throw 'Credential provider registry entries do not match this installation.'
    }
}

function New-ProviderRegistration {
    param(
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey,
        [Parameter(Mandatory = $true)][string] $DllPath
    )
    if ((Test-Path -LiteralPath $ClassKey) -or (Test-Path -LiteralPath $ProviderKey)) {
        throw 'The credential provider CLSID is already registered.'
    }
    $classCreated = $false
    $providerCreated = $false
    try {
        New-Item -Path $ClassKey -Value 'RiAuth Credential Provider' | Out-Null
        $classCreated = $true
        New-Item -Path $InprocKey -Value $DllPath | Out-Null
        New-ItemProperty -LiteralPath $InprocKey -Name 'ThreadingModel' -PropertyType String -Value 'Apartment' | Out-Null
        New-Item -Path $ProviderKey -Value 'RiAuth' | Out-Null
        $providerCreated = $true
        Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
    } catch {
        $failure = $_
        if ($providerCreated) {
            try { Remove-Item -LiteralPath $ProviderKey -Recurse -Force } catch {
                Write-Warning "Could not remove incomplete credential provider registration: $_"
            }
        }
        if ($classCreated) {
            try { Remove-Item -LiteralPath $ClassKey -Recurse -Force } catch {
                Write-Warning "Could not remove incomplete COM registration: $_"
            }
        }
        throw $failure
    }
}

function Restore-ProviderRegistration {
    param(
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey,
        [Parameter(Mandatory = $true)][string] $DllPath
    )
    if (-not (Test-Path -LiteralPath $ClassKey)) {
        New-Item -Path $ClassKey -Value 'RiAuth Credential Provider' | Out-Null
    }
    if (-not (Test-Path -LiteralPath $InprocKey)) {
        New-Item -Path $InprocKey -Value $DllPath | Out-Null
    }
    if (-not (Test-Path -LiteralPath $ProviderKey)) {
        New-Item -Path $ProviderKey -Value 'RiAuth' | Out-Null
    }
    # Existing entries are never overwritten; a foreign registration fails
    # closed and needs an operator to inspect the recorded rollback paths.
    $inproc = Get-Item -LiteralPath $InprocKey
    if ($null -eq $inproc.GetValue('ThreadingModel')) {
        New-ItemProperty -LiteralPath $InprocKey -Name 'ThreadingModel' -PropertyType String -Value 'Apartment' | Out-Null
    }
    Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
}

function Assert-InstalledProvider {
    param(
        [Parameter(Mandatory = $true)][string] $DllPath,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey
    )
    Assert-SignedBy -LiteralPath $DllPath -ExpectedThumbprint $ExpectedThumbprint
    $item = Get-Item -LiteralPath $DllPath
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'The installed credential provider must not be a reparse point.'
    }
    Assert-X64NativeDll -LiteralPath $DllPath
    Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
}

function Get-InstalledProviderState {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)][string] $ExpectedClsid,
        [Parameter(Mandatory = $true)][string] $DllPath,
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey
    )
    $value = (Get-Item -LiteralPath $RegistryKey).GetValue('CredentialProviderClsid')
    if ($null -eq $value) {
        if ((Test-Path -LiteralPath $DllPath) -or
            (Test-Path -LiteralPath $ClassKey) -or
            (Test-Path -LiteralPath $ProviderKey)) {
            throw 'Credential provider files or registration exist without the installer marker.'
        }
        return $false
    }
    if ($value -cne $ExpectedClsid) {
        throw 'Installed credential provider CLSID does not match this installer.'
    }
    return $true
}

if (-not $IsWindows) {
    throw 'The device host installer runs only on Windows.'
}
if (-not [Environment]::Is64BitProcess) {
    throw 'Run the device host installer in 64-bit PowerShell.'
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run the device host installer as an administrator.'
}
$programFilesDirectory = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles)
$programDataDirectory = [Environment]::GetFolderPath([Environment+SpecialFolder]::CommonApplicationData)
if ([string]::IsNullOrWhiteSpace($programFilesDirectory) -or
    [string]::IsNullOrWhiteSpace($programDataDirectory)) {
    throw 'Program Files and ProgramData must be available.'
}

$installParent = Join-Path $programFilesDirectory 'RiAuth'
$installDirectory = Join-Path $installParent 'DeviceHost'
$installedExecutable = Join-Path $installDirectory 'RiAuth.DeviceHost.exe'
$installedProviderDll = Join-Path $installDirectory 'RiAuth.CredentialProvider.dll'
$stateDirectory = Join-Path $programDataDirectory 'RiAuth\DeviceHost'
$registryParent = 'HKLM:\SOFTWARE\RiAuth'
$registryKey = 'HKLM:\SOFTWARE\RiAuth\DeviceHost'
$providerClsid = '{64A6A7BF-BA56-4463-9692-83A4EA3DBC6C}'
$providerClassKey = "HKLM:\SOFTWARE\Classes\CLSID\$providerClsid"
$providerInprocKey = "$providerClassKey\InprocServer32"
$providerRegistryKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\Credential Providers\$providerClsid"

if ($Action -eq 'Install') {
    if ([string]::IsNullOrWhiteSpace($PackagePath) -or
        [string]::IsNullOrWhiteSpace($SignerThumbprint)) {
        throw 'Install requires -PackagePath and -SignerThumbprint.'
    }
} elseif ($Action -eq 'Update') {
    if ([string]::IsNullOrWhiteSpace($PackagePath) -or $PSBoundParameters.ContainsKey('SignerThumbprint')) {
        throw 'Update requires -PackagePath and uses the installed signer pin.'
    }
} elseif ($PSBoundParameters.ContainsKey('PackagePath') -or
          $PSBoundParameters.ContainsKey('CredentialProviderPath') -or
          $PSBoundParameters.ContainsKey('SignerThumbprint')) {
    throw 'Uninstall accepts no package or signer parameters.'
}

$package = $null
$credentialPackage = $null
if ($Action -ne 'Uninstall') {
    $package = (Resolve-Path -LiteralPath $PackagePath).ProviderPath
    if ([IO.Path]::GetExtension($package) -ine '.exe') {
        throw 'PackagePath must refer to a signed .exe file.'
    }
    if ($PSBoundParameters.ContainsKey('CredentialProviderPath')) {
        if ([string]::IsNullOrWhiteSpace($CredentialProviderPath)) {
            throw 'CredentialProviderPath must name a signed x64 DLL.'
        }
        $credentialPackage = (Resolve-Path -LiteralPath $CredentialProviderPath).ProviderPath
        if ([IO.Path]::GetExtension($credentialPackage) -ine '.dll') {
            throw 'CredentialProviderPath must refer to a signed .dll file.'
        }
        Assert-X64NativeDll -LiteralPath $credentialPackage
    }
}

# Only one elevated installer may swap the machine-wide executable or pin.
$mutex = [Threading.Mutex]::new($false, 'Global\RiAuth.DeviceHost.Install')
$hasMutex = $false
try {
    $hasMutex = $mutex.WaitOne(30000)
    if (-not $hasMutex) {
        throw 'Another device host installation is in progress.'
    }

    if ($Action -eq 'Install') {
        if ((Test-Path -LiteralPath $registryKey) -or
            (Test-Path -LiteralPath $installDirectory)) {
            throw 'A device host installation or signer pin already exists.'
        }
        $pin = Normalize-Thumbprint $SignerThumbprint
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-SignedBy -LiteralPath $package -ExpectedThumbprint $pin
        if ($null -ne $credentialPackage) {
            if ((Test-Path -LiteralPath $providerClassKey) -or
                (Test-Path -LiteralPath $providerRegistryKey)) {
                throw 'The credential provider CLSID is already registered.'
            }
            Assert-SignedBy -LiteralPath $credentialPackage -ExpectedThumbprint $pin
        }

        New-Item -ItemType Directory -Path $installParent -Force | Out-Null
        $stageDirectory = Join-Path $installParent ("DeviceHost.stage.$([guid]::NewGuid().ToString('N'))")
        $installed = $false
        $providerRegistrationStarted = $false
        try {
            New-Item -ItemType Directory -Path $stageDirectory | Out-Null
            $stageExecutable = Join-Path $stageDirectory 'RiAuth.DeviceHost.exe'
            Copy-Item -LiteralPath $package -Destination $stageExecutable
            Assert-SignedBy -LiteralPath $stageExecutable -ExpectedThumbprint $pin
            if ($null -ne $credentialPackage) {
                $stageProviderDll = Join-Path $stageDirectory 'RiAuth.CredentialProvider.dll'
                Copy-Item -LiteralPath $credentialPackage -Destination $stageProviderDll
                Assert-SignedBy -LiteralPath $stageProviderDll -ExpectedThumbprint $pin
                Assert-X64NativeDll -LiteralPath $stageProviderDll
            }
            Move-Item -LiteralPath $stageDirectory -Destination $installDirectory
            $installed = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin

            New-Item -Path $registryParent -Force | Out-Null
            New-Item -Path $registryKey | Out-Null
            New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' `
                -PropertyType String -Value $pin | Out-Null
            if ($null -ne $credentialPackage) {
                New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid | Out-Null
                $providerRegistrationStarted = $true
                New-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
                Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
            }
        } catch {
            $failure = $_
            if ($providerRegistrationStarted) {
                try {
                    if (Test-Path -LiteralPath $providerRegistryKey) {
                        Remove-Item -LiteralPath $providerRegistryKey -Recurse -Force
                    }
                    if (Test-Path -LiteralPath $providerClassKey) {
                        Remove-Item -LiteralPath $providerClassKey -Recurse -Force
                    }
                } catch {
                    throw "Install failed ($failure); credential provider rollback also failed ($_). Inspect $installDirectory and $registryKey before retrying."
                }
            }
            if (Test-Path -LiteralPath $registryKey) {
                try { Remove-Item -LiteralPath $registryKey -Recurse -Force } catch {
                    Write-Warning "Could not remove incomplete signer pin: $_"
                }
            }
            if ($installed -and (Test-Path -LiteralPath $installDirectory)) {
                try { Remove-Item -LiteralPath $installDirectory -Recurse -Force } catch {
                    Write-Warning "Could not remove incomplete installation: $_"
                }
            }
            throw $failure
        } finally {
            if (Test-Path -LiteralPath $stageDirectory) {
                Remove-Item -LiteralPath $stageDirectory -Recurse -Force
            }
        }
        if ($null -ne $credentialPackage) {
            Write-Output "Installed $installedExecutable and credential provider $installedProviderDll with signer $pin"
        } else {
            Write-Output "Installed $installedExecutable with signer $pin"
        }
    } elseif ($Action -eq 'Update') {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
        Assert-SignedBy -LiteralPath $package -ExpectedThumbprint $pin
        $hasProvider = Get-InstalledProviderState -RegistryKey $registryKey -ExpectedClsid $providerClsid -DllPath $installedProviderDll -ClassKey $providerClassKey -ProviderKey $providerRegistryKey
        if ($hasProvider) {
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        }
        if ($null -ne $credentialPackage) {
            Assert-SignedBy -LiteralPath $credentialPackage -ExpectedThumbprint $pin
        }

        $nonce = [guid]::NewGuid().ToString('N')
        $stagedExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.stage.$nonce.exe"
        $backupExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.backup.$nonce.exe"
        $stagedProviderDll = Join-Path $installDirectory "RiAuth.CredentialProvider.stage.$nonce.dll"
        $backupProviderDll = Join-Path $installDirectory "RiAuth.CredentialProvider.backup.$nonce.dll"
        $oldHostMoved = $false
        $newHostMoved = $false
        $oldProviderMoved = $false
        $newProviderMoved = $false
        $providerRegistrationStarted = $false
        $providerMarkerAdded = $false
        try {
            Copy-Item -LiteralPath $package -Destination $stagedExecutable
            Assert-SignedBy -LiteralPath $stagedExecutable -ExpectedThumbprint $pin
            if ($null -ne $credentialPackage) {
                Copy-Item -LiteralPath $credentialPackage -Destination $stagedProviderDll
                Assert-SignedBy -LiteralPath $stagedProviderDll -ExpectedThumbprint $pin
                Assert-X64NativeDll -LiteralPath $stagedProviderDll
            }
            Move-Item -LiteralPath $installedExecutable -Destination $backupExecutable
            $oldHostMoved = $true
            Move-Item -LiteralPath $stagedExecutable -Destination $installedExecutable
            $newHostMoved = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
            if ($null -ne $credentialPackage) {
                if ($hasProvider) {
                    Move-Item -LiteralPath $installedProviderDll -Destination $backupProviderDll
                    $oldProviderMoved = $true
                }
                Move-Item -LiteralPath $stagedProviderDll -Destination $installedProviderDll
                $newProviderMoved = $true
                Assert-SignedBy -LiteralPath $installedProviderDll -ExpectedThumbprint $pin
                Assert-X64NativeDll -LiteralPath $installedProviderDll
                if (-not $hasProvider) {
                    New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid | Out-Null
                    $providerMarkerAdded = $true
                    $providerRegistrationStarted = $true
                    New-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
                }
                Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
            }
        } catch {
            $failure = $_
            try {
                if ($providerRegistrationStarted) {
                    if (Test-Path -LiteralPath $providerRegistryKey) {
                        Remove-Item -LiteralPath $providerRegistryKey -Recurse -Force
                    }
                    if (Test-Path -LiteralPath $providerClassKey) {
                        Remove-Item -LiteralPath $providerClassKey -Recurse -Force
                    }
                }
                if ($providerMarkerAdded) {
                    Remove-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid'
                }
                if ($newProviderMoved) {
                    Remove-Item -LiteralPath $installedProviderDll -Force
                }
                if ($oldProviderMoved) {
                    Move-Item -LiteralPath $backupProviderDll -Destination $installedProviderDll
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
                if ($newHostMoved) {
                    Remove-Item -LiteralPath $installedExecutable -Force
                }
                if ($oldHostMoved) {
                    Move-Item -LiteralPath $backupExecutable -Destination $installedExecutable
                    Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
                }
            } catch {
                throw "Update failed ($failure); rollback also failed ($_). Inspect signed backups at $backupExecutable and $backupProviderDll"
            }
            throw $failure
        } finally {
            if (Test-Path -LiteralPath $stagedExecutable) {
                Remove-Item -LiteralPath $stagedExecutable -Force
            }
            if (Test-Path -LiteralPath $stagedProviderDll) {
                Remove-Item -LiteralPath $stagedProviderDll -Force
            }
        }
        try { Remove-Item -LiteralPath $backupExecutable -Force } catch {
            Write-Warning "Update installed, but the signed backup could not be removed: $backupExecutable ($_)"
        }
        if ($oldProviderMoved) {
            try { Remove-Item -LiteralPath $backupProviderDll -Force } catch {
                Write-Warning "Update installed, but the signed provider backup could not be removed: $backupProviderDll ($_)"
            }
        }
        if ($null -ne $credentialPackage) {
            Write-Output "Updated $installedExecutable and registered credential provider $installedProviderDll with pinned signer $pin"
        } else {
            Write-Output "Updated $installedExecutable with pinned signer $pin"
        }
    } else {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
        $hasProvider = Get-InstalledProviderState -RegistryKey $registryKey -ExpectedClsid $providerClsid -DllPath $installedProviderDll -ClassKey $providerClassKey -ProviderKey $providerRegistryKey
        if ($hasProvider) {
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        }

        # The host stores device credentials under ProgramData. Revocation must
        # happen remotely before an operator removes that state. An empty secured
        # directory can remain; links and any child entry are treated as state.
        if (Test-Path -LiteralPath $stateDirectory) {
            $stateRoot = Get-Item -LiteralPath $stateDirectory -Force
            if (($stateRoot.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Device state path is a reparse point: $stateDirectory"
            }
            if (-not $stateRoot.PSIsContainer) {
                throw "Device state path is not a directory: $stateDirectory"
            }
            $stateEntry = Get-ChildItem -LiteralPath $stateDirectory -Force |
                Select-Object -First 1
            if ($null -ne $stateEntry) {
                throw "Device state remains at $($stateEntry.FullName). Revoke the device and purge state before uninstalling."
            }
        }
        $otherFiles = Get-ChildItem -LiteralPath $installDirectory -Force |
            Where-Object {
                $_.Name -cne 'RiAuth.DeviceHost.exe' -and
                -not ($hasProvider -and $_.Name -ceq 'RiAuth.CredentialProvider.dll')
            } |
            Select-Object -First 1
        if ($null -ne $otherFiles) {
            throw "Unexpected installation file remains at $($otherFiles.FullName). Resolve it before uninstalling."
        }

        # Move the whole verified installation aside before changing registry
        # registration. Keep it intact until registry removal succeeds, so an
        # error can restore the signed host and provider together.
        $quarantineDirectory = Join-Path $installParent ("DeviceHost.uninstall.$([guid]::NewGuid().ToString('N'))")
        Move-Item -LiteralPath $installDirectory -Destination $quarantineDirectory
        try {
            if ($hasProvider) {
                Remove-Item -LiteralPath $providerRegistryKey -Recurse -Force
                Remove-Item -LiteralPath $providerClassKey -Recurse -Force
            }
            Remove-Item -LiteralPath $registryKey -Recurse -Force
        } catch {
            $failure = $_
            try {
                Move-Item -LiteralPath $quarantineDirectory -Destination $installDirectory
                Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
                New-Item -Path $registryParent -Force | Out-Null
                New-Item -Path $registryKey -Force | Out-Null
                New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' -PropertyType String -Value $pin -Force | Out-Null
                if ($hasProvider) {
                    New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid -Force | Out-Null
                    Restore-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
            } catch {
                throw "Uninstall failed ($failure); rollback also failed ($_). Inspect $quarantineDirectory, $registryKey and credential provider registration before retrying."
            }
            throw $failure
        }
        try { Remove-Item -LiteralPath $quarantineDirectory -Recurse -Force } catch {
            Write-Warning "Device host registration was removed, but signed binaries remain at $quarantineDirectory and require cleanup after LogonUI releases them: $_"
        }
        Write-Output 'Uninstalled riAuth device host and its optional credential provider. The empty ProgramData directory, if present, was retained.'
    }
} finally {
    if ($hasMutex) { $mutex.ReleaseMutex() }
    $mutex.Dispose()
}
