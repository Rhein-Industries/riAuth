# riauthctl remote client

`riauthctl` is a standalone HTTP client. Its Cargo workspace has no dependency on the `riauth` server, local configuration, or storage crates. It provides routine remote administration while the existing `riauth` executable retains local maintenance and broader commands during the split.

```sh
cargo install --locked --path crates/riauthctl
riauthctl --server https://id.example.com status
riauthctl --server https://id.example.com discovery
riauthctl --server https://id.example.com login alice
riauthctl --server https://id.example.com whoami
riauthctl --server https://id.example.com revision
riauthctl --server https://id.example.com inventory users --limit 100
riauthctl --server https://id.example.com user list
riauthctl --server https://id.example.com user get alice
riauthctl --server https://id.example.com group list
riauthctl --server https://id.example.com group get operators
riauthctl --server https://id.example.com client list
riauthctl --server https://id.example.com client get dashboard
riauthctl --server https://id.example.com session list
riauthctl --server https://id.example.com request inspect CODE
riauthctl --server https://id.example.com device inspect USER-CODE
riauthctl --server https://id.example.com plan --file manifest.json --out plan.json
riauthctl --server https://id.example.com apply --plan plan.json
riauthctl --server https://id.example.com export --out manifest.json
riauthctl --server https://id.example.com logout
```

Set `RIAUTH_SERVER` instead of repeating `--server`. An issuer is always required; the client never reads `riauth.toml` or chooses a local default. The URL must be canonical HTTPS, except that HTTP is accepted for loopback hosts. The client checks the exact discovery issuer and requires its `token_endpoint` to equal `--server` plus `/oauth/token` before sending a password or saved bearer token. A provider-specific issuer may have valid discovery but no management API; use the primary management issuer reported in the error. The client never switches credential destinations automatically. It rejects redirects and caps request time and response size. Requests connect directly and ignore proxy environment variables. Use `--ca-cert private-ca.pem` for a private HTTPS trust root and `--request-timeout SECONDS` to change the per-request deadline.

`login --password-stdin` reads one line from standard input for scripts; interactive login prompts without echo. `RIAUTH_OTP` supplies an optional one-time code; `login --mfa` prompts for one when the variable is unset. Passwords, session tokens, and server error descriptions are not printed. The saved session is bound to the exact issuer and written atomically to an owner-only file. By default it lives at `$XDG_CONFIG_HOME/riauthctl/session.json` or `~/.config/riauthctl/session.json`; use `--session-file` or `RIAUTH_SESSION_FILE` to select another path. Treat that file as a credential.

Inventory accepts `users`, `groups`, `clients`, `sources`, or `audit`, with one page of 1–1000 items per request. Use `--after` for a returned cursor and `--filter` where the server supports it. `--json` emits the versioned CLI envelope.

Routine People, Groups, and Applications commands use `/api/users`, `/api/groups`, and `/api/clients`. They call the server's management service, so the server decides permissions, validation, revision conflicts, and audit records. Examples:

```sh
printf '%s\n' "$NEW_PASSWORD" | riauthctl --server https://id.example.com user create alice --email alice@example.com --password-stdin
riauthctl --server https://id.example.com user update alice --enabled false --revoke-sessions
riauthctl --server https://id.example.com user disable alice
riauthctl --server https://id.example.com user revoke-sessions alice
riauthctl --server https://id.example.com user list --filter ali --limit 100
riauthctl --server https://id.example.com group create operators
riauthctl --server https://id.example.com group list --filter oper --limit 100
riauthctl --server https://id.example.com group has-member operators alice
riauthctl --server https://id.example.com group add-member operators alice
riauthctl --server https://id.example.com group remove-member operators alice
riauthctl --server https://id.example.com client create dashboard --redirect-uri https://dashboard.example.com/callback
riauthctl --server https://id.example.com client update dashboard --name "Dashboard"
riauthctl --server https://id.example.com client list --filter dash --limit 100
riauthctl --server https://id.example.com client create worker --service --secret-file worker-credential.json
riauthctl --server https://id.example.com revision
riauthctl --server https://id.example.com --if-revision REVISION --idempotency-key ROTATION_ID client rotate-secret worker --secret-file worker-new-credential.json
riauthctl --server https://id.example.com session revoke SESSION_ID
```

