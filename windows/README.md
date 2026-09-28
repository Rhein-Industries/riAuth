# Windows device host and credential provider

`RiAuth.DeviceHost` is a Windows x64 command-line device client for the Platform
Windows device protocol. It enrolls with an authorized bearer token, stores the
device secret under `%ProgramData%\RiAuth\DeviceHost` using machine DPAPI and a
SYSTEM/Administrators-only ACL, requests an online sign-in ticket, redeems it
once, and checks the returned user and device identity. Enrollment pins one
enabled local Windows account by SID. The native credential provider uses that
binding for an interactive logon or unlock tile. It never sends the separate
local Windows password to riAuth.

## Build and install

Publish from a machine with the .NET 9 SDK:

```powershell
dotnet publish .\windows\RiAuth.DeviceHost\RiAuth.DeviceHost.csproj -c Release -o .\publish\device-host
```

Release configuration targets `win-x64` and produces a self-contained,
single-file `RiAuth.DeviceHost.exe`. On Windows with MSVC C++ Desktop tools and
the Windows SDK, build the x64 native provider:

```powershell
cmake -S .\windows\RiAuth.CredentialProvider -B .\build\credential-provider -A x64
cmake --build .\build\credential-provider --config Release
```

Sign the executable, `RiAuth.CredentialProvider.dll`, and
`Install-DeviceHost.ps1` with the same trusted Authenticode certificate. From an
elevated 64-bit PowerShell 7 process on Windows, install using the certificate's
40-hex-character thumbprint:

```powershell
.\windows\Install-DeviceHost.ps1 -Action Install -PackagePath .\publish\device-host\RiAuth.DeviceHost.exe -SignerThumbprint <thumbprint>
```

The installer pins that signer under `HKLM:\SOFTWARE\RiAuth\DeviceHost`.
After enrollment, register the provider with an update:

```powershell
.\windows\Install-DeviceHost.ps1 -Action Update -PackagePath .\publish\device-host\RiAuth.DeviceHost.exe -CredentialProviderPath .\build\credential-provider\Release\RiAuth.CredentialProvider.dll
```

The same signed DLL can be supplied on first install. Updates verify signatures
and stage both binaries with rollback. The provider is never installed unless
`-CredentialProviderPath` is supplied. The script does not disable Windows
system credential providers or install a service.

## Device lifecycle

Run the installed executable as Administrator or SYSTEM. The management bearer
token is one line on redirected standard input; it is never accepted in a
command argument. Use an administrator session or an agent authorized for
`device.enroll` on the exact device, and supply the current numeric configuration
revision plus a stable idempotency key. Reuse the same key and request on an
ambiguous enrollment or revoke retry.

```text
RiAuth.DeviceHost.exe enroll --issuer https://id.example.test --device-id laptop --username alice --local-account .\alice --display-name "Alice laptop" --revision 42 --idempotency-key <stable-key> --token-stdin
RiAuth.DeviceHost.exe status
RiAuth.DeviceHost.exe login --issuer https://id.example.test --proof-stdin
RiAuth.DeviceHost.exe revoke --issuer https://id.example.test --revision 43 --idempotency-key <stable-key> --token-stdin
```

For `login`, redirected standard input contains one JSON line with `password`
and optional `otp`. Feed it from a trusted secret-entry process; avoid putting
passwords in shell commands or files. A successful login prints only a status,
never a ticket or assertion. Server denial, timeout, and network loss all fail
closed. Offline tickets are not requested or stored.

The credential provider obtains the pinned local SID from `cp-account` and
calls `cp-login --proof-stdin` for every credential submission. `cp-login`
returns an approval record only after a fresh online ticket redemption, matching
riAuth identity, and a second check that the local SID is an enabled local user.
The provider then serializes the entered **local Windows password** for that
account to Windows LSA. riAuth approval alone cannot authenticate a Windows
account. The provider fails closed when the host, network, server, or account
check fails. Existing Windows system providers remain available for recovery,
so this provider does not enforce riAuth approval across every possible Windows
sign-in route.

`revoke` clears local state only after the server confirms revocation. If an
administrator revoked the device elsewhere,
`purge-local --confirm-remote-revoked` removes local state for recovery. Uninstall refuses
while any file remains in the device state directory; revoke and clear state
first, then run `Install-DeviceHost.ps1 -Action Uninstall` with the pinned signed
script. Re-enrollment rotates the server secret and invalidates outstanding
tickets. `enroll --replace` is required when local state already exists.

The local protocol invariant check is `dotnet run --project
windows/RiAuth.DeviceHost -- selftest`. Windows signing, installer registration
and rollback, DPAPI and ACL behavior, LogonUI interaction, and actual LSA logon
and unlock require Windows testing.
