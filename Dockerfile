# syntax=docker/dockerfile:1
#
FROM --platform=$BUILDPLATFORM rust:1-bookworm AS dashboard

RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked

WORKDIR /code
COPY Cargo.toml Cargo.lock /code/
COPY crates/domain /code/crates/domain
COPY crates/app/Cargo.toml /code/crates/app/Cargo.toml
COPY crates/adapters/adapter_dashboard_leptos /code/crates/adapters/adapter_dashboard_leptos
COPY crates/adapters/adapter_http_axum/Cargo.toml /code/crates/adapters/adapter_http_axum/Cargo.toml
COPY crates/adapters/adapter_storage_sqlite_sqlx/Cargo.toml /code/crates/adapters/adapter_storage_sqlite_sqlx/Cargo.toml
COPY crates/adapters/adapter_virtual/Cargo.toml /code/crates/adapters/adapter_virtual/Cargo.toml
COPY crates/adapters/adapter_mqtt/Cargo.toml /code/crates/adapters/adapter_mqtt/Cargo.toml
COPY crates/adapters/adapter_ble/Cargo.toml /code/crates/adapters/adapter_ble/Cargo.toml
COPY crates/bin/minihubd/Cargo.toml /code/crates/bin/minihubd/Cargo.toml

RUN set -eux; \
    for crate in domain app; do \
    mkdir -p "crates/${crate}/src" && touch "crates/${crate}/src/lib.rs"; \
    done; \
    for adapter in adapter_http_axum adapter_storage_sqlite_sqlx adapter_virtual adapter_mqtt adapter_ble; do \
    mkdir -p "crates/adapters/${adapter}/src" && touch "crates/adapters/${adapter}/src/lib.rs"; \
    done; \
    mkdir -p crates/bin/minihubd/src && echo "fn main() {}" > crates/bin/minihubd/src/main.rs


# The dashboard has its own Cargo.toml (excluded from workspace).
WORKDIR /code/crates/adapters/adapter_dashboard_leptos

RUN trunk build --release

FROM --platform=$BUILDPLATFORM rust:1-bookworm AS vendor

WORKDIR /code
COPY Cargo.toml Cargo.lock ./
COPY crates/domain/Cargo.toml /code/crates/domain/Cargo.toml
COPY crates/app/Cargo.toml /code/crates/app/Cargo.toml
COPY crates/adapters/adapter_http_axum/Cargo.toml /code/crates/adapters/adapter_http_axum/Cargo.toml
COPY crates/adapters/adapter_storage_sqlite_sqlx/Cargo.toml /code/crates/adapters/adapter_storage_sqlite_sqlx/Cargo.toml
COPY crates/adapters/adapter_virtual/Cargo.toml /code/crates/adapters/adapter_virtual/Cargo.toml
COPY crates/adapters/adapter_mqtt/Cargo.toml /code/crates/adapters/adapter_mqtt/Cargo.toml
COPY crates/adapters/adapter_ble/Cargo.toml /code/crates/adapters/adapter_ble/Cargo.toml
COPY crates/bin/minihubd/Cargo.toml /code/crates/bin/minihubd/Cargo.toml

RUN set -eux; \
    for crate in domain app; do \
    mkdir -p "crates/${crate}/src" && touch "crates/${crate}/src/lib.rs"; \
    done; \
    for adapter in adapter_http_axum adapter_storage_sqlite_sqlx adapter_virtual adapter_mqtt adapter_ble; do \
    mkdir -p "crates/adapters/${adapter}/src" && touch "crates/adapters/${adapter}/src/lib.rs"; \
    done; \
    mkdir -p crates/bin/minihubd/src && echo "fn main() {}" > crates/bin/minihubd/src/main.rs

# https://docs.docker.com/engine/reference/builder/#run---mounttypecache
RUN --mount=type=cache,target=$CARGO_HOME/git,sharing=locked \
    --mount=type=cache,target=$CARGO_HOME/registry,sharing=locked \
    mkdir -p /code/.cargo \
    && cargo vendor > /code/.cargo/config.toml

FROM rust:1-bookworm AS builder

# Install cross-compilation toolchains when needed
RUN apt-get update \
    && apt-get install -y --no-install-recommends libdbus-1-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /code

COPY --from=vendor /code/.cargo /code/.cargo
COPY --from=vendor /code/vendor /code/vendor

# Copy full workspace source
COPY Cargo.toml Cargo.lock /code/
COPY crates /code/crates

# Build for the target architecture
RUN cargo build --release --locked --offline

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 minihub \
    && useradd --uid 1000 --gid minihub --create-home minihub

WORKDIR /app

COPY --from=builder /code/target/release/minihubd /app/minihubd
COPY --from=dashboard /code/crates/adapters/adapter_dashboard_leptos/dist /app/dashboard

RUN mkdir -p /app/data && chown -R minihub:minihub /app

USER minihub

ENV MINIHUB_HOST=0.0.0.0
ENV MINIHUB_PORT=8080
ENV MINIHUB_DATABASE_URL=sqlite:///app/data/minihub.db
ENV MINIHUB_DASHBOARD_DIR=/app/dashboard

EXPOSE 8080

VOLUME ["/app/data"]

ENTRYPOINT ["/app/minihubd"]
