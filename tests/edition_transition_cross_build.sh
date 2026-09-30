#!/bin/sh
set -eu

for storage in plain encrypted; do
    RIAUTH_CROSS_BUILD_DIR="$(mktemp -d)"
    export RIAUTH_CROSS_BUILD_DIR
    trap 'rm -rf "$RIAUTH_CROSS_BUILD_DIR"' EXIT
    if [ "$storage" = encrypted ]; then
        RIAUTH_CROSS_BUILD_ENCRYPTED=1
        export RIAUTH_CROSS_BUILD_ENCRYPTED
    else
        unset RIAUTH_CROSS_BUILD_ENCRYPTED
    fi
    cargo test --locked --no-default-features --features platform --test edition_transition_cross_build -- --ignored
    cargo test --locked --no-default-features --features essentials --test edition_transition_cross_build -- --ignored
    rm -rf "$RIAUTH_CROSS_BUILD_DIR"
    trap - EXIT
done
