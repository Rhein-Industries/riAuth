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

    [switch] $AllowLegacySignedInstall,

    [switch] $AllowDowngrade,

    [string] $RecoveryReason
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

function ConvertTo-ReleaseVersion {
    param([Parameter(Mandatory = $true)][string] $Value)
    if ($Value.Length -gt 64 -or
        $Value -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
        throw 'Bundle version must be canonical major.minor.patch decimal components.'
    }
    try {
        return [Version]::Parse($Value)
    } catch {
        throw 'Bundle version components must fit in a signed 32-bit integer.'
    }
}

function Test-CanonicalReleaseVersion {
    param([Parameter(Mandatory = $true)][string] $Value)
    if ($Value.Length -gt 64 -or
        $Value -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
        return $false
    }
    $parsed = $null
    return [Version]::TryParse($Value, [ref] $parsed)
}

function Write-DowngradeIntent {
    param(
        [Parameter(Mandatory = $true)][string] $CurrentVersion,
        [Parameter(Mandatory = $true)][string] $HighestVersion,
        [Parameter(Mandatory = $true)][string] $TargetVersion,
        [Parameter(Mandatory = $true)][string] $TargetHostSha256,
        [Parameter(Mandatory = $true)][string] $TargetProviderSha256,
        [Parameter(Mandatory = $true)][string] $Reason
    )
    $eventCreate = Join-Path $env:SystemRoot 'System32\eventcreate.exe'
    if (-not (Test-Path -LiteralPath $eventCreate -PathType Leaf)) {
        throw 'Windows eventcreate.exe is unavailable; refusing an unaudited downgrade.'
    }
    $operator = [Security.Principal.WindowsIdentity]::GetCurrent().Name
    $description = "riAuth DeviceHost recovery by $operator; installed=$CurrentVersion; highest=$HighestVersion; target=$TargetVersion; host_sha256=$TargetHostSha256; provider_sha256=$TargetProviderSha256; reason=$Reason"
    $output = & $eventCreate /L APPLICATION /T WARNING /ID 801 /SO RiAuthDeviceHost /D $description 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Could not write the Windows Application downgrade audit event; refusing update: $output"
    }
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

