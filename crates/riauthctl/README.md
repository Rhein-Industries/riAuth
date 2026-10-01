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

`session list` shows only the current human user's live sessions. `session revoke ID` uses `/api/sessions/{id}`; the server permits the owner, an administrator, or an agent with exact `session.revoke` authority. Revoking another session sends an `Idempotency-Key` to the shared session writer. Supply `--idempotency-key KEY` to replay the exact selected revocation while the caller remains live; otherwise the client creates a new key for this invocation. A different target with the same key fails. Revoking the current terminal session removes the saved session file and cannot replay after invalidating that credential, so that command rejects an explicit key. Session revocation has no configuration revision precondition and rejects `--if-revision`. The API does not expose a list of another user's session IDs; use `user revoke-sessions USER` to revoke that user's sessions through the conditional management route.

Shared-secret confidential and service client creation requires `--secret-file`; `private_key_jwt` applications do not receive a shared secret. Rotation always requires the file. The destination must be a new file in an existing directory. It is reserved with owner-only permissions before the mutation, and the full one-time response is written there if the server issues a shared secret. Standard output contains the client result and `credential_file` path without the secret. Keep that file private. Every private destination in this client (`--secret-file` and `--out`) is reserved before the request and removed again when the command fails normally, but SIGKILL, a power loss or Ctrl-C can leave the empty reserved file behind. Delete it before retrying at the same path, or use a new path. Supply your own `--idempotency-key` on the first attempt to identify an uncertain commit. An exact retry returns `409 credential_already_issued` without writing a new secret file. Inspect the client and rotate with a fresh key and current revision if the first private file was lost.

`plan` reads a bounded JSON manifest with `api_version: "riauth/v1"` and asks the server to compute an immutable plan. It creates the output file with owner-only permissions and refuses to overwrite an existing file. Review its changes, hash, base revision, and expiry before applying it. `apply` requires that private plan file, checks the exact stored plan and issuer with the server, and resolves only the secret references needed by its changes (`env:NAME` or owner-only `file:PATH`). The server checks the revision and commits atomically. Repeating `apply` returns the stored result for the same plan without rereading secrets. `export` reads `GET /api/state/export`, refuses a response that reports included secrets or contains a delivery authorization field, and writes only the manifest to a new owner-only file. Set `--run-id` or `RIAUTH_RUN_ID` to attach an audit correlation value. None of these commands opens local server configuration or storage.

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
