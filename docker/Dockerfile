###########################
# Final image: glibc slim
# docker build --target phlow-slim -t phlow:slim -f Dockerfile.multi .
###########################
FROM ubuntu:24.04 AS phlow-slim

ENV PHLOW_MAIN="https://github.com/lowcarboncode/phlow-mirror-request/tarball/main"
ENV PHLOW_LOG="info"
ENV PHLOW_OTEL="true"

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY phlow-x86_64-unknown-linux-gnu /phlow
RUN chmod +x /phlow

ENTRYPOINT ["/phlow"]
