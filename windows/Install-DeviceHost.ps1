#requires -Version 7.0
<#
.SYNOPSIS
Verifies, installs, updates, or removes a signed riAuth Windows bundle.

.DESCRIPTION
Install, update and uninstall require elevated 64-bit PowerShell 7 on Windows.
Verify does not require elevation. The four-file bundle contains this signed
installer, a signed manifest, a signed host executable and a signed native x64
credential provider. The first install pins their common certificate thumbprint;
subsequent actions require the pin. Updates verify the current installed hashes
and keep both binaries for rollback. Legacy installations without hash metadata
require an explicit migration switch after verifying pinned signatures.

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Verify -BundlePath . -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Install -BundlePath . -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Update -BundlePath .

.EXAMPLE
.\Install-DeviceHost.ps1 -Action Uninstall
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Verify', 'Install', 'Update', 'Uninstall')]
    [string] $Action,

    [string] $BundlePath,

    [string] $SignerThumbprint,

    [switch] $AllowLegacySignedInstall
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

function Assert-Sha256 {
    param(
        [Parameter(Mandatory = $true)][string] $LiteralPath,
        [Parameter(Mandatory = $true)][string] $ExpectedHash
    )
    $actual = (Get-FileHash -LiteralPath $LiteralPath -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($actual -cne $ExpectedHash) {
        throw "SHA-256 does not match the signed bundle manifest: $LiteralPath"
    }
}

function Assert-NoReparsePoint {
    param([Parameter(Mandatory = $true)][string] $LiteralPath)
    $item = Get-Item -LiteralPath $LiteralPath -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Bundle or installed path must not be a reparse point: $LiteralPath"
    }
}

