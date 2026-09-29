# Platform forward-auth recipe

One Platform integration: browser SSO in front of an application, using the repository templates for nginx `auth_request` and Traefik forwardAuth. The application receives identity headers. Login is an authorization-code flow with S256 PKCE, redeemed inside riAuth. This page is the configuration and the results the fixtures assert. It is not a record that those fixtures ran on this revision, and it is not a compatibility result for a deployed nginx or Traefik.

The configuration bodies are [deploy/nginx-forward-auth.conf](../../deploy/nginx-forward-auth.conf) and [deploy/traefik-forward-auth.yml](../../deploy/traefik-forward-auth.yml). Behavior, header names, and operator notes are in [proxy.md](../proxy.md). Routes are registered only in a Platform build ([src/api.rs](../../src/api.rs) `platform_routes`).

## Three different claims

| Claim | What it means here |
| --- | --- |
| CI script | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) is written to run the two ignored tests below on the `ubuntu-24.04` integration job. That text is not a result for this revision. |
| This revision | Neither ignored test was executed while this page was written. No Cargo command was run. |
| Deployment peer | The fixtures are HTTP loopback processes. They do not install a production vhost, a customer nginx or Traefik build, TLS, a load balancer, or a real application. |

## Clients the fixtures create

Both ignored tests call `Core::create_client` in process. They do not run `riauth client create`. Each client is public (`confidential: false`, `service: false`), has an empty `allowed_groups`, `require_mfa: false`, and `implicit_consent` left at its default `false`. `settings.proxy.domain` is unset. `session_ttl` is 3600. Scopes are `openid`, `profile`, `email`, and `groups`. The callback is exactly `{external_origin}/outpost/{client_id}/callback`.

| Fixture | Client id | Name | `external_origin` | `trusted_proxies` |
| --- | --- | --- | --- | --- |
| nginx, [tests/outpost.rs](../../tests/outpost.rs) `nginx_forward_auth_terminal_sso_headers_and_revocation` | `reports` | Protected reports | `http://127.0.0.1:{listen port}` | `127.0.0.1` |
| Traefik, [tests/outpost_traefik.rs](../../tests/outpost_traefik.rs) `traefik_forward_auth_real` | `dashboard` | Dashboard | `http://localhost:{port}` | `127.0.0.1` |

The example in [proxy.md](../proxy.md) is a different client: `https://reports.example.test`, group `staff`, `--require-mfa`, and `implicit_consent: true`. Use that shape for an operator install. The statuses below are what the loopback fixtures assert, not what that example was run against.

`POST /api/clients` with the same JSON, or `riauth client create`, is the supported way to create the client. An agent needs `client.write` on `client/{client_id}` ([src/core.rs](../../src/core.rs) `create_client`). Installing nginx or Traefik, and editing their files, is an infrastructure action. riAuth does not do it. `trusted_proxies` is server configuration: at most 64 exact IPs, no CIDR ([proxy.md](../proxy.md)).

An Essentials configuration that sets `proxy_listeners`, or a `forward_auth` or `outpost_start` rate-limit override, fails validation with `{field} requires the Platform build` ([src/edition.rs](../../src/edition.rs)).

## nginx configuration the ignored test renders

`nginx_forward_auth_terminal_sso_headers_and_revocation` is `#[ignore = "Set RIAUTH_TEST_NGINX, optionally RIAUTH_TEST_BROWSER"]`. It reads the deploy template and replaces only these markers:

| Marker | Fixture value |
| --- | --- |
| `{{RUNTIME}}` | empty temp directory (pid, error log, and temp paths live here) |
| `{{LISTEN}}` | `127.0.0.1:{port}` |
| `{{SERVER_NAME}}` | `127.0.0.1` |
| `{{ISSUER}}` | loopback riAuth origin, no trailing slash |
| `{{CLIENT_ID}}` | `reports` |
| `{{ORIGIN}}` | the same `http://127.0.0.1:{port}` as the client's `external_origin` |
| `{{UPSTREAM}}` | loopback application |

