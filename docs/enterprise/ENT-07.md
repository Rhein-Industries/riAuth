# ENT-07 Shared Signals

[Implementation](../../src/ssf.rs) and [tests](../../tests/ssf.rs).

Receiver and transmitter for a subset of [OpenID Shared Signals Framework 1.0](https://openid.net/specs/openid-sharedsignals-framework-1_0-final.html) and [RFC 8417](https://www.rfc-editor.org/rfc/rfc8417) Security Event Tokens. Push delivery only. Apple Business Manager was **not** tested.

## Event URIs

Only these three events are accepted or emitted:

| Event | URI |
| --- | --- |
| account-disabled | `https://schemas.openid.net/secevent/risc/event-type/account-disabled` |
| session-revoked | `https://schemas.openid.net/secevent/caep/event-type/session-revoked` |
| credential-change | `https://schemas.openid.net/secevent/caep/event-type/credential-change` |

Push delivery method: `urn:ietf:rfc:8935` (alias `https://schemas.openid.net/secevent/risc/delivery-method/push`). Poll (`urn:ietf:rfc:8936`) is not implemented.

## SET profile

The receiver requires `iat`, a bounded `jti`, `typ: secevent+jwt`, and `sub_id` in `iss_sub`, `opaque`, or `email` format. Top-level `sub` and `exp` are rejected. The `iss_sub` mapping includes both the subject issuer and subject value; another format or issuer with the same value cannot select that user. The `iss_sub.iss` value follows JWT StringOrURI syntax and need not be HTTPS; the stream's pinned SET transmitter issuer remains an HTTPS URL. Every event payload must be a JSON object; a nested primary `subject`, if present, must match top-level `sub_id`. CAEP credential-change additionally requires a nonempty `credential_type` and a `change_type` of `create`, `revoke`, `update`, or `delete`. The SET issuer, audience and signature must match a registered stream and its pinned public keys. `iat` may be at most 60 seconds ahead and seven days old; the issuer/JTI replay record remains for that window. A duplicate valid SET receives the same empty HTTP 202 acknowledgement without reapplying its events.

Outbound SETs are signed with the instance signing key and include `iat`, `jti`, and `sub_id`; they omit top-level `sub` and `exp`. The `iat` and event timestamp come from the queued transition time and remain fixed across retries for the same `jti`. Outbound push sets `Content-Type: application/secevent+jwt` and `Accept: application/json`.

Not implemented, and not advertised in metadata: poll delivery, status, add-subject, remove-subject, and verification endpoints. Local subject bindings are approved through the separate administrative API.

## Metadata

`GET /.well-known/ssf-configuration` (when the issuer has a path, `/.well-known/ssf-configuration` is inserted before that path, per SSF discovery).

The document reports `spec_version` `1_0`, this issuer, `jwks_uri` (`/oauth/jwks`), push as the only delivery method, `configuration_endpoint` `/api/ssf/streams`, OAuth bearer authorization (`urn:ietf:rfc:6749`), and the three event URIs. Those URLs are HTTPS only when the issuer is HTTPS. An HTTP loopback issuer is a development exception and does not satisfy the specification's TLS requirement.

## Receiver configuration

The advertised `/api/ssf/streams` endpoint accepts bearer authentication from an administrator, an agent with `ssf.configure`, or an OAuth service client's unbound `client_credentials` access token carrying the `ssf.configure` scope. Resource-bound OAuth tokens for another API are rejected. The OAuth option makes the advertised `urn:ietf:rfc:6749` authorization scheme operational. An agent requires `ssf.configure=*` to create a stream because the transmitter generates its ID; it can read or change only its owned streams. Service clients can manage only their own streams. Non-administrator receivers have a limit of eight standard streams per owner and 24 shared standard stream slots, reserving eight of the 32 total slots for administrators. An `ssf.manage` agent cannot use this outbound configuration API. Client-secret Basic authentication and ordinary client access tokens do not grant SSF configuration authority. A receiver supplies `events_requested`, `delivery`, and optional `description`; the server generates `stream_id`, `iss`, and `aud`. The audience is the authenticated administrator's user ID, agent principal ID or `client:<id>` service identity and remains fixed for the stream. It returns the standard stream configuration object with HTTP 201. `GET /api/ssf/streams` returns an array, while `GET /api/ssf/streams?stream_id=ID` returns one object. `PATCH` updates supplied receiver fields; `PUT` replaces them; both require `stream_id` in the JSON body and return the full object. `DELETE /api/ssf/streams?stream_id=ID` returns empty HTTP 204. Responses use `Cache-Control: no-store`.

```json
{
  "events_requested": ["https://schemas.openid.net/secevent/risc/event-type/account-disabled"],
  "delivery": {"method": "urn:ietf:rfc:8935", "endpoint_url": "https://receiver.example/events"},
  "description": "Receiver A"
}
```

A new receiver stream has no local subject bindings, so it cannot receive a user's events until an administrator links exact subject identifiers to approved local users. The configuration endpoint cannot set signing trust or user mappings.

Push `delivery.authorization_header` is optional. If supplied, the transmitter sends its exact value as the `Authorization` header on every POST. It is write only: configuration and administrative responses omit it, and it is never copied into queue or audit records. The database must have `database_key_file` configured before a stream can store this secret. PATCH with the same endpoint and no `authorization_header` retains the value; explicit `null`, a changed endpoint, or PUT without the property clears it. Changing the authorization value cancels queued deliveries so old events are not sent with a new credential. A delivery already claimed by a worker may complete one in-flight POST after an endpoint, credential, binding, or stream change.

## Administrative trust and subject bindings

The private `/api/ssf/admin/streams` API and CLI create inbound trust registrations with pinned peer JWKS and local subject bindings. Authorization is an administrator bearer session or an agent bearer with `ssf.manage` (`ssf/<id>` or `*`). Responses list key IDs only; JWKS material is write-only. These routes replaced the earlier `/api/ssf/streams` administrative contract. Existing `riauth ssf stream` commands use the new route automatically. Run the example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory.

```sh
mkdir -p deployment-private
riauth ssf stream create --file deployment-private/stream.json
riauth ssf stream list
riauth ssf stream delete tenant-a
```

`list` includes `inbound_push_url`. Inbound SETs are pushed to:

`POST {issuer}/api/ssf/events`

`Content-Type: application/secevent+jwt` and a raw SET body. No bearer is required; the signature is the authenticator. Success, including a duplicate, is HTTP 202 with an empty body. Validation failures use the RFC 8935 `err`/`description` JSON shape and `Content-Language: en`. The receiver distinguishes malformed requests, unknown issuer, wrong audience, untrusted key, and failed signature with `invalid_request`, `invalid_issuer`, `invalid_audience`, `invalid_key`, and `authentication_failed` respectively. Unknown subjects are silently ignored with HTTP 202, so that case does not return `access_denied`.

Example `stream.json`:

```json
{
  "id": "tenant-a",
  "issuer": "https://transmitter.example",
  "audience": "https://idp.example",
  "events_requested": [
    "https://schemas.openid.net/secevent/risc/event-type/account-disabled",
    "https://schemas.openid.net/secevent/caep/event-type/session-revoked",
    "https://schemas.openid.net/secevent/caep/event-type/credential-change"
  ],
  "delivery": {
    "method": "urn:ietf:rfc:8935",
    "endpoint_url": "https://receiver.example/events"
  },
  "jwks": { "keys": [{ "kty": "EC", "crv": "P-256", "kid": "peer-key", "alg": "ES256", "use": "sig", "x": "REPLACE_WITH_PEER_X", "y": "REPLACE_WITH_PEER_Y" }] },
  "subjects": { "external-subject": "local-username" }
}
```

Replace the example JWKS coordinates with the peer's actual public key before creating a stream; an empty or invalid key set is rejected.

`issuer` is the peer transmitter expected on inbound SETs, not this instance's issuer. `audience` is required on inbound SETs and is the `aud` of outbound SETs. `subjects` maps an exact external identifier to an existing username. A bare string is shorthand for `{"format":"iss_sub","iss":"<issuer>","sub":"<string>"}`. To bind a different `iss_sub` issuer, `opaque`, or `email`, use its canonical JSON object serialized as the map key, for example `"{\"format\":\"opaque\",\"id\":\"external-123\"}"`. Users are never created. `jwks` is the peer's public verification set for inbound SETs.

For a receiver-created outbound stream, set or replace the entire approved mapping with `PUT /api/ssf/admin/streams/{id}/subjects` and a JSON body of `{"subjects":{"<subject-id>":"local-username"}}`. This operation requires the same `ssf.manage` permission. Removing a stream, replacing bindings, or changing its delivery endpoint cancels pending deliveries for that stream.

`iss` in the create response is this instance's issuer (the transmitter identity on SETs it signs).

## Inbound behavior

Signature, `iss`, `aud`, `iat`, `jti`, and the full `sub_id` are checked against a stream whose pinned JWKS verifies the token. The raw SET is not written to the audit log or to the delivery queue. Inbound action audit actors include the stream ID and its owner.

- `account-disabled` disables the linked user and revokes sessions.
- `session-revoked` revokes sessions and bumps the epoch.
- `credential-change` revokes sessions and bumps the epoch. It does not change the local password.

An unknown subject, or a subject linked only on a different stream, is HTTP 202 with no local change and an audit action `ssf.ignored` that does not contain the token. The last enabled administrator is not disabled. A stream for tenant A cannot change a user who is not linked on that stream. Inbound account-disable uses the shared durable revocation transition: child agents, Windows devices and outstanding sign-in tickets are revoked, and re-enabling the account does not restore them.

## Outbound behavior

Account-disable and credential-change notifications are produced by the shared durable record transition, within the same database transaction as the change. Events are deduplicated per user, event and credential type within a transaction; unchanged records and repeated disable requests do not enqueue a second event. Rollback, manifest preview and successful apply retries do not create deliveries. Re-enabling a legacy disabled account repairs any unrevoked child credentials and advances its epoch, without emitting another account-disabled event.

| Transition | Entry points |
| --- | --- |
| Account disabled | Administrator update, SCIM PUT/PATCH/DELETE, manifest apply, LDAP/cloud-directory sync, offboarding and inbound account-disable |
| Password replaced | Administrator/self-service password change, email reset/invitation completion, local administrator recovery, SCIM and manifest apply |
| Authenticator changed | TOTP enrollment/import/reset, recovery-code rotation, passkey enrollment/removal/reset |
| Session revoked | Core logout/session revocation and explicit administrative session revocation |

Outbound credential types are `password`, `otp`, `recovery-code` and `public-key`; the latter three are local extension values that require receiver agreement. Outbound `change_type` is currently always `update`, including passkey enrollment or deletion, so receivers that need exact create/delete semantics should not rely on this event alone. Transparent password rehashing, recovery-code consumption and passkey authentication-counter updates do not signal credential replacement. An inbound credential-change only invalidates local sessions; it does not replace a local credential or echo a credential-change notification.

The transition only enqueues. `deliver_once` signs and POSTs `application/secevent+jwt`. HTTP 500 and 429 retry with backoff, up to 5 attempts. Other 4xx and redirects stop and record failure. Tokens are not logged or stored on the queue record. Delivery remains at-least-once: recipients should deduplicate by SET `jti`, which stays stable across retries.

## What an Apple Business Manager test still requires

- A real ABM (or other) SSF transmitter, its issuer, and its JWKS.
- A stream registered with that transmitter's management API, not only this local API.
- SETs actually produced by ABM, including whatever subject format it sends, and a reviewed exact local subject mapping.
- Evidence that an event for one tenant does not disable another tenant's user, observed in that federation rather than with a local signing key.

`tests/ssf.rs` uses local keys and a local HTTP listener only.