function Read-SignedBundle {
    param(
        [Parameter(Mandatory = $true)][string] $Directory,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [Parameter(Mandatory = $true)][string] $ExpectedProviderClsid
    )

    $bundle = (Resolve-Path -LiteralPath $Directory).ProviderPath
    if (-not (Test-Path -LiteralPath $bundle -PathType Container)) {
        throw 'BundlePath must be a directory.'
    }
    Assert-NoReparsePoint -LiteralPath $bundle
    $names = @(
        'Install-DeviceHost.ps1',
        'RiAuth.DeviceHost.manifest.ps1',
        'RiAuth.DeviceHost.exe',
        'RiAuth.CredentialProvider.dll'
    )
    $entries = @(Get-ChildItem -LiteralPath $bundle -Force)
    if ($entries.Count -ne $names.Count) {
        throw 'The signed bundle must contain exactly four files.'
    }
    foreach ($entry in $entries) {
        if ($entry.PSIsContainer -or -not ($names -ccontains $entry.Name)) {
            throw "Unexpected bundle entry: $($entry.FullName)"
        }
        Assert-NoReparsePoint -LiteralPath $entry.FullName
    }

    $manifestPath = Join-Path $bundle 'RiAuth.DeviceHost.manifest.ps1'
    $installerPath = Join-Path $bundle 'Install-DeviceHost.ps1'
    $hostPath = Join-Path $bundle 'RiAuth.DeviceHost.exe'
    $providerPath = Join-Path $bundle 'RiAuth.CredentialProvider.dll'
    if (-not [IO.Path]::GetFullPath($PSCommandPath).Equals(
            [IO.Path]::GetFullPath($installerPath), [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Run Install-DeviceHost.ps1 directly from the bundle being verified.'
    }
    # Authenticating the manifest precedes even parsing its first line.
    Assert-SignedBy -LiteralPath $manifestPath -ExpectedThumbprint $ExpectedThumbprint
    $firstLine = Get-Content -LiteralPath $manifestPath -TotalCount 1
    if ($firstLine -cnotmatch '^# RIAUTH-BUNDLE-V1 ([A-Za-z0-9+/]+={0,2})$') {
        throw 'The signed bundle manifest has no valid v1 envelope.'
    }
    try {
        $utf8 = [Text.UTF8Encoding]::new($false, $true)
        $json = $utf8.GetString([Convert]::FromBase64String($Matches[1]))
        $document = [Text.Json.JsonDocument]::Parse($json)
    } catch {
        throw "The signed bundle manifest has invalid UTF-8 JSON: $_"
    }
    try {
        if ($document.RootElement.ValueKind -ne [Text.Json.JsonValueKind]::Object) {
            throw 'The signed bundle manifest must contain one JSON object.'
        }
        $expectedKeys = @('format', 'version', 'provider_clsid', 'host_sha256', 'provider_sha256', 'installer_sha256')
        $manifest = @{}
        $propertyCount = 0
        foreach ($property in $document.RootElement.EnumerateObject()) {
            $propertyCount++
            if (-not ($expectedKeys -ccontains $property.Name) -or $manifest.ContainsKey($property.Name) -or
                $property.Value.ValueKind -ne [Text.Json.JsonValueKind]::String) {
                throw 'The signed bundle manifest has an invalid, duplicate, or non-string field.'
            }
            $manifest[$property.Name] = $property.Value.GetString()
        }
        if ($propertyCount -ne $expectedKeys.Count) {
            throw 'The signed bundle manifest has missing or extra fields.'
        }
    } finally {
        $document.Dispose()
    }
    if ($manifest['format'] -cne 'riauth-windows-device-bundle/v1' -or
        $manifest['provider_clsid'] -cne $ExpectedProviderClsid -or
        $manifest['version'] -cnotmatch '^[0-9A-Za-z][0-9A-Za-z._+-]{0,63}$') {
        throw 'The signed bundle format, version or provider CLSID is invalid.'
    }
    foreach ($key in @('host_sha256', 'provider_sha256', 'installer_sha256')) {
        if ($manifest[$key] -cnotmatch '^[0-9A-F]{64}$') {
            throw "The signed bundle has an invalid $key value."
        }
    }
    Assert-SignedBy -LiteralPath $installerPath -ExpectedThumbprint $ExpectedThumbprint
    Assert-SignedBy -LiteralPath $hostPath -ExpectedThumbprint $ExpectedThumbprint
    Assert-SignedBy -LiteralPath $providerPath -ExpectedThumbprint $ExpectedThumbprint
    Assert-Sha256 -LiteralPath $installerPath -ExpectedHash $manifest['installer_sha256']
    Assert-Sha256 -LiteralPath $hostPath -ExpectedHash $manifest['host_sha256']
    Assert-Sha256 -LiteralPath $providerPath -ExpectedHash $manifest['provider_sha256']
    Assert-X64NativeDll -LiteralPath $providerPath
    return [pscustomobject]@{
        Directory = $bundle
        Host = $hostPath
        Provider = $providerPath
        Version = $manifest['version']
        HostSha256 = $manifest['host_sha256']
        ProviderSha256 = $manifest['provider_sha256']
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
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [string] $ExpectedHash
    )
    Assert-SignedBy -LiteralPath $Executable -ExpectedThumbprint $ExpectedThumbprint
    Assert-NoReparsePoint -LiteralPath $Executable
    if ($ExpectedHash) {
        Assert-Sha256 -LiteralPath $Executable -ExpectedHash $ExpectedHash
    }
}

function Get-InstalledHashes {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)][bool] $HasProvider,
        [Parameter(Mandatory = $true)][bool] $AllowLegacySignedInstall
    )
    $key = Get-Item -LiteralPath $RegistryKey
    $hostHash = $key.GetValue('HostSha256')
    $providerHash = $key.GetValue('ProviderSha256')
    $version = $key.GetValue('BundleVersion')
    if ($null -eq $hostHash -and $null -eq $providerHash -and $null -eq $version) {
        if (-not $AllowLegacySignedInstall) {
            throw 'Installed hashes are absent. A signed legacy installation requires -AllowLegacySignedInstall for migration or removal.'
        }
        return $null
    }
    if ($hostHash -cnotmatch '^[0-9A-F]{64}$' -or
        $providerHash -cnotmatch '^[0-9A-F]{64}$' -or
        [string]::IsNullOrWhiteSpace($version)) {
        throw 'Installed bundle hash/version registry values are incomplete or invalid.'
    }
    if (-not $HasProvider) {
        throw 'Installed bundle metadata exists without a registered credential provider.'
    }
    return [pscustomobject]@{
        HostSha256 = $hostHash
        ProviderSha256 = $providerHash
        Version = $version
    }
}

