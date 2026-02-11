use once_cell::sync::OnceCell;
use prometheus::{Counter, Histogram, IntGauge, Encoder, TextEncoder, Registry};
use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry as TracingRegistry};
use lazy_static::lazy_static;

static METRICS_HANDLE: OnceCell<Arc<MetricsHandle>> = OnceCell::new();
static REGISTRY: OnceCell<Registry> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct MetricsHandle {
    registry: Arc<Registry>,
}

impl MetricsHandle {
    pub fn new() -> Self {
        let registry = Registry::new();
        Self {
            registry: Arc::new(registry),
        }
    }

    pub fn render(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap_or_else(|_| String::from("Failed to encode metrics")))
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub log_format: LogFormat,
    pub metrics: bool,
}

#[derive(Clone)]
pub struct TelemetryHandles {
    pub metrics: Option<Arc<MetricsHandle>>,
}

impl std::fmt::Debug for TelemetryHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetryHandles")
            .field("metrics", &self.metrics.is_some())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("failed to set global tracing subscriber")]
    SubscriberAlreadySet,
    #[error("prometheus registry error: {0}")]
    Prometheus(#[from] prometheus::Error),
}

lazy_static! {
    pub static ref CONTENT_OPERATIONS_TOTAL: Counter = register_counter!(
        "rustok_content_operations_total",
        "Total content operations",
        &["operation", "kind", "status"]
    ).expect("Failed to register content_operations_total");

    pub static ref CONTENT_OPERATION_DURATION_SECONDS: Histogram = register_histogram!(
        "rustok_content_operation_duration_seconds",
        "Duration of content operations",
        &["operation", "kind"]
    ).expect("Failed to register content_operation_duration_seconds");

    pub static ref CONTENT_NODES_TOTAL: IntGauge = register_int_gauge!(
        "rustok_content_nodes_total",
        "Total number of content nodes"
    ).expect("Failed to register content_nodes_total");

    pub static ref COMMERCE_OPERATIONS_TOTAL: Counter = register_counter!(
        "rustok_commerce_operations_total",
        "Total commerce operations",
        &["operation", "kind", "status"]
    ).expect("Failed to register commerce_operations_total");

    pub static ref COMMERCE_OPERATION_DURATION_SECONDS: Histogram = register_histogram!(
        "rustok_commerce_operation_duration_seconds",
        "Duration of commerce operations",
        &["operation", "kind"]
    ).expect("Failed to register commerce_operation_duration_seconds");

    pub static ref COMMERCE_PRODUCTS_TOTAL: IntGauge = register_int_gauge!(
        "rustok_commerce_products_total",
        "Total number of products"
    ).expect("Failed to register commerce_products_total");

    pub static ref COMMERCE_ORDERS_TOTAL: IntGauge = register_int_gauge!(
        "rustok_commerce_orders_total",
        "Total number of orders"
    ).expect("Failed to register commerce_orders_total");

    pub static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "rustok_http_requests_total",
        "Total HTTP requests",
        &["method", "path", "status"]
    ).expect("Failed to register http_requests_total");

    pub static ref HTTP_REQUEST_DURATION_SECONDS: Histogram = register_histogram!(
        "rustok_http_request_duration_seconds",
        "HTTP request duration",
        &["method", "path"]
    ).expect("Failed to register http_request_duration_seconds");
}

pub fn init(config: TelemetryConfig) -> Result<TelemetryHandles, TelemetryError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer: Box<dyn Layer<_> + Send + Sync> = match config.log_format {
        LogFormat::Json => fmt::layer()
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .json()
            .boxed(),
        LogFormat::Pretty => fmt::layer()
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .pretty()
            .boxed(),
    };

    let subscriber = TracingRegistry::default().with(env_filter).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| TelemetryError::SubscriberAlreadySet)?;

    let metrics_handle = if config.metrics {
        let handle = Arc::new(MetricsHandle::new());
        let _ = REGISTRY.set(handle.registry().clone());
        let _ = METRICS_HANDLE.set(handle.clone());
        Some(handle)
    } else {
        None
    };

    Ok(TelemetryHandles {
        metrics: metrics_handle,
    })
}

pub fn metrics_handle() -> Option<Arc<MetricsHandle>> {
    METRICS_HANDLE.get().cloned()
}

pub fn render_metrics() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.get()
        .ok_or(prometheus::Error::Msg("Registry not initialized"))?
        .gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).unwrap_or_else(|_| String::from("Failed to encode metrics")))
}

pub fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    span.id().map(|id| id.into_u64().to_string())
}
