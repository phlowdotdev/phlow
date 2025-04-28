###############################################################################
# Phlow Runtime - Environment
###############################################################################
FROM debian:bookworm-slim AS environment
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    zlib1g-dev \
    liblzma-dev \
    gcc \
    libgcc-s1 \
    libssl3 \
    perl \
    make \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

###############################################################################
# Phlow Runtime - Builder
###############################################################################
FROM debian:bookworm-slim AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    zlib1g-dev \
    liblzma-dev \
    gcc \
    libgcc-s1 \
    libssl3 \
    perl \
    make \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . . 

RUN cargo build --release -p phlow-runtime && strip --strip-unneeded /app/target/release/phlow

###############################################################################
# Phlow Runtime - Runtime
###############################################################################
FROM busybox:1.34.1-glibc AS runtime

COPY --from=builder /lib/x86_64-linux-gnu/libz.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/liblzma.so.5 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libssl.so.3 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/
COPY --from=builder /etc/ssl/certs /etc/ssl/certs
COPY --from=builder /app/target/release/phlow /app/phlow

WORKDIR /app
RUN chmod +x /app/phlow

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

ENTRYPOINT ["/app/phlow"]