function Assert-SignedScriptBytesBy {
    param(
        [Parameter(Mandatory = $true)][byte[]] $Content,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )

    if ($PSVersionTable.PSVersion.Major -lt 7 -or
        ($PSVersionTable.PSVersion.Major -eq 7 -and $PSVersionTable.PSVersion.Minor -lt 4)) {
        throw 'Bundle verification, install and update require PowerShell 7.4 or newer for UTF-8 content signature verification.'
    }
    # PowerShell 7.4 supports UTF-8 script content here. The signature is
    # checked against the same byte array that Read-SignedBundle parses.
    $signature = Get-AuthenticodeSignature -Content $Content -SourcePathOrExtension 'ps1'
    if ($signature.Status -ne 'Valid' -or $null -eq $signature.SignerCertificate) {
        throw "Authenticode signature is not valid for the bundle manifest ($($signature.Status))"
    }
    $actual = Normalize-Thumbprint $signature.SignerCertificate.Thumbprint
    if ($actual -cne $ExpectedThumbprint) {
        throw 'Authenticode signer does not match the pinned certificate for the bundle manifest.'
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
    # Never reopen the source manifest after signature verification: an
    # extracted bundle can be replaced between two path-based reads.
    $manifestStream = [IO.File]::Open(
        $manifestPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($manifestStream.Length -gt 1048576) {
            throw 'The signed bundle manifest exceeds the 1 MiB limit.'
        }
        $manifestBytes = [byte[]]::new([int]$manifestStream.Length)
        $manifestStream.ReadExactly($manifestBytes, 0, $manifestBytes.Length)
    } finally {
        $manifestStream.Dispose()
    }
    Assert-SignedScriptBytesBy -Content $manifestBytes -ExpectedThumbprint $ExpectedThumbprint
    $utf8 = [Text.UTF8Encoding]::new($false, $true)
    try {
        $manifestText = $utf8.GetString($manifestBytes)
    } catch {
        throw "The signed bundle manifest is not valid UTF-8: $_"
    }
    $lineEnd = $manifestText.IndexOf("`n")
    if ($lineEnd -lt 0) {
        throw 'The signed bundle manifest has no first-line terminator.'
    }
    $firstLine = $manifestText.Substring(0, $lineEnd)
    if ($firstLine.EndsWith("`r")) {
        $firstLine = $firstLine.Substring(0, $firstLine.Length - 1)
    }
    if ($firstLine -cnotmatch '^# RIAUTH-BUNDLE-V1 ([A-Za-z0-9+/]+={0,2})$') {
        throw 'The signed bundle manifest has no valid v1 envelope.'
    }
    try {
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
        $manifest['version'] -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
        throw 'The signed bundle format, version or provider CLSID is invalid.'
    }
    $null = ConvertTo-ReleaseVersion $manifest['version']
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
        InstallerSha256 = $manifest['installer_sha256']
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
    $hadHighestVersion = $key.GetValueNames() -contains 'HighestBundleVersion'
    $highestVersion = $key.GetValue('HighestBundleVersion')
    if ($null -eq $hostHash -and $null -eq $providerHash -and $null -eq $version) {
        if ($hadHighestVersion) {
            throw 'Highest bundle version exists without installed bundle metadata.'
        }
        if (-not $AllowLegacySignedInstall) {
            throw 'Installed hashes are absent. A signed legacy installation requires -AllowLegacySignedInstall for migration or removal.'
        }
        return $null
    }
    if ($hostHash -cnotmatch '^[0-9A-F]{64}$' -or
        $providerHash -cnotmatch '^[0-9A-F]{64}$' -or
        $version -isnot [string] -or
        $version -cnotmatch '^[0-9A-Za-z][0-9A-Za-z._+-]{0,63}$') {
        throw 'Installed bundle hash/version registry values are incomplete or invalid.'
    }
    # The first signed-bundle builder accepted opaque release labels. Their
    # hash metadata remains trustworthy, but their order cannot be inferred.
    $isOpaqueVersion = -not (Test-CanonicalReleaseVersion $version)
    if ($hadHighestVersion) {
        if ($isOpaqueVersion) {
            throw 'An opaque installed version cannot have a numeric version floor.'
        }
        $currentParsed = ConvertTo-ReleaseVersion $version
        $highestParsed = ConvertTo-ReleaseVersion $highestVersion
        if ($highestParsed.CompareTo($currentParsed) -lt 0) {
            throw 'Highest bundle version is older than the installed bundle version.'
        }
    } elseif (-not $isOpaqueVersion) {
        # Bundles installed before the version floor existed start at their
        # verified current version, without silently lowering that baseline.
        $highestVersion = $version
    }
    if (-not $HasProvider) {
        throw 'Installed bundle metadata exists without a registered credential provider.'
    }
    return [pscustomobject]@{
        HostSha256 = $hostHash
        ProviderSha256 = $providerHash
        Version = $version
        HighestVersion = $highestVersion
        HadHighestVersion = $hadHighestVersion
        IsOpaqueVersion = $isOpaqueVersion
    }
}

function Set-InstalledHashes {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)][string] $HostSha256,
        [Parameter(Mandatory = $true)][string] $ProviderSha256,
        [Parameter(Mandatory = $true)][string] $Version,
        [Parameter(Mandatory = $true)][string] $HighestVersion
    )
    # Advance the floor first. A power loss between registry writes must never
    # make a newer installed version appear to be the highest permitted one.
    New-ItemProperty -LiteralPath $RegistryKey -Name 'HighestBundleVersion' -PropertyType String -Value $HighestVersion -Force | Out-Null
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
        foreach ($name in @('HostSha256', 'ProviderSha256', 'BundleVersion', 'HighestBundleVersion')) {
            if ($key.GetValueNames() -contains $name) {
                Remove-ItemProperty -LiteralPath $RegistryKey -Name $name
            }
        }
    } elseif ($Previous.HadHighestVersion) {
        Set-InstalledHashes -RegistryKey $RegistryKey -HostSha256 $Previous.HostSha256 -ProviderSha256 $Previous.ProviderSha256 -Version $Previous.Version -HighestVersion $Previous.HighestVersion
    } else {
        $key = Get-Item -LiteralPath $RegistryKey
        if ($key.GetValueNames() -contains 'HighestBundleVersion') {
            Remove-ItemProperty -LiteralPath $RegistryKey -Name 'HighestBundleVersion'
        }
        New-ItemProperty -LiteralPath $RegistryKey -Name 'HostSha256' -PropertyType String -Value $Previous.HostSha256 -Force | Out-Null
        New-ItemProperty -LiteralPath $RegistryKey -Name 'ProviderSha256' -PropertyType String -Value $Previous.ProviderSha256 -Force | Out-Null
        New-ItemProperty -LiteralPath $RegistryKey -Name 'BundleVersion' -PropertyType String -Value $Previous.Version -Force | Out-Null
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

function Write-PendingUpdate {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)] $Update
    )
    $fields = @(
        'riauth-update/v1', $Update.Nonce,
        $(if ($Update.OldHasProvider) { '1' } else { '0' }),
        $(if ($Update.OldHadMetadata) { '1' } else { '0' }),
        $Update.OldHostSha256, $Update.OldProviderSha256,
        $Update.OldVersion, $Update.OldHighestVersion,
        $(if ($Update.OldHadHighestVersion) { '1' } else { '0' }),
        $Update.NewHostSha256, $Update.NewProviderSha256,
        $Update.NewVersion, $Update.NewHighestVersion,
        $Update.NewInstallerSha256
    )
    $key = Get-Item -LiteralPath $RegistryKey
    if ($key.GetValueNames() -contains 'PendingUpdate') {
        throw 'A pending device host update already exists.'
    }
    # One registry value is the write-ahead record. Flush it before the first
    # copy or rename, including the stage files, so retry can classify every
    # interrupted state without trusting incomplete bundle metadata.
    $key.SetValue('PendingUpdate', ($fields -join '|'), [Microsoft.Win32.RegistryValueKind]::String)
    $key.Flush()
}

