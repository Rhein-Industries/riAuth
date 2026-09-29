#requires -Version 7.0
<#
.SYNOPSIS
Builds a signed, hash-bound riAuth Windows device-host delivery directory.

.DESCRIPTION
Run on Windows with PowerShell 7 and a trusted code-signing certificate whose
private key is available in CurrentUser\My or LocalMachine\My. Supply the
published win-x64 host EXE, native x64 provider DLL, and installer script.
The output directory must not exist. The script signs copies of those files,
then signs a manifest binding their exact signed bytes to one release version.
It also creates a versioned ZIP next to the output directory containing exactly
those four signed files.

.EXAMPLE
.\New-DeviceHostBundle.ps1 -HostPath .\RiAuth.DeviceHost.exe -ProviderPath .\RiAuth.CredentialProvider.dll -InstallerPath .\Install-DeviceHost.ps1 -OutputDirectory .\release-1.0.0 -SignerThumbprint 0123456789ABCDEF0123456789ABCDEF01234567 -ReleaseVersion 1.0.0
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string] $HostPath,

    [Parameter(Mandatory = $true)]
    [string] $ProviderPath,

    [Parameter(Mandatory = $true)]
    [string] $InstallerPath,

    [Parameter(Mandatory = $true)]
    [string] $OutputDirectory,

    [Parameter(Mandatory = $true)]
    [string] $SignerThumbprint,

    [Parameter(Mandatory = $true)]
    [string] $ReleaseVersion,

    [string] $TimestampServer
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

function Assert-SourceFile {
    param([Parameter(Mandatory = $true)][string] $LiteralPath)
    if (-not (Test-Path -LiteralPath $LiteralPath -PathType Leaf)) {
        throw "Bundle source file is missing: $LiteralPath"
    }
    $item = Get-Item -LiteralPath $LiteralPath -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Bundle source file must not be a reparse point: $LiteralPath"
    }
    return $item.FullName
}

function Assert-SignedBy {
    param(
        [Parameter(Mandatory = $true)][string] $LiteralPath,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint
    )
    $signature = Get-AuthenticodeSignature -LiteralPath $LiteralPath
    if ($signature.Status -ne 'Valid' -or $null -eq $signature.SignerCertificate) {
        throw "Authenticode signature is not valid: $LiteralPath ($($signature.Status))"
    }
    if ((Normalize-Thumbprint $signature.SignerCertificate.Thumbprint) -cne $ExpectedThumbprint) {
        throw "Authenticode signer does not match the selected certificate: $LiteralPath"
    }
}

function Sign-And-Verify {
    param(
        [Parameter(Mandatory = $true)][string] $LiteralPath,
        [Parameter(Mandatory = $true)][Security.Cryptography.X509Certificates.X509Certificate2] $Certificate,
        [Parameter(Mandatory = $true)][string] $ExpectedThumbprint,
        [string] $TimestampUrl
    )
    $signing = @{
        LiteralPath = $LiteralPath
        Certificate = $Certificate
        HashAlgorithm = 'SHA256'
        Force = $true
        ErrorAction = 'Stop'
    }
    if (-not [string]::IsNullOrWhiteSpace($TimestampUrl)) {
        $signing.TimestampServer = $TimestampUrl
    }
    Set-AuthenticodeSignature @signing | Out-Null
    Assert-SignedBy -LiteralPath $LiteralPath -ExpectedThumbprint $ExpectedThumbprint
}

if (-not $IsWindows) {
    throw 'Bundle signing requires Windows Authenticode support.'
}
$pin = Normalize-Thumbprint $SignerThumbprint
if ($ReleaseVersion.Length -gt 64 -or
    $ReleaseVersion -cnotmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$') {
    throw 'ReleaseVersion must be canonical major.minor.patch decimal components.'
}
try {
    $null = [Version]::Parse($ReleaseVersion)
} catch {
    throw 'ReleaseVersion components must fit in a signed 32-bit integer.'
}
if (-not [string]::IsNullOrWhiteSpace($TimestampServer)) {
    $timestampUri = $null
    if (-not [Uri]::TryCreate($TimestampServer, [UriKind]::Absolute, [ref] $timestampUri) -or
        $timestampUri.Scheme -cne 'http') {
        throw 'TimestampServer must be an absolute HTTP URL supported by Set-AuthenticodeSignature.'
    }
}

$hostSource = Assert-SourceFile $HostPath
$providerSource = Assert-SourceFile $ProviderPath
$installerSource = Assert-SourceFile $InstallerPath
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) {
    throw "OutputDirectory already exists: $output"
}
$parent = [IO.Path]::GetDirectoryName($output)
if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
    throw "OutputDirectory parent does not exist: $parent"
}
$zipOutput = Join-Path $parent ("RiAuth.DeviceHost-$ReleaseVersion-win-x64.zip")
if (Test-Path -LiteralPath $zipOutput) {
    throw "Bundle ZIP already exists: $zipOutput"
}

