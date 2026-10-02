# A09 actual native ARM64 artifact review

The single corrected manual job [37016520583](https://github.com/Rhein-Industries/riAuth/actions/runs/37016520583)
completed successfully on native Linux ARM64. Workflow definition
`036a392656b4b5070cc86a11d5ca3258b7b868d2` and product source
`9a819317efb3a13fa27cd86f884be2be00898fc0` are distinct verified pins.
The original HTTP422 dispatch refusal and invalid-workflow push metadata remain
in the earlier reports. No automatic retry or guard reduction occurred.

Root downloaded the one uploaded artifact and read its full evidence. The
[retained receipt](evidence/wave30-a09-native-arm64-37016520583.json) contains the
five archive/member hashes, exact command results, inputs and resource summary.
Root rehashed every archive, extracted member in memory and step log, compared
inputs to pinned Git bodies, and inspected the native ELF ARM64 headers without
executing any binary. LICENSE and THIRD_PARTY_NOTICES match pinned source.
GitHub's outer artifact digest is recorded as API-reported; the extracted
download does not establish a locally rehashed outer ZIP digest.

Three sequential locked release builds produced Essentials and Platform servers,
both matched maintenance tools and the base client. The observed server
capabilities identify the expected editions/build features. The unchanged
focused archive checker completed successfully, exercising route exposure,
Essentials agent-scope refusals, Platform-only state and direct/preflight
downgrade refusal. Root read that checker fully; it does not establish the
complete ordinary-user/client identity and shared configuration transition gate.

The actual runner reports four CPUs and 16,722,042,880 bytes RAM. Across 1,308
resource samples, minimum host/workspace/private free space was
112,151,941,120 bytes (104.45 GiB). Maximum observed sample interval was
5.40448 seconds; nominal monitoring is not a hard storage quota. The actual
allocation passed the 30 GiB launch guard despite the published runner reference
not guaranteeing it. No 10 GiB stop or 8 GiB floor breach was observed.
Cleanup error list is empty. No independent post-VM process inventory is claimed.

A09 remains open. This is five LOCAL ARM64 artifacts at the exact supplied
product pin, not an official release, registry/container cohort, x86 cohort,
physical factor, TLS/deployment or full shared-gate pass. The evidence explicitly
records `shared_full_gate=not_run` and `official_release=false`. The next bounded
source proposal must preserve exact uploaded artifact provenance and test current
format3 behavior; the historical format2 encrypted helper is not a current gate.

The job's serialized Cargo reservation was released on observing completion.
The queued local SCIM invocation was first offered to the existing CI shell,
which refused before Cargo because its login expired. Root did not operate that
login. The same single target was reassigned to an existing Sol worker with a
history-preserving fixed-source alignment and fresh capacity preflight.
That local result is separate from this remote artifact receipt.