function Read-PendingUpdate {
    param([Parameter(Mandatory = $true)][string] $RegistryKey)
    $key = Get-Item -LiteralPath $RegistryKey
    $raw = $key.GetValue('PendingUpdate')
    if ($null -eq $raw) { return $null }
    if ($raw -isnot [string]) { throw 'Pending update record is not a string.' }
    $fields = $raw.Split('|')
    if ($fields.Count -ne 14 -or $fields[0] -cne 'riauth-update/v1' -or
        $fields[1] -cnotmatch '^[0-9a-f]{32}$' -or
        $fields[2] -cnotmatch '^[01]$' -or $fields[3] -cnotmatch '^[01]$' -or
        $fields[4] -cnotmatch '^[0-9A-F]{64}$' -or
        $fields[5] -cnotmatch '^(-|[0-9A-F]{64})$' -or
        $fields[8] -cnotmatch '^[01]$' -or
        $fields[9] -cnotmatch '^[0-9A-F]{64}$' -or
        $fields[10] -cnotmatch '^[0-9A-F]{64}$' -or
        $fields[13] -cnotmatch '^[0-9A-F]{64}$') {
        throw 'Pending update record is malformed.'
    }
    $oldHasProvider = $fields[2] -ceq '1'
    $oldHadMetadata = $fields[3] -ceq '1'
    $oldHadHighestVersion = $fields[8] -ceq '1'
    if (($oldHasProvider -and $fields[5] -ceq '-') -or
        (-not $oldHasProvider -and $fields[5] -cne '-') -or
        ($oldHadMetadata -and -not $oldHasProvider) -or
        (-not $oldHadMetadata -and ($fields[6] -cne '-' -or $fields[7] -cne '-' -or $oldHadHighestVersion))) {
        throw 'Pending update record has inconsistent prior installation state.'
    }
    $newVersion = ConvertTo-ReleaseVersion $fields[11]
    $newHighestVersion = ConvertTo-ReleaseVersion $fields[12]
    if ($oldHadMetadata) {
        if ($fields[6] -cnotmatch '^[0-9A-Za-z][0-9A-Za-z._+-]{0,63}$') {
            throw 'Pending update has an invalid previous version label.'
        }
        $oldOpaqueVersion = -not (Test-CanonicalReleaseVersion $fields[6])
        if ($oldOpaqueVersion) {
            if ($oldHadHighestVersion -or $fields[7] -cne '-') {
                throw 'Pending update has an ambiguous opaque prior version floor.'
            }
            # The prior signer and hashes can be checked, but v1 opaque labels
            # cannot be compared with the canonical target release version.
            $expectedHighest = $fields[11]
        } else {
            $oldVersion = ConvertTo-ReleaseVersion $fields[6]
            $oldHighestVersion = ConvertTo-ReleaseVersion $fields[7]
            if ($oldHighestVersion.CompareTo($oldVersion) -lt 0 -or
                (-not $oldHadHighestVersion -and $fields[7] -cne $fields[6])) {
                throw 'Pending update has an invalid previous version floor.'
            }
            $expectedHighest = if ($newVersion.CompareTo($oldHighestVersion) -gt 0) { $fields[11] } else { $fields[7] }
        }
    } else {
        $expectedHighest = $fields[11]
    }
    if ($fields[12] -cne $expectedHighest -or $newHighestVersion.CompareTo($newVersion) -lt 0) {
        throw 'Pending update would lower the recorded version floor.'
    }
    return [pscustomobject]@{
        Nonce = $fields[1]
        OldHasProvider = $oldHasProvider
        OldHadMetadata = $oldHadMetadata
        OldHostSha256 = $fields[4]
        OldProviderSha256 = $fields[5]
        OldVersion = $fields[6]
        OldHighestVersion = $fields[7]
        OldHadHighestVersion = $oldHadHighestVersion
        NewHostSha256 = $fields[9]
        NewProviderSha256 = $fields[10]
        NewVersion = $fields[11]
        NewHighestVersion = $fields[12]
        NewInstallerSha256 = $fields[13]
    }
}

function Clear-PendingUpdate {
    param([Parameter(Mandatory = $true)][string] $RegistryKey)
    Remove-ItemProperty -LiteralPath $RegistryKey -Name 'PendingUpdate'
    (Get-Item -LiteralPath $RegistryKey).Flush()
}

function Get-UpdatePayloadState {
    param(
        [Parameter(Mandatory = $true)][string] $LiteralPath,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [Parameter(Mandatory = $true)][string] $NewHash,
        [string] $OldHash,
        [switch] $Provider
    )
    if (-not (Test-Path -LiteralPath $LiteralPath)) { return 'missing' }
    Assert-NoReparsePoint -LiteralPath $LiteralPath
    if (-not (Test-Path -LiteralPath $LiteralPath -PathType Leaf)) {
        throw "Update payload path is not a file: $LiteralPath"
    }
    Assert-SignedBy -LiteralPath $LiteralPath -ExpectedThumbprint $ExpectedThumbprint
    if ($Provider) { Assert-X64NativeDll -LiteralPath $LiteralPath }
    $hash = (Get-FileHash -LiteralPath $LiteralPath -Algorithm SHA256).Hash.ToUpperInvariant()
    if ($hash -ceq $NewHash) { return 'new' }
    if ($OldHash -and $hash -ceq $OldHash) { return 'old' }
    throw "Installed update payload matches neither recorded signed version: $LiteralPath"
}

function Assert-UpdateEntries {
    param(
        [Parameter(Mandatory = $true)][string] $Directory,
        [Parameter(Mandatory = $true)] $Update
    )
    $allowed = @(
        'RiAuth.DeviceHost.exe', 'RiAuth.CredentialProvider.dll',
        "RiAuth.DeviceHost.stage.$($Update.Nonce).exe",
        "RiAuth.DeviceHost.backup.$($Update.Nonce).exe",
        "RiAuth.CredentialProvider.stage.$($Update.Nonce).dll"
    )
    if ($Update.OldHasProvider) {
        $allowed += "RiAuth.CredentialProvider.backup.$($Update.Nonce).dll"
    }
    foreach ($entry in @(Get-ChildItem -LiteralPath $Directory -Force)) {
        if ($entry.PSIsContainer -or -not ($allowed -ccontains $entry.Name)) {
            throw "Unexpected installation entry during pending update: $($entry.FullName)"
        }
        Assert-NoReparsePoint -LiteralPath $entry.FullName
    }
}

function Assert-UpdateRegistryValues {
    param(
        [Parameter(Mandatory = $true)][string] $RegistryKey,
        [Parameter(Mandatory = $true)] $Update,
        [Parameter(Mandatory = $true)][string] $ExpectedClsid
    )
    $key = Get-Item -LiteralPath $RegistryKey
    $allowed = @{
        HostSha256 = @($Update.NewHostSha256)
        ProviderSha256 = @($Update.NewProviderSha256)
        BundleVersion = @($Update.NewVersion)
        HighestBundleVersion = @($Update.NewHighestVersion)
    }
    if ($Update.OldHadMetadata) {
        $allowed.HostSha256 += $Update.OldHostSha256
        $allowed.ProviderSha256 += $Update.OldProviderSha256
        $allowed.BundleVersion += $Update.OldVersion
        if ($Update.OldHadHighestVersion) {
            $allowed.HighestBundleVersion += $Update.OldHighestVersion
        }
    }
    foreach ($name in @('HostSha256', 'ProviderSha256', 'BundleVersion', 'HighestBundleVersion')) {
        if ($Update.OldHadMetadata -and
            $name -in @('HostSha256', 'ProviderSha256', 'BundleVersion') -and
            -not ($key.GetValueNames() -contains $name)) {
            throw "Pending update is missing prior registry metadata: $name"
        }
        if ($key.GetValueNames() -contains $name) {
            if (-not ($allowed[$name] -ccontains $key.GetValue($name))) {
                throw "Pending update found unexpected registry value: $name"
            }
        }
    }
    $marker = $key.GetValue('CredentialProviderClsid')
    if (($null -ne $marker -and $marker -cne $ExpectedClsid) -or
        ($Update.OldHasProvider -and $marker -cne $ExpectedClsid)) {
        throw 'Pending update found an unexpected provider marker.'
    }
}

