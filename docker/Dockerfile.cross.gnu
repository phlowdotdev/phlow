# syntax=docker/dockerfile:1.4

ARG TARGETARCH
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY modules modules
COPY phs phs
COPY phlow-sdk phlow-sdk
COPY phlow-engine phlow-engine
COPY phlow-runtime phlow-runtime

ARG TARGETARCH
RUN rustup target add \
    $(test "${TARGETARCH}" = "arm64" && echo aarch64-unknown-linux-gnu || echo x86_64-unknown-linux-gnu)

RUN cargo build --release --target $(test "${TARGETARCH}" = "arm64" && echo aarch64-unknown-linux-gnu || echo x86_64-unknown-linux-gnu) -p phlow-runtime

FROM debian:bullseye-slim AS final
WORKDIR /app
COPY --from=builder /app/target/*/release/phlow-runtime .
ENTRYPOINT ["./phlow-runtime"]
