# Windows device host recovery

The riAuth credential-provider tile requires a reachable server and a fresh
ticket redemption for every submission. It does not use offline tickets. Keep
an ordinary Windows sign-in method and an authorized administrator available
for recovery. Other Windows credential providers are not controlled by this
tile, so revoking riAuth access alone does not disable every Windows logon path.

## Connection or riAuth credentials fail

1. Use an existing Windows system provider and the local Windows password to
   enter the machine. Do not remove all system credential providers.
2. Check the device's network, DNS, certificate trust, clock, and the riAuth
   issuer. The tile denies access during a timeout, server denial, or network
   outage; it has no offline fallback. Retry the tile after restoring service.
3. If the riAuth password or OTP is lost, recover that identity through the
   normal riAuth administrator or MFA recovery path. Retry with current proof.
   The separate local Windows password is still required by this provider.
4. If the local Windows password is lost, use Windows account recovery through
   an authorized administrator. A riAuth assertion cannot replace that password.

## Device revoked, local secret lost, or local account replaced

1. From an authorized riAuth administrator session, confirm the device ID and
   revoke it. If the enrolled host can reach riAuth, its `revoke --issuer ...
   --revision ... --idempotency-key ... --token-stdin` command sends the remote
   revoke and purges local state only after server confirmation. Reuse the same
   idempotency key on an ambiguous retry.
2. If revocation was completed elsewhere and the local host cannot reach the
   server, sign in through a Windows system provider as an administrator and run
   `RiAuth.DeviceHost.exe purge-local --confirm-remote-revoked`. This command
   trusts the operator's confirmation; it does not check server state. Never
   use it as a substitute for remote revocation.
3. Re-enroll with a current authorized bearer token on redirected standard
   input, the current configuration revision, a new idempotency key, and
   `--local-account .\name`. Enrollment rotates the device secret and pins the
   current local account SID. A deleted and recreated account has a new SID and
   needs this re-enrollment. `--replace` is required if local state remains.

The server cannot return an old device secret. A lost secret requires remote
revocation and enrollment. A riAuth user disable also revokes enrolled devices;
reenabling the user does not restore those device credentials.

## Uninstall or repair a failed update

1. Revoke the device remotely and clear local state as above. The installer
   refuses uninstall while any entry remains in
   `%ProgramData%\RiAuth\DeviceHost`; inspect unexpected files only after
   confirming remote revocation.
2. Run the pinned-signer `Install-DeviceHost.ps1 -Action Uninstall` from a
   trusted signed release as an elevated 64-bit PowerShell 7 administrator.
   Independently check the script's Authenticode signer before execution as
   shown in [README.md](README.md). For a previous signed installation without
   bundle hash metadata, add `-AllowLegacySignedInstall`.
   It deregisters the provider and removes the host together. If LogonUI still
   holds the DLL open, the script reports any retained quarantine path; finish
   cleanup after a restart and confirm registration is absent.
3. For an update failure, use a Windows system provider to enter the machine.
   Inspect the installer error and recorded backup paths before retrying the
   same verified bundle or a previously signed bundle from the pinned signer.
   The installer attempts to restore the prior signed binaries and registry
   registration. If LogonUI has the provider DLL open, restart, sign in using a
   Windows system provider, and retry. Do not manually replace a registered
   provider DLL with an unverified file.

Signer rotation and an expired or unavailable signing certificate need a
separate authorized recovery plan. Normal updates require the originally
pinned signer. The provider does not enforce riAuth denial against Windows
system providers; machine-wide policy requires a separate design.
