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
