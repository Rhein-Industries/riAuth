# Browser SSO for forward-auth proxies

riAuth's embedded outpost supplies browser SSO to nginx `auth_request` and to [Traefik forwardAuth](#traefik-forwardauth). The proxy handles application HTTP and WebSocket traffic; riAuth authorizes each request. The application receives identity headers. Users sign in on riAuth's sign-in page (passkey, or password plus an optional code) or approve from the terminal, using the same OIDC policies, factors and SSO session as other applications.

Create a public web client with `riauth client create` and a settings file. Run the example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory:

```sh
mkdir -p deployment-private
cat > deployment-private/reports-settings.json <<'EOF'
{"proxy": {"external_origin": "https://reports.example.test", "session_ttl": 3600}, "implicit_consent": true}
EOF
riauth client create reports --name 'Protected reports' \
  --redirect-uri https://reports.example.test/outpost/reports/callback \
  --scope openid,profile,email,groups --group staff --require-mfa \
  --settings-file deployment-private/reports-settings.json
```

`implicit_consent` skips the consent screen for this first-party application; omit it to show consent once. The same client as a `POST /api/clients` body or agent manifest entry:

```json
{
  "client_id": "reports",
  "name": "Protected reports",
  "confidential": false,
  "service": false,
  "redirect_uris": ["https://reports.example.test/outpost/reports/callback"],
  "scopes": ["openid", "profile", "email", "groups"],
  "allowed_groups": ["staff"],
  "require_mfa": true,
  "settings": {
    "proxy": {"external_origin": "https://reports.example.test", "session_ttl": 3600},
    "implicit_consent": true
  }
}
```

Use a dedicated client for each protected application. Its callback must match exactly, and its policy controls access. The embedded outpost uses authorization code with S256 PKCE, state, nonce and issuer validation. Confidential clients and clients requiring PAR/JAR/DPoP are rejected for this embedded profile. Codes issued to a proxy client are redeemed only inside riAuth: `/oauth/token` refuses every grant for a proxy client with 400 `unauthorized_client`, so a code seen in a proxy access log cannot be exchanged. A proxy session is an independent opaque cookie bound to the originating client and parent user session, capped by both lifetimes. Tokens and PKCE verifiers never reach the upstream application. Disabling a client, changing its configuration, removing required group membership, expiry/revocation of a required temporary group grant, or revoking the parent session stops subsequent authorization checks. A configured device-trust requirement checks the originating user session's verification, capped by proof and session expiry; another session cannot inherit that trust. Existing application connections, including an already-upgraded WebSocket, require the proxy/application to close them; authorization checks apply to new requests/connections.

Configure the proxy's actual socket IP in `trusted_proxies` on every riAuth node. The outpost rejects other peers and duplicate cookies/forwarded URL headers. It ignores supplied identity headers and checks `X-Original-URL` against the configured external origin. Return URLs cannot target another origin or a reserved outpost path. Callbacks require the state and browser-binding cookie from the same login attempt. HTTPS cookies use `__Host-`, Secure, HttpOnly and SameSite=Lax; they carry no OAuth credentials. Authentication cookies are removed from `X-Riauth-App-Cookie`, which the integrating proxy forwards as the application's Cookie header.

The [nginx template](../deploy/nginx-forward-auth.conf) is exercised by the integration test. Replace every `{{...}}` marker, add the normal TLS/vhost directives, and review the resulting configuration. `ISSUER` includes any issuer path and omits a trailing slash; `ORIGIN` is the protected application's exact origin. `RUNTIME` is a private, writable nginx runtime directory. Set `UPSTREAM` to the application, reachable only through the trusted proxy. The template replaces all documented identity headers and removes Authorization before sending traffic upstream. Configure the application to trust only those headers from that proxy. It preserves application cookies while filtering riAuth cookies. It authorizes WebSocket upgrades and streams the resulting connection through nginx.

The protocol routes are:

