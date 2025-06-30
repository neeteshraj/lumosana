use prometheus::{
    Counter, Histogram, IntCounter, IntGauge, Registry, Encoder, TextEncoder,
    HistogramOpts, Opts,
};
use actix_web::{HttpResponse, Result};
use std::sync::Once;

static INIT: Once = Once::new();
static mut REGISTRY: Option<Registry> = None;

// Metrics
static mut HTTP_REQUESTS_TOTAL: Option<Counter> = None;
static mut HTTP_REQUEST_DURATION: Option<Histogram> = None;
static mut ACTIVE_CONNECTIONS: Option<IntGauge> = None;
static mut TRANSACTION_ANALYSES_TOTAL: Option<IntCounter> = None;
static mut TRANSACTION_ANALYSIS_DURATION: Option<Histogram> = None;

pub fn init_metrics() {
    INIT.call_once(|| {
        let registry = Registry::new();
        
        // HTTP metrics
        let http_requests_total = Counter::with_opts(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("transaction_debugger")
        ).unwrap();
        
        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .namespace("transaction_debugger")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        ).unwrap();
        
        let active_connections = IntGauge::with_opts(
            Opts::new("active_connections", "Number of active connections")
                .namespace("transaction_debugger")
        ).unwrap();
        
        // Business metrics
        let transaction_analyses_total = IntCounter::with_opts(
            Opts::new("transaction_analyses_total", "Total number of transaction analyses")
                .namespace("transaction_debugger")
        ).unwrap();
        
        let transaction_analysis_duration = Histogram::with_opts(
            HistogramOpts::new("transaction_analysis_duration_seconds", "Transaction analysis duration in seconds")
                .namespace("transaction_debugger")
                .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
        ).unwrap();

        // Register metrics
        registry.register(Box::new(http_requests_total.clone())).unwrap();
        registry.register(Box::new(http_request_duration.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(transaction_analyses_total.clone())).unwrap();
        registry.register(Box::new(transaction_analysis_duration.clone())).unwrap();

        unsafe {
            REGISTRY = Some(registry);
            HTTP_REQUESTS_TOTAL = Some(http_requests_total);
            HTTP_REQUEST_DURATION = Some(http_request_duration);
            ACTIVE_CONNECTIONS = Some(active_connections);
            TRANSACTION_ANALYSES_TOTAL = Some(transaction_analyses_total);
            TRANSACTION_ANALYSIS_DURATION = Some(transaction_analysis_duration);
        }
    });
}

pub fn get_registry() -> &'static Registry {
    unsafe {
        REGISTRY.as_ref().expect("Metrics not initialized")
    }
}

pub fn inc_http_requests() {
    unsafe {
        if let Some(counter) = &HTTP_REQUESTS_TOTAL {
            counter.inc();
        }
    }
}

pub fn observe_http_request_duration(duration: f64) {
    unsafe {
        if let Some(histogram) = &HTTP_REQUEST_DURATION {
            histogram.observe(duration);
        }
    }
}

pub fn inc_active_connections() {
    unsafe {
        if let Some(gauge) = &ACTIVE_CONNECTIONS {
            gauge.inc();
        }
    }
}

pub fn dec_active_connections() {
    unsafe {
        if let Some(gauge) = &ACTIVE_CONNECTIONS {
            gauge.dec();
        }
    }
}

pub fn inc_transaction_analyses() {
    unsafe {
        if let Some(counter) = &TRANSACTION_ANALYSES_TOTAL {
            counter.inc();
        }
    }
}

pub fn observe_transaction_analysis_duration(duration: f64) {
    unsafe {
        if let Some(histogram) = &TRANSACTION_ANALYSIS_DURATION {
            histogram.observe(duration);
        }
    }
}

/// HTTP handler for Prometheus metrics endpoint
pub async fn metrics_handler() -> Result<HttpResponse> {
    let registry = get_registry();
    let metric_families = registry.gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer))
}