A changed `--email` is saved unverified, and `user update` has no flag that marks an address verified: the shared user writer refuses a changed address marked verified in the same change (400) for every interface, so the owner confirms it.

`user get`, `group get`, and `client get` use `/api/resources` with exact resource read permission. Plain `user list`, `group list`, and `client list` retain the existing array response from `/api/users`, `/api/groups`, and `/api/clients`. Adding `--filter`, `--limit`, or `--after` uses the bounded `/api/inventory` page response (`items`, `next_cursor`, `revision`); filters match case-sensitive name substrings. Continue a page with `--after` and the same filter. The server filters every read by the caller's authority. Group reads return member IDs as stored by the server. `group has-member GROUP USERNAME` checks the exact group and user resources, so it requires both `group.read` and `user.read`; it returns a `member` boolean only after both reads succeed.

Each direct user, group, or client mutation sends a quoted `If-Match` configuration revision and an `Idempotency-Key`. By default the client reads `/api/state/revision` and generates a fresh key. Use `--if-revision N` when the caller has a known revision or lacks `state.read` permission. Supply `--idempotency-key KEY` to retry the **same command and body** after an uncertain result; the server retains a matching receipt for 24 hours. Shared-secret client creation and secret rotation return only `409 credential_already_issued` on an exact retry, with no secret in the receipt. If private-file delivery failed, inspect the client and rotate with a new key and current revision. `--run-id` adds audit correlation. `user disable` also requests session revocation through the shared user management writer; `user revoke-sessions` leaves the enabled state alone. The shared client writer refuses a direct change of an existing application's enabled state, allowed groups, MFA requirement, or callback, origin and logout endpoints, so `client disable` and the matching `client update` flags answer `409` unless the value is unchanged; use the reviewed commands below for those changes. Client creation uses the usual interactive scopes by default, or `api` for a service client. `--settings-file` supplies a complete ProviderSettings JSON object; on update it replaces the complete settings object. The server applies all client type and setting rules.

Reviewed changes use the same server writers and routes as the server CLI and the browser. Each class has `stage` with a JSON file of at most 32 KiB (the server rejects a larger request body), `change ID` to read the digest, and `approve`, `execute` and `cancel` with `--digest DIGEST`. The server enforces who may author, approve and execute (distinct full administrators), expiry, and every dependency, and binds each decision to the immutable digest. `change` reads carry no revision; each write sends a quoted `If-Match` revision and an `Idempotency-Key` as other direct writes do, so `--if-revision` and `--idempotency-key` retry the same decision.

```sh
riauthctl --server https://id.example.com grants get alice
riauthctl --server https://id.example.com grants set alice --file low-risk-grants.json
riauthctl --server https://id.example.com grants stage alice --file grants.json
riauthctl --server https://id.example.com grants change CHANGE_ID
riauthctl --server https://id.example.com grants approve CHANGE_ID --digest DIGEST
riauthctl --server https://id.example.com grants execute CHANGE_ID --digest DIGEST
riauthctl --server https://id.example.com grants cancel CHANGE_ID --digest DIGEST
riauthctl --server https://id.example.com group review stage operators --file members.json
riauthctl --server https://id.example.com client review stage dashboard --file policy.json
riauthctl --server https://id.example.com client endpoint-review stage dashboard --file endpoints.json
riauthctl --server https://id.example.com client status-review stage dashboard --file status.json
riauthctl --server https://id.example.com client creation-review stage --file new-client.json
riauthctl --server https://id.example.com client creation-review execute CHANGE_ID --digest DIGEST --secret-file new-client-credential.json
```

`grants set` is only for low-risk grants; the server answers `409` for a high-privilege change, which must be staged. `group review`, `client review`, `client endpoint-review`, `client status-review` and `client creation-review` take the same file shapes as the matching `riauth` commands: `{"members":[…]}`, `{"allowed_groups":[…],"require_mfa":bool}`, the five endpoint fields with explicit `null` for a removed front or back channel, `{"enabled":bool}`, and a complete new-client object. `change` always prints the digest and the immutable proposal. Creation `execute` requires `--secret-file`: a generated shared secret is written once to that new owner-only file, and standard output carries only `credential_file`. The file is removed again when the client has no secret or the execution fails. SIGKILL or Ctrl-C can still leave the reserved file behind; delete it before retrying at the same path. The response must say whether a secret was generated (`change.proposal.generate_client_secret`); if it does not and carries no secret, the command fails instead of guessing, and a secret that is present is always kept.

