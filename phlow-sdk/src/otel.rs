use std::env;

use opentelemetry::{
    global::{self, BoxedTracer},
    trace::TracerProvider,
    KeyValue,
};
use opentelemetry_otlp::ExporterBuildError;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use opentelemetry_semantic_conventions::{
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};
use tracing::Dispatch;
use tracing::{debug, Level};
use tracing_core::LevelFilter;
use tracing_opentelemetry::MetricsLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

use crate::prelude::ApplicationData;
// otel active
static PHLOW_OTEL_ACTIVE: once_cell::sync::Lazy<bool> =
    once_cell::sync::Lazy::new(|| match std::env::var("PHLOW_OTEL") {
        Ok(active) => active.parse::<bool>().unwrap_or(false),
        Err(_) => false,
    });

static PHLOW_SPAN_ACTIVE: once_cell::sync::Lazy<Level> =
    once_cell::sync::Lazy::new(|| match std::env::var("PHLOW_SPAN") {
        Ok(level) => level.parse::<Level>().unwrap_or(Level::INFO),
        Err(_) => Level::INFO,
    });

static PHLOW_LOG: once_cell::sync::Lazy<Level> =
    once_cell::sync::Lazy::new(|| match std::env::var("PHLOW_LOG") {
        Ok(level) => level.parse::<Level>().unwrap_or(Level::INFO),
        Err(_) => Level::INFO,
    });

fn resource(app_data: ApplicationData) -> Resource {
    let service_name = env::var("OTEL_SERVICE_NAME")
        .unwrap_or_else(|_| app_data.name.unwrap_or_else(|| "phlow".to_string()));
    let service_version = env::var("OTEL_SERVICE_VERSION").unwrap_or_else(|_| {
        app_data.version.unwrap_or_else(|| {
            env::var("PHLOW_VERSION")
                .unwrap_or_else(|_| "".to_string())
                .to_string()
        })
    });
    let deployment_environment_name =
        env::var("OTEL_DEPLOYMENT_ENVIRONMENT_NAME").unwrap_or_else(|_| {
            app_data.environment.unwrap_or_else(|| {
                env::var("PHLOW_ENV")
                    .unwrap_or_else(|_| "development".to_string())
                    .to_string()
            })
        });

    let attributes = vec![
        KeyValue::new(SCHEMA_URL, "https://opentelemetry.io/schemas/1.4.0"),
        KeyValue::new(SERVICE_NAME, service_name),
        KeyValue::new(SERVICE_VERSION, service_version),
        KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, deployment_environment_name),
    ];

    Resource::builder()
        .with_schema_url(
            [
                KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
            ],
            SCHEMA_URL,
        )
        .with_attributes(attributes)
        .build()
}

pub fn get_tracer() -> BoxedTracer {
    global::tracer("phlow-tracing-otel-subscriber")
}

fn init_meter_provider(resource: Resource) -> Result<SdkMeterProvider, ExporterBuildError> {
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
        .with_resource(resource)
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

fn init_tracer_provider(resource: Resource) -> Result<SdkTracerProvider, ExporterBuildError> {
    let exporter = if env::var("OTEL_EXPORTER_OTLP_PROTOCOL") == Ok("grpc".to_string()) {
        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()?
    } else {
        opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .build()?
    };

    Ok(SdkTracerProvider::builder()
        // Customize sampling strategy
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            1.0,
        ))))
        // If export trace to AWS X-Ray, you can use XrayIdGenerator
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

pub fn get_log_level() -> Level {
    *PHLOW_LOG
}

fn get_span_level() -> Level {
    *PHLOW_SPAN_ACTIVE
}

pub fn get_otel_active() -> bool {
    *PHLOW_OTEL_ACTIVE
}

pub fn init_tracing_subscriber(app_data: ApplicationData) -> OtelGuard {
    if !get_otel_active() {
        let subscriber = Registry::default()
            .with(fmt::layer().with_filter(LevelFilter::from_level(get_log_level())));

        let dispatch = Dispatch::new(subscriber);

        debug!("PHLOW_OTEL is set to false, using default subscriber");

        return OtelGuard {
            tracer_provider: None,
            meter_provider: None,
            dispatch,
        };
    }

    let resource = resource(app_data);
    let tracer_provider = init_tracer_provider(resource.clone()).ok();
    let meter_provider = init_meter_provider(resource.clone()).ok();

    if let (Some(tp), Some(mp)) = (&tracer_provider, &meter_provider) {
        let tracer = tp.tracer("tracing-otel-subscriber");

        let fmt_layer = fmt::layer().with_filter(LevelFilter::from_level(get_log_level()));
        let otel_layer =
            OpenTelemetryLayer::new(tracer).with_filter(LevelFilter::from_level(get_span_level()));
        let metrics_layer = MetricsLayer::new(mp.clone());

        let subscriber = Registry::default()
            .with(fmt_layer)
            .with(otel_layer)
            .with(metrics_layer);

        let dispatch = Dispatch::new(subscriber);

        debug!("OpenTelemetry provider found, using OpenTelemetry subscriber");

        OtelGuard {
            tracer_provider: Some(tp.clone()),
            meter_provider: Some(mp.clone()),
            dispatch,
        }
    } else {
        let subscriber = Registry::default()
            .with(fmt::layer().with_filter(LevelFilter::from_level(get_log_level())));
        let dispatch = Dispatch::new(subscriber);

        debug!("No OpenTelemetry provider found, using default subscriber");

        OtelGuard {
            tracer_provider: None,
            meter_provider: None,
            dispatch,
        }
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
