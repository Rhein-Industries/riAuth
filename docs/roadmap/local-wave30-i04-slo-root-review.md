# I04 Lasso SLO source review and held execution

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original I04 remains open.
This records source review, not a native peer execution or a whole-profile pass.

## Reserved source

Source `e028106c3d7c9a236056730ad500aad876616540` adds 267 C helper lines and
one 450-line ignored Rust fixture. Root read both complete additions and the
static report `1991672a3adf1ff9b3af5843013c009894ad3290`. Removing the new
includes, helper block and three dispatch additions reconstructs the entire
original C helper. The existing 19,341 Rust bytes are an exact prefix.
No production source or existing authentication/refusal assertion changes.

The added helper saves the identity/session accepted by Lasso, reloads it in a
fresh process, checks exact NameID/session index, processes a signed Redirect
logout using Lasso's validation, and serializes the actual emptied session.
Root read the selected Lasso 2.9 primary implementation and ownership contracts:
returned NameID lists own object references, index lists own strings, and logout
validation removes the saved assertion/index after matching the request.
The installed dylib hash matches the selected source-phase pin. Those file reads
are not a library/compiler invocation.

The new fixture defines local revocation before peer delivery, unrelated-session
preservation, wrong-metadata refusal, fresh-process saved-session presence and
absence, spent-session refusal, one server confirmation/audit, and consumed
response replay with unchanged full snapshot. These are unexecuted assertions.
Child process ownership, private captures and finite helper/test deadlines were
reviewed. No synthetic emptied-session marker, clock repair, direct ledger write
or trust relaxation is accepted.

## Alignment and preparation

Preparation report `020a32429863d2ca656b794d634efe3b8d91e70f` was read fully.
The history-preserving merge `abd01f6a51307011a7c41c711b3aeaec09341614` retains
published `ae8937800254a1ad4296ea257de1eccc4780e45b`. Root independently compared
the complete `src` and `crates` trees, both Cargo inputs and toolchain object IDs;
all equal that published pin. The only report conflict retained the entire
supporting report, whose original prefix equals the published report.
The reviewed C/test blobs remain `ce0a928adb22a4d00a612cf3fb7a113125303e48` and
`917364806b5be106d9e102bd5d30ba93455a8f0f` respectively.

The read-only cache inventory estimates 2–4 GiB additional peak for the proposed
default-feature test, with uncertain cache reuse. At about 11.13 GiB free, the
upper estimate exceeds the margin above the 9 GiB stop point. Root therefore
keeps the compiler and Cargo invocation held, pending serialized predecessors
and a fresh credible capacity check. No cache deletion is authorized.

The report supplies one bounded five-second pkg-config child and one 60-second
compiler child, private captures and owned process-group cleanup. The proposed
ignored filter is only
`lasso_idp_initiated_redirect_logout_revokes_only_bound_session_and_consumes_response_once`.
No compiler, helper, Lasso protocol, Rust typecheck or Cargo invocation has run
for this slice. A separate exact release is required for each execution step.

## Disposition

Accept the preserved source and preparation evidence for review. I04 remains
in progress: the real-library SLO receiver oracles remain pending, and this
slice does not establish every SAML/LDAP/RADIUS/RadSec/EAP profile, tenant,
device, release or independent operator outcome. Primary ownership is unchanged.
D04/O06/I10/R05 remain closed; O07's observed mapping blocker is preserved.

Root verification here consists of full changed-body/report reads, original
source reconstruction, selected primary-library reads, immutable source/input
comparison, documentation and whitespace checks. No root native/product runtime
was executed. The worker's corrected literal-ignore annotation checker is a
static verification correction, not a product failure.

## Own cache inventory and continuing capacity hold

Root fully read b02e9dea2c819b3d8ba9a0abc2603e653b3f8d60. Its only change is
a148-line report append; source/test and all earlier report prefixes remain
unchanged. The private600 inventory contains an EMPTY cleanup allowlist and
zero proposed relief. All six integration binaries have accepted execution
citations; absence of a historical executed-file hash cannot prove nonexecution.
Their combined0.8577GiB would also not provide the requested2GiB relief. Fresh
lsof/ps metadata snapshots show no current use, not historical nonexecution.
No deletion is authorized by this review.

