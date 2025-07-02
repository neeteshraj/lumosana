use opentelemetry::{global, KeyValue, InstrumentationScope};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter};
use opentelemetry_sdk::{Resource, runtime, trace::SdkTracerProvider, logs::SdkLoggerProvider, metrics::SdkMeterProvider};
use std::sync::OnceLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE.get_or_init(|| {
        Resource::builder()
            .with_service_name("transaction-debugger")
            .build()
    }).clone()
}

pub fn init_telemetry() -> (SdkTracerProvider, SdkMeterProvider, SdkLoggerProvider) {
    // Logs
    let log_exporter = LogExporter::builder().with_tonic().build().expect("Log exporter failed");
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(log_exporter)
        .build();

    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    // Console log output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap()));

    tracing_subscriber::registry()
        .with(otel_layer.with_filter(filter_otel))
        .with(fmt_layer)
        .init();

    // Traces
    let span_exporter = SpanExporter::builder().with_tonic().build().expect("Span exporter failed");
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(span_exporter)
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    // Metrics
    let metric_exporter = MetricExporter::builder().with_tonic().build().expect("Metric exporter failed");
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(get_resource())
        .build();
    global::set_meter_provider(meter_provider.clone());

    (tracer_provider, meter_provider, logger_provider)
}

pub fn shutdown_telemetry(
    tracer: SdkTracerProvider,
    meter: SdkMeterProvider,
    logger: SdkLoggerProvider
) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = vec![];

    if let Err(e) = tracer.shutdown() {
        errors.push(format!("tracer shutdown: {e}"));
    }
    if let Err(e) = meter.shutdown() {
        errors.push(format!("meter shutdown: {e}"));
    }
    if let Err(e) = logger.shutdown() {
        errors.push(format!("logger shutdown: {e}"));
    }

    if !errors.is_empty() {
        Err(errors.join("\n").into())
    } else {
        Ok(())
    }
}
