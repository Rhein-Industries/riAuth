#requires -Version 7.0
<#
.SYNOPSIS
Installs, updates, or removes the separately distributed riAuth device host.

.DESCRIPTION
Run from an elevated 64-bit PowerShell 7 process on Windows. Sign this script
and RiAuth.DeviceHost.exe with a trusted Authenticode code-signing certificate
before use. The first install requires the operator to supply that certificate's
SHA-1 thumbprint. Subsequent updates and uninstall require the exact same signer.
This script does not register a Windows credential provider or log on to Windows.

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Install -PackagePath .\RiAuth.DeviceHost.exe -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567

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
if ([string]::IsNullOrWhiteSpace($env:ProgramFiles) -or
    [string]::IsNullOrWhiteSpace($env:ProgramData)) {
    throw 'Program Files and ProgramData must be available.'
}

$installParent = Join-Path $env:ProgramFiles 'RiAuth'
$installDirectory = Join-Path $installParent 'DeviceHost'
$installedExecutable = Join-Path $installDirectory 'RiAuth.DeviceHost.exe'
$stateDirectory = Join-Path $env:ProgramData 'RiAuth\DeviceHost'
$registryParent = 'HKLM:\SOFTWARE\RiAuth'
$registryKey = 'HKLM:\SOFTWARE\RiAuth\DeviceHost'

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
          $PSBoundParameters.ContainsKey('SignerThumbprint')) {
    throw 'Uninstall accepts neither -PackagePath nor -SignerThumbprint.'
}

$package = $null
if ($Action -ne 'Uninstall') {
    $package = (Resolve-Path -LiteralPath $PackagePath).ProviderPath
    if ([IO.Path]::GetExtension($package) -ine '.exe') {
        throw 'PackagePath must refer to a signed .exe file.'
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

        New-Item -ItemType Directory -Path $installParent -Force | Out-Null
        $stageDirectory = Join-Path $installParent ("DeviceHost.stage.$([guid]::NewGuid().ToString('N'))")
        $installed = $false
        try {
            New-Item -ItemType Directory -Path $stageDirectory | Out-Null
            $stageExecutable = Join-Path $stageDirectory 'RiAuth.DeviceHost.exe'
            Copy-Item -LiteralPath $package -Destination $stageExecutable
            Assert-SignedBy -LiteralPath $stageExecutable -ExpectedThumbprint $pin
            Move-Item -LiteralPath $stageDirectory -Destination $installDirectory
            $installed = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin

            New-Item -Path $registryParent -Force | Out-Null
            New-Item -Path $registryKey | Out-Null
            New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' `
                -PropertyType String -Value $pin | Out-Null
        } catch {
            $failure = $_
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
        Write-Output "Installed $installedExecutable with signer $pin"
    } elseif ($Action -eq 'Update') {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
        Assert-SignedBy -LiteralPath $package -ExpectedThumbprint $pin

        $nonce = [guid]::NewGuid().ToString('N')
        $stagedExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.stage.$nonce.exe"
        $backupExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.backup.$nonce.exe"
        $oldMoved = $false
        $newMoved = $false
        try {
            Copy-Item -LiteralPath $package -Destination $stagedExecutable
            Assert-SignedBy -LiteralPath $stagedExecutable -ExpectedThumbprint $pin
            Move-Item -LiteralPath $installedExecutable -Destination $backupExecutable
            $oldMoved = $true
            Move-Item -LiteralPath $stagedExecutable -Destination $installedExecutable
            $newMoved = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
        } catch {
            $failure = $_
            try {
                if ($newMoved -and (Test-Path -LiteralPath $installedExecutable)) {
                    Remove-Item -LiteralPath $installedExecutable -Force
                }
                if ($oldMoved) {
                    Move-Item -LiteralPath $backupExecutable -Destination $installedExecutable
                    Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
                }
            } catch {
                throw "Update failed ($failure); rollback also failed ($_). Original executable may remain at $backupExecutable"
            }
            throw $failure
        } finally {
            if (Test-Path -LiteralPath $stagedExecutable) {
                Remove-Item -LiteralPath $stagedExecutable -Force
            }
        }
        try { Remove-Item -LiteralPath $backupExecutable -Force } catch {
            Write-Warning "Update installed, but the signed backup could not be removed: $backupExecutable ($_)"
        }
        Write-Output "Updated $installedExecutable with pinned signer $pin"
    } else {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin

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
            Where-Object { $_.Name -cne 'RiAuth.DeviceHost.exe' } |
            Select-Object -First 1
        if ($null -ne $otherFiles) {
            throw "Unexpected installation file remains at $($otherFiles.FullName). Resolve it before uninstalling."
        }

        # Move the whole verified installation aside before changing the pin.
        # If pin removal or executable removal fails, put both back. Deleting
        # the now-empty quarantine directory is cleanup, not an uninstall gate.
        $quarantineDirectory = Join-Path $installParent ("DeviceHost.uninstall.$([guid]::NewGuid().ToString('N'))")
        $quarantineExecutable = Join-Path $quarantineDirectory 'RiAuth.DeviceHost.exe'
        Move-Item -LiteralPath $installDirectory -Destination $quarantineDirectory
        try {
            Remove-Item -LiteralPath $registryKey -Recurse -Force
            Remove-Item -LiteralPath $quarantineExecutable -Force
        } catch {
            $failure = $_
            try {
                New-Item -Path $registryParent -Force | Out-Null
                New-Item -Path $registryKey -Force | Out-Null
                New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' `
                    -PropertyType String -Value $pin -Force | Out-Null
                if (-not (Test-Path -LiteralPath $quarantineExecutable -PathType Leaf)) {
                    throw 'The original executable is missing from quarantine.'
                }
                Move-Item -LiteralPath $quarantineDirectory -Destination $installDirectory
                Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin
            } catch {
                throw "Uninstall failed ($failure); rollback also failed ($_). Inspect $quarantineDirectory and $registryKey before retrying."
            }
            throw $failure
        }
        try { Remove-Item -LiteralPath $quarantineDirectory -Force } catch {
            Write-Warning "Device host was removed, but empty quarantine directory remains: $quarantineDirectory ($_)"
        }
        Write-Output 'Uninstalled riAuth device host. The empty ProgramData directory, if present, was retained.'
    }
} finally {
    if ($hasMutex) { $mutex.ReleaseMutex() }
    $mutex.Dispose()
}