function Complete-ProviderRegistration {
    param(
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey,
        [Parameter(Mandatory = $true)][string] $DllPath
    )
    foreach ($entry in @(
        @{ Path = $ClassKey; Name = ''; Value = 'RiAuth Credential Provider' },
        @{ Path = $InprocKey; Name = ''; Value = $DllPath },
        @{ Path = $InprocKey; Name = 'ThreadingModel'; Value = 'Apartment' },
        @{ Path = $ProviderKey; Name = ''; Value = 'RiAuth' }
    )) {
        if (-not (Test-Path -LiteralPath $entry.Path)) {
            New-Item -Path $entry.Path | Out-Null
        }
        $key = Get-Item -LiteralPath $entry.Path
        $actual = $key.GetValue($entry.Name)
        if ($null -ne $actual -and $actual -cne $entry.Value) {
            throw "A foreign provider registration blocks update recovery: $($entry.Path)"
        }
        if ($null -eq $actual) {
            $key.SetValue($entry.Name, $entry.Value, [Microsoft.Win32.RegistryValueKind]::String)
            $key.Flush()
        }
    }
    Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
}

function Assert-CompatibleProviderRegistration {
    param(
        [Parameter(Mandatory = $true)][string] $ClassKey,
        [Parameter(Mandatory = $true)][string] $InprocKey,
        [Parameter(Mandatory = $true)][string] $ProviderKey,
        [Parameter(Mandatory = $true)][string] $DllPath,
        [Parameter(Mandatory = $true)][bool] $OldHasProvider
    )
    if ($OldHasProvider) {
        # An ordinary update never changes provider registration. Missing or
        # foreign entries are not an interrupted binary swap.
        Assert-ProviderRegistration -ClassKey $ClassKey -InprocKey $InprocKey -ProviderKey $ProviderKey -DllPath $DllPath
        return
    }
    # A legacy host-only migration may have stopped partway through adding
    # these keys. Accept absent values, but never overwrite a foreign value.
    foreach ($entry in @(
        @{ Path = $ClassKey; Name = ''; Value = 'RiAuth Credential Provider' },
        @{ Path = $InprocKey; Name = ''; Value = $DllPath },
        @{ Path = $InprocKey; Name = 'ThreadingModel'; Value = 'Apartment' },
        @{ Path = $ProviderKey; Name = ''; Value = 'RiAuth' }
    )) {
        if (Test-Path -LiteralPath $entry.Path) {
            $actual = (Get-Item -LiteralPath $entry.Path).GetValue($entry.Name)
            if ($null -ne $actual -and $actual -cne $entry.Value) {
                throw "A foreign provider registration blocks update recovery: $($entry.Path)"
            }
        }
    }
}

