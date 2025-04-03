## 🧪 OpenTelemetry + Jaeger (Local Dev Setup)

To enable observability with **Jaeger** during development, you can run a full OpenTelemetry-compatible collector locally in seconds.

### 🔄 1. Run Jaeger with OTLP support

!!!bash
docker run -d \
  -p4318:4318 \  # OTLP HTTP
  -p4317:4317 \  # OTLP gRPC
  -p16686:16686 \  # Jaeger UI
  jaegertracing/all-in-one:latest

This container supports OTLP over HTTP and gRPC, which are both compatible with Phlow's OpenTelemetry output.

---

### ⚙️ 2. Configure environment variables

Set the following environment variables in your shell or `.env` file:

!!!bash
export OTEL_RESOURCE_ATTRIBUTES="service.name=phlow-dev,service.version=0.1.0"
export OTEL_SERVICE_NAME="phlow-dev"

You can change the `service.name` to any label that helps identify your instance in Jaeger.

---

### 🔍 3. Open the Jaeger UI

Once running, access the Jaeger web interface at:

[http://localhost:16686](http://localhost:16686)

Search for your service using the name defined in `OTEL_SERVICE_NAME`.

---

### ✅ Tips

- Combine this with `PHLOW_OTEL=true`, `PHLOW_SPAN=INFO`, and `PHLOW_LOG=DEBUG` for full observability.
- You can also integrate with **Grafana Tempo** or **AWS X-Ray** by replacing the collector backend.