$certificate = @(Get-ChildItem -Path 'Cert:\CurrentUser\My', 'Cert:\LocalMachine\My' |
    Where-Object {
        $_.Thumbprint -and
        (Normalize-Thumbprint $_.Thumbprint) -ceq $pin -and
        $_.HasPrivateKey
    } | Select-Object -First 1)
if ($certificate.Count -ne 1) {
    throw 'The selected code-signing certificate with private key was not found.'
}
$codeSigningOid = '1.3.6.1.5.5.7.3.3'
$codeSigningUsage = @($certificate[0].Extensions |
    Where-Object { $_ -is [Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension] } |
    ForEach-Object { $_.EnhancedKeyUsages } |
    Where-Object { $_.Value -ceq $codeSigningOid })
if ($codeSigningUsage.Count -eq 0) {
    throw 'The selected certificate does not permit code signing.'
}

$staging = Join-Path $parent ('.riauth-bundle-' + [guid]::NewGuid().ToString('N'))
$zipStaging = $staging + '.zip'
[IO.Directory]::CreateDirectory($staging) | Out-Null
try {
    $hostFile = Join-Path $staging 'RiAuth.DeviceHost.exe'
    $providerFile = Join-Path $staging 'RiAuth.CredentialProvider.dll'
    $installerFile = Join-Path $staging 'Install-DeviceHost.ps1'
    $manifestFile = Join-Path $staging 'RiAuth.DeviceHost.manifest.ps1'
    Copy-Item -LiteralPath $hostSource -Destination $hostFile -ErrorAction Stop
    Copy-Item -LiteralPath $providerSource -Destination $providerFile -ErrorAction Stop
    Copy-Item -LiteralPath $installerSource -Destination $installerFile -ErrorAction Stop

    foreach ($file in @($hostFile, $providerFile, $installerFile)) {
        Sign-And-Verify -LiteralPath $file -Certificate $certificate[0] -ExpectedThumbprint $pin -TimestampUrl $TimestampServer
    }

    # Hash the final, signed bytes. The manifest is signed last, so its own
    # signature cannot create a circular hash dependency.
    $metadata = [ordered]@{
        format = 'riauth-windows-device-bundle/v1'
        version = $ReleaseVersion
        provider_clsid = '{64A6A7BF-BA56-4463-9692-83A4EA3DBC6C}'
        host_sha256 = (Get-FileHash -LiteralPath $hostFile -Algorithm SHA256).Hash.ToUpperInvariant()
        provider_sha256 = (Get-FileHash -LiteralPath $providerFile -Algorithm SHA256).Hash.ToUpperInvariant()
        installer_sha256 = (Get-FileHash -LiteralPath $installerFile -Algorithm SHA256).Hash.ToUpperInvariant()
    }
    $compactJson = ConvertTo-Json -InputObject $metadata -Compress -Depth 3
    $firstLine = '# RIAUTH-BUNDLE-V1 ' + [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($compactJson))
    [IO.File]::WriteAllText($manifestFile, $firstLine + "`r`n", [Text.UTF8Encoding]::new($false))
    Sign-And-Verify -LiteralPath $manifestFile -Certificate $certificate[0] -ExpectedThumbprint $pin -TimestampUrl $TimestampServer

    $entries = @(Get-ChildItem -LiteralPath $staging -Force)
    $expected = @('Install-DeviceHost.ps1', 'RiAuth.CredentialProvider.dll', 'RiAuth.DeviceHost.exe', 'RiAuth.DeviceHost.manifest.ps1')
    if ($entries.Count -ne $expected.Count -or
        @($entries | Where-Object { -not $_.PSIsContainer -and $expected -ccontains $_.Name }).Count -ne $expected.Count) {
        throw 'The output bundle does not contain exactly the four expected files.'
    }

    [IO.Compression.ZipFile]::CreateFromDirectory(
        $staging, $zipStaging, [IO.Compression.CompressionLevel]::Optimal, $false)
    $archive = [IO.Compression.ZipFile]::OpenRead($zipStaging)
    try {
        $zipEntries = @($archive.Entries)
        if ($zipEntries.Count -ne $expected.Count -or
            @($zipEntries | Where-Object { $expected -ccontains $_.FullName }).Count -ne $expected.Count) {
            throw 'The bundle ZIP does not contain exactly the four expected files.'
        }
    } finally {
        $archive.Dispose()
    }

    # Publish only after all signatures, hashes, and archive entries validate.
    [IO.File]::Move($zipStaging, $zipOutput)
    try {
        [IO.Directory]::Move($staging, $output)
    } catch {
        [IO.File]::Delete($zipOutput)
        throw
    }
    Write-Output "Signed riAuth Windows bundle: $output"
    Write-Output "ZIP containing signed riAuth Windows bundle files: $zipOutput"
} finally {
    if (Test-Path -LiteralPath $zipStaging) {
        Remove-Item -LiteralPath $zipStaging -Force
    }
    if (Test-Path -LiteralPath $staging) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
}