Fresh reported free space10.8849GiB remains below the proposed13GiB planning
start for the estimated4GiB upper transient allowance above9GiB stop. Compiler
and ignored Cargo test remain held, with no helper/native/runtime evidence.
The unrelated later SCIM repeat owns the serialized Cargo lane. This resource
hold does not establish a product or protocol defect, and I04 remains open.

## Separately reviewed root cache relief and compiler release

Root subsequently reviewed14 further unused regenerable CI-prewarm executables
in the other private cache, using its immutable64-candidate inventory. Root
revalidated exact parent, nonsymlink regular identity, UID, single link,755 mode,
size/mtime/SHA and matching old test-support-without-fuzzing fingerprints.
Fresh14-path lsof and ps comm snapshots had no reference, and accepted tracked
docs/planning contained none of their basenames/hashes. The historical
never-executed classification is attributed to the prior CI worker inventory.
No cited run artifact, product, helper, library, fingerprint, archive or log
was removed. The exact new root allowance was exhausted by one14-path removal.

The [prune receipt](evidence/wave30-root-unused-prewarm-prune.json) retains
all14 identities,2835757632 bytes, rm exit0/all absent, deps2043→2029, and
free bytes11712270336→14548258816. The original inventory is unchanged.
A first root verification guard used the executable basename for a Cargo
fingerprint directory and failed before mutation; using actual riauth-hash
directories passed. Earlier TSV header/filter wrapper mistakes likewise
changed nothing. I04's own empty allowlist remains empty.

With about13.55GiB observed free and SCIM released, root separately released
ONE bounded pkg-config5s plus compiler60s preparation step per020a324.
No helper/protocol or Cargo invocation is included. Fresh source/library
identity and disk preflight, private capped captures,9GiB stop/8floor and owned
process-group cleanup are required. Compile failure stops without correction
or retry. The ignored Rust filter stays held pending actual compile success
and fresh capacity; this source decision adds no interoperability result.

## Actual compiler output and first ignored filter

Root fully read compiler receipt f481938a999cf3c95e953f0c1ddd402be9706364
and independently hashed the selected C source, library, four private captures
and output executable. One pkg-config metadata child and one cc-O2 child exited0;
the wrapper completed in0.356856 seconds. The39472-byte arm64 Mach-O helper
has SHA-25618f148c0a3119d4a1268597c829c680a743c4978ccc96924dac337368d3a891c.
This establishes compile/link output and load names, not native lifecycle success.
Preparation released before its report; root then separately released one filter.

That exact ignored filter compiled in56.22 seconds and exited101:0passed1failed
1filtered, test2.58 seconds, wrapper59.686627 seconds. Root read all1304 raw log
bytes and hashed private0600 log a253f09fb45a4641d8373eb9318ca66bf11ff237e7570b490b24d02d6c2b4f03
and supervisor metadata22104b9b038bf784a28d51e5d05b360715be17a11a2f60544296225ad9eed2d6.
The owned group63914 was empty after joined exit; no cleanup signals or deadline/
disk stop occurred. Thirty disk samples had minimum13705166848 bytes.

The assertion at tests/saml_sp_peer.rs897 combines native exit1 with stderr
containing(-111), but prints neither actual native status nor finite Lasso code.
Which conjunct failed is unknown. Earlier SSO persistence/reload, local revocation
before peer delivery, unrelated identity and exact issued NameID/index checks
completed. Negative no-output/unchanged-input assertions and all positive logout
processing, session retirement, response confirmation and consumed-replay checks
were not reached. No signature bypass or expected-code correction is established.
Cargo released immediately before reporting. No repeat is authorized by this
review; a finite status/stage/code observation must precede any concrete correction.

## Finite refusal projection: source only

Root fully read bf7c394150141f8dda6b41c6c0644dd6a9a7d870 and the161-line
static appendix328228552f3b07240482bbf3dd70c8d60242221c. Only the appended
ignored function changes. Numeric exit/signal and an optional tuple of one of
11 static C-stage literals plus canonical i32 are added to the failed assertion;
no raw stderr/message/path/protocol/name/index is printed. C/helper/library are
unchanged. Missing, malformed, unknown or multiple failure-family lines yield
unclassified None. The boolean exit1-and-contains(-111) remains exact.

