use opentelemetry::{
    global::{self, BoxedTracer},
    trace::TracerProvider,
};
use opentelemetry_otlp::ExporterBuildError;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use tracing::{dispatcher, Dispatch};
use tracing_core::Level;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn resource() -> Resource {
    Resource::builder().build()
}

pub fn get_tracer() -> BoxedTracer {
    global::tracer("tracing-otel-subscriber")
}

fn init_meter_provider() -> Result<SdkMeterProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()?;
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

fn init_tracer_provider() -> Result<SdkTracerProvider, ExporterBuildError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()?;

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

fn get_log_level() -> Level {
    match std::env::var("PHLOW_LOG") {
        Ok(level) => match level.parse::<Level>() {
            Ok(level) => level,
            Err(_) => Level::ERROR,
        },
        Err(_) => Level::INFO,
    }
}

pub fn init_tracing_subscriber() -> Result<OtelGuard, ExporterBuildError> {
    let tracer_provider = init_tracer_provider()?;
    let meter_provider = init_meter_provider()?;

    let tracer = tracer_provider.tracer("tracing-otel-subscriber");

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            get_log_level(),
        ))
        .with(tracing_subscriber::fmt::layer())
        .with(MetricsLayer::new(meter_provider.clone()))
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    let dispatch = dispatcher::get_default(|d| d.clone());

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
        dispatch,
    })
}

pub struct OtelGuard {
    pub tracer_provider: SdkTracerProvider,
    pub meter_provider: SdkMeterProvider,
    pub dispatch: Dispatch,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("{err:?}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("{err:?}");
        }
    }
}
