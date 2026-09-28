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
   confirming remote revocation. Complete any pending installer update before
   uninstalling.
2. Run the pinned-signer `Install-DeviceHost.ps1 -Action Uninstall` from a
   trusted signed release as an elevated 64-bit PowerShell 7.0 or newer administrator.
   Independently check the script's Authenticode signer before execution as
   shown in [README.md](README.md). For a previous signed installation without
   bundle hash metadata, add `-AllowLegacySignedInstall`.
   It deregisters the provider and removes the host together. If LogonUI still
   holds the DLL open, the script reports any retained quarantine path; finish
   cleanup after a restart and confirm registration is absent.
3. For an interrupted update, use a Windows system provider to enter the
   machine, independently check the installer signature, and retry `-Action
   Update -BundlePath .` from the **same** verified bundle. A pending-update
   record binds the retry to its original version and hashes. The installer
   checks canonical, staged, and backup files, finishes the update, and clears
   the record only after registration, metadata, and cleanup verify. If
   LogonUI holds the DLL open, restart and retry from the same bundle.
4. A release installed with the earlier installer had no pending-update
   record. The corrected installer can recover a single recognizable stage or
   backup generation when registry metadata and file hashes establish a safe
   state. It restores the prior pair if the earlier update attempted to reuse
   the same version with different bytes. Mixed registry metadata, absent
   hash metadata, or ambiguous files stop for operator investigation. Keep the
   old and target signed bundles available; do not manually replace or delete
   registered binaries.
5. To deliberately install older binaries after recovery, prepare a signed
   recovery bundle that contains this corrected installer, the intended older
   host and provider, and a lower numeric version. Use the pinned release
   signer, then supply `-AllowDowngrade -RecoveryReason '<specific reason>'`
   with `-Action Update`. The installer requires a Windows Application warning
   event before changing files and retains the highest installed version as
   the floor for later ordinary updates. Do not execute an old installer that
   lacks the version check.

Signer rotation and an expired or unavailable signing certificate need a
separate authorized recovery plan. Normal updates require the originally
pinned signer. The provider does not enforce riAuth denial against Windows
system providers; machine-wide policy requires a separate design.