Reviewed creation `execute` is the documented receipt exception. A direct `client create` or `client rotate-secret` stores only a redacted marker, so an exact retry answers `409 credential_already_issued` and never returns the secret again. The reviewed-creation receipt keeps the generated secret for 24 hours so the executing administrator can recover it: an exact retry with the same `--idempotency-key` and `--if-revision` replays the committed result, secret included, and writes it to a **new** `--secret-file` (an existing path is still refused before any request). If the execution committed but the file could not be written, or the response was lost, the error says so. Retry the exact command with a new `--secret-file`; if no `--idempotency-key` was supplied, if 24 hours have passed, or if the retry is refused, inspect the client and rotate its secret with `client rotate-secret`.

Scoped agents use `/api/agents` through the same server writers as `riauth agent`. Only a human administrator session can manage agents; an agent credential is refused locally and by the server.

```sh
riauthctl --server https://id.example.com agent create deployer --permission state.read=state/revision --permission user.write=user/alice --ttl 86400 --out deployer-credential.json
riauthctl --server https://id.example.com agent rotate deployer --out deployer-replacement.json
riauthctl --server https://id.example.com agent revoke deployer
riauthctl --server https://id.example.com agent list
riauthctl --server https://id.example.com --agent-file deployer-credential.json whoami
```

`--permission ACTION=KIND/NAME` (or `ACTION=*`) is repeated for each permission; `--parent USER` names an enabled non-administrator owner. The server validates every action, resource, lifetime and parent. Create, rotate and revoke send a quoted `If-Match` revision and an `Idempotency-Key`, read automatically unless `--if-revision` and `--idempotency-key` are given. The credential is returned only by the first committed response. It is written to the new owner-only `--out` file in the JSON form `--agent-file` reads, and standard output carries only the agent and `credential_file`. An existing `--out` path is refused before any request. A refused or malformed issuance removes the reserved file again. SIGKILL or Ctrl-C can still leave it behind; delete it before retrying at the same path. An exact retry with the same key and revision answers `409 credential_already_issued` without a credential and without a second file; inspect the agent and rotate with a new key and the current revision if the first file was lost. Rotation replaces the token immediately and keeps permissions and parent. Revocation stops the credential at once.

Dynamic-registration templates, signing keys and account invitations use the same server writers and routes as `riauth registration`, `riauth keys` and `riauth account`. Each write sends a quoted `If-Match` revision and an `Idempotency-Key` as other direct writes do, with the same `--if-revision` and `--idempotency-key` retry rules.

```sh
riauthctl --server https://id.example.com registration list
riauthctl --server https://id.example.com registration create --file template.json --out registration-credential.json
riauthctl --server https://id.example.com registration revoke TEMPLATE_ID
riauthctl --server https://id.example.com key list
riauthctl --server https://id.example.com key generate signing-eu --algorithm ES256
riauthctl --server https://id.example.com key bind signing-hsm --signer vault-signer --algorithm RS256
riauthctl --server https://id.example.com key import signing-old --file private-key.pem --algorithm RS256 --kid existing-kid
riauthctl --server https://id.example.com key rotate
riauthctl --server https://id.example.com invitation list
riauthctl --server https://id.example.com invitation create alice --email alice@example.com --name "Alice A." --group operators
riauthctl --server https://id.example.com invitation revoke alice
```

