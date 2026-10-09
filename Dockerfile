FROM rust:1.98.1-trixie@sha256:a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546 AS build
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libssl-dev clang && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src
COPY assets/riauth-mark.svg ./assets/riauth-mark.svg
ARG RIAUTH_EDITION=essentials
ARG CARGO_BUILD_JOBS=1
ARG CARGO_INCREMENTAL=0
ARG CARGO_PROFILE_RELEASE_DEBUG=0
RUN case "$RIAUTH_EDITION" in essentials|platform) ;; *) exit 2;; esac \
    && cargo build --release --locked --no-default-features --features "$RIAUTH_EDITION" --bin riauth

FROM debian:trixie-slim@sha256:a29215f6a35e51e22adffa17f89e9d2ef06214e64a2bad10d765c46aea49f11f
LABEL org.opencontainers.image.source="https://github.com/Rhein-Industries/riAuth"
ARG RIAUTH_EDITION=essentials
ARG RIAUTH_COMMIT
LABEL org.riauth.edition="$RIAUTH_EDITION"
LABEL org.opencontainers.image.revision="$RIAUTH_COMMIT"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3t64 && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 riauth && useradd --uid 10001 --gid 10001 --no-create-home --shell /usr/sbin/nologin riauth \
    && install -d -o 10001 -g 10001 -m 0700 /data
COPY --from=build /build/target/release/riauth /usr/local/bin/riauth
COPY LICENSE /usr/share/doc/riauth/LICENSE
COPY THIRD_PARTY_NOTICES.md /usr/share/doc/riauth/THIRD_PARTY_NOTICES.md
USER 10001:10001
WORKDIR /data
VOLUME ["/data"]
EXPOSE 9000
ENTRYPOINT ["riauth", "--config", "/data/riauth.toml"]
CMD ["serve"]
