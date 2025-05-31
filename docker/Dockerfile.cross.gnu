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
ENV RUST_TARGET=x86_64-unknown-linux-gnu

RUN if [ "$TARGETARCH" = "arm64" ]; then \
        rustup target add aarch64-unknown-linux-gnu && \
        export RUST_TARGET="aarch64-unknown-linux-gnu"; \
    else \
        rustup target add x86_64-unknown-linux-gnu && \
        export RUST_TARGET="x86_64-unknown-linux-gnu"; \
    fi && \
    echo "Building for target $RUST_TARGET" && \
    cargo build --target $RUST_TARGET --release -p phlow-runtime

FROM debian:bullseye-slim AS final
WORKDIR /app
COPY --from=builder /app/target/*/release/phlow-runtime .
ENTRYPOINT ["./phlow-runtime"]