function Set-InstalledHashes {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)][string] $HostSha256,
        [Parameter(Mandatory = $true)][string] $ProviderSha256,
        [Parameter(Mandatory = $true)][string] $Version
    )
    New-ItemProperty -LiteralPath $RegistryKey -Name 'HostSha256' -PropertyType String -Value $HostSha256 -Force | Out-Null
    New-ItemProperty -LiteralPath $RegistryKey -Name 'ProviderSha256' -PropertyType String -Value $ProviderSha256 -Force | Out-Null
    New-ItemProperty -LiteralPath $RegistryKey -Name 'BundleVersion' -PropertyType String -Value $Version -Force | Out-Null
}

function Restore-InstalledHashes {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        $Previous
    )
    if ($null -eq $Previous) {
        $key = Get-Item -LiteralPath $RegistryKey
        foreach ($name in @('HostSha256', 'ProviderSha256', 'BundleVersion')) {
            if ($key.GetValueNames() -contains $name) {
                Remove-ItemProperty -LiteralPath $RegistryKey -Name $name
            }
        }
    } else {
        Set-InstalledHashes -RegistryKey $RegistryKey -HostSha256 $Previous.HostSha256 -ProviderSha256 $Previous.ProviderSha256 -Version $Previous.Version
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
    try {
        New-Item -Path $ClassKey -Value 'RiAuth Credential Provider' | Out-Null
        New-Item -Path $InprocKey -Value $DllPath | Out-Null
        New-ItemProperty -LiteralPath $InprocKey -Name 'ThreadingModel' -PropertyType String -Value 'Apartment' | Out-Null
        New-Item -Path $ProviderKey -Value 'RiAuth' | Out-Null
        Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
    } catch {
        $failure = $_
        try {
            # Either New-Item can create a key and then report an error. The
            # preflight above proved these keys were absent before this call.
            if (Test-Path -LiteralPath $ProviderKey) {
                Remove-Item -LiteralPath $ProviderKey -Recurse -Force
            }
            if (Test-Path -LiteralPath $ClassKey) {
                Remove-Item -LiteralPath $ClassKey -Recurse -Force
            }
        } catch {
            throw "Provider registration failed ($failure); partial registry cleanup also failed ($_)."
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
        [string] $ExpectedHash,
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey
    )
    Assert-SignedBy -LiteralPath $DllPath -ExpectedThumbprint $ExpectedThumbprint
    Assert-NoReparsePoint -LiteralPath $DllPath
    if ($ExpectedHash) {
        Assert-Sha256 -LiteralPath $DllPath -ExpectedHash $ExpectedHash
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
if ($Action -ne 'Verify') {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'Run the device host installer as an administrator.'
    }
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

if ($Action -in @('Install', 'Verify')) {
    if ([string]::IsNullOrWhiteSpace($BundlePath) -or
        [string]::IsNullOrWhiteSpace($SignerThumbprint)) {
        throw "$Action requires -BundlePath and -SignerThumbprint."
    }
} elseif ($Action -eq 'Update') {
    if ([string]::IsNullOrWhiteSpace($BundlePath) -or $PSBoundParameters.ContainsKey('SignerThumbprint')) {
        throw 'Update requires -BundlePath and uses the installed signer pin.'
    }
} elseif ($PSBoundParameters.ContainsKey('BundlePath') -or
          $PSBoundParameters.ContainsKey('SignerThumbprint')) {
    throw 'Uninstall accepts no bundle or signer parameters.'
}
if ($AllowLegacySignedInstall -and $Action -notin @('Update', 'Uninstall')) {
    throw '-AllowLegacySignedInstall is available only for update or uninstall of a signed legacy installation.'
}

if ($Action -eq 'Verify') {
    $pin = Normalize-Thumbprint $SignerThumbprint
    $verifiedBundle = Read-SignedBundle -Directory $BundlePath -ExpectedThumbprint $pin -ExpectedProviderClsid $providerClsid
    Write-Output "Verified signed bundle version $($verifiedBundle.Version) with signer $pin"
    return
}

# Only one elevated installer may swap the machine-wide binaries or pin.
$mutex = [Threading.Mutex]::new($false, 'Global\RiAuth.DeviceHost.Install')
$hasMutex = $false
try {
    $hasMutex = $mutex.WaitOne(30000)
    if (-not $hasMutex) {
        throw 'Another device host installation is in progress.'
    }

    if ($Action -eq 'Install') {
        if ((Test-Path -LiteralPath $registryKey) -or
            (Test-Path -LiteralPath $installDirectory) -or
            (Test-Path -LiteralPath $providerClassKey) -or
            (Test-Path -LiteralPath $providerRegistryKey)) {
            throw 'A device host installation, signer pin, or provider registration already exists.'
        }
        $pin = Normalize-Thumbprint $SignerThumbprint
        $bundle = Read-SignedBundle -Directory $BundlePath -ExpectedThumbprint $pin -ExpectedProviderClsid $providerClsid
        New-Item -ItemType Directory -Path $installParent -Force | Out-Null
        Assert-NoReparsePoint -LiteralPath $installParent
        $stageDirectory = Join-Path $installParent ("DeviceHost.stage.$([guid]::NewGuid().ToString('N'))")
        $installed = $false
        $providerRegistrationStarted = $false
        try {
            New-Item -ItemType Directory -Path $stageDirectory | Out-Null
            $stageExecutable = Join-Path $stageDirectory 'RiAuth.DeviceHost.exe'
            $stageProviderDll = Join-Path $stageDirectory 'RiAuth.CredentialProvider.dll'
            Copy-Item -LiteralPath $bundle.Host -Destination $stageExecutable
            Copy-Item -LiteralPath $bundle.Provider -Destination $stageProviderDll
            Assert-InstalledPayload -Executable $stageExecutable -ExpectedThumbprint $pin -ExpectedHash $bundle.HostSha256
            Assert-SignedBy -LiteralPath $stageProviderDll -ExpectedThumbprint $pin
            Assert-NoReparsePoint -LiteralPath $stageProviderDll
            Assert-Sha256 -LiteralPath $stageProviderDll -ExpectedHash $bundle.ProviderSha256
            Assert-X64NativeDll -LiteralPath $stageProviderDll
            Move-Item -LiteralPath $stageDirectory -Destination $installDirectory
            $installed = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $bundle.HostSha256
            Assert-SignedBy -LiteralPath $installedProviderDll -ExpectedThumbprint $pin
            Assert-Sha256 -LiteralPath $installedProviderDll -ExpectedHash $bundle.ProviderSha256

            New-Item -Path $registryParent -Force | Out-Null
            New-Item -Path $registryKey | Out-Null
            New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' -PropertyType String -Value $pin | Out-Null
            New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid | Out-Null
            Set-InstalledHashes -RegistryKey $registryKey -HostSha256 $bundle.HostSha256 -ProviderSha256 $bundle.ProviderSha256 -Version $bundle.Version
            $providerRegistrationStarted = $true
            New-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $bundle.ProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
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
                if (Test-Path -LiteralPath $registryKey) {
                    Remove-Item -LiteralPath $registryKey -Recurse -Force
                }
                if ($installed -and (Test-Path -LiteralPath $installDirectory)) {
                    Remove-Item -LiteralPath $installDirectory -Recurse -Force
                }
            } catch {
                throw "Install failed ($failure); rollback also failed ($_). Inspect $installDirectory, $registryKey and provider registration."
            }
            throw $failure
        } finally {
            if (Test-Path -LiteralPath $stageDirectory) {
                try { Remove-Item -LiteralPath $stageDirectory -Recurse -Force } catch {
                    Write-Warning "Staging directory requires cleanup: $stageDirectory ($_)"
                }
            }
        }
        Write-Output "Installed bundle $($bundle.Version): $installedExecutable and provider $installedProviderDll with signer $pin"
    } elseif ($Action -eq 'Update') {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        $bundle = Read-SignedBundle -Directory $BundlePath -ExpectedThumbprint $pin -ExpectedProviderClsid $providerClsid
        Assert-NoReparsePoint -LiteralPath $installDirectory
        $hasProvider = Get-InstalledProviderState -RegistryKey $registryKey -ExpectedClsid $providerClsid -DllPath $installedProviderDll -ClassKey $providerClassKey -ProviderKey $providerRegistryKey
        $previousHashes = Get-InstalledHashes -RegistryKey $registryKey -HasProvider $hasProvider -AllowLegacySignedInstall ([bool]$AllowLegacySignedInstall)
        $oldHostHash = if ($null -ne $previousHashes) { $previousHashes.HostSha256 } else { $null }
        $oldProviderHash = if ($null -ne $previousHashes) { $previousHashes.ProviderSha256 } else { $null }
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $oldHostHash
        if ($hasProvider) {
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $oldProviderHash -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        }
        $otherFiles = Get-ChildItem -LiteralPath $installDirectory -Force |
            Where-Object {
                $_.Name -cne 'RiAuth.DeviceHost.exe' -and
                -not ($hasProvider -and $_.Name -ceq 'RiAuth.CredentialProvider.dll')
            } | Select-Object -First 1
        if ($null -ne $otherFiles) {
            throw "Unexpected installation file remains at $($otherFiles.FullName). Resolve it before updating."
        }
        if ($null -ne $previousHashes -and $hasProvider -and $previousHashes.HostSha256 -ceq $bundle.HostSha256 -and
            $previousHashes.ProviderSha256 -ceq $bundle.ProviderSha256 -and
            $previousHashes.Version -ceq $bundle.Version) {
            Write-Output "Bundle $($bundle.Version) is already installed and verified."
            return
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
            Copy-Item -LiteralPath $bundle.Host -Destination $stagedExecutable
            Copy-Item -LiteralPath $bundle.Provider -Destination $stagedProviderDll
            Assert-InstalledPayload -Executable $stagedExecutable -ExpectedThumbprint $pin -ExpectedHash $bundle.HostSha256
            Assert-SignedBy -LiteralPath $stagedProviderDll -ExpectedThumbprint $pin
            Assert-NoReparsePoint -LiteralPath $stagedProviderDll
            Assert-Sha256 -LiteralPath $stagedProviderDll -ExpectedHash $bundle.ProviderSha256
            Assert-X64NativeDll -LiteralPath $stagedProviderDll
            Move-Item -LiteralPath $installedExecutable -Destination $backupExecutable
            $oldHostMoved = $true
            Move-Item -LiteralPath $stagedExecutable -Destination $installedExecutable
            $newHostMoved = $true
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $bundle.HostSha256
            if ($hasProvider) {
                Move-Item -LiteralPath $installedProviderDll -Destination $backupProviderDll
                $oldProviderMoved = $true
            }
            Move-Item -LiteralPath $stagedProviderDll -Destination $installedProviderDll
            $newProviderMoved = $true
            Assert-SignedBy -LiteralPath $installedProviderDll -ExpectedThumbprint $pin
            Assert-Sha256 -LiteralPath $installedProviderDll -ExpectedHash $bundle.ProviderSha256
            Assert-X64NativeDll -LiteralPath $installedProviderDll
            if (-not $hasProvider) {
                $providerMarkerAdded = $true
                New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid | Out-Null
                $providerRegistrationStarted = $true
                New-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
            }
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $bundle.ProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
            Set-InstalledHashes -RegistryKey $registryKey -HostSha256 $bundle.HostSha256 -ProviderSha256 $bundle.ProviderSha256 -Version $bundle.Version
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
                    $currentKey = Get-Item -LiteralPath $registryKey
                    if ($currentKey.GetValueNames() -contains 'CredentialProviderClsid') {
                        Remove-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid'
                    }
                }
                if ($newProviderMoved) {
                    Remove-Item -LiteralPath $installedProviderDll -Force
                }
                if ($oldProviderMoved) {
                    Move-Item -LiteralPath $backupProviderDll -Destination $installedProviderDll
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $oldProviderHash -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
                if ($newHostMoved) {
                    Remove-Item -LiteralPath $installedExecutable -Force
                }
                if ($oldHostMoved) {
                    Move-Item -LiteralPath $backupExecutable -Destination $installedExecutable
                    Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $oldHostHash
                }
                Restore-InstalledHashes -RegistryKey $registryKey -Previous $previousHashes
            } catch {
                throw "Update failed ($failure); rollback also failed ($_). Inspect signed backups at $backupExecutable and $backupProviderDll"
            }
            throw $failure
        } finally {
            if (Test-Path -LiteralPath $stagedExecutable) {
                try { Remove-Item -LiteralPath $stagedExecutable -Force } catch {
                    Write-Warning "Staged executable requires cleanup: $stagedExecutable ($_)"
                }
            }
            if (Test-Path -LiteralPath $stagedProviderDll) {
                try { Remove-Item -LiteralPath $stagedProviderDll -Force } catch {
                    Write-Warning "Staged provider requires cleanup: $stagedProviderDll ($_)"
                }
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
        Write-Output "Updated bundle $($bundle.Version): $installedExecutable and provider $installedProviderDll with pinned signer $pin"
    } else {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        Assert-NoReparsePoint -LiteralPath $installDirectory
        $hasProvider = Get-InstalledProviderState -RegistryKey $registryKey -ExpectedClsid $providerClsid -DllPath $installedProviderDll -ClassKey $providerClassKey -ProviderKey $providerRegistryKey
        $installedHashes = Get-InstalledHashes -RegistryKey $registryKey -HasProvider $hasProvider -AllowLegacySignedInstall ([bool]$AllowLegacySignedInstall)
        $oldHostHash = if ($null -ne $installedHashes) { $installedHashes.HostSha256 } else { $null }
        $oldProviderHash = if ($null -ne $installedHashes) { $installedHashes.ProviderSha256 } else { $null }
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $oldHostHash
        if ($hasProvider) {
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $oldProviderHash -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        }

        # Remote revocation must precede local purge. Any child entry, including
        # a hidden file or link, blocks removal of the binaries and registration.
        if (Test-Path -LiteralPath $stateDirectory) {
            $stateRoot = Get-Item -LiteralPath $stateDirectory -Force
            if (($stateRoot.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "Device state path is a reparse point: $stateDirectory"
            }
            if (-not $stateRoot.PSIsContainer) {
                throw "Device state path is not a directory: $stateDirectory"
            }
            $stateEntry = Get-ChildItem -LiteralPath $stateDirectory -Force | Select-Object -First 1
            if ($null -ne $stateEntry) {
                throw "Device state remains at $($stateEntry.FullName). Revoke the device and purge state before uninstalling."
            }
        }
        $otherFiles = Get-ChildItem -LiteralPath $installDirectory -Force |
            Where-Object {
                $_.Name -cne 'RiAuth.DeviceHost.exe' -and
                -not ($hasProvider -and $_.Name -ceq 'RiAuth.CredentialProvider.dll')
            } | Select-Object -First 1
        if ($null -ne $otherFiles) {
            throw "Unexpected installation file remains at $($otherFiles.FullName). Resolve it before uninstalling."
        }

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
                Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $oldHostHash
                New-Item -Path $registryParent -Force | Out-Null
                New-Item -Path $registryKey -Force | Out-Null
                New-ItemProperty -LiteralPath $registryKey -Name 'SignerThumbprint' -PropertyType String -Value $pin -Force | Out-Null
                if ($hasProvider) {
                    New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid -Force | Out-Null
                    Restore-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $oldProviderHash -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
                Restore-InstalledHashes -RegistryKey $registryKey -Previous $installedHashes
            } catch {
                throw "Uninstall failed ($failure); rollback also failed ($_). Inspect $quarantineDirectory, $registryKey and provider registration before retrying."
            }
            throw $failure
        }
        try { Remove-Item -LiteralPath $quarantineDirectory -Recurse -Force } catch {
            Write-Warning "Device host registration was removed, but signed binaries remain at $quarantineDirectory and require cleanup after LogonUI releases them: $_"
        }
        Write-Output 'Uninstalled riAuth device host and credential provider. The empty ProgramData directory, if present, was retained.'
    }
} finally {
    if ($hasMutex) { $mutex.ReleaseMutex() }
    $mutex.Dispose()
}
