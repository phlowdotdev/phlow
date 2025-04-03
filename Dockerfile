FROM ubuntu:22.04

# Instala o Alloy (Grafana Agent) com variáveis de ambiente
ENV GCLOUD_HOSTED_METRICS_ID="2351705"
ENV GCLOUD_HOSTED_METRICS_URL="https://prometheus-prod-56-prod-us-east-2.grafana.net/api/prom/push"
ENV GCLOUD_HOSTED_LOGS_ID="1171530"
ENV GCLOUD_HOSTED_LOGS_URL="https://logs-prod-036.grafana.net/loki/api/v1/push"
ENV GCLOUD_FM_URL="https://fleet-management-prod-008.grafana.net"
ENV GCLOUD_FM_POLL_FREQUENCY="60s"
ENV GCLOUD_FM_HOSTED_ID="1213733"
ENV ARCH="amd64"
ENV GCLOUD_RW_API_KEY="glc_eyJvIjoiMTM1MTc3MSIsIm4iOiJzdGFjay0xMjEzNzMzLWFsbG95LXBobG93LWJldGEiLCJrIjoiUDZlMjZaeDRTeFM1Nk5jdzFiVHJ2NDc1IiwibSI6eyJyIjoicHJvZC11cy1lYXN0LTAifX0="

RUN /bin/sh -c "$(curl -fsSL https://storage.googleapis.com/cloud-onboarding/alloy/scripts/install-linux.sh)"

# Copia o config da Alloy
COPY config.alloy /etc/alloy/config.alloy

# Cria diretório de trabalho da aplicação
WORKDIR /app

# Comando final para rodar a aplicação
CMD ["./test_otlp"]
