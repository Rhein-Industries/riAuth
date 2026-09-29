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
riauthctl --server https://id.example.com client update dashboard --enabled false
riauthctl --server https://id.example.com client disable dashboard
riauthctl --server https://id.example.com client list --filter dash --limit 100
riauthctl --server https://id.example.com client create worker --service --secret-file worker-credential.json
riauthctl --server https://id.example.com revision
riauthctl --server https://id.example.com --if-revision REVISION --idempotency-key ROTATION_ID client rotate-secret worker --secret-file worker-new-credential.json
riauthctl --server https://id.example.com session revoke SESSION_ID
```

`user get`, `group get`, and `client get` use `/api/resources` with exact resource read permission. Plain `user list`, `group list`, and `client list` retain the existing array response from `/api/users`, `/api/groups`, and `/api/clients`. Adding `--filter`, `--limit`, or `--after` uses the bounded `/api/inventory` page response (`items`, `next_cursor`, `revision`); filters match case-sensitive name substrings. Continue a page with `--after` and the same filter. The server filters every read by the caller's authority. Group reads return member IDs as stored by the server. `group has-member GROUP USERNAME` checks the exact group and user resources, so it requires both `group.read` and `user.read`; it returns a `member` boolean only after both reads succeed.

Each direct user, group, or client mutation sends a quoted `If-Match` configuration revision and an `Idempotency-Key`. By default the client reads `/api/state/revision` and generates a fresh key. Use `--if-revision N` when the caller has a known revision or lacks `state.read` permission. Supply `--idempotency-key KEY` to retry the **same command and body** after an uncertain result; the server retains a matching receipt for 24 hours. `--run-id` adds audit correlation. `user disable` also requests session revocation through the shared user management writer; `user revoke-sessions` leaves the enabled state alone. `client disable` uses the shared client writer, including its dependent-grant revocation. Client creation uses the usual interactive scopes by default, or `api` for a service client. `--settings-file` supplies a complete ProviderSettings JSON object; on update it replaces the complete settings object. The server applies all client type and setting rules.

`session list` shows only the current human user's live sessions. `session revoke ID` uses `/api/sessions/{id}`; the server permits the owner, an administrator, or an agent with exact `session.revoke` authority. Revoking the current terminal session removes the saved session file. This endpoint has no revision precondition or idempotency receipt, so `session revoke` rejects `--if-revision` and `--idempotency-key`. The API does not expose a list of another user's session IDs; use `user revoke-sessions USER` to revoke that user's sessions through the conditional management route.

Shared-secret confidential and service client creation requires `--secret-file`; `private_key_jwt` applications do not receive a shared secret. Rotation always requires the file. The destination must be a new file in an existing directory. It is reserved with owner-only permissions before the mutation, and the full one-time response is written there if the server issues a shared secret. Standard output contains the client result and `credential_file` path without the secret. Keep that file private. Supply your own `--idempotency-key` on the first attempt when you need a recoverable retry: if the response is uncertain, repeat the same command with that key and a new `--secret-file` path so the server can return the stored receipt.

`plan` reads a bounded JSON manifest with `api_version: "riauth/v1"` and asks the server to compute an immutable plan. It creates the output file with owner-only permissions and refuses to overwrite an existing file. Review its changes, hash, base revision, and expiry before applying it. `apply` requires that private plan file, checks the exact stored plan and issuer with the server, and resolves only the secret references needed by its changes (`env:NAME` or owner-only `file:PATH`). The server checks the revision and commits atomically. Repeating `apply` returns the stored result for the same plan without rereading secrets. Set `--run-id` or `RIAUTH_RUN_ID` to attach an audit correlation value. Neither command opens local server configuration or storage.

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
