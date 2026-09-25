# Get started locally

This guide starts one riAuth instance on your computer, signs in as its first administrator, and registers an OpenID Connect (OIDC) web application. The default HTTP listener is bound to loopback for local evaluation.

## Prerequisites

Install the Rust **1.93** toolchain, a C/C++ compiler, and CMake. Linux builds also need `pkg-config`, OpenSSL development headers, and udev development headers. See the [project README](../README.md) for the release scope and [operations](operations.md) for deployment options.

## 1. Install and initialize

```sh
git clone https://github.com/Rhein-Industries/riAuth.git
cd riAuth
cargo install --locked --path .
riauth init
```

`riauth init` prompts for the first administrator password and creates `riauth.toml` plus the `data/` directory in the current directory. The default username is `admin`, the issuer is `http://localhost:9000`, and the listener is `127.0.0.1:9000`. The generated files are ignored by Git. Run the remaining commands from this directory so the CLI finds `riauth.toml`.

Start the service and leave this terminal open:

```sh
riauth serve
```

## 2. Sign in

Open [http://localhost:9000/apps](http://localhost:9000/apps) and sign in with `admin` and the password you just set. The portal may be empty until you register an application.

In another terminal, from the same directory:

```sh
riauth login admin
riauth doctor
riauth discovery
```

`login` saves a private CLI session. `doctor` checks the connected instance; `discovery` prints the OIDC metadata that applications use. If the second terminal is elsewhere, pass `--config /path/to/riAuth/riauth.toml` to the CLI.

## 3. Register a web application

Use the callback URL your application actually serves. This local example assumes an OIDC-capable web application at `http://localhost:3000` with a callback at `/callback`:

```sh
mkdir -p deployment-private
riauth client create local-demo --name 'Local demo' --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile \
  --output-file deployment-private/local-demo-client.json
```

The response file contains the client secret. Keep it private; `deployment-private/` is ignored by this repository. Configure the application with:

| Application setting | Value |
| --- | --- |
| Issuer / discovery | `http://localhost:9000` / [`/.well-known/openid-configuration`](http://localhost:9000/.well-known/openid-configuration) |
| Client ID and secret | `local-demo` and `client_secret` from the private response file |
| Redirect URI | `http://localhost:3000/callback`, exactly as registered |
| Scopes and flow | `openid profile`; authorization code with S256 PKCE, `state`, and `nonce` |

Start the application and use its sign-in action. riAuth presents the browser sign-in and consent pages, then redirects to the registered callback. Registration alone does not start the example application. Without an application launch URL, the portal can show this client as **Setup pending**; see [Configure an application](PORTAL.md#configure-an-application) to add a launcher.

## Next steps

- **Connect an application:** [OIDC profiles](oidc-profiles.md), [SAML](saml.md), [proxy SSO](proxy.md), and the [HTTP API](api.md).
- **Add users and factors:** [My applications portal](PORTAL.md), [passkeys](passkeys.md), and [account lifecycle](lifecycle.md).
- **Operate a deployment:** [TLS, backup, restore, and probes](operations.md), [availability](availability.md), and [deployment testing](testing.md).
- **Migrate an existing estate:** [Authentik migration](migration.md) and [release limitations](limitations.md).

The loopback HTTP default is only for local use. For a reachable deployment, choose the intended HTTPS issuer before initialization, configure native TLS or a trusted TLS proxy, protect signing and backup keys, and rehearse recovery as described in [operations](operations.md). Review the [release limitations](limitations.md) before a production cutover.
