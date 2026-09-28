# riauthctl remote client

`riauthctl` is a standalone HTTP client. Its Cargo workspace has no dependency on the `riauth` server, local configuration, or storage crates. It provides a bounded set of remote commands while the existing `riauth` executable retains broader administration during the split.

```sh
cargo install --locked --path crates/riauthctl
riauthctl --server https://id.example.com status
riauthctl --server https://id.example.com discovery
riauthctl --server https://id.example.com login alice
riauthctl --server https://id.example.com whoami
riauthctl --server https://id.example.com revision
riauthctl --server https://id.example.com inventory users --limit 100
riauthctl --server https://id.example.com plan --file manifest.json --out plan.json
riauthctl --server https://id.example.com apply --plan plan.json
riauthctl --server https://id.example.com logout
```

Set `RIAUTH_SERVER` instead of repeating `--server`. An issuer is always required; the client never reads `riauth.toml` or chooses a local default. The URL must be canonical HTTPS, except that HTTP is accepted for loopback hosts. The client checks the exact discovery issuer and requires its `token_endpoint` to equal `--server` plus `/oauth/token` before sending a password or saved bearer token. A provider-specific issuer may have valid discovery but no management API; use the primary management issuer reported in the error. The client never switches credential destinations automatically. It rejects redirects and caps request time and response size. Requests connect directly and ignore proxy environment variables. Use `--ca-cert private-ca.pem` for a private HTTPS trust root and `--request-timeout SECONDS` to change the per-request deadline.

`login --password-stdin` reads one line from standard input for scripts; interactive login prompts without echo. `RIAUTH_OTP` supplies an optional one-time code; `login --mfa` prompts for one when the variable is unset. Passwords, session tokens, and server error descriptions are not printed. The saved session is bound to the exact issuer and written atomically to an owner-only file. By default it lives at `$XDG_CONFIG_HOME/riauthctl/session.json` or `~/.config/riauthctl/session.json`; use `--session-file` or `RIAUTH_SESSION_FILE` to select another path. Treat that file as a credential.

Inventory accepts `users`, `groups`, `clients`, `sources`, or `audit`, with one page of 1–1000 items per request. Use `--after` for a returned cursor and `--filter` where the server supports it. `--json` emits the versioned CLI envelope.

`plan` reads a bounded JSON manifest with `api_version: "riauth/v1"` and asks the server to compute an immutable plan. It creates the output file with owner-only permissions and refuses to overwrite an existing file. Review its changes, hash, base revision, and expiry before applying it. `apply` requires that private plan file, checks the exact stored plan and issuer with the server, and resolves only the secret references needed by its changes (`env:NAME` or owner-only `file:PATH`). The server checks the revision and commits atomically. Repeating `apply` returns the stored result for the same plan without rereading secrets. Set `--run-id` or `RIAUTH_RUN_ID` to attach an audit correlation value. Neither command opens local server configuration or storage.

For scoped automation, `--agent-file` or `RIAUTH_AGENT_FILE` selects an owner-only JSON credential containing `issuer`, `agent_id`, `token`, and `expires_at` from the primary management issuer. An agent token never falls back to a human session and cannot run `login`, `logout`, or end-user passkey commands. Keep plan files, agent credentials, and secret files private. The server remains responsible for permissions, plan freshness, and applying the requested changes.

Terminal USB passkey login and enrollment are optional:

```sh
cargo install --locked --path crates/riauthctl --features terminal-usb
riauthctl --server https://id.example.com passkey login alice
riauthctl --server https://id.example.com passkey enroll --name security-key
```

The base build rejects those commands locally before contacting the server. The USB feature requires a supported CTAP2 authenticator and its native USB prerequisites. Other mutable administration and local maintenance commands remain in the existing executables during the split.
