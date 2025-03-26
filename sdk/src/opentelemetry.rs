use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, MetricError, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use tracing::error;
use tracing_core::Level;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
pub enum OtelError {
    TracerError(opentelemetry::trace::TraceError),
    MeterError(MetricError),
}

pub fn resource() -> Resource {
    Resource::builder().build()
}

fn init_meter_provider() -> Result<SdkMeterProvider, OtelError> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .map_err(OtelError::MeterError)?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    // For debugging in development
    let stdout_reader =
        PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource())
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

// Construct TracerProvider for OpenTelemetryLayer
fn init_tracer_provider() -> Result<SdkTracerProvider, OtelError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .map_err(OtelError::TracerError)?;

    Ok(SdkTracerProvider::builder()
        // Customize sampling strategy
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        // If export trace to AWS X-Ray, you can use XrayIdGenerator
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build())
}

fn log_level() -> Level {
    let env = std::env::var("PHLOW_LOG").unwrap_or_else(|_| "info".to_string());

    match env.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
pub fn init_tracing_subscriber() -> Result<OtelGuard, OtelError> {
    let env = std::env::var("PHLOW_OTEL").unwrap_or_else(|_| "false".to_string());

    // if false log only stdout with log level
    if env.to_lowercase() != "true" {
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::from_level(
                log_level(),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
        return Ok(OtelGuard {
            tracer_provider: SdkTracerProvider::default(),
            meter_provider: SdkMeterProvider::default(),
        });
    }

    let tracer_provider: SdkTracerProvider = init_tracer_provider()?;
    let meter_provider = init_meter_provider()?;

    let tracer = tracer_provider.tracer("tracing-otel-subscriber");

    tracing_subscriber::registry()
        // The global level filter prevents the exporter network stack
        // from reentering the globally installed OpenTelemetryLayer with
        // its own spans while exporting, as the libraries should not use
        // tracing levels below DEBUG. If the OpenTelemetry layer needs to
        // trace spans and events with higher verbosity levels, consider using
        // per-layer filtering to target the telemetry layer specifically,
        // e.g. by target matching.
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            log_level(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(MetricsLayer::new(meter_provider.clone()))
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}

pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            error!("{err:?}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            error!("{err:?}");
        }
    }
}