`registration create` reads a template JSON file of at most 32 KiB (the server's request-body limit); the server validates every field. The initial access token is returned only by the first committed response. It is written to the new owner-only `--out` file as `{"issuer","token"}`, the form `riauth registration register --credential-file` reads, and standard output carries only the template and `credential_file`. An existing `--out` path is refused before any request. A refused or malformed issuance removes the reserved file again. SIGKILL or Ctrl-C can still leave it behind; delete it before retrying at the same path. An exact retry with the same key and revision answers `409 credential_already_issued` without a token and without a second file; revoke the template and create another if the first file was lost. `registration revoke` stops the credential at once. `key generate` (alias `key create`) creates a key domain or rotates its active key, keeping verification keys. `key bind` names a signer the server already configures. `key import` reads a private key from an owner-only file of at most 16 KiB; world-readable files are refused locally and the key is never printed. `key rotate` rotates the instance signing key. Key responses hold public key data only. An invitation response carries no link or code: the one-time link is mailed to the invitee, so there is no secret to save. `invitation create` reissues a pending invitation for the same username; the server requires configured mail.

Windows devices and client and RADIUS certificate bindings are Platform features; upstream sources exist in both editions. They use the same server writers and routes as `riauth windows-device`, `riauth certificate`, `riauth radius` and `riauth source`. Writes send a quoted `If-Match` revision and an `Idempotency-Key` as other direct writes do. The server rejects the Platform routes in a build without them.

```sh
riauthctl --server https://id.example.com windows-device list
riauthctl --server https://id.example.com windows-device enroll laptop --username alice --display-name "Alice laptop" --offline-ttl 43200 --out laptop-credential.json
riauthctl --server https://id.example.com windows-device revoke laptop
riauthctl --server https://id.example.com certificate list
riauthctl --server https://id.example.com certificate bind alice --file leaf.pem --san-uri spiffe://example.com/alice
riauthctl --server https://id.example.com certificate revoke BINDING_ID
riauthctl --server https://id.example.com radius certificates
riauthctl --server https://id.example.com radius bind-certificate alice --listener eap-tls --file chain.pem
riauthctl --server https://id.example.com radius revoke-certificate BINDING_ID
riauthctl --server https://id.example.com source list
riauthctl --server https://id.example.com source put --file source.json
```

`windows-device enroll` enrolls or replaces a device. Its secret and optional offline ticket are returned only by the first committed response. They are written to the new owner-only `--out` file as the complete server response, the same content the server CLI writes with `--output-file`, and standard output carries only the device view, `offline_expires_at` and `credential_file`. The file is for operator use: pipe `device_secret` to `riauth windows-device login --secret-stdin`, or send the secret and ticket to the offline-verify route. The Windows DeviceHost does not read it. It enrolls itself and refuses offline tickets. The response must name the requested device and username, show the device not revoked, and carry a `ri_windev_` secret of 32 to 1024 characters, as the DeviceHost also requires; the display name is 1 to 200 bytes. An existing `--out` path is refused before any request. A refused or malformed issuance removes the reserved file again. SIGKILL or Ctrl-C can still leave it behind; delete it before retrying at the same path. An exact retry with the same key and revision answers `409 credential_already_issued` without a secret and without a second file; enroll again with a new key and the current revision to issue a new secret. Revoking a device also stops its sign-in tickets. Certificates are public material: `certificate bind` takes `--file` (a PEM of at most 32 KiB), `--san-uri` and `--san-email` (at least one), `radius bind-certificate` takes a chain file and a listener name of the same form as any name (1 to 64 of `A-Z a-z 0-9 - _ . @`), a bind response must name the requested user (and listener), a revoke response must show `revoked` for the requested binding, and the request body that is actually sent must also fit the server's 32 KiB limit, which a newline-heavy PEM can exceed. `source put` reads `{"source":{…},"client_secret":…}` from a regular file of at most 32 KiB. A file that carries a non-empty `client_secret` must be owner-only (0600 or 0400) or the command refuses it locally, and the secret is never printed. The server answers with the source profile only. Source login, linking and unlinking stay end-user commands in the legacy executable.

Temporary access, offboarding, directory sync and SCIM provisioning use the same server writers and routes as `riauth access`, `riauth offboard`, `riauth directory` and `riauth provision`. Access, offboarding and the provisioning job and deactivation writes send an `Idempotency-Key` and a quoted `If-Match` revision like other direct writes. A principal that may not read the revision (a configured approver without `state.read`) sends the key alone for `access` writes unless `--if-revision` names it; the browser review adapters require both headers, the bearer API does not.

```sh
riauthctl --server https://id.example.com access request ops --reason "Deploy window" --ttl 3600
riauthctl --server https://id.example.com access requests
riauthctl --server https://id.example.com access approve REQUEST_ID
riauthctl --server https://id.example.com access deny REQUEST_ID
riauthctl --server https://id.example.com access revoke GRANT_ID
riauthctl --server https://id.example.com offboard schedule alice --execute-at 2030-01-02T03:04:05+01:00 --timezone Europe/Berlin
riauthctl --server https://id.example.com offboard reschedule JOB_ID --execute-at 1900000000 --timezone UTC
riauthctl --server https://id.example.com offboard cancel JOB_ID
riauthctl --server https://id.example.com directory plan corp --out ldap-plan.json
riauthctl --server https://id.example.com directory apply --plan ldap-plan.json --confirm-removals PLAN_ID
riauthctl --server https://id.example.com directory workspace plan corp --out workspace-plan.json
riauthctl --server https://id.example.com directory entra apply --plan entra-plan.json
riauthctl --server https://id.example.com provision plan scim-main --out scim-plan.json
riauthctl --server https://id.example.com provision apply --plan scim-plan.json
riauthctl --server https://id.example.com provision resolve JOB_ID --observed applied --evidence TICKET-123
```

`directory plan`, `directory workspace|entra plan` and `provision plan` page the server's snapshot until it completes and save the reviewed plan to a new owner-only file; an existing output, and an `--out` path that is not valid UTF-8 (the summary names the file as JSON text), are refused before any request or write. The same UTF-8 check applies to `plan --out`. `apply` takes that file, checks it against the server's copy of the plan (a changed or foreign plan is refused), and applies exactly that plan id. These plan and apply requests carry no revision or request key, as the server CLI sends none: a plan is bound by its id, and a repeated page request must not replay an old page. For that reason `plan` and `apply` (desired state), `directory plan|apply` (LDAP, Workspace and Entra) and `provision plan|apply` refuse a global `--if-revision` or `--idempotency-key` locally with an explanatory message instead of silently ignoring it; the server checks the revision a plan was made at when it applies it. A plan that removes users, memberships or access is refused locally unless `--confirm-removals PLAN_ID` names its exact id, which also goes out in the two confirmation headers; nothing is confirmed implicitly. These directories and provisioning targets are configured on the server; this client lists, plans and applies their existing operations. `provision resolve`, `resolve-deactivation`, `dismiss-deactivation` and the two `recover-dispatch` commands take the server's enumerations (`applied`, `not_applied`, `absent`; `remote_absent`, `permanently_unverifiable`; `worker_lost`, `legacy_untracked`), 1 to 280 characters of evidence without control characters, and the two attestation flags that recovery requires. `offboard schedule` sends unix seconds as a number and any other value as text; the server accepts only an absolute instant. `--timezone` is an audit label that follows the server's grammar: one to three slash-separated components, each 1 to 64 characters from `A-Z a-z 0-9 _ + -` (the bound is per component, so `<32 characters>/<32 characters>` is valid); a label outside it, such as one with four components, an empty component, a leading or trailing slash, a space, a dot or a non-ASCII character, is refused locally before any request.

Workflow approval (Platform) uses the same services as `riauth workflow`:
`POST /api/workflow-approvals/review`, `/activate` and `/revoke`. Author one workflow
in a desired-state plan; a second enabled administrator reviews it and a third
activates it. Every client write sends an `Idempotency-Key` and quoted `If-Match`;
the current revision is a first-operation guard. The server refuses agent and
delegated callers before receipts. Plan, workflow and approval IDs are JSON body
fields bounded to 1-128 bytes without controls, and may start with a hyphen.
Responses must name the requested plan/decision or both workflow and approval.
Revocation requires `--approval-id` from the inspected activation result; old
untargeted requests fail closed with 400. Never fetch-and-retarget on retry.

Every retry validates its receipt's expiry, exact request fingerprint and
permissions, then reconstructs its outcome from live service checks. Review
revalidates author/reviewer authority and dependencies; its decision view does
not assert an active selection. A completed retirement must still match its
canonical ledger, live revoker authority, current withdrawal and retained fence;
a replacement or inconsistent state conflicts. Former execution dependencies
may be unavailable during retirement. Valid same/new-key replay creates no new
review/revocation, audit, revision or receipt and does not reserve a new key.
Activation retains its separate live selection and stale-run sealing rules. A
same-key retry must retain its original request and If-Match bytes; refreshing
the validator is a fingerprint conflict. The plan's expiry/base revision is
checked for a first review/activation. Review/revoke errors roll back all writes.

```sh
riauthctl --server https://id.example.com workflow review PLAN_ID --decision approve
riauthctl --server https://id.example.com workflow activate PLAN_ID
riauthctl --server https://id.example.com workflow revoke WORKFLOW_ID --approval-id APPROVAL_ID
```

Shared Signals streams (Platform) and the encrypted backup use the same routes as `riauth ssf stream` and `riauth backup`.

```sh
riauthctl --server https://id.example.com ssf stream create --file stream.json
riauthctl --server https://id.example.com ssf stream list
riauthctl --server https://id.example.com ssf stream delete STREAM_ID
riauthctl --server https://id.example.com backup --key-file backup.key --out backup.riauth
```

`ssf stream create|list|delete` act on `/api/ssf/admin/streams` for an administrator or an agent holding `ssf.manage`; the server owns trust pinning, subject bindings, the revision precondition, the receipt and the audit record, and `create` and `delete` send `If-Match` and an `Idempotency-Key`. `create` reads a regular JSON file of at most 32 KiB (the server's request limit also applies to the body sent) with an `id` of 1 to 64 of `A-Z a-z 0-9 - _ . @`; a stream id may start with a hyphen. A file that carries `delivery.authorization_header`, the secret the server sends with each outbound signal, must be owner-only (0600 or 0400) or the command refuses it locally, and the value must follow the server's bound (1 to 2048 bytes, no control characters other than tab); an empty value, a non-string and a control character are refused before any request, and the server decides the exact charset. The value is never printed or put in an error. Zeroization is best-effort: this client wipes the copies it holds when the request ends, but the HTTP client's request body and serde's intermediate buffers are not zeroized. A response that has an `authorization_header` field anywhere (case-insensitively), the secret as a key or an exact string value, or a string containing a secret of at least eight bytes, is refused with a fixed message that never echoes the value and notes that the stream may already exist (inspect it with `ssf stream list`); shorter secrets are checked as whole strings to avoid matching ordinary text; a coincidental property-name match can still refuse an already committed creation. A response is accepted only if it names the requested stream. `delete` refuses a response that names `authorization_header` anywhere and prints only `deleted` and `stream_id`, rebuilt from the fields it validated, never the rest of the response. The server CLI offers no subject-replacement command, so this client has none either (`PUT /api/ssf/admin/streams/{id}/subjects` stays an API route).

`backup` streams the server's `riauth.backup/v3` archive into a new private file beside `--out` (created exclusively, with mode 0600 on Unix) and publishes `--out` with a hard link only after the whole archive authenticates, so nothing is ever replaced and an existing `--out`, even a dangling symbolic link, is refused before any request. `--key-file` is a private file of 32 random bytes as base64url, as `riauth keygen` writes it; as for the server CLI, the key is sent to the server for the export, so use it only over HTTPS or loopback and treat the serving host as able to read it while a backup runs. The transfer is bounded by `--max-bytes` (32 bytes to 4 GiB, default 4 GiB) and by the quota the server reports in `x-riauth-backup-max-bytes`, whichever is lower; `--request-timeout` bounds each wait for data, not the whole transfer. The archive is authenticated with this client's own reader of the format: every frame is AES-256-GCM sealed under the key with associated data that binds it to the archive, position and kind, the records are strictly ordered, the trailer's record count, frame count and SHA-256 transcript match, nothing follows the trailer, and the archive's issuer record agrees with its header. Typed frame envelopes refuse unknown or repeated fields, and the embedded configuration refuses repeated keys recursively. Arbitrary record payloads remain JSON values; this is not a duplicate-key guarantee for every nested record object. A short, oversized, wrongly keyed, tampered, truncated or cancelled transfer attempts to remove the partial file and does not publish `--out`; cleanup is best effort and an unlink error can leave the private partial behind. SIGINT and SIGTERM (Ctrl-C, `kill`) cancel cooperatively and exit non-zero when observed; their handlers are installed before the partial file is created. After verification the client checks for a visible pending notification before the hard link. Notification delivery can lag or a signal can arrive after the last poll, so this does not guarantee atomic signal-versus-publication ordering; synchronous publication and cleanup are not interrupted by the cooperative checks. SIGHUP is deliberately not handled (that would need `unsafe` code, which this crate forbids, and would override an inherited "ignore"), so under `nohup` a closed terminal does not touch the backup, but without `nohup` closing the terminal ends the process by its default action and can leave a partial behind; run a long backup under `nohup`, a service manager or a terminal multiplexer. A signal this client does not handle (SIGHUP without `nohup`, SIGKILL, SIGQUIT), a crash or a power loss can likewise leave the partial: it is the private (0600) file `.riauth-backup-<uuid>.partial` in the directory of `--out`, and it is safe to delete once no backup is running. On a platform without Unix signals only Ctrl-C is handled, from its first poll. Owner-only file permission checks and creation mode are enforced on Unix; this port does not establish equivalent Windows ACL protection. `--out` must be a valid UTF-8 path, which is checked before any request. This client zeroizes the backup key it holds on a best-effort basis; the HTTP client's request body and serde's intermediate buffers are not zeroized. No archive byte or key is ever printed: standard output carries only `backup_file`, `api_version`, `created_at`, `encrypted`, `verified`, `issuer`, `stream_id`, `frames`, `records`, `bytes` and `transcript`. This client does not decide whether the archive's database schema is one a given server build can import; `riauth restore` does. To rehearse a backup, restore it into a scratch directory as [Encrypted backup and restore](../../docs/operations.md#encrypted-backup-and-restore) describes.

`session list` shows only the current human user's live sessions. `session revoke ID` uses `/api/sessions/{id}`; the server permits the owner, an administrator, or an agent with exact `session.revoke` authority. Revoking another session sends an `Idempotency-Key` to the shared session writer. Supply `--idempotency-key KEY` to replay the exact selected revocation while the caller remains live; otherwise the client creates a new key for this invocation. A different target with the same key fails. Revoking the current terminal session removes the saved session file and cannot replay after invalidating that credential, so that command rejects an explicit key. Session revocation has no configuration revision precondition and rejects `--if-revision`. The API does not expose a list of another user's session IDs; use `user revoke-sessions USER` to revoke that user's sessions through the conditional management route.

Shared-secret confidential and service client creation requires `--secret-file`; `private_key_jwt` applications do not receive a shared secret. Rotation always requires the file. The destination must be a new file in an existing directory. It is reserved with owner-only permissions before the mutation, and the full one-time response is written there if the server issues a shared secret. Standard output contains the client result and `credential_file` path without the secret. Keep that file private. Every private destination in this client (`--secret-file` and `--out`) is reserved before the request and removed again when the command fails normally, but SIGKILL, a power loss or Ctrl-C can leave the empty reserved file behind. Delete it before retrying at the same path, or use a new path. Supply your own `--idempotency-key` on the first attempt to identify an uncertain commit. An exact retry returns `409 credential_already_issued` without writing a new secret file. Inspect the client and rotate with a fresh key and current revision if the first private file was lost.

`plan` reads a bounded JSON manifest with `api_version: "riauth/v1"` and asks the server to compute an immutable plan. It creates the output file with owner-only permissions and refuses to overwrite an existing file. Review its changes, hash, base revision, and expiry before applying it. `apply` requires that private plan file, checks the exact stored plan and issuer with the server, and resolves only the secret references needed by its changes (`env:NAME` or owner-only `file:PATH`). The server checks the revision and commits atomically. Repeating `apply` returns the stored result for the same plan without rereading secrets. A plan that removes access (a user or client disable, a membership removal, a retired connector) is never applied implicitly: `apply` refuses it locally unless you pass `--confirm-removals PLAN_ID` with the plan's exact `plan_id`. A different id is refused before any request, and the id is sent in the `X-riAuth-Confirm-Removals` and `X-riAuth-Confirm-Cloud-Removals` headers only when you pass it. The server enforces the same rule independently. `export` reads `GET /api/state/export`, refuses a response that reports included secrets or contains a delivery authorization field, and writes only the manifest to a new owner-only file. When the server reports stored connector definitions (to a full human administrator, once a definition is stored or the operator has set `connector_secret_dir`), the command also prints them as `connectors`: `connector_secret_dir`, and per definition only `kind` (`ldap`, `workspace`, `entra` or `scim`), `id`, `revision`, `digest`, `loaded_in_this_process`, `loaded_revision` and `restart_required`, plus `loaded_digest` and `retired` on a definition retired while this process still runs it. Each is validated before it is copied: the id follows the server's name rule, a digest is the server's 43-character base64url form, a revision is an integer or null, the flags are booleans, and at most 1024 definitions are accepted. Any other field of the status or a definition is never printed, and a status outside that shape is refused with the fixed error "Export connector status is malformed" (nothing is echoed) before the manifest file is written. A stored definition takes effect only when the server process starts, so `restart_required: true` means the operator still has to restart. Set `--run-id` or `RIAUTH_RUN_ID` to attach an audit correlation value. None of these commands opens local server configuration or storage.

For scoped automation, `--agent-file` or `RIAUTH_AGENT_FILE` selects an owner-only JSON credential containing `issuer`, `agent_id`, `token`, and `expires_at` from the primary management issuer. An agent token never falls back to a human session and cannot run `login`, `logout`, or end-user passkey commands. Keep plan files, agent credentials, and secret files private. The server remains responsible for permissions, plan freshness, and applying the requested changes.

Terminal USB passkey login and enrollment are optional:

```sh
cargo install --locked --path crates/riauthctl --features terminal-usb
riauthctl --server https://id.example.com passkey login alice
riauthctl --server https://id.example.com passkey login alice --transaction-id "$TRANSACTION"
riauthctl --server https://id.example.com passkey enroll --name security-key
```

The base build rejects those commands locally before contacting the server. The USB feature requires a supported CTAP2 authenticator and its native USB prerequisites. USB commands need interactive touch/PIN input; `--non-interactive` fails locally. The optional transaction ID binds a passkey sign-in to a pending server authentication transaction. Local maintenance commands remain in the existing executable during the split.

Terminal authorization uses the same server decisions as the browser. First sign in with `riauthctl login` or `riauthctl passkey login`, then review and decide a pending browser request or device code:

```sh
riauthctl --server https://id.example.com request inspect ABCDE-FGHIJ
riauthctl --server https://id.example.com request approve ABCDE-FGHIJ --passkey
riauthctl --server https://id.example.com request deny ABCDE-FGHIJ
riauthctl --server https://id.example.com device inspect ABCDE-FGHIJ
riauthctl --server https://id.example.com device approve ABCDE-FGHIJ --passkey
riauthctl --server https://id.example.com device deny ABCDE-FGHIJ
riauthctl --server https://id.example.com authorize 'https://id.example.com/oauth/authorize?...' --passkey --callback-file callback.txt
```

Approval prints the server's application, client ID, scopes, and destination before prompting. `--yes` confirms after that review and is required with `--non-interactive`; declining the prompt leaves the request pending. `--password-stdin` or an interactive password prompt can replace `--passkey`; `--mfa` prompts for a one-time code, and `RIAUTH_OTP` supplies one without a prompt. A browser request carries its server-issued authentication transaction through fresh sign-in and decision, including passkey sign-in. Device approval always signs in again after review, keeps the same account, and relies on the server's fresh identity and client policy check. Denials do not require fresh sign-in. Agent credentials cannot decide end-user requests.

`authorize` accepts a complete URL on the selected issuer and returns the callback without following it. The callback may contain a one-time authorization code; use `--callback-file` to reserve a new owner-only file before making the decision and keep the code out of stdout. Terminal delivery does not support `form_post` response modes; use the original browser for those. An embedded upstream source stage also remains in the browser. The client never sends a session bearer to an application callback. The server continues to decide consent, authentication proof, policy, audit, and revocation behavior.

The credential outputs (`client create` and `client rotate-secret` with `--secret-file`, `agent create` and `agent rotate`, `registration create`, `windows-device enroll`, each with `--out`) and `export --out` name their file in the JSON they print, so the path must be valid UTF-8. A path that is not is refused with a fixed message ("Credential output path must be valid UTF-8" or "Export output path must be valid UTF-8", never the path) before any request, credential issuance or private reservation, rather than after a secret was issued and written.

`authorize --callback-file` is named in the printed result too, so it must be valid UTF-8: a path that is not is refused with a fixed message ("Callback output path must be valid UTF-8") before any request, reservation or private write, never after a one-time callback was written.
