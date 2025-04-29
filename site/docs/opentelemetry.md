---
sidebar_position: 8
title: OpenTelemetry
---

Phlow provides built-in support for **OpenTelemetry** to enable observability and monitoring of your workflows. This includes tracing, metrics, and logging capabilities.
This allows you to gain insights into the performance and behavior of your workflows, making it easier to identify bottlenecks and optimize your processes.

## OpenTelemetry Providers
Phlow includes the following OpenTelemetry providers:
- **Tracing**: Captures spans and logs for distributed tracing.
- **Metrics**: Collects and exports metrics data for performance monitoring.
- **Logging**: Integrates with the logging system to provide structured logs.


### Configuration
To enable OpenTelemetry in your Phlow instance, you can set the following environment variables:

| Variable  | Description  | Default Value |
|-----------|----------------------------|---------|
| PHLOW_OTEL | Enable OpenTelemetry | `false` |
| PHLOW_LOG | Log level | `WARN` |
| PHLOW_SPAN | Span level | `INFO` |

### Enabling OpenTelemetry

By default, OpenTelemetry is not enabled in Phlow. To activate it, you need to set the `PHLOW_OTEL` environment variable to `true`:

```bash
export PHLOW_OTEL=true
```

This ensures that OpenTelemetry features, such as tracing, metrics, and logging, are activated for your workflows.

### OpenTelemetry Standard Environment Variables

You can use the standard OpenTelemetry environment variables to configure your project. Below are some commonly used variables:

| Variable                          | Description                                                                                  |
|-----------------------------------|----------------------------------------------------------------------------------------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT`     | The endpoint for sending OTLP data (e.g., `http://localhost:4317` for gRPC).                 |
| `OTEL_RESOURCE_ATTRIBUTES`        | Key-value pairs to describe the resource (e.g., `service.name=phlow,service.version=1.0.0`). |
| `OTEL_SERVICE_NAME`               | The name of the service emitting telemetry data.                                             |
| `OTEL_LOG_LEVEL`                  | The log level for the SDK (e.g., `info`, `debug`).                                           |
| `OTEL_TRACES_SAMPLER`             | The sampler to use for traces (e.g., `always_on`, `always_off`, `traceidratio`).            |
| `OTEL_TRACES_SAMPLER_ARG`         | Arguments for the sampler (e.g., `0.25` for 25% sampling rate with `traceidratio`).         |

For a complete list of environment variables, refer to the [OpenTelemetry documentation](https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/).

##  OpenTelemetry + Jaeger (Local Dev Setup)

To enable observability with **Jaeger** during development, you can run a full OpenTelemetry-compatible collector locally in seconds.

### 1. Run Jaeger with OTLP support

```bash
docker run -d \
  -p4318:4318 \  # OTLP HTTP
  -p4317:4317 \  # OTLP gRPC
  -p16686:16686 \  # Jaeger UI
  jaegertracing/all-in-one:latest
```

This container supports OTLP over HTTP and gRPC, which are both compatible with Phlow's OpenTelemetry output.


### 2. Configure environment variables

Set the following environment variables in your shell or `.env` file:

```bash
export OTEL_RESOURCE_ATTRIBUTES="service.name=phlow-dev,service.version=0.1.0"
export OTEL_SERVICE_NAME="phlow-dev"
```

You can change the `service.name` to any label that helps identify your instance in Jaeger.

###  3. Open the Jaeger UI

Once running, access the Jaeger web interface at:

[http://localhost:16686](http://localhost:16686)

Search for your service using the name defined in `OTEL_SERVICE_NAME`.


