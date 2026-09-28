#!/bin/sh
set -eu

RIAUTH_CROSS_BUILD_DIR="$(mktemp -d)"
export RIAUTH_CROSS_BUILD_DIR
trap 'rm -rf "$RIAUTH_CROSS_BUILD_DIR"' EXIT

cargo test --locked --no-default-features --features platform --test edition_transition_cross_build -- --ignored
cargo test --locked --no-default-features --features essentials --test edition_transition_cross_build -- --ignored
