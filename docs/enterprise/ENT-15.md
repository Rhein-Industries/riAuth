# ENT-15 CSV export

[Implementation](../../src/reports.rs) and [tests](../../tests/reports.rs).

Authorized CSV exports of users and audit events. This is an operational report. It is **not** a SOC, ISO, or other compliance certification.

Review filters, retention, and redaction are defined in [ENT-09.md](ENT-09.md).

## Endpoints and CLI

| Surface | Permission | Notes |
| --- | --- | --- |
| `GET /api/reports/users.csv` | Same as user inventory: caller must be an administrator, or an agent with some `user.read` permission. Each row still requires `user.read` on `user/{username}` (exact resource or `*`). | Query: `filter` (username substring), `limit`, `cursor`. |
| `GET /api/reports/audit.csv` | `audit.read` on `audit/events`, same as review. | Same filters as `GET /api/audit/review`. |
| `riauth report users --out FILE` | The saved admin or agent credential. | Walks every page and writes one CSV. |
| `riauth report audit --out FILE` | The saved admin or agent credential. | Same, with `--action`, `--actor`, `--target`, `--run-id`, `--from`, `--to`. |

`--out` is published with mode `0600` by an exclusive hard link from a private temporary file in the same directory. Existing files are never overwritten, and failed exports remove the temporary file without publishing a partial report. The CLI writes each page directly to that file, retaining only the current page in memory. The global `--output-file` flag, if also set, receives only the JSON acknowledgement (`output_file`, `rows`, `pages`), not the CSV. Page size defaults to 1,000. The command rejects exports exceeding 10,000 pages, responses exceeding 8 MiB while receiving them (including chunked responses), incomplete CSV records, or a cursor that does not advance. The acknowledgement counts logical CSV records, including quoted newlines and doubled quotes correctly. `--request-timeout SECONDS` or `RIAUTH_REQUEST_TIMEOUT` sets the deadline for each page request, including body receipt; the default is 30 seconds and the allowed range is 1..=86,400.

CSV exports contain identity or audit data. From the repository root, create `deployment-private/` and choose a path such as `deployment-private/users.csv` for `FILE`; outside a checkout, use a private operator directory.

Response headers: `content-type: text/csv; charset=utf-8`, `content-disposition: attachment`, `cache-control: no-store`, `x-content-type-options: nosniff`. When another page exists, `x-next-cursor` is set. The cursor is the same encrypted cursor as review (audit) or a user cursor bound to the actor, username filter, and configuration revision (`riauth.users.page/v1`, one hour). A user-export cursor is rejected after a configuration change so a concurrent insert cannot skip or duplicate UUID keys; restart the export.

`limit` is clamped to 1..=1,000 (default 100) for both CSV endpoints. Review stays 1..=500.

## Columns

User CSV, in order:

`id, username, email, display_name, enabled, admin, created_at, groups`

`enabled` and `admin` are `true` or `false`. Empty email is an empty field. `groups` is `;`-separated durable group names, sorted. Temporary PAM grants are not included because export reads `Group.members`. Group names cannot contain `;`.

Audit CSV, in order:

`id, at, actor, action, target, run_id, request_id`

There is no details or changes column. `run_id` and `request_id` are empty when absent. Secrets are not given a column.

## Encoding

- UTF-8, no byte-order mark.
- Record separator is LF (`\n`), not CRLF. Every row, including the header and the last row, ends with LF.
- Header row on every HTTP page. The CLI writes the header once and drops it on later pages.
- RFC 4180 quoting: fields that contain comma, double quote, LF, or CR are wrapped in double quotes, and internal quotes are doubled.
- Spreadsheet formula injection: a field whose first character is `=`, `+`, `-`, `@`, tab, or CR is prefixed with one single quote (`=cmd` becomes `'=cmd`) before quoting is decided.
- Unicode is stored as UTF-8 and is not escaped.

## Pagination and bounds

Each request scans at most 256 keys at a time and stops after `limit` accepted rows or 10,000 examined rows, whichever comes first. The response is only that page. Callers repeat with `cursor` / `x-next-cursor` until the cursor is absent. An empty last page is not required when the final accepted row is also the last stored key.

User export reads the group collection once per request (at most 10,000 groups) to fill the `groups` column. It does not load every user into memory; only the page is materialized.

## Secrets and authorization

The user CSV is not a dump of the user record. It omits `password_hash`, TOTP secrets, recovery codes, pairwise seeds, and any other credential. The audit CSV omits `details` entirely, so a redacted change document cannot leak through a summary column.

An agent with `user.read` on `user/alice` receives Alice and does not receive other users. A normal user receives `access_denied` for both report endpoints. Audit export uses `audit.read` on `audit/events` only.

## CLI regression evidence

`tests/reports.rs` runs the real binary against a paged HTTP fixture: 96 pages produce over 24 MiB of UTF-8 CSV with multiline quoted fields, exactly 96 acknowledged records, one header, and mode `0600`. A chunked oversized second page leaves neither a published output nor a temporary file. A delayed response verifies that one-second and three-second request deadlines fail and succeed respectively. These tests establish pagination and enforced byte boundaries; they do not promise a process RSS or deployment throughput target.