It starts `nginx -p {RUNTIME} -c {RUNTIME}/nginx.conf`. The template already sets `daemon off` and `master_process off`. It does not add TLS. `location = /_riauth_auth` is internal and calls `{ISSUER}/outpost/reports/auth` with the body off and `X-Original-URL {ORIGIN}$request_uri`. `location /outpost/reports/` is proxied to riAuth without `auth_request`. `location /` runs the provenance maps, then `auth_request`. A 401 from the subrequest becomes `@riauth_login`: methods other than GET and HEAD return 401; GET and HEAD return 302 to `$riauth_login`.

The same `exercise` function runs the embedded Rust listener when nginx is absent (`rust_reverse_proxy_terminal_sso_headers_websocket_and_revocation`, not ignored). Assertions that sit in `if builtin` belong to that listener. They are listed under [What this recipe does not use](#what-this-recipe-does-not-use) and are not nginx results.

## Traefik configuration the ignored test renders

`traefik_forward_auth_real` is `#[ignore = "Set RIAUTH_TEST_TRAEFIK"]`. CI is written to download Traefik **v3.7.13** `linux/amd64` and require sha256 `52cd039a34258dd61c617a95d69252bc6bcae27c520f338186c31c7fef8f6394` before that test. This page did not download that binary or check the hash.

The dynamic file is the deploy template with only these replacements: `{{CLIENT_ID}}` = `dashboard`, `{{RIAUTH_URL}}` = the loopback riAuth origin, `{{APP_HOST}}` = `localhost`, `{{ENTRYPOINT}}` = `web`, `{{APP_UPSTREAM}}` = the loopback application. `trustForwardHeader` stays `false`. No `tls: {}` is added. The test also writes this static file, which the dynamic template cannot set:

```yaml
global: { checkNewVersion: false, sendAnonymousUsage: false }
log: { level: ERROR }
entryPoints:
  web:
    address: "127.0.0.1:{port}"
    http:
      aliasHeadersStrategy: delete
providers:
  file:
    directory: "{temp}/dynamic"
```

`delete` is the fixture's choice. [proxy.md](../proxy.md) also accepts `reject`. The default `keep` is documented as unsafe. The test starts `traefik --configFile={temp}/traefik.yml`.

The issuer router is a second dynamic file, matching the rule in proxy.md, on HTTP and without TLS:

```yaml
http:
  routers:
    issuer:
      rule: "Host(`127.0.0.1`) && !PathRegexp(`^/outpost/[^/]+/(auth|traefik)$`)"
      entryPoints: ["web"]
      service: issuer
  services:
    issuer: { loadBalancer: { servers: [ { url: "{riAuth origin}" } ] } }
```

`{{RIAUTH_URL}}` in the template is the internal address. The fixture uses loopback and does not send that address back through Traefik.

## Permissions

| Action | Who | Fixture |
| --- | --- | --- |
| Create or update the proxy client | Administrator session, or an agent with `client.write` on `client/{id}`. Manifest plan/apply uses the same permission. | In-process `create_client` as the initial administrator. |
| Approve the browser login | The signed-in end user. `riauth request approve` refuses agent credentials (`Agent credentials cannot authenticate or consent as an end user`). | nginx: the `riauth` binary. Traefik binary test: `Core::authorize` in process, not the CLI. |
| Deploy the proxy | Whoever manages that host. Not a riAuth permission. | The test process starts the binary. |
| Revoke the parent session | That session, via `POST /api/logout`, or an allowed caller of `DELETE /api/sessions/{id}`. | `Core::logout` on the session token. Neither binary test shells out to `riauth logout` or `riauth session revoke`. |

The proxy client cannot be confidential, and it cannot require PAR, JAR, or DPoP ([proxy.md](../proxy.md)). Codes for a proxy client are refused at the token endpoint. The in-process test `proxy_client_codes_cannot_be_redeemed_at_the_token_endpoint` in [tests/signin_core.rs](../../tests/signin_core.rs) asserts `400 unauthorized_client` for `authorization_code` and `refresh_token`. That test was not run here, and it does not start nginx or Traefik.

Cookie names come from [src/outpost.rs](../../src/outpost.rs) `cookie_name`: `riauth_bind_` or `riauth_proxy_`, plus the first 16 characters of the unpadded base64url SHA-256 of the client id. HTTPS adds the `__Host-` prefix and `Secure`. Both values are opaque (`ri_proxy_binding_…` and `ri_proxy_…` in [src/assembly/outpost.rs](../../src/assembly/outpost.rs)). The fixtures use HTTP origins, so they do not exercise `__Host-` or `Secure`.

## Redirect, session, and revocation results

Every status in the two tables is an assertion in an ignored test. Writing this page did not produce that status.

### nginx fixture

The application origin is `http://127.0.0.1:{port}`. The target is `{origin}/reports?one=1&two=2`. The user is `admin`.

| Step | Asserted result |
| --- | --- |
| GET the target with `x-authentik-username: root` and no session | 302. The test follows that `Location` and does not assert its text. |
| GET the start `Location` | 302, with a `Set-Cookie` containing `ri_proxy_` or `ri_proxy_binding_`. |
| GET the authorization `Location` with `Accept: text/html` | 303 and a `riauth_return` cookie. |
| GET the interaction URL as JSON with that cookie | Body contains `user_code`. |
| `riauth --server {issuer} --session-file {file} --json request approve {user_code} --yes`, with `RIAUTH_AGENT_FILE` and `RIAUTH_RUN_ID` removed | Process succeeds. |
| GET the stored callback with the binding cookie | 302. `Location` is exactly the original target. |
| GET the target with the proxy cookie, `app-session=fixture`, and planted identity, `Authorization`, `Forwarded`, and forwarding headers | Upstream JSON: `username` `admin`, `authorization` null, `cookie` `app-session=fixture`, `uri` `/reports?one=1&two=2`. |
| POST the target with the proxy cookie and no provenance | 403. The application channel stays empty. |
| POST with the proxy cookie and one `X-Riauth-Request-Intent: api` | 200. Upstream `username` is `admin` and `intent` is null. |
| POST with the proxy cookie and (`http://evil.example.test`, `same-site`) or (the application origin, `same-site`) | 403. The application channel stays empty. |
| POST with the proxy cookie, `Origin` equal to the application origin, and `Sec-Fetch-Site: same-origin` | 200. Upstream `username` is `admin`. |
| POST with `X-Riauth-Request-Intent: api` and no proxy cookie | 401. |
| POST `{origin}/outpost/reports/logout` with the proxy cookie and either of the mismatched origin/site pairs above | 403. The next GET of the target with that cookie is still 200. |
| POST logout with `Origin: https://attacker.test` and no `Sec-Fetch-Site` | 403. The test does not issue a following GET for this case. |
| POST logout with `Origin` equal to the application origin and `Sec-Fetch-Site: same-origin` | 200. The next GET of the target with that cookie is 302. |

If `RIAUTH_TEST_BROWSER` is set, the test then starts that browser headless on the target, approves the new `user_code` with the same `riauth request approve` command, and expects upstream `username` `admin`, the same `uri`, and `cookie` null. CI is written to set the variable to `command -v riauth-chrome`. This page did not set it. After that optional path, `Core::logout` on the original administrator session token makes the next GET 302. Without the browser variable, that logout runs only after the application cookie is already dead, so the nginx fixture's parent-session check is the 302 above from the outpost logout.

### Traefik v3.7.13 fixture

The application origin is `http://localhost:{port}`. The target is `{origin}/ui/?one=1&two=2`. Approval is user `alice`, in process.

| Step | Asserted result |
| --- | --- |
| GET the target with `Sec-Fetch-Mode: navigate` and `X-Original-URL: http://attacker.test/` | 302. `Location` is exactly `{origin}/outpost/dashboard/start?rd=` plus the target. The planted URL is not the return target. |
| GET `{origin}/v1/jobs` with `Sec-Fetch-Mode: cors` | 401. `X-Riauth-Login` starts with `{origin}/outpost/dashboard/start?rd=`. JSON `error` is `invalid_token`. |
| GET the start URL | 302. A `Set-Cookie` contains `riauth_bind_` and is not `Max-Age=0`. |
| `Core::authorize(&alice)` with `decision` `approve` | Callback starts with `{origin}/outpost/dashboard/callback?`. |
| GET that callback with the binding cookie | 302. `Location` is exactly the target. A `Set-Cookie` contains `riauth_proxy_` and is not `Max-Age=0`. |
| GET the target with that cookie, `app-session=fixture`, planted `riauth_sso` and `riauth_bind_` cookies, planted identity headers, dotted/bang/underscore aliases, `Authorization: Bearer attacker`, and a planted `X-Original-URL` | Upstream `username` and `auth_user` are `alice`. Alias fields are null. `authorization` is null. `cookie` is `app-session=fixture`. `uri` is `/ui/?one=1&two=2`. |
| GET with only the proxy cookie and a planted `riauth_sso` cookie | `username` is `alice`. `cookie` is null. |
| POST the target with the proxy cookie and `Origin: https://attacker.test` | 403. |
| WebSocket to `ws://localhost:{port}/ws` with the proxy cookie and no `Origin`, or `Origin: https://attacker.test` | The handshake fails. |
| WebSocket with `Origin` equal to the application origin | The server echoes `protected echo`. |
| GET `{http://127.0.0.1:{port}}/outpost/dashboard/traefik` and `.../auth` with the proxy cookie | 404 for each. |
| `Core::logout(&alice)`, then GET the target with the proxy cookie | 302. |

The open WebSocket is not read again after logout. The test does not assert that the socket closes.

## Negative and failure checks

Binary-fixture failures are in the tables above: planted identity does not become the upstream user, a missing or cross-site provenance does not reach the nginx application, a rejected logout does not clear the nginx proxy cookie, a Traefik background call is 401 rather than a redirect, and the issuer host does not expose `/outpost/{client}/auth` or `/traefik`.

These checks live in the same Traefik file and call the router in process. They do not start a Traefik binary. They were not run while this page was written.

| Test | Asserted failure |
| --- | --- |
| `traefik_endpoint_requires_trusted_peer` | Peer `192.0.2.10` or `::1` is 403 `access_denied`, with no `X-Authentik-Username` and, when unsigned-in, no `Location`. Peer `127.0.0.1` with the proxy cookie is 200. |
| `traefik_endpoint_validates_forwarded_headers_and_ignores_original_url` | A bad, missing, or duplicate `X-Forwarded-Proto`, `X-Forwarded-Host`, or `X-Forwarded-Uri` is 400 with no username. A planted `X-Original-URL` does not change the 302 `Location`. |
| `unauthenticated_background_and_unsafe_requests_get_401` | `cors`, `no-cors`, a duplicate `navigate`, OPTIONS, and unauthenticated unsafe methods are 401 with `X-Riauth-Login`, JSON `invalid_token`, and no `Location` or `Set-Cookie`. |
| `cross_site_unsafe_methods_are_refused_before_authentication` | Cross-site POST, DELETE, PATCH, and PUT are 403 before a username is issued, with or without the proxy cookie. Duplicate `Origin` or `Sec-Fetch-Site` is 400. |
| `unsafe_writes_need_browser_provenance_or_explicit_api_intent` | POST with no provenance, or with intent `browser` or `api, api`, is 403. Two `api` intent headers are 400. One `api` intent with the proxy cookie is 200 and does not echo the intent header. `api` plus an attacker `Origin` is 403. |
| `policy_denial_returns_html_for_documents` | After Alice loses required group `dashboard-operators`, a document navigation is 403 HTML containing `You don't have access to this application.` and a link to `{issuer}/apps`, with `default-src 'none'` and no username. `Sec-Fetch-Dest: empty` or `Sec-Fetch-Mode: cors` is 403 JSON `access_denied`. Restoring the group returns 200. The binary Traefik client has no required group, so the binary test does not assert this page. |
| `nginx_auth_endpoint_output_is_unchanged` | `GET /outpost/dashboard/auth` with a trusted peer and `X-Original-URL` of the application returns 200, `X-Authentik-Username: alice`, and `X-Riauth-App-Cookie: app-session=abc`. An attacker `X-Original-URL` is 400. Peer `192.0.2.10` is 403. No cookie yields 401 `invalid_token` and `X-Riauth-Login`, with no `Location`. |
| `revoked_parent_session_is_unauthorized_next_time` | After `Core::logout`, navigation is 302 to the absolute start URL, and `Sec-Fetch-Mode: cors` is 401 with no username. |
| `forward_auth_admission_is_separate_from_workers` | Sixteen busy forward-auth permits make another forward-auth check wait. This is the in-process queue, not the Traefik process. |

`forward_auth` is 6000 requests per minute per client address, counted in memory on each node. `outpost_start` is 30 per minute. Both are in [docs/api.md](../api.md). The binary fixtures do not fill those buckets. A full admission queue is documented as 503 `temporarily_unavailable` after two seconds ([docs/operations.md](../operations.md)).

## Closest API and CLI

There is no forward-auth subcommand.

| Fixture step | Supported equivalent |
| --- | --- |
| `Core::create_client` | `riauth client create`, or `POST /api/clients`. The proxy.md example is the operator form; the fixture fields are the table above. Agents use `client.write` and manifest plan/apply. |
| nginx `riauth request approve {user_code} --yes` | `GET /api/authorization/{code}`, then `POST /api/authorization/decision` with `approve: true` ([src/cli.rs](../../src/cli.rs)). |
| Traefik `Core::authorize` | The same `riauth request approve`. The binary test does not execute that command. |
| `POST /outpost/{client}/logout` | That route. It revokes the application cookie. The nginx fixture asserts 200, then 302 on the next application GET. |
| `Core::logout` | `riauth logout`, which is `POST /api/logout` for the saved session file. `riauth session revoke ID` is `DELETE /api/sessions/{id}` for one listed session ([docs/lifecycle.md](../lifecycle.md)). |
| Bearer check | `GET /api/proxy/auth?client_id=` validates an OAuth access token. It is not the browser outpost ([docs/api.md](../api.md)). |

## What this recipe does not use

The embedded listener test in `tests/outpost.rs` shares `exercise` and adds checks the nginx process does not run: alias headers rejected with 400, rewritten `X-Forwarded-*` values, a 413 body over 4096 bytes, host `attacker.test` returning 404, WebSocket protocol `test` echoing `protected echo`, and the open socket closing within 35 seconds after logout. [proxy.md](../proxy.md) documents that 30-second recheck for the Rust listener only. The nginx template sets `Upgrade` and `Connection`, and the nginx ignored test does not open a socket.

The Traefik dashboard variant in proxy.md (`service: api@internal`, client name not `traefik`) is not `traefik_forward_auth_real`. `trustForwardHeader: true` and a load-balancer address list are not the binary fixture; the template hard-codes `trustForwardHeader: false`. The in-process header test does accept a single `X-Forwarded-Proto: wss` as success, which is not a load balancer running in front of Traefik.

The nginx template replaces the identity headers, `Cookie`, `Authorization`, and `X-Riauth-Request-Intent` that it names. The nginx ignored test asserts the username, the cleared authorization, the filtered cookie, and the cleared intent. It does not assert that `Forwarded`, `X-Forwarded-Host`, `X-Forwarded-Proto`, `X-Forwarded-Uri`, `X-Original-Host`, `X-Real-IP`, or `Remote-User` are absent upstream. Those names are not cleared in the template.

No customer nginx package, no Traefik version other than the CI pin, no TLS vhost, and no protected application were run. The CI nginx package is the distro `nginx` package, version not pinned. The Traefik hash is workflow text until that job runs on a commit.

## Other D03 recipes

Still without a recipe page: outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [LDAP provider](platform-ldap-provider.md), the [OIDC relying party](oidc-relying-party.md), the [SAML IdP](platform-saml-idp.md), the [LDAP import](ldap-import.md), the [upstream OIDC recipe](upstream-oidc.md), the [inbound SCIM recipe](platform-inbound-scim.md), and the [SAML source recipe](platform-saml-source.md) are separate. The SAML source IdP is the in-process `Upstream` helper, and no named external IdP is connected. The inbound SCIM client is in-process `oneshot`, and no named SCIM client is connected. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. The import page follows disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. The relying-party client is the in-tree axum fixture, not a named application. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
