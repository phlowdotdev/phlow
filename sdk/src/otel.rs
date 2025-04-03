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
use tracing::{debug, Level};
use tracing::{dispatcher, Dispatch};
use tracing_core::LevelFilter;
use tracing_opentelemetry::MetricsLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn resource() -> Resource {
    Resource::builder().build()
}

pub fn get_tracer() -> BoxedTracer {
    global::tracer("phlow-tracing-otel-subscriber")
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
        Ok(level) => level.parse::<Level>().unwrap_or(Level::ERROR),
        Err(_) => Level::WARN,
    }
}

fn get_span_level() -> Level {
    match std::env::var("PHLOW_SPAN") {
        Ok(level) => level.parse::<Level>().unwrap_or(Level::INFO),
        Err(_) => Level::INFO,
    }
}

pub fn init_tracing_subscriber() -> OtelGuard {
    let tracer_provider = init_tracer_provider().ok();
    let meter_provider = init_meter_provider().ok();

    let (tracer_provider, meter_provider) = if tracer_provider.is_some() && meter_provider.is_some()
    {
        let tracer_provider = tracer_provider.unwrap();
        let meter_provider = meter_provider.unwrap();
        let tracer = tracer_provider.tracer("tracing-otel-subscriber");

        let fmt_layer = fmt::layer().with_filter(LevelFilter::from_level(get_log_level())); // logs (ex: WARN)

        let otel_layer =
            OpenTelemetryLayer::new(tracer).with_filter(LevelFilter::from_level(get_span_level())); // spans (ex: INFO)

        Registry::default()
            .with(fmt_layer)
            .with(otel_layer)
            .with(MetricsLayer::new(meter_provider.clone()))
            .init();

        debug!("OpenTelemetry provider found, using OpenTelemetry subscriber");

        (Some(tracer_provider), Some(meter_provider))
    } else {
        Registry::default()
            .with(fmt::layer().with_filter(LevelFilter::from_level(get_log_level())))
            .init();

        debug!("No OpenTelemetry provider found, using default subscriber");
        (None, None)
    };

    let dispatch = dispatcher::get_default(|d| d.clone());

    OtelGuard {
        tracer_provider,
        meter_provider,
        dispatch,
    }
}

pub struct OtelGuard {
    pub tracer_provider: Option<SdkTracerProvider>,
    pub meter_provider: Option<SdkMeterProvider>,
    pub dispatch: Dispatch,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(tracer_provider) = &self.tracer_provider {
            if let Err(err) = tracer_provider.shutdown() {
                eprintln!("{err:?}");
            }
        }
        if let Some(meter_provider) = &self.meter_provider {
            if let Err(err) = meter_provider.shutdown() {
                eprintln!("{err:?}");
            }
        }
    }
}
