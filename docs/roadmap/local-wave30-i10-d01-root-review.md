# I10 diagnostic correction and D01 confidential application review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. This review distinguishes the
I10 operator-interface gate from the selected D01/D05 documented application
journey. No root local product build, test, service or browser was executed.

## I10: active Entra private-file diagnosis

Source `ef80983e13fa5923ae5c2a29b520e415e1205240` (accepted port `61bff87`)
changes only the Entra credential selector and appends one focused regression.
Root read both changes fully. Replacing only that selector with the original
secret-file call reconstructs the entire published assembly source at
`9a819317efb3a13fa27cd86f884be2be00898fc0`. Every pre-existing cloud fixture is
an exact byte prefix. No credential helper, configuration, signing/token,
provider, browser, authority, API, Store or writer change is included.

Supported certificate mode requires an empty shared-secret path and nonempty
certificate/private-key paths. The selector now reads that private key using
the existing 16,384-byte bound; all other modes keep the existing 4,096-byte
shared-secret check. Fixed shape, scope-first authorization,
`provider_verified=false` and next-token reload semantics remain unchanged.
A readable file does not establish PEM, pair validity or provider acceptance.

Root released exactly one locked `test-support,fuzzing` cloud-directory filter:
`cloud_operations_entra_certificate_private_file_status_is_scoped_and_redacted`.
At report-only HEAD `0e0d658f1f148d16db50c7f9efa6e78f6b8e6502`, on unchanged
reviewed source bytes, it passed **1/0, 47 filtered, 2.12 seconds**. Build was
1m06s; monitored invocation 72.178s. There was no failure, correction or repeat.
The only warning was the recorded native compact-unwind linker warning.

Root independently read the complete actual log and observation JSON and
rehash-checked both private 0600 files:

- log SHA-256 `8561bc484c37c18e7589924c5e4b84d18d86e5a77b9734dfdc9cfdd264468502`;
- observation SHA-256 `11d09656ca2681b3600dd6260ce4852ed7c38401462b811b837f428b0e531a48`.

The observation records exact argv/source/HEAD, Darwin 25.2 arm64,
Rust/Cargo 1.98.1, owned Cargo PID 50974 reaped and 37 nominal two-second disk
samples. Minimum 10,429,796,352 bytes (9.714 GiB), completion 10,688,733,184
bytes (9.955 GiB) remained above the 9 GiB stop and 8 GiB floor. The worker
released the sole Cargo slot immediately after exit, before its report append.
This was a local private-file diagnostic regression, not a tenant, Linux,
release, remote delivery or provider-health test.

The passed function covers generated certificate/key files, real pair
replacement and deterministic modified metadata, 16-KiB exact boundary and
oversize, missing/nonregular/Unix permission refusal, unchanged secret bound,
redaction, scoped refusal and complete durable snapshot equality. No peer token
or directory request was made. Authority-before-selector is additionally
verified by unchanged source order; no instrumented file-open claim is made.

### Original I10 disposition

Root reread the original requested seven facets and the exact supported
LDAP/SCIM/Workspace/Entra catalog. The dated full matrix at worker
`cae69dd2f31e98cb80b5a15b173cb1cbba37eb8f` is preserved in the
[operator-interface report](local-wave30-i10-operational-interface-plan.md).
Configuration validation, mappings, intentional connection checks, bounded
controller schedules, file rotation, scoped history and fixed operator errors
have connected Core/API/configuration/CLI paths. LDAP's authenticated planning
crawl is a deliberate connection path, with draft writes distinguished from
account apply. Cloud scoped probes and credential verification are explicit.
The new SCIM first-page probe closes its pre-delivery gap with eight actual
fresh passes; it does not claim complete crawl, Groups/filter or write capability.

Accepted historical OpenLDAP, stored-definition/CLI, outbound two-router and
cloud lifecycle/authority fixtures remain credited only at their executed pins.
Their mocks and local peers do not become live SaaS, AD or tenant observations.
The only demonstrated local operational residual in that matrix was the wrong
Entra file selector; the reviewed correction and focused actual pass resolve it.

Root considers the **original local operator-interface outcome a completion
candidate after integration/publication**. No extra GUI button, duplicate CLI
alias or universal new campaign is imposed. Named tenant delegation/consent,
cloud certificate acceptance, real SaaS/AD configuration, multi-node deployment
and shipped artifacts remain explicit external inputs, owned by their original
rows; none is inferred healthy. I04 real-peer profiles, I06 tenant rollout and
P08 downstream delivery retain their distinct gates. Status remains unchanged
until the source and actual evidence are published and the exact live row is
updated by root. The concurrent CI SCIM OAuth failures are separately under
investigation; this review does not assert a current green broad suite.

## D01: reviewed confidential browser adapter, runtime pending

Helper source `0b0d15cda6e6c61388ce40338de707f84851989f` (accepted port
`8a04979`) adds only `scripts/d01-confidential-browser-demo.py`, 651 lines,
29,933 bytes, SHA-256
`d8446bfdf22f2a345b019673fe93828d5f75024a87b530b100eb6c027da9a863`.
Root read the complete source and the reused verifier's discovery,
callback parsing, signature/claims validation and HTTP/process code. AST and
in-memory compilation passed without import or execution. Entry is guarded;
no product, guide, existing verifier, dependency or crypto implementation changes.

