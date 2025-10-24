FROM debian:bookworm-slim

# Arquivo alterado para baixar binários pré-compilados em vez de buildar
ARG ARCH=amd64
WORKDIR /app

# Instala deps mínimas de runtime e utilitários para download
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Escolhe o binário pela arquitetura e baixa da release
RUN set -eux; \
    case "$ARCH" in \
    amd64) FILE=phlow-amd64 ;; \
    arm64) FILE=phlow-arm64 ;; \
    *) echo "Unsupported ARCH: $ARCH" >&2; exit 1 ;; \
    esac; \
    URL="https://github.com/phlowdotdev/phlow/releases/download/$PHLOW_VERSION/$FILE"; \
    echo "Downloading $URL"; \
    curl -fsSL -o /app/phlow "$URL"; \
    chmod +x /app/phlow; \
    ls -lh /app/phlow

ENTRYPOINT ["/app/phlow"]
