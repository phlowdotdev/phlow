###############################################################################
# Dockerfile para o Phlow Runtime
###############################################################################
FROM clux/muslrust:stable AS builder

WORKDIR /app
COPY . .

# 🔧 Reinstala o target musl, caso não esteja ativo na imagem
RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --release --target x86_64-unknown-linux-musl -p phlow-runtime

###############################################################################
# Dockerfile para o Phlow Modules
###############################################################################
FROM clux/muslrust:stable AS modules

WORKDIR /app
COPY . .

RUN rustup target add x86_64-unknown-linux-musl

RUN chmod +x /app/scripts/modules-build.sh

RUN cargo build --release --target x86_64-unknown-linux-musl
RUN bash /app/scripts/modules-build.sh
RUN echo "📦 Conteúdo de /app no estágio builder:" && ls -la /
RUN echo "📦 Conteúdo recursivo de /app:" && find /


###############################################################################
# Estágio auxiliar com os certificados
###############################################################################
FROM alpine:latest AS certs
RUN apk add --no-cache ca-certificates
RUN mkdir -p /etc/ssl/certs && cp -r /etc/ssl/certs /ca-certs

###############################################################################
# Imagem final com binário estático + certificados
###############################################################################
FROM scratch

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    PHLOW_MAIN=https://github.com/lowcarboncode/phlow-mirror-request/archive/refs/heads/main.zip \
    PHLOW_LOG=info \
    PHLOW_OTEL=true

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/phlow /app/phlow
COPY --from=certs /ca-certs /etc/ssl/certs
COPY --from=modules /phlow_modules /app/phlow_modules

ENTRYPOINT ["/app/phlow"]
# CMD ["--help"]                                                                        