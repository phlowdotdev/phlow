FROM rust:1.87.0 AS builder

WORKDIR /app

COPY . .

RUN cargo build --release -p phlow-runtime

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/phlow-runtime /usr/local/bin/phlow-runtime
RUN apt-get update && \
    apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
ENTRYPOINT ["/usr/local/bin/phlow-runtime"]