| Route | Behavior |
| --- | --- |
| `GET /outpost/{client}/auth` | nginx: 200 with validated identity headers, 401 with an encoded `X-Riauth-Login` URL, or 403 for policy denial. Requires trusted-peer `X-Original-URL`. |
| `GET /outpost/{client}/traefik` | Traefik forwardAuth: 200 with identity headers and the filtered `Cookie`, 302 to sign-in for top-level navigation, 401 with `X-Riauth-Login` otherwise, or 403. Requires a trusted peer and validated `X-Forwarded-*`. Read-only. See [below](#traefik-forwardauth). |
| `GET /outpost/{client}/start?rd=...` | Sets a browser-binding cookie and redirects to OIDC authorization, which shows riAuth's sign-in page (or the terminal alternative). |
| `GET /outpost/{client}/callback` | Exchanges the bound code, creates an application cookie and returns to the original application URL. |
| `POST /outpost/{client}/logout` | Revokes that application cookie; requires an Origin matching the exact outward logout target and `Sec-Fetch-Site` absent, `same-origin` or `none`. CLI/central session revocation also works. |

Route the application's `/outpost/{client}/` prefix to riAuth without an authentication subrequest. Keep the nginx auth subrequest location internal. Use the `X-Riauth-Login` URL from the trusted response, including its encoding; concatenating a raw original query into `rd` loses parameters. The template redirects unauthenticated GET/HEAD requests and returns 401 for writes, so a login redirect does not silently convert a POST into a GET.

For shared-domain logout, the trusted proxy must overwrite `X-Original-URL` with the exact outward `/outpost/{client}/logout` URL. The nginx template does this. Without that header, riAuth accepts logout only from the provider's central `external_origin`.
The single-app Traefik template removes any client-supplied `X-Original-URL`, so logout uses its configured `external_origin`.

`/outpost/{client}/auth` and `/outpost/{client}/traefik` share the `forward_auth` rate bucket (6000 requests per minute per client address, always counted in memory, so the limit applies per riAuth node) and a separate 16-permit admission queue. `/outpost/{client}/start` has its own `outpost_start` bucket (30 per minute). See [operations](operations.md#rate-limits-and-admission).

Agents manage the settings through existing `client.write`, conditional updates and immutable manifest plan/apply. Deploying nginx or Traefik configuration remains an infrastructure action. riAuth does not install, update or orchestrate remote nginx or Traefik instances. The embedded reverse proxy and shared-domain profile below are available. No bypass-path rules or Basic credential injection are enabled by this profile.

Run `RIAUTH_TEST_NGINX=/path/to/nginx cargo test --test outpost -- --ignored`; set `RIAUTH_TEST_BROWSER=/path/to/chrome` to exercise a real browser as well. Tests use disposable loopback services and a private nginx configuration, with actual CLI approval, encoded return queries, header/cookie isolation, logout and policy checks. See [Authentik forward auth](https://docs.goauthentik.io/add-secure-apps/providers/proxy/forward_auth/) for the upstream integration model.


## Traefik forwardAuth

`GET /outpost/{client}/traefik` implements Traefik's forwardAuth contract. It was built against the Traefik v3.7.13 source (`pkg/middlewares/auth/forward.go`) and is exercised end to end with a real Traefik binary by `traefik_forward_auth_real`.

### Setup

1. **One client per application**, created as [above](#browser-sso-for-forward-auth-proxies), for example `reports` with `external_origin` `https://reports.example.test` and callback `https://reports.example.test/outpost/reports/callback`. Forward auth needs single-application mode: do not set `proxy.domain`.
2. **`trusted_proxies`** on every riAuth node must list every address Traefik connects from, as exact IPs. Run Traefik with host networking or a static address. Ranges (CIDR) are deliberately not supported: `trusted_proxies` also decides whose `X-Forwarded-For` and forwarded client-certificate headers riAuth believes, so a range would let any host in it assert both.
3. **Static configuration.** On every entrypoint that serves protected applications, set `http.aliasHeadersStrategy: delete` (the default is `keep`, and Traefik warns about it). It removes header-name aliases such as `X_Auth_User` that forwardAuth does not replace. If the Traefik dashboard is enabled, keep `api.insecure: false`.

   ```yaml
   entryPoints:
     websecure:
       address: ":443"
       http:
         aliasHeadersStrategy: delete
   api:
     dashboard: true
     insecure: false
   ```

4. **Dynamic configuration.** Copy [`deploy/traefik-forward-auth.yml`](../deploy/traefik-forward-auth.yml) once per application and replace `{{CLIENT_ID}}`, `{{RIAUTH_URL}}`, `{{APP_HOST}}`, `{{ENTRYPOINT}}` and `{{APP_UPSTREAM}}`. The template defines the forwardAuth middleware, the application router, and a priority-1000 router that sends `/outpost/{client}/start`, `/callback` and `/logout` on the application's host to riAuth.
   - `{{RIAUTH_URL}}` is riAuth's **internal** address, for example `http://192.0.2.10:9000`. The address shown is a documentation placeholder; use riAuth's actual service address. Never route it back through Traefik: the forwardAuth request must arrive from Traefik's own address, and the issuer router below hides the endpoint.
   - Add `tls: {}` (or a `certResolver`) to both routers on an HTTPS entrypoint.
   - Keep the forwardAuth middleware before any `stripPrefix` or `replacePath`, so riAuth sees the path the browser requested.
   - For an issuer with a path (for example `https://id.example.com/riauth/`), add an `addPrefix` middleware to the `-outpost` router and append the path to the forwardAuth address.
5. **Issuer host.** If Traefik also serves riAuth's issuer host, exclude the forward-auth endpoints so nobody can reach them through Traefik:

   ```yaml
   http:
     routers:
       riauth-issuer:
         rule: "Host(`id.example.com`) && !PathRegexp(`^/outpost/[^/]+/(auth|traefik)$`)"
         entryPoints: ["websecure"]
         service: riauth-issuer
         tls: {}
     services:
       riauth-issuer:
         loadBalancer:
           servers: [{ url: "http://192.0.2.10:9000" }]
   ```

### What riAuth checks

The endpoint never writes to the store and never sets a cookie. It fails closed, in this order:

1. **Peer.** The TCP peer must be in `trusted_proxies`; otherwise 403.
2. **Target.** Exactly one each of `X-Forwarded-Proto` (`http` or `https`; `ws`/`wss` only arrive with `trustForwardHeader: true` and map to `http`/`https`), `X-Forwarded-Host` (1–255 bytes of letters, digits, `.`, `-`, `:`, `[`, `]`) and `X-Forwarded-Uri` (starts with `/` but not `//`, at most 8192 bytes, no control characters, whitespace, `#` or `\`). The rebuilt URL must keep that origin; otherwise 400. It replaces any `X-Original-URL`; a client-supplied one is never read.
3. **Client.** The client must exist and the target must belong to its `external_origin` and avoid `/outpost/` paths.
4. **WebSocket.** A request with `Sec-WebSocket-Key` (or a `ws`/`wss` proto) needs exactly one `Origin` equal to the application's origin; otherwise 403.
5. **Unsafe methods.** For anything other than GET, HEAD and OPTIONS (a missing `X-Forwarded-Method` counts as unsafe), an `Origin` must be the application's origin if present, and `Sec-Fetch-Site` must be `same-origin` or `none` if present; otherwise 403. Cross-site form posts are refused before authentication.
6. **Authorization** runs the same proxy-session and policy check as the nginx endpoint.

| Result | Response |
| --- | --- |
| Allowed | 200 with `X-Authentik-Username`, `-Uid`, `-Name`, `-Email`, `-Groups` (group names separated by a vertical bar), `X-Auth-User` and `X-Auth-Sub`, plus `Cookie` holding the application's own cookies without riAuth's. When no application cookie is left, `Cookie` is omitted and Traefik removes the client's. |
| Not signed in, top-level GET or HEAD navigation (`Sec-Fetch-Mode` absent or `navigate`, not a WebSocket) | 302 with an **absolute** `Location` to `https://{app}/outpost/{client}/start?rd=…`. Traefik would resolve a relative one against the forwardAuth address. |
| Not signed in, anything else (API calls, POSTs, WebSockets) | 401 with `X-Riauth-Login: <that URL>`, so background requests are never redirected |
| Denied for a document request (`Sec-Fetch-Dest: document`) | 403 with a minimal page, "You don't have access to this application.", linking to `{issuer}/apps` |

The template's `authResponseHeaders` also lists `X-Authentik-Entitlements` and `Authorization`, which riAuth never sets: Traefik deletes those names from the client's request on every allowed request, so a client cannot inject them. `authRequestHeaders` passes only `Accept`, `Cookie`, `Origin`, `Sec-Fetch-Dest`, `Sec-Fetch-Mode`, `Sec-Fetch-Site` and `Sec-WebSocket-Key` to riAuth, besides the `X-Forwarded-*` set Traefik adds itself.

### Variants

- **Load balancer in front of Traefik.** Set `trustForwardHeader: true` on the forwardAuth middleware and `forwardedHeaders.trustedIPs` on the entrypoint to the load balancer's addresses; Traefik then forwards the `X-Forwarded-*` set it trusts. Add the load balancer's addresses to `trusted_proxies` as well, so riAuth walks past them in `X-Forwarded-For` to the real client address; otherwise every user shares the balancer's rate-limit bucket.
- **Traefik dashboard.** Use `service: api@internal`, the rule ``Host(`traefik.example.com`) && (PathPrefix(`/api`) || PathPrefix(`/dashboard`))``, and a `redirectRegex` middleware on a `Path(`/`)` router that sends `/` to `/dashboard/`. `api.insecure` must be `false`, or the dashboard is also served unprotected on port 8080. Do not name the client `traefik`. Test this variant separately; the real-Traefik test covers the generic application template.
- **Client certificates.** Traefik becomes a trusted proxy, so if `client_certificates` sets `forwarded_header`, Traefik must remove that header from client requests on the issuer router, for example with a `headers` middleware whose `customRequestHeaders` sets it to `""`.

### Limits

- **Direct application access.** Forward auth protects only requests that pass through Traefik. Restrict the application's listener so users cannot bypass the proxy.
- **Application authorization.** Forward auth supplies identity headers; it does not issue application-specific API or ACL tokens. Configure the application's own permissions and test the resulting access level.
- **WebSockets are not revalidated** after the handshake. Revoking the riAuth session stops new requests and new sockets, not an open one. The Rust reverse proxy below does recheck open sockets.
- **Cross-site form posts** to protected applications are refused (step 5), even with a valid proxy cookie.
- **Rate limits** for `forward_auth` are counted per riAuth node, even with PostgreSQL.

Run `RIAUTH_TEST_TRAEFIK=/path/to/traefik cargo test --test outpost_traefik --locked -- --ignored` to repeat the real-Traefik test. It renders the deploy template and the issuer router above, and covers the absolute login redirect, background 401s, sign-in through the outpost routes, identity headers, removal of client `Authorization`, alias and riAuth cookies, a planted `X-Original-URL`, cross-site POST refusal, WebSocket `Origin` checks, hidden forward-auth endpoints on the issuer host and revocation. The other tests in that file cover the contract without Traefik.

## Embedded Rust reverse proxy

`riauth serve` can host protected applications directly. Add listener routes to the private server TOML, and manage each referenced policy client through the existing CLI or agent manifest. Every route must also be an allowed origin in that client's proxy settings. Missing or disabled policy clients fail closed.

```toml
[proxy_listeners.edge]
listen = "0.0.0.0:8443"
tls_cert_file = "tls/apps.pem"
tls_key_file = "tls/apps.key"
max_body_bytes = 8388608
upstream_timeout_seconds = 30

[proxy_listeners.edge.routes."https://reports.example.com"]
client_id = "reports"
upstream = "https://reports-backend.example.test:9443"
ca_file = "tls/private-app-ca.pem"
```

Native TLS is required for non-loopback listeners. HTTP loopback is available for local tests or a local TLS ingress. TLS certificates reload every 60 seconds, retaining the active certificate if replacement fails. Route or upstream trust changes require restarting the service. An upstream is a fixed exact origin; paths and queries come from the authenticated request. HTTP upstreams outside loopback require explicit `allow_plain_http = true`. A supplied CA file replaces public roots for that upstream. Environment proxy settings and HTTP redirects are disabled for upstream requests.

The listener handles its client's `/outpost/…` terminal SSO routes internally. All application paths require a live authorized session. GET/HEAD without a session redirect to login; writes return 401. Requests cannot select an arbitrary upstream, proxy CONNECT, or reach management routes on this listener. Those paths remain normal protected application paths. The service replaces identity and forwarding headers, removes inbound Authorization and authentication cookies, filters hop headers and reserved response cookies, bounds request bodies, and streams application responses with backpressure. It forwards the actual TCP peer in `X-Forwarded-For`. Applications behind another ingress should account for that ingress address.

The current upstream profile uses HTTP/1.1. It supports WebSocket upgrades with same-origin validation, checked handshake keys and negotiated subprotocols. Active sockets recheck authorization every 30 seconds and close after revocation, session expiry, loss of a required temporary group grant or device verification, a policy change or service shutdown. A listener allows 256 simultaneous forwarded responses or sockets. Ordinary upstream requests, including response streams, use the configured timeout; WebSockets use session lifetime and periodic authorization instead.

## Shared-domain SSO

One provider can cover an explicit set of HTTPS applications under the same registrable domain:

```json
{
  "proxy": {
    "external_origin": "https://auth.example.com",
    "session_ttl": 3600,
    "domain": {
      "cookie_domain": "example.com",
      "application_origins": ["https://reports.example.com", "https://files.example.com"]
    }
  }
}
```

Register only `https://auth.example.com/outpost/CLIENT_ID/callback` as this provider's callback. Route the central authentication origin and the listed application origins to the outpost. In the embedded proxy, assign each origin a route referencing this same client. Forward-auth integrations use the central `X-Riauth-Login` URL returned by the auth endpoint.

The browser-binding cookie remains host-only at the central authentication origin. The application session uses a Secure, HttpOnly, SameSite=Lax `__Secure-riauth_…` cookie with the explicit parent Domain. Public suffixes, unrelated domains, HTTP applications and unlisted return origins are rejected. All subdomains receiving this cookie must be trusted: a domain cookie is shared with sibling hosts. This mode shares one policy and consent across the listed applications; use separate clients when their permissions differ.

`cargo test --test outpost rust_reverse_proxy` exercises the actual Rust listener, CLI approval, cookie/header filtering, body limits, unknown hosts, WebSocket protocol/echo and revocation closure. The domain regression test covers shared sessions, exact origins and ports, public-suffix rejection, host-only login binding, shared logout and live scope-policy changes. Set `RIAUTH_TEST_BROWSER` to include real Chrome navigation through the Rust listener.
