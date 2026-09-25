# ENT-14 — Self-hosted event map

[Implementation](../../src/event_map.rs) and [tests](../../tests/event_map.rs).

riAuth can show where audited activity already says it happened, without calling a map tile server, a font host or a geolocation service. The picture is an original equirectangular grid compiled into the binary. It has no country borders and no copied tileset.

## Who can open it

`GET /api/audit/map` uses the same permission as `GET /api/audit`: `audit.read` on `audit/events`. A human administrator has that permission. An agent needs it explicitly; `audit.read` on a different resource, or another action, is rejected with 403. A normal user is rejected with 403. Missing credentials are 401.

The browser page is `GET /events` (and `/events/`), a public static shell served by the existing portal HTTP stack. The page exposes no audit data until its API request passes authorization. It is not an application in the catalogue and the end-user launcher does not link to it. Page script never sees the session token. A signed-in administrator's SSO cookie (`__Host-riauth_sso` on https, `riauth_sso` on loopback http) is enough, even for a password-only browser session; agents and CLI callers send `Authorization: Bearer`. A password-only administrator session can open the map; configure administrator MFA policy according to your deployment requirements. The page's content security policy is `default-src 'none'` with same-origin script, style and connect sources only.

## Where coordinates come from

Only an explicit local `location` object is plotted:

```json
{"latitude": 37.77, "longitude": -122.42, "label": "San Francisco"}
```

`latitude` is -90 through 90 and `longitude` is -180 through 180. `label` is optional, at most 80 characters, and cannot contain control characters. The object may be stored in either place:

1. `details.location` on the audit event. If the key is present but the coordinates are missing or out of range, the event is unknown. The map does not fall back to the user.
2. Otherwise, the actor's user attribute `location`, set through the existing user update or manifest `attributes`. Changing that attribute changes how older events without their own coordinates are grouped. Store `details.location` when a historical point must stay put. The current audit writer does not copy the attribute into new events.

Events without a usable coordinate, including actors that are not users, increment `unknown`. They are not omitted. IP addresses, forwarding headers and nested audit `changes` are ignored. riAuth does not ship or query a GeoIP database, and the page does not call the browser location API. This view does not add tracking: it only reads coordinates operators already stored, and the read itself is not an audit event.

Do not put secrets, passwords or tokens in the location label or in other attribute fields you expect this view to ignore. The response is limited to rounded coordinates, an optional label and counts.

## Aggregation and filters

Nearby events in the same 0.1 degree cell are summed. Halfway cases round away from zero. Longitude 180 and -180 share one cell. Cells are ordered by count, then latitude, then longitude. At most 500 cells are returned (`omitted_cells` counts the rest). The scan walks timestamp-prefixed audit storage keys in reverse order and stops after 10,000 events or at the start of the time window. `truncated` is true when the cap stopped the scan before the window was finished.

| Query | Meaning |
| --- | --- |
| `since`, `until` | Inclusive unix seconds. A reversed range is 400 |
| `action` | Case-sensitive action prefix, at most 128 characters. Not a regular expression |

```json
{"points":[{"latitude":37.8,"longitude":-122.4,"count":4,"label":"San Francisco"}],"unknown":2,"scanned":40,"matched":6,"truncated":false,"omitted_cells":0}
```

## Internet-free deployment

The grid, its stylesheet and its script are `include_str` assets inside the riAuth binary, same as the application portal. A deployment with no internet access can serve `/events` and `/api/audit/map` from local storage. The page does not request tiles, fonts, scripts or geolocation from the network. No schema migration is required.

See also [the HTTP API](../api.md) and [the portal](../PORTAL.md).
