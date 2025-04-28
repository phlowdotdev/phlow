###############################################################################
# Phlow Runtime
###############################################################################
FROM rust:1.86-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    zlib1g \
    liblzma5 \
    libgcc-s1 \
    libssl3 \
    perl \
    make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release -p phlow-runtime && strip --strip-unneeded /app/target/release/phlow

###############################################################################
# BusyBox
####################################################################a###########
FROM busybox:glibc

COPY --from=builder /lib/x86_64-linux-gnu/libz.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/liblzma.so.5 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libssl.so.3 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/
COPY --from=builder /etc/ssl/certs /etc/ssl/certs
COPY --from=builder /app/target/release/phlow /app/phlow

WORKDIR /app
RUN chmod +x /app/phlow

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    PHLOW_MAIN=https://github.com/phlowdotdev/phlow-mirror-request/archive/refs/heads/main.zip \
    PHLOW_LOG=info \
    PHLOW_OTEL=true

ENTRYPOINT ["/app/phlow"]