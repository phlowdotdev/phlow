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

# Instala Rust manualmente
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release -p phlow-runtime \
    && strip --strip-unneeded /app/target/release/phlow

###############################################################################
# Phlow Runtime - Final image
###############################################################################
FROM debian:trixie-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copia apenas o binário (não precisa mais copiar as libs separadas)
COPY --from=builder /app/target/release/phlow /app/phlow

RUN chmod +x /app/phlow

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt 

ENTRYPOINT ["/app/phlow"]
