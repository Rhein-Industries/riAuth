FROM rust:1.93.0-bookworm@sha256:d0a4aa3ca2e1088ac0c81690914a0d810f2eee188197034edf366ed010a2b382 AS build
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libssl-dev libudev-dev clang && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src
COPY assets/riauth-mark.svg ./assets/riauth-mark.svg
RUN cargo build --release --locked

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
LABEL org.opencontainers.image.source="https://github.com/Rhein-Industries/riAuth"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 libudev1 && rm -rf /var/lib/apt/lists/* \
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