function Complete-PendingUpdate {
    param(
        [Parameter(Mandatory = $true)] $Update,
        [Parameter(Mandatory = $true)] $Bundle,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )
    if ($Bundle.Version -cne $Update.NewVersion -or
        $Bundle.HostSha256 -cne $Update.NewHostSha256 -or
        $Bundle.ProviderSha256 -cne $Update.NewProviderSha256 -or
        $Bundle.InstallerSha256 -cne $Update.NewInstallerSha256) {
        throw 'Pending update requires the exact originally verified signed bundle.'
    }
    Assert-NoReparsePoint -LiteralPath $installDirectory
    Assert-UpdateEntries -Directory $installDirectory -Update $Update
    Assert-UpdateRegistryValues -RegistryKey $registryKey -Update $Update -ExpectedClsid $providerClsid
    Assert-CompatibleProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll -OldHasProvider $Update.OldHasProvider
    $hostStage = Join-Path $installDirectory "RiAuth.DeviceHost.stage.$($Update.Nonce).exe"
    $hostBackup = Join-Path $installDirectory "RiAuth.DeviceHost.backup.$($Update.Nonce).exe"
    $providerStage = Join-Path $installDirectory "RiAuth.CredentialProvider.stage.$($Update.Nonce).dll"
    $providerBackup = Join-Path $installDirectory "RiAuth.CredentialProvider.backup.$($Update.Nonce).dll"

    $hostState = Get-UpdatePayloadState -LiteralPath $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -OldHash $Update.OldHostSha256 -NewHash $Update.NewHostSha256
    $providerOldHash = if ($Update.OldHasProvider) { $Update.OldProviderSha256 } else { $null }
    $providerState = Get-UpdatePayloadState -LiteralPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -OldHash $providerOldHash -NewHash $Update.NewProviderSha256 -Provider
    $hostBackupState = Get-UpdatePayloadState -LiteralPath $hostBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $Update.OldHostSha256
    $providerBackupState = if ($Update.OldHasProvider) {
        Get-UpdatePayloadState -LiteralPath $providerBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $Update.OldProviderSha256 -Provider
    } else { 'missing' }
    if (($hostState -eq 'missing' -and $hostBackupState -eq 'missing') -or
        ($Update.OldHasProvider -and $providerState -eq 'missing' -and $providerBackupState -eq 'missing')) {
        throw 'Pending update lost both the installed file and its signed backup.'
    }
    # A crash during Copy-Item can leave an incomplete stage. It is never
    # activated; replace only this exact journal-owned path from the verified
    # bundle. A backup or canonical payload with invalid bytes always fails.
    foreach ($stage in @(
        @{ Path = $hostStage; Hash = $Update.NewHostSha256; Provider = $false },
        @{ Path = $providerStage; Hash = $Update.NewProviderSha256; Provider = $true }
    )) {
        if (Test-Path -LiteralPath $stage.Path) {
            Assert-NoReparsePoint -LiteralPath $stage.Path
            if (-not (Test-Path -LiteralPath $stage.Path -PathType Leaf)) {
                throw "Pending update stage is not a file: $($stage.Path)"
            }
            try {
                $null = Get-UpdatePayloadState -LiteralPath $stage.Path -ExpectedThumbprint $ExpectedThumbprint -NewHash $stage.Hash -Provider:($stage.Provider)
            } catch {
                Remove-Item -LiteralPath $stage.Path -Force
            }
        }
    }
    if ($hostState -ne 'new') {
        if ($hostState -eq 'old') {
            if ($hostBackupState -eq 'missing') {
                Move-Item -LiteralPath $installedExecutable -Destination $hostBackup
            } else {
                Remove-Item -LiteralPath $installedExecutable -Force
            }
        }
        if (-not (Test-Path -LiteralPath $hostStage)) {
            Copy-Item -LiteralPath $Bundle.Host -Destination $hostStage
        }
        Assert-InstalledPayload -Executable $hostStage -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $Update.NewHostSha256
        Move-Item -LiteralPath $hostStage -Destination $installedExecutable
    }
    Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $Update.NewHostSha256
    if ($providerState -ne 'new') {
        if ($providerState -eq 'old') {
            if ($providerBackupState -eq 'missing') {
                Move-Item -LiteralPath $installedProviderDll -Destination $providerBackup
            } else {
                Remove-Item -LiteralPath $installedProviderDll -Force
            }
        }
        if (-not (Test-Path -LiteralPath $providerStage)) {
            Copy-Item -LiteralPath $Bundle.Provider -Destination $providerStage
        }
        $null = Get-UpdatePayloadState -LiteralPath $providerStage -ExpectedThumbprint $ExpectedThumbprint -NewHash $Update.NewProviderSha256 -Provider
        Move-Item -LiteralPath $providerStage -Destination $installedProviderDll
    }
    $null = Get-UpdatePayloadState -LiteralPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -NewHash $Update.NewProviderSha256 -Provider
    if (-not $Update.OldHasProvider) {
        New-ItemProperty -LiteralPath $registryKey -Name 'CredentialProviderClsid' -PropertyType String -Value $providerClsid -Force | Out-Null
        Complete-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
    }
    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $Update.NewProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
    Set-InstalledHashes -RegistryKey $registryKey -HostSha256 $Update.NewHostSha256 -ProviderSha256 $Update.NewProviderSha256 -Version $Update.NewVersion -HighestVersion $Update.NewHighestVersion
    $verified = Get-InstalledHashes -RegistryKey $registryKey -HasProvider $true -AllowLegacySignedInstall $false
    if ($verified.HostSha256 -cne $Update.NewHostSha256 -or
        $verified.ProviderSha256 -cne $Update.NewProviderSha256 -or
        $verified.Version -cne $Update.NewVersion -or
        $verified.HighestVersion -cne $Update.NewHighestVersion) {
        throw 'Pending update did not establish the recorded installed metadata.'
    }
    foreach ($path in @($hostStage, $providerStage, $hostBackup, $providerBackup)) {
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Force
        }
    }
    Clear-PendingUpdate -RegistryKey $registryKey
}

function Restore-UnjournaledPayload {
    param(
        [Parameter(Mandatory = $true)][string] $CanonicalPath,
        [Parameter(Mandatory = $true)][string] $BackupPath,
        [Parameter(Mandatory = $true)][string] $OldHash,
        [Parameter(Mandatory = $true)][string] $AttemptedHash,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [switch] $Provider
    )
    $state = Get-UpdatePayloadState -LiteralPath $CanonicalPath -ExpectedThumbprint $ExpectedThumbprint -OldHash $OldHash -NewHash $AttemptedHash -Provider:$Provider
    if ($OldHash -ceq $AttemptedHash -and $state -eq 'new') { $state = 'old' }
    $backupState = Get-UpdatePayloadState -LiteralPath $BackupPath -ExpectedThumbprint $ExpectedThumbprint -NewHash $OldHash -Provider:$Provider
    if ($state -ne 'old') {
        if ($backupState -eq 'missing') {
            throw "Cannot restore the prior signed payload without its hash-matching backup: $CanonicalPath"
        }
        if ($state -eq 'new') { Remove-Item -LiteralPath $CanonicalPath -Force }
        Move-Item -LiteralPath $BackupPath -Destination $CanonicalPath
    }
    $null = Get-UpdatePayloadState -LiteralPath $CanonicalPath -ExpectedThumbprint $ExpectedThumbprint -NewHash $OldHash -Provider:$Provider
}

