# Cloud directory operations

The Platform build exposes the same operational surface for configured Google Workspace and Microsoft Entra directories. Signed-in administrators and delegated directory operators can open **Connectors** under `/admin` for directories they may read; its same-origin routes call the same scoped Core methods and job service.

| Method | Endpoint | Required authority | Behavior |
| --- | --- | --- | --- |
| `GET` | `/api/cloud-directories/{kind}/{id}/operations` | `directory.read` on `{kind}/{id}` | Shows sanitized configuration, mappings, validation, and available reconciliation history. |
| `POST` | `/api/cloud-directories/{kind}/{id}/test-connection` | `directory.sync` on `{kind}/{id}` | Obtains a token and reads one users page without saving a plan or changing accounts. |

The browser uses `/api/admin/cloud-directories` to list only readable connectors, plus `/api/admin/cloud-directories/{kind}/{id}/operations` and `/api/admin/cloud-directories/{kind}/{id}/test-connection` for the same read and probe. These routes require a signed-in administrator or a delegated directory operator with the exact connector grant, plus the portal request guard. The probe uses the browser write guard because it is a POST, while its Core operation remains read-only.

`kind` is `workspace` or `entra`. Configuration validation uses the same structural connector checks as startup and also reports mapped local groups that do not exist. The operations response includes the connector's mapped groups, attributes, username prefix, and reconciliation mode. Its credential status reports only whether the bounded private file can be read with owner-only permissions, its modification time, and that the next token request rereads it. This is a file check, not proof of a valid key, successful rotation, or provider acceptance. The response never includes credential paths, client secrets, private keys, access tokens, or raw scheduler authority, errors, and outcome records.

The optional schedule and ten most recent job summaries come from the existing reconciliation service and appear only when the caller also has its `directory.sync` authority. Job states include a generic next action and never assert remote completion. A `completed` reconciliation job records local controller completion; downstream delivery may still be queued. No schedule or job is synthesized for a connector without a configured controller or history. Schedule configuration and credential replacement remain server-side operations.

The connection test returns `connected`, `checked_at`, and a bounded error code and message on failure. It checks only token acquisition and the first users response shape. It does not validate group membership pages, the complete directory crawl, local mapping targets, an apply, or downstream account changes. A successful result is therefore an operational reachability check, not permission to skip plan review. The probe does not persist connector state or consume the sync retry budget; use it sparingly when diagnosing connectivity.
