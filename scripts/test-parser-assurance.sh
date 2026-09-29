#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS=2

# A finite regression/corpus campaign. No nightly tooling or network peer needed.
cargo test --locked --features test-support,fuzzing --lib fuzzing::tests
cargo test --locked --features test-support,fuzzing --lib scim::patch_tests
cargo test --locked --features test-support,fuzzing --lib ldap_server::ber_tests
cargo test --locked --features test-support,fuzzing --lib provisioning::tests::outbound_response_parser_keeps_remote_identity_bound
cargo test --locked --features test-support,fuzzing --lib saml::wire::signed_redirect_tests
cargo test --locked --features test-support,fuzzing --lib jose::signed_claim_tests
cargo test --locked --features test-support,fuzzing --no-fail-fast \
  --test fuzz_corpus --test parser_assurance --test operations -- --nocapture