The printed client is confidential `local-demo`, fixed issuer
`http://localhost:9000` and exact callback `http://localhost:3000/callback`.
Owner-only bounded credential input is validated without printing its secret.
The adapter initiates browser authorization with fresh state/nonce/S256 and a
browser-flow cookie; consumes one exact callback, performs client-secret-post
code exchange, and delegates RS256/JWKS/issuer/audience/nonce/time/access-hash
validation to the hash-pinned recovery verifier. Userinfo subject agreement
precedes a fresh protected application cookie. It never invokes the verifier's
scripted login or supplies an IdP service session/approval substitute.

The reused verifier bytes are checked before loading and no constructor or
listener from that module is called. Native OpenSSL executable/version/hash
are explicitly pinned. Headers, request lines, body shape, Host/Origin,
callback fields, own cookies, redirects, HTTP/process operations and evidence
are bounded. Unrelated localhost cookies are tolerated while duplicate own
cookies refuse. Fixed finite failure tags avoid raw query/secret/error logs.
The single-thread POSIX alarm monitors free space and total/pending/phase time;
Halt traverses exception wrappers into owned finally cleanup. Listener,
connection, verifier temporaries and private references are checked at exit.
Outer owner still supplies the exact bounded IdP/browser lifecycle and final
60-second cleanup, isolated Driver session and private-lab deletion proof.

This is a deliberately supplied example-app input to the documented section 3,
not a discovered product or printed-guide defect. Prior actual operator,
password-portal and public recovery RP observations remain distinct. Actual
confidential sign-in, consent, exact callback and fresh protected access are
**not yet executed**. Root will separately release one reviewed checkpoint
through RiWork Cua.ai Driver only, with matching c01 build artifacts and the
published guide, finite owned listeners and no private-value evidence. D01/D05
status remains in_progress at this phase.

## Publication decision and concurrent CI boundary

The complete actual Entra appendix `d9c5309a5f27eccc2425564b58ac4ce2ac17a369`
is ported as `f53d41a`; root compared its entire previous report prefix before
integration and read the actual appendix. D01 source-only evidence
`6b3a1790ff80cb205bb403854725a779541adbac` is ported as `7b3dc9e`; all previous
browser/plan bytes are preserved. Repository documentation and tracked-file
hygiene checks pass on the combined accepted tree, and Git whitespace is clean.

Root's exact live I10 read confirms the unchanged original seven facets,
project/task/worktree, prerequisites and in_progress status. Following this
publication, root accepts the original local I10 operational-interface outcome
and will mark that row DONE. This decision credits inspected connected source,
accepted historical setup/lifecycle/refusal evidence, the actual eight-case
SCIM probe and actual certificate-mode diagnostic regression, rather than a
report alone. Release/tenant and other original protocol-profile rows remain
open where their inputs and gates remain unmeasured.

Public CI `37013383299` at `9a819317efb3a13fa27cd86f884be2be00898fc0`
finished with audit and integration successful, check failed. The corrected
reports target passes and the next SCIM OAuth target records 16 passed,
8 failed, 18.66 seconds. Root independently downloaded and read its exact
failure block: private log 312,377 bytes, SHA-256
`d71f45cbf65285336738a99a9d77e48201e09d0010757e638e47dbb205e84a64`.
This is preserved in the separate
[CI diagnosis](local-wave30-scim-oauth-ci-report.md), accepted port `3a214a0`.

The inspected common path is direct Core stepping with no persistent App
executor holder, causing queued target-admission releases to lose their
executor. The proposed fixture holders retain the same executor, including
across the private staff-removal helper return. Root approved only those
fixture-lifetime hunks and one later target after source review; no admission,
lease, production, assertion, retry/page/removal/rotation or receipt semantics
may change. Those hunks are still unexecuted at this review. The historical
Linux failure, local I10 passes and current ARM artifact run are separate
observations; no current broad-green CI or Linux correction is asserted.
# Dated D01 provider preflight follow-up

The separately released fixture stopped during preflight, before creating a lab,
credentials, store, server, helper or browser. Worker evidence
`93643763a1fc3360491a0a4a32424fd353e92cc0` retains and withdraws its initial
hash-mismatch/no-provider-execution report: the provider hash matched and one
version subprocess ran, but its exit/output were lost after a combined assertion.
Root read the full appendix. No application outcome follows from that attempt.

Root separately authorized one five-second provider diagnostic, whose private
1103-byte mode0600 metadata has SHA-256
`8f47d8fbd384ccf9746de938ef5c6e9cbba8d0e072484b443e2c2cbc4fa7439d`.
Root independently read and rehashed the metadata. That distinct invocation
returned zero, 63 ASCII stdout bytes and empty stderr, with the matching artifact
hash. Its complete text is `OpenSSL 3.6.4 25 Aug 2026 (Library: OpenSSL 3.6.4
25 Aug 2026)` plus newline. The first invocation's lost values remain unknown.

Diagnostic report `5c7665c0e1cd9f23ad6c8d912cdc67ab27609a2c` and static report
`fb9f474db779fb655b43026f1089be19661bb645` retain all previous report bytes.
Source `16395f1be5a65b0d3cb0a1ad2d88daa0c17b7e1b` changes only the expected
version constant to that exact full text. Root reversed the one constant and
reconstructed the entire original helper, parsed the new source and compiled a
code object in memory without executing it. Every other guard and flow byte
remains unchanged. The new helper SHA-256 is
`f94af72ab613bc332ea684e5c4244c38d53c6540e1ea3d883938aecd265ec6fc`.

No prefix match, alternate provider, relaxed hash/path/ASCII/output/time guard or
cryptography change is accepted. The metadata establishes the concrete expected
text correction, not a browser pass. A new full fixture requires publication,
separate root release and fresh capacity at or above 8.5 GiB. Desktop is free;
A09 continues to own the independent serialized Cargo slot. D01/D05 remain open.
