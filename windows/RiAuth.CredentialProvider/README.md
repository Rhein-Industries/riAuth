# riAuth Windows credential provider

This x64 COM DLL exposes a Windows logon and unlock tile for the one local SAM
account pinned at device enrollment. It does not replace or filter Windows'
other credential providers. The tile asks for a riAuth password, optional OTP,
and the separate local Windows password. On every submission it invokes the
installed DeviceHost by an absolute Program Files path, sends only the riAuth
proof through a restricted inherited pipe, and waits at most 20 seconds.

DeviceHost must redeem a new one-use server ticket, validate the assertion and
the pinned local SID, then return exactly `RIAUTH-CP-APPROVED-V1\n<SID>\n` on
stdout with exit code zero. The CP also uses `cp-account`, which returns exactly
`RIAUTH-CP-ACCOUNT-V1\n<SID>\n` from protected local state, to associate its
tile with the pinned Windows user. A timeout, transport failure, server denial,
unexpected output, changed SID, disabled local user, or nonlocal SID produces no
Windows credential serialization. The CP never sends the local Windows password
to DeviceHost. It uses the Windows Negotiate package and
`KERB_INTERACTIVE_UNLOCK_LOGON` only after fresh approval.

Build with Visual Studio 2022 and the Windows SDK on an x64 Windows machine:

```powershell
cmake -S windows\RiAuth.CredentialProvider -B build\riauth-cp -A x64
cmake --build build\riauth-cp --config Release
```

Package `RiAuth.CredentialProvider.dll` with DeviceHost through
[`New-DeviceHostBundle.ps1`](../New-DeviceHostBundle.ps1). The bundle builder
signs the DLL, host, installer, and hash manifest with the selected release
signer; [`Install-DeviceHost.ps1`](../Install-DeviceHost.ps1) verifies the
complete bundle and owns COM registration and removal. A credential provider DLL must
be tested in an isolated Windows VM before deployment: this repository has no
Windows LogonUI, secure desktop, domain join, or hardware test environment.
The local Windows password is still required; the server assertion alone is
not an LSA credential. The serialization shape follows Microsoft's
[Credential Provider sample](https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/CredentialProvider/cpp).
