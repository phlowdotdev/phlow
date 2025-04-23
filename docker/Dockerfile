FROM scratch

ENV PHLOW_MAIN="https://github.com/lowcarboncode/phlow-mirror-request/tarball/main"
ENV PHLOW_LOG="info"
ENV PHLOW_OTEL="true"

COPY phlow /phlow

# Executa o binário
ENTRYPOINT ["./phlow"]