function Recover-UnjournaledUpdate {
    param(
        [Parameter(Mandatory = $true)] $Bundle,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )
    # Releases before the write-ahead record could leave stage/backup files in
    # the install directory. Accept only one recognizable nonce and complete
    # registry metadata; ambiguous older states need operator investigation.
    Assert-NoReparsePoint -LiteralPath $installDirectory
    $entries = @(Get-ChildItem -LiteralPath $installDirectory -Force)
    $nonces = @()
    foreach ($entry in $entries) {
        if ($entry.Name -cmatch '^RiAuth\.(?:DeviceHost|CredentialProvider)\.(?:stage|backup)\.([0-9a-f]{32})\.(?:exe|dll)$') {
            $nonces += $Matches[1]
        }
    }
    $nonces = @($nonces | Sort-Object -Unique)
    if ($nonces.Count -eq 0) { return $null }
    if ($nonces.Count -ne 1) {
        throw 'Unjournaled update contains multiple backup/stage generations; recovery requires operator investigation.'
    }
    $hasProvider = Get-InstalledProviderState -RegistryKey $registryKey -ExpectedClsid $providerClsid -DllPath $installedProviderDll -ClassKey $providerClassKey -ProviderKey $providerRegistryKey
    $metadata = Get-InstalledHashes -RegistryKey $registryKey -HasProvider $hasProvider -AllowLegacySignedInstall ([bool]$AllowLegacySignedInstall)
    if ($null -eq $metadata -or -not $hasProvider) {
        throw 'Unjournaled legacy update has no complete version/hash baseline; restore it from a verified release before retrying.'
    }
    $newHighestVersion = if ($metadata.IsOpaqueVersion -or
        (ConvertTo-ReleaseVersion $Bundle.Version).CompareTo((ConvertTo-ReleaseVersion $metadata.HighestVersion)) -gt 0) {
        $Bundle.Version
    } else { $metadata.HighestVersion }
    $record = [pscustomobject]@{
        Nonce = $nonces[0]
        OldHasProvider = $true
        OldHadMetadata = $true
        OldHostSha256 = $metadata.HostSha256
        OldProviderSha256 = $metadata.ProviderSha256
        OldVersion = $metadata.Version
        OldHighestVersion = if ($metadata.IsOpaqueVersion) { '-' } else { $metadata.HighestVersion }
        OldHadHighestVersion = $metadata.HadHighestVersion
        NewHostSha256 = $Bundle.HostSha256
        NewProviderSha256 = $Bundle.ProviderSha256
        NewVersion = $Bundle.Version
        NewHighestVersion = $newHighestVersion
        NewInstallerSha256 = $Bundle.InstallerSha256
    }
    Assert-UpdateEntries -Directory $installDirectory -Update $record
    $hostStage = Join-Path $installDirectory "RiAuth.DeviceHost.stage.$($record.Nonce).exe"
    $hostBackup = Join-Path $installDirectory "RiAuth.DeviceHost.backup.$($record.Nonce).exe"
    $providerStage = Join-Path $installDirectory "RiAuth.CredentialProvider.stage.$($record.Nonce).dll"
    $providerBackup = Join-Path $installDirectory "RiAuth.CredentialProvider.backup.$($record.Nonce).dll"

    if ($metadata.Version -ceq $Bundle.Version -and
        $metadata.HostSha256 -ceq $Bundle.HostSha256 -and
        $metadata.ProviderSha256 -ceq $Bundle.ProviderSha256) {
        # The old installer committed metadata and the new pair, then failed
        # to delete backups. These unused backups have no recorded old hash;
        # verify their pinned signatures before deleting them, never restore
        # or execute them.
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $Bundle.HostSha256
        Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $Bundle.ProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        foreach ($stage in @(
            @{ Path = $hostStage; Hash = $Bundle.HostSha256; Provider = $false },
            @{ Path = $providerStage; Hash = $Bundle.ProviderSha256; Provider = $true }
        )) {
            if (Test-Path -LiteralPath $stage.Path) {
                $null = Get-UpdatePayloadState -LiteralPath $stage.Path -ExpectedThumbprint $ExpectedThumbprint -NewHash $stage.Hash -Provider:($stage.Provider)
            }
        }
        foreach ($backup in @(
            @{ Path = $hostBackup; Provider = $false },
            @{ Path = $providerBackup; Provider = $true }
        )) {
            if (Test-Path -LiteralPath $backup.Path) {
                Assert-NoReparsePoint -LiteralPath $backup.Path
                Assert-SignedBy -LiteralPath $backup.Path -ExpectedThumbprint $ExpectedThumbprint
                if ($backup.Provider) { Assert-X64NativeDll -LiteralPath $backup.Path }
            }
        }
        foreach ($path in @($hostStage, $providerStage, $hostBackup, $providerBackup)) {
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
        }
        return "Removed verified residue from completed bundle $($Bundle.Version)."
    }

    if ($metadata.Version -ceq $Bundle.Version) {
        # The old installer could republish different bytes under one version.
        # Never finish that replacement. Restore the registry-bound prior pair
        # from exact signed backups, then remove the abandoned stage files.
        Assert-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
        foreach ($stage in @($hostStage, $providerStage)) {
            if (Test-Path -LiteralPath $stage) {
                Assert-NoReparsePoint -LiteralPath $stage
                if (-not (Test-Path -LiteralPath $stage -PathType Leaf)) {
                    throw "Interrupted update stage is not a file: $stage"
                }
                # A crash during Copy-Item can leave an incomplete stage. It
                # is discarded after the old canonical pair is verified.
            }
        }
        $oldHostState = Get-UpdatePayloadState -LiteralPath $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -OldHash $metadata.HostSha256 -NewHash $Bundle.HostSha256
        $oldProviderState = Get-UpdatePayloadState -LiteralPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -OldHash $metadata.ProviderSha256 -NewHash $Bundle.ProviderSha256 -Provider
        $oldHostBackupState = Get-UpdatePayloadState -LiteralPath $hostBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $metadata.HostSha256
        $oldProviderBackupState = Get-UpdatePayloadState -LiteralPath $providerBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $metadata.ProviderSha256 -Provider
        if (($oldHostState -ne 'old' -and $metadata.HostSha256 -cne $Bundle.HostSha256 -and $oldHostBackupState -eq 'missing') -or
            ($oldHostState -eq 'missing' -and $oldHostBackupState -eq 'missing') -or
            ($oldProviderState -ne 'old' -and $metadata.ProviderSha256 -cne $Bundle.ProviderSha256 -and $oldProviderBackupState -eq 'missing') -or
            ($oldProviderState -eq 'missing' -and $oldProviderBackupState -eq 'missing')) {
            throw 'Interrupted same-version update cannot restore both prior signed files; no files were changed.'
        }
        Restore-UnjournaledPayload -CanonicalPath $installedExecutable -BackupPath $hostBackup -OldHash $metadata.HostSha256 -AttemptedHash $Bundle.HostSha256 -ExpectedThumbprint $ExpectedThumbprint
        Restore-UnjournaledPayload -CanonicalPath $installedProviderDll -BackupPath $providerBackup -OldHash $metadata.ProviderSha256 -AttemptedHash $Bundle.ProviderSha256 -ExpectedThumbprint $ExpectedThumbprint -Provider
        Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $metadata.HostSha256
        Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -ExpectedHash $metadata.ProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
        foreach ($path in @($hostStage, $providerStage, $hostBackup, $providerBackup)) {
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
        }
        return "Restored verified prior bundle $($metadata.Version) after an interrupted same-version replacement."
    }

    if ($metadata.IsOpaqueVersion) {
        # There is no defensible ordering between an older opaque label and
        # this numeric target. Require explicit audited intent before using
        # the still-verifiable prior signatures and hashes for recovery.
        $needsRecovery = $true
        $highestForAudit = 'unknown-opaque-prior'
    } else {
        $currentVersion = ConvertTo-ReleaseVersion $metadata.Version
        $targetVersion = ConvertTo-ReleaseVersion $Bundle.Version
        if ($targetVersion.CompareTo($currentVersion) -eq 0) {
            throw 'Unjournaled update has a different bundle under the same release version.'
        }
        $needsRecovery = $targetVersion.CompareTo((ConvertTo-ReleaseVersion $metadata.HighestVersion)) -lt 0
        $highestForAudit = $metadata.HighestVersion
    }
    if ($needsRecovery) {
        if (-not $AllowDowngrade) {
            throw 'Unjournaled update has a lower or unordered prior version; audited recovery intent is required.'
        }
        Write-DowngradeIntent -CurrentVersion $metadata.Version -HighestVersion $highestForAudit -TargetVersion $Bundle.Version -TargetHostSha256 $Bundle.HostSha256 -TargetProviderSha256 $Bundle.ProviderSha256 -Reason $RecoveryReason
    } elseif ($AllowDowngrade) {
        throw '-AllowDowngrade is permitted only below the highest installed version.'
    }
    Assert-ProviderRegistration -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey -DllPath $installedProviderDll
    $hostState = Get-UpdatePayloadState -LiteralPath $installedExecutable -ExpectedThumbprint $ExpectedThumbprint -OldHash $metadata.HostSha256 -NewHash $Bundle.HostSha256
    $providerState = Get-UpdatePayloadState -LiteralPath $installedProviderDll -ExpectedThumbprint $ExpectedThumbprint -OldHash $metadata.ProviderSha256 -NewHash $Bundle.ProviderSha256 -Provider
    $hostBackupState = Get-UpdatePayloadState -LiteralPath $hostBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $metadata.HostSha256
    $providerBackupState = Get-UpdatePayloadState -LiteralPath $providerBackup -ExpectedThumbprint $ExpectedThumbprint -NewHash $metadata.ProviderSha256 -Provider
    if (($hostState -eq 'missing' -and $hostBackupState -eq 'missing') -or
        ($providerState -eq 'missing' -and $providerBackupState -eq 'missing')) {
        throw 'Unjournaled update has lost an installed file and its expected signed backup.'
    }
    # Stage bytes may be incomplete after power loss. Complete-PendingUpdate
    # validates or replaces only the journal-owned stage from the signed bundle.
    Write-PendingUpdate -RegistryKey $registryKey -Update $record
    Complete-PendingUpdate -Update $record -Bundle $Bundle -ExpectedThumbprint $ExpectedThumbprint
    return "Recovered unjournaled update and completed signed bundle $($Bundle.Version)."
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
if ($AllowDowngrade -ne $PSBoundParameters.ContainsKey('RecoveryReason')) {
    throw 'A downgrade requires both -AllowDowngrade and -RecoveryReason.'
}
if ($AllowDowngrade) {
    if ($Action -ne 'Update') {
        throw '-AllowDowngrade is available only for update.'
    }
    if ($RecoveryReason -ne $RecoveryReason.Trim() -or
        $RecoveryReason.Length -lt 8 -or $RecoveryReason.Length -gt 256 -or
        $RecoveryReason -match '[\x00-\x1F\x7F]') {
        throw 'RecoveryReason must be 8 to 256 visible characters with no leading or trailing whitespace.'
    }
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
            Set-InstalledHashes -RegistryKey $registryKey -HostSha256 $bundle.HostSha256 -ProviderSha256 $bundle.ProviderSha256 -Version $bundle.Version -HighestVersion $bundle.Version
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
        $pendingUpdate = Read-PendingUpdate -RegistryKey $registryKey
        if ($null -ne $pendingUpdate) {
            Complete-PendingUpdate -Update $pendingUpdate -Bundle $bundle -ExpectedThumbprint $pin
            Write-Output "Recovered and completed signed bundle update $($bundle.Version) with pinned signer $pin"
            return
        }
        $unjournaledRecovery = Recover-UnjournaledUpdate -Bundle $bundle -ExpectedThumbprint $pin
        if ($null -ne $unjournaledRecovery) {
            Write-Output $unjournaledRecovery
            return
        }
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
            if ($AllowDowngrade) {
                throw '-AllowDowngrade cannot be used for an idempotent update.'
            }
            Write-Output "Bundle $($bundle.Version) is already installed and verified."
            return
        }

        if ($null -ne $previousHashes -and $previousHashes.IsOpaqueVersion) {
            # Signed bundles before the numeric version contract accepted
            # labels such as v1. Their order is unknown, so migration needs
            # the same audited recovery intent as a known downgrade.
            $needsRecovery = $true
            $highestForAudit = 'unknown-opaque-prior'
        } elseif ($null -ne $previousHashes) {
            $currentVersion = ConvertTo-ReleaseVersion $previousHashes.Version
            $highestVersion = ConvertTo-ReleaseVersion $previousHashes.HighestVersion
            $targetVersion = ConvertTo-ReleaseVersion $bundle.Version
            if ($targetVersion.CompareTo($currentVersion) -eq 0) {
                throw 'A different signed bundle cannot replace the same release version.'
            }
            $needsRecovery = $targetVersion.CompareTo($highestVersion) -lt 0
            $highestForAudit = $previousHashes.HighestVersion
        } else {
            # A legacy signed installation has no version from which to infer
            # a downgrade. Its explicit migration gate is handled above.
            $needsRecovery = $false
        }
        if ($needsRecovery) {
            if (-not $AllowDowngrade) {
                throw "Bundle $($bundle.Version) is below the highest installed version or follows an unordered legacy label. Supply audited recovery intent to update."
            }
            Write-DowngradeIntent -CurrentVersion $previousHashes.Version -HighestVersion $highestForAudit -TargetVersion $bundle.Version -TargetHostSha256 $bundle.HostSha256 -TargetProviderSha256 $bundle.ProviderSha256 -Reason $RecoveryReason
        } elseif ($AllowDowngrade) {
            throw '-AllowDowngrade is permitted only for a bundle below the highest installed version.'
        }

        $newHighestVersion = if ($null -eq $previousHashes -or $previousHashes.IsOpaqueVersion -or
            (ConvertTo-ReleaseVersion $bundle.Version).CompareTo((ConvertTo-ReleaseVersion $previousHashes.HighestVersion)) -gt 0) {
            $bundle.Version
        } else {
            $previousHashes.HighestVersion
        }
        $nonce = [guid]::NewGuid().ToString('N')
        $stagedExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.stage.$nonce.exe"
        $backupExecutable = Join-Path $installDirectory "RiAuth.DeviceHost.backup.$nonce.exe"
        $stagedProviderDll = Join-Path $installDirectory "RiAuth.CredentialProvider.stage.$nonce.dll"
        $backupProviderDll = Join-Path $installDirectory "RiAuth.CredentialProvider.backup.$nonce.dll"
        $updateRecord = [pscustomobject]@{
            Nonce = $nonce
            OldHasProvider = [bool]$hasProvider
            OldHadMetadata = $null -ne $previousHashes
            OldHostSha256 = (Get-FileHash -LiteralPath $installedExecutable -Algorithm SHA256).Hash.ToUpperInvariant()
            OldProviderSha256 = if ($hasProvider) { (Get-FileHash -LiteralPath $installedProviderDll -Algorithm SHA256).Hash.ToUpperInvariant() } else { '-' }
            OldVersion = if ($null -ne $previousHashes) { $previousHashes.Version } else { '-' }
            OldHighestVersion = if ($null -ne $previousHashes -and -not $previousHashes.IsOpaqueVersion) { $previousHashes.HighestVersion } else { '-' }
            OldHadHighestVersion = $null -ne $previousHashes -and $previousHashes.HadHighestVersion
            NewHostSha256 = $bundle.HostSha256
            NewProviderSha256 = $bundle.ProviderSha256
            NewVersion = $bundle.Version
            NewHighestVersion = $newHighestVersion
            NewInstallerSha256 = $bundle.InstallerSha256
        }
        Write-PendingUpdate -RegistryKey $registryKey -Update $updateRecord
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
            Set-InstalledHashes -RegistryKey $registryKey -HostSha256 $bundle.HostSha256 -ProviderSha256 $bundle.ProviderSha256 -Version $bundle.Version -HighestVersion $newHighestVersion
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
                    Assert-SignedBy -LiteralPath $backupProviderDll -ExpectedThumbprint $pin
                    Assert-NoReparsePoint -LiteralPath $backupProviderDll
                    Assert-Sha256 -LiteralPath $backupProviderDll -ExpectedHash $updateRecord.OldProviderSha256
                    Assert-X64NativeDll -LiteralPath $backupProviderDll
                    Move-Item -LiteralPath $backupProviderDll -Destination $installedProviderDll
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $oldProviderHash -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
                if ($newHostMoved) {
                    Remove-Item -LiteralPath $installedExecutable -Force
                }
                if ($oldHostMoved) {
                    Assert-InstalledPayload -Executable $backupExecutable -ExpectedThumbprint $pin -ExpectedHash $updateRecord.OldHostSha256
                    Move-Item -LiteralPath $backupExecutable -Destination $installedExecutable
                    Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $oldHostHash
                }
                Restore-InstalledHashes -RegistryKey $registryKey -Previous $previousHashes
                Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $updateRecord.OldHostSha256
                if ($hasProvider) {
                    Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $updateRecord.OldProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
                }
                foreach ($path in @($stagedExecutable, $stagedProviderDll, $backupExecutable, $backupProviderDll)) {
                    if (Test-Path -LiteralPath $path) {
                        Remove-Item -LiteralPath $path -Force
                    }
                }
                Clear-PendingUpdate -RegistryKey $registryKey
            } catch {
                throw "Update failed ($failure); rollback also failed ($_). Inspect signed backups at $backupExecutable and $backupProviderDll"
            }
            throw $failure
        }
        try {
            Assert-InstalledPayload -Executable $installedExecutable -ExpectedThumbprint $pin -ExpectedHash $bundle.HostSha256
            Assert-InstalledProvider -DllPath $installedProviderDll -ExpectedThumbprint $pin -ExpectedHash $bundle.ProviderSha256 -ClassKey $providerClassKey -InprocKey $providerInprocKey -ProviderKey $providerRegistryKey
            foreach ($path in @($stagedExecutable, $stagedProviderDll, $backupExecutable, $backupProviderDll)) {
                if (Test-Path -LiteralPath $path) {
                    Remove-Item -LiteralPath $path -Force
                }
            }
            Clear-PendingUpdate -RegistryKey $registryKey
        } catch {
            throw "New bundle is installed, but update cleanup is pending ($_). Retry the same signed bundle to complete recovery."
        }
        Write-Output "Updated bundle $($bundle.Version): $installedExecutable and provider $installedProviderDll with pinned signer $pin"
    } else {
        $pin = Get-PinnedSigner -RegistryKey $registryKey
        Assert-SignedBy -LiteralPath $PSCommandPath -ExpectedThumbprint $pin
        if ($null -ne (Read-PendingUpdate -RegistryKey $registryKey)) {
            throw 'A signed update is pending. Retry that exact bundle to complete verified recovery before uninstalling.'
        }
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
