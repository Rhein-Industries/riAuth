# Cloud directory operations

The Platform build exposes the same operational surface for configured Google Workspace and Microsoft Entra directories:

| Method | Endpoint | Required authority | Behavior |
| --- | --- | --- | --- |
| `GET` | `/api/cloud-directories/{kind}/{id}/operations` | `directory.read` on `{kind}/{id}` | Shows sanitized configuration, mappings, validation, and available reconciliation history. |
| `POST` | `/api/cloud-directories/{kind}/{id}/test-connection` | `directory.sync` on `{kind}/{id}` | Obtains a token and reads one users page without saving a plan or changing accounts. |

`kind` is `workspace` or `entra`. Configuration validation uses the same structural connector checks as startup; credential availability is checked by the connection test. The operations response includes the connector's mapped groups, attributes, username prefix, and reconciliation mode. It never includes credential files, client secrets, private keys, access tokens, or raw scheduler authority and outcome records. The optional schedule and ten most recent job summaries come from the existing reconciliation service and appear only when the caller also has its `directory.sync` authority. No schedule or job is synthesized for a connector without a configured controller or history.

The connection test returns `connected`, `checked_at`, and a bounded error code and message on failure. It checks only token acquisition and the first users response shape. It does not validate group membership pages, the complete directory crawl, local mapping targets, an apply, or downstream account changes. A successful result is therefore an operational reachability check, not permission to skip plan review. The probe does not persist connector state or consume the sync retry budget; use it sparingly when diagnosing connectivity.
