# Custom frontend layouts and styling

Essentials and Platform support the same operator-supplied frontend themes.
You can replace complete page layouts, stylesheets, external JavaScript and
images without rebuilding the server. Omitted files keep their embedded
defaults. With no theme configured, the embedded HTML and asset bodies remain
identical; HTML CSP additionally permits same-origin fonts.

## Configure and start

Put a trusted theme in a local deployment directory. Add this table **after all
top-level configuration keys** in `riauth.toml`:

```toml
[frontend]
theme_dir = "frontend-theme"
```

Relative paths resolve beside the configuration file, rather than beside the
binary. Programmatic `Config` values use paths relative to the process working
directory. The default configuration omits this table when serialized.

The server reads the entire theme once, before opening storage, migration or
binding its listener. First-administrator setup uses the same configured theme
before Core initialization and retains that snapshot when setup completes.
Requests look up validated names in an immutable shared bundle and never read
the directory. Changing or removing files does not change a running instance;
restart to load a new snapshot. An invalid theme refuses startup before storage
writes. Removing `[frontend]` restores the embedded frontend on the next start.
`browser_ui = false` continues to disable browser pages and assets.

Theme files are trusted deployment code: custom JavaScript executes with the
instance's browser origin and can interact with signed-in users. Restrict write
access to the operator, keep credentials and private files outside this tree,
and finish deploying files before starting the server. This is a bounded
startup snapshot, not an atomic filesystem-tree transaction. There is no
upload endpoint, browser editor, agent permission or database theme record.

## Supported file layout

The following page overrides are exact names under `pages/`:

| Theme file | Embedded source to copy | Existing route or use |
| --- | --- | --- |
| `pages/apps.html` | `src/portal/index.html` | `/apps` and HTML root |
| `pages/signin.html` | `src/portal/signin.html` | OIDC/SAML sign-in, consent and sign-out interaction |
| `pages/admin.html` | `src/portal/admin.html` | `/admin` |
| `pages/account.html` | `src/portal/account.html` | `/account/accept`, `/account/verify`, `/account/reset` |
| `pages/security.html` | `src/portal/self_service/security.html` | `/account/security` |
| `pages/device.html` | `src/portal/device.html` | `/device` |
| `pages/source-stage.html` | `src/portal/source-stage.html` | Embedded source-stage browser continuation |
| `pages/setup.html` | `src/portal/setup.html` | Pending first-administrator `/setup` |
| `pages/sources.html` | `src/portal/sources.html` | `/account/sources/continue` |
| `pages/events.html` | `src/portal/events.html` | Platform `/events` |
| `pages/access-review.html` | `src/portal/access-review.html` | Platform `/access/review` |

These files replace only the representations of existing routes. A theme does
not activate a disabled feature or add a page route. Script-free error/notice
pages, completed-setup notices, native protocol form-post responses, discovery
and JSON/API responses are outside the template override scope. Their backend
behavior and protections remain unchanged.

Existing asset overrides use `assets/<filename>` and retain their existing
`__BASE__portal/assets/<filename>` URLs. The exact supported names are:

```text
app.css admin.css security.css map.css access-review.css riauth-mark.svg
app.js admin.js auth.js account.js capabilities.js signin.js setup.js device.js
sources.js source-login.js source-stage.js security.js access-review.js map.js
grant-review.js membership-review.js client-creation-review.js
client-policy-review.js client-status-review.js client-endpoint-review.js
```

Copy these from `src/portal/`, except `security.css`/`security.js` from
`src/portal/self_service/` and `riauth-mark.svg` from `assets/`. Asset routes
retain their existing edition/feature availability; pending setup serves its
existing four baseline assets (`setup.js`, `auth.js`, `app.css`, the mark).

