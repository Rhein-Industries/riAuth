# Published 88790de — completed integration job review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Root read the completed integration job of
[CI run 37006213470](https://github.com/Rhein-Industries/riAuth/actions/runs/37006213470),
job `110834995871`, at published source
`88790deb62d32c84fa17dceb12cd93a727224e94`.
The job is **success**. Audit is also success. At this observation the separate
check job is still running its full-target tests; whole-run success is not claimed.

The downloaded raw integration log is 265774 bytes, SHA-256
`0d327b38cb4999bedc20df3f739eafac161baa6d571aa3a81a4d139189c67859`,
retained at `/tmp/riauth-wave30-ci-37006213470-110834995871.log`.
The root verified its checkout SHA, named command blocks and returned results.
These are remote CI executions, not new root-local services or tests.

| Actual selected job command / stage | Result |
| --- | --- |
| Identity `saml_independent_xmlsec` filter, locked / ignored | 1 passed, 177 filtered, 7.07 s |
| Identity `saml_source_independent_xmlsec` filter, locked / ignored | 1 passed, 177 filtered, 5.81 s |
| Identity `saml_logout_independent_xmlsec` filter, locked / ignored | 1 passed, 177 filtered, 7.09 s |
| `scripts/test-postgres.sh` default target | 2 passed, 17.33 s |
| `bash scripts/test-contracts-postgres.sh` selected shared contracts | 91 passed, 90 filtered, 211.58 s |
| Secure first-administrator browser setup | 9 passed, 53.5 s |
| Headless portal authenticator journeys | 22 passed, 2 skipped, 6.5 min |
| PostgreSQL Q05 replay/concurrency selected schedule | 1 passed, 1 filtered, 14.15 s |
| `scripts/test-ldap.sh` disposable OpenLDAP | 1 passed, 8.15 s |
| `--test browser --locked -- --ignored` with selected Chrome | 1 passed, 11.83 s |
| `--test portal_browser --locked -- --ignored` with selected Chrome | 1 passed, 10.40 s |
| `--test outpost --locked -- --ignored` with nginx / Chrome | 1 passed, 1 filtered, 17.77 s |
| `--test outpost_traefik --locked -- --ignored` with selected Traefik | 1 passed, 13 filtered, 3.15 s |

The 91 are selected shared PostgreSQL contracts, not 91 O03 checks, recipe
walkthroughs or all database targets. Browser setup uses Chromium, Firefox and
WebKit. The two skipped portal cases remain skipped; the aggregate is not 24
passes or physical-authenticator evidence. The named Rust browser test is
`browser_terminal_login_callback_and_signed_backchannel_logout`; LDAP is
`openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync`; nginx is
`nginx_forward_auth_terminal_sso_headers_and_revocation`; Traefik is
`traefik_forward_auth_real`.

These fresh published-source peer/fixture results supplement the precisely dated
[D03 recipe audit](local-wave30-d03-tested-recipe-plan.md) and
[D05 evaluation](d05-acceptance-evidence.md). They do not relabel earlier failed
runs, introduce a release artifact, or execute every recipe's printed CLI.
The independent [D01 password browser checkpoint](local-wave30-d01-user-browser-review.md)
has separate macOS, Driver and c01-binary provenance; it is not these CI journeys.

No actual customer tenant, physical key/phone, deployment secret reprovisioning,
physical PITR promotion, production multi-node availability or paused-I/O proof
is inferred. Accepted protocol, receipt, header, PAM, Group and lease limitations
remain. No task status changed from this job observation.

Root validation: raw log hash/checkout/command-result inspection, documentation
link/layout check and Git whitespace. Root ran no Cargo, service or browser.
