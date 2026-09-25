<img src="assets/riauth-mark.svg" alt="riAuth logo" width="88">

# riAuth

**Your identity layer, on your infrastructure.**

riAuth brings sign-in, single sign-on, application access, and identity administration into one self-hosted service. People get a familiar place to find their apps and sign in. Administrators get standards-based connections, clear access policies, and a way to preview desired-state changes before applying them.

**v0.1.0 · [MIT licensed](LICENSE)**

## What you can do with riAuth

| Goal | How riAuth helps |
| --- | --- |
| **Give people a better sign-in experience** | Offer browser passkeys, passwords with optional authenticator codes, recovery codes, and terminal approval. The application portal shows each person the apps they can access, with search, categories, and favorites. |
| **Connect applications and directories** | Use OpenID Connect, SAML, LDAP, SCIM, RADIUS, and forward authentication. Federate with upstream identity sources and apply group, application, and assurance policies across sign-in paths. |
| **Make access changes with confidence** | Manage users, clients, and policy through a remote CLI. Grant automation agents scoped, expiring permissions; preview desired-state plans and apply them with revision checks and audit records. |
| **Run it your way** | Start with one service and embedded storage, or use external database storage for multiple service processes. Health probes, metrics, native HTTPS, and encrypted backup and restore support day-to-day operations. |

Explore the [documentation](docs/README.md) for configuration and protocol details.

## Try it locally

Install the build prerequisites in the [getting-started guide](docs/getting-started.md), then:

```sh
git clone https://github.com/Rhein-Industries/riAuth.git
cd riAuth
cargo install --locked --path .
riauth init
riauth serve
```

`riauth init` prompts for the first administrator password. Open **[http://localhost:9000/apps](http://localhost:9000/apps)** to sign in and explore the application portal. In another terminal, from the same directory, try:

```sh
riauth login admin
riauth doctor
riauth --help
```

The local instance listens on loopback. The [first-application walkthrough](docs/getting-started.md) shows how to register an OpenID Connect application and test its sign-in.

## Connect an application

Register a confidential web client with your application's actual callback URL:

```sh
mkdir -p deployment-private
riauth client create reports --name 'Reports' --confidential \
  --redirect-uri https://reports.example.com/oauth/callback \
  --scope openid,profile \
  --output-file deployment-private/reports-credential.json
```

The output file holds the client secret; `deployment-private/` is excluded from version control. Configure the application with the issuer's `/.well-known/openid-configuration` document and the saved client credentials. The [OpenID Connect guide](docs/oidc-profiles.md) covers the authorization flow, while the [portal guide](docs/PORTAL.md) shows how to add an application launcher.

## Choose your next step

- **Create a polished app experience:** Configure the [application portal](docs/PORTAL.md), [passkeys](docs/passkeys.md), and [account lifecycle](docs/lifecycle.md).
- **Connect your environment:** Follow the [SAML](docs/saml.md), [directory and provisioning](docs/README.md#interfaces-and-identity), or [proxy sign-in](docs/proxy.md) guides.
- **Automate administration:** Use [scoped agents and reviewed plans](docs/agent.md) or the [HTTP API](docs/api.md).
- **Bring over existing identities:** Follow the [import and migration guide](docs/migration.md) for a staged cutover.
- **Prepare a reachable deployment:** Set an HTTPS issuer and use the [operations guide](docs/operations.md) for keys, backups, monitoring, and recovery. The [release notes](docs/release-notes.md) describe this version.

## Contribute

The [contributing guide](CONTRIBUTING.md) covers the development workflow and checks. To run the core checks locally:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --features test-support,fuzzing --locked -- -D warnings
cargo test --all-targets --features test-support,fuzzing --locked
python3 scripts/check-repo-hygiene.py
python3 scripts/check-docs.py
```

## Security and license

Report vulnerabilities privately through the [security policy](SECURITY.md). riAuth is released under the [MIT License](LICENSE); dependency terms and source links are in the [third-party notices](THIRD_PARTY_NOTICES.md).
