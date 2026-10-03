# Completed CI at source 143d99d: root review

Run **37098106793**, source **143d99dbe43fefd6aeb3d274a0d974fd025292e0**, completed successfully in audit, check and integration. Root retained completed job metadata and hashed the complete check/integration logs before attribution. [The receipt](evidence/wave30-ci-37098106793-completed-root-review.json) records identities, counts and limits. Earlier failing runs remain failed.

The check command `cargo test --all-targets --features test-support,fuzzing --locked` produced 182 result blocks: 1330 passed, zero failed, 182 ignored. These are counts within that command, not a distinct global test inventory. The named background-capacity regression passed. Selected complete targets:

| Target | Actual result |
| --- | --- |
| reports | 12 passed, zero failed, 8.40s; attribution test passed |
| scim_oauth | 24 passed, zero failed, 26.70s |
| source_stage | 24 passed, zero failed, 40.33s; browser transport rendering fixture passed |
| saml_sp_peer | zero passed, two ignored; local Lasso evidence remains separate |
| s02_group_listing_paging | zero passed, one ignored; local materialization evidence remains separate |

The completed integration job has its [separate root review](evidence/wave30-ci-37098106793-integration-root-review.json). It credits the actual scoped XMLsec, PostgreSQL, OpenLDAP, RP, portal, proxy and browser results at this same source. Two browser skips remain skips.

Product `src`, `crates`, manifests, toolchain, build.rs and .cargo paths match published 544d1340 byte-for-byte; helper/workflow/doc differences are excluded from that comparison. This is one actual successful CI run at 143d99d, not a later-source, shipped-artifact, tenant/device or rare-race-eradication claim. ARM run 37101183416 remains a separate active validation. No root Cargo or retry was executed.

The first parser index missed ANSI-colored target headings. Root stripped those headings and rechecked named results before this receipt. Multiline step headers are labeled as such; they are not silently assigned as the producing command for every later result.