Additional theme files go under `theme-assets/`, with up to three nested
directories within it. For example, `theme-assets/images/logo.svg` is served at
`__BASE__portal/theme-assets/images/logo.svg`. Accepted extensions and fixed MIME
types are CSS (`text/css`), JS (`text/javascript`), SVG (`image/svg+xml`), PNG,
JPEG (`jpg`/`jpeg`), GIF, WebP, ICO, WOFF, WOFF2, TTF and OTF. HTML is accepted
only as a known page template. No arbitrary file path is resolved from a request.

All theme responses use `no-store` and `nosniff`. Each tree is limited to 256
entries including directories, 128 files, four directory levels below its root,
1 MiB per file and 8 MiB of file content in total. Names are ASCII letters,
digits, hyphens, underscores and dots; each segment is at most 64 bytes and each
relative key at most 240 bytes. Empty/dot/hidden segments, trailing dots,
Windows device names, encoded names, backslashes and duplicate case-folded
paths are refused. Symlinks, reparse points, special files, unknown files and
malformed UTF-8 in HTML/CSS/JS/SVG refuse the whole bundle. File reads use
no-follow, nonblocking opens on Linux x86_64/AArch64 and macOS, and reparse-point
opens with metadata rejection on Windows. Custom themes fail on unsupported
platforms; the default frontend remains available.

## Template and frontend contracts

Use defaults from the **same server source revision** as your binary. The
server substitutes these explicit markers, escaping values for HTML:

- `__BASE__` is the configured issuer path with a trailing slash, such as
  `/identity/`. Keep it in the `riauth-base` meta tag and local asset/link URLs.
- `signin.html` may use `__CODE__` and `__COMMAND__` for terminal instructions.
- `source-stage.html` uses `__AUTHORIZATION__`, `__STAGE__`, `__RESUME__`,
  `__CANCEL__` and `__CONTINUE__` for the existing browser continuation.

There is no templating engine or server-side script execution. Files are trusted
literal HTML/CSS/JS, with only the existing marker substitutions. CSS/JS files
are served unchanged. Use relative URLs in CSS and the existing base meta tag
in JavaScript when constructing local URLs.

For functional custom layouts, keep the default DOM IDs, forms, data attributes,
hidden initial states and corresponding frontend modules. Move or restyle those
elements rather than deleting them. `auth.js` supplies the existing Origin,
portal-header and Fetch Metadata contracts; management UI operations also keep
their route-specific revisions, idempotency and review requirements. A theme
cannot change backend authorization, cookies, policy, consent or admission.
Operator scripts must still implement those contracts if replacing the modules.

Keep scripts and styles in same-origin external files. The existing HTML CSP
continues to refuse inline code, remote scripts/styles/images, external fetches,
frames and form submission, with only `font-src 'self'` added for local fonts.
For a font in `theme-assets/fonts/team.woff2`, a stylesheet in `theme-assets/`
can use `@font-face { font-family: Team; src: url("fonts/team.woff2") format("woff2"); }`.
Error/notice pages remain script-free. CSP is not a substitute for trusting the
operator who supplies the theme or for reviewing custom JavaScript.

## Starter theme

From a source checkout matching your binary, copy the included example:

```sh
mkdir -p deployment-private/frontend-theme
cp -R examples/frontend-theme/. deployment-private/frontend-theme/
```

Configure `theme_dir = "deployment-private/frontend-theme"` beside that checkout's
`riauth.toml`, then restart. The example replaces the applications page layout,
name, logo and colors, adds a workspace banner and runs a same-origin external
UI script. It preserves the existing page IDs and authentication modules. Other
pages and baseline assets continue to use embedded defaults.

To start a different page from its exact default, copy its source from the table:

```sh
mkdir -p deployment-private/frontend-theme/pages
cp src/portal/signin.html deployment-private/frontend-theme/pages/signin.html
```

Edit layout and add a link such as
`<link rel="stylesheet" href="__BASE__portal/theme-assets/workspace.css">`.
To replace a full existing stylesheet instead, copy it into `assets/` under the
same filename. Restart after deployment and verify sign-in, cancellation, consent,
account security, administration and setup flows that your changed pages use.
The supplied theme and router tests are not a real-browser execution claim.
