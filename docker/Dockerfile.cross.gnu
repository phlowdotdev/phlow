FROM rust:1.87.0 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release -p phlow-runtime

RUN ls

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/phlow /app/phlow

RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["/app/phlow"]