Root removed the local projector, reversed the local import and failure-message
arguments, and reconstructed the entire e028 test37798 bytes exactly, retaining
the19341-byte original prefix. New source SHA-256 is
862564f9154cd1de98378bc853e47948a731b994d22a1cf8d5be94083a1f52da.
This is not a changed refusal oracle or an explanation of the prior failure.
The diagnostic remains uncompiled/unrun at review. Its same ignored warm-cache
filter stays held while I02 owns Cargo. A later release requires fresh source/
helper/library/cache/settings and capacity review, without C rebuild, additional
filter, blind retry, threshold reduction or deletion. The earlier cold-build
13GiB planning condition is not an immutable peak measurement; unchanged warm
libraries may justify a separately reviewed smaller allowance. No reservation
or execution is inferred merely from this source review.

## Second finite observation and selected-library refusal expectation

Root read the full `56c75be46a33c0485243dddaf5e64b40b740f2b7` actual appendix and reverified the 1,388-byte private log SHA-256 `17783cf450501bac6f621674a13285740a23ac892e9a39ce589391e4f6fd3c40` and evidence `6d326ac9e6697e9fe1c1ef5f44ff5c65f3f53501a6f7d22c0d15d348958a8e86`. The one same ignored filter compiled in 2.66 seconds and failed in 2.27 seconds: Cargo 101, zero passed, one failed, one filtered. This invocation observed native exit 1, no signal, fixed stage `lasso_logout_process_request_msg`, integer code 102. The prior run's unretained native result remains unknown. Positive receiver retirement, confirmation and replay checks remain unreached. Owned group 12249 was reaped and empty; Cargo was released before report work.

Root hash-verified the primary Lasso 2.9.0 archive `63816c8219df48cdefeccb1acb35e04014ca6395b5263c70aacd5470ea95c351`, read its errors header (identical to installed SHA-256 `00d4659947b59cecaff5c63911811eb1b65440567a9ab974c2d25ff7bdfc6784`), logout request-processing body and profile request/signature-check bodies. The header defines `LASSO_DS_ERROR_INVALID_SIGNATURE` as 102. Redirect processing assigns verification status; the forced signature check propagates it through logout processing. This supports a narrow exact selected-library expectation correction, retaining exit 1 and requiring the fixed projected stage/code 102, rather than the stale text `(-111)`. It does not permit arbitrary failure or relax signature verification. Only the appended test assertion and report are reserved; C/helper/production and all negative/positive postconditions remain protected. No further runtime is released by this source reservation.

## Corrected selected-library filter: actual local lifecycle pass

Root read the complete new 758-byte mode-0600 log, SHA-256 `11969084b9a31173adb95c9dcd4eb21022525f6f0d6b8bb8bacb717303d0704e`, and 10,177-byte evidence `55fc3bb6b237f231de4cf684436edf256d8377d6b07eb9e2cbc905ead2e2d7b7`. On clean 47a5/source61c, the one released same ignored filter exited 0: compile 2.59 seconds, one passed, zero failed, one filtered, test 2.09 seconds, total 5.581404 seconds. Source/helper/library/manifests stayed pinned; no C rebuild, additional target, native probe or retry followed.

The passing function reaches the exact exit-1/request-processing/code-102 assertion, negative no-output/unchanged-state checks and positive real Lasso matched-session retirement, fresh reload, signed correlated response, one confirmation/audit and consumed-response replay assertions. Its failure-only diagnostic prints nothing on success: no new native tuple was separately captured. The earlier finite diagnostic supplies its own tuple, and the first historical failure remains unknown. Minimum free space was 12,146,814,976 bytes; owned group 45365 was reaped and empty, with no cleanup signals, before immediate Cargo release. This establishes the selected local macOS Lasso 2.9 lifecycle regression, not whole I04, Linux, tenant, browser or release acceptance. Worker append-only actual report remains pending at this root entry.
