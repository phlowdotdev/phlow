###############################################################################
# Phlow Runtime - Environment
###############################################################################
FROM ubuntu:20.04 AS environment

# Dependências mínimas para build do Rust
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    pkg-config \
    libssl-dev \
    libssl1.1 \
    ca-certificates \
    zlib1g-dev \
    liblzma-dev \
    gcc \
    libgcc-s1 \
    perl \
    make \
    binutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Instala Rust travado na versão 1.83.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.83.0
ENV PATH="/root/.cargo/bin:${PATH}"

###############################################################################
# Phlow Runtime - Builder
###############################################################################
FROM environment AS builder

WORKDIR /app

# Copia apenas os arquivos de dependência primeiro (melhor uso do cache)
COPY Cargo.toml Cargo.lock . 

# Depois copia o restante
COPY . .

# Build com strip na sequência
RUN cargo build --release -p phlow-runtime \
    && strip --strip-unneeded /app/target/release/phlow

###############################################################################
# Phlow Runtime - Runtime
###############################################################################
FROM busybox:1.34.1-glibc AS runtime

# Copia somente as libs necessárias
COPY --from=builder /lib/x86_64-linux-gnu/libz.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/liblzma.so.5 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libssl.so.1.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.1.1 /lib/
COPY --from=builder /lib/x86_64-linux-gnu/libdl.so.2 /lib/
COPY --from=builder /etc/ssl/certs /etc/ssl/certs

COPY --from=builder /app/target/release/phlow /app/phlow

WORKDIR /app
RUN chmod +x /app/phlow

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

ENTRYPOINT ["/app/phlow"]
