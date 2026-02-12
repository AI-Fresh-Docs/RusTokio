//! Custom Prometheus Metrics for RusToK
//!
//! This module provides application-specific metrics for:
//! - EventBus throughput and lag
//! - Circuit breaker states
//! - Cache hit/miss rates
//! - Error rates by module
//!
//! # Example
//!
//! ```rust
//! use rustok_telemetry::metrics::{eventbus_metrics, circuit_breaker_metrics};
//!
//! // Record EventBus event published
//! eventbus_metrics().events_published.inc();
//!
//! // Record circuit breaker state change
//! circuit_breaker_metrics().record_state_change("tenant_cache", "closed", "open");
//! ```

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, register_int_counter,
    register_int_gauge, CounterVec, GaugeVec, HistogramVec, IntCounter, IntGauge,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ============================================================================
// EventBus Metrics
// ============================================================================

lazy_static! {
    /// Total events published to EventBus
    pub static ref EVENTBUS_EVENTS_PUBLISHED_TOTAL: IntCounter = register_int_counter!(
        "rustok_eventbus_events_published_total",
        "Total number of events published to EventBus"
    )
    .unwrap();

    /// Total events dropped by EventBus (channel full)
    pub static ref EVENTBUS_EVENTS_DROPPED_TOTAL: IntCounter = register_int_counter!(
        "rustok_eventbus_events_dropped_total",
        "Total number of events dropped by EventBus (channel full)"
    )
    .unwrap();

    /// Current number of EventBus subscribers
    pub static ref EVENTBUS_SUBSCRIBERS: IntGauge = register_int_gauge!(
        "rustok_eventbus_subscribers",
        "Current number of active EventBus subscribers"
    )
    .unwrap();

    /// Events published by type
    pub static ref EVENTBUS_EVENTS_BY_TYPE: CounterVec = register_counter_vec!(
        "rustok_eventbus_events_by_type_total",
        "Events published by event type",
        &["event_type"]
    )
    .unwrap();

    /// EventBus publish duration
    pub static ref EVENTBUS_PUBLISH_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "rustok_eventbus_publish_duration_seconds",
        "Duration of EventBus publish operations",
        &["result"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    )
    .unwrap();

    /// Event lag (queue depth approximation)
    pub static ref EVENTBUS_LAG: IntGauge = register_int_gauge!(
        "rustok_eventbus_lag",
        "Approximate event lag (published - processed estimate)"
    )
    .unwrap();
}

/// EventBus metrics handle
#[derive(Debug, Clone)]
pub struct EventBusMetrics;

impl EventBusMetrics {
    pub fn events_published(&self) -> &IntCounter {
        &EVENTBUS_EVENTS_PUBLISHED_TOTAL
    }

    pub fn events_dropped(&self) -> &IntCounter {
        &EVENTBUS_EVENTS_DROPPED_TOTAL
    }

    pub fn subscribers(&self) -> &IntGauge {
        &EVENTBUS_SUBSCRIBERS
    }

    pub fn record_publish(&self, event_type: &str, success: bool, duration_secs: f64) {
        EVENTBUS_EVENTS_PUBLISHED_TOTAL.inc();
        EVENTBUS_EVENTS_BY_TYPE.with_label_values(&[event_type]).inc();

        let result = if success { "success" } else { "failure" };
        EVENTBUS_PUBLISH_DURATION_SECONDS
            .with_label_values(&[result])
            .observe(duration_secs);
    }

    pub fn record_drop(&self) {
        EVENTBUS_EVENTS_DROPPED_TOTAL.inc();
    }

    pub fn set_subscribers(&self, count: i64) {
        EVENTBUS_SUBSCRIBERS.set(count);
    }

    pub fn set_lag(&self, lag: i64) {
        EVENTBUS_LAG.set(lag);
    }
}

/// Get EventBus metrics handle
pub fn eventbus_metrics() -> EventBusMetrics {
    EventBusMetrics
}

// ============================================================================
// Circuit Breaker Metrics
// ============================================================================

lazy_static! {
    /// Circuit breaker state (1 = active, 0 = inactive) by name and state
    pub static ref CIRCUIT_BREAKER_STATE: GaugeVec = register_gauge_vec!(
        "rustok_circuit_breaker_state",
        "Circuit breaker state (1 = current state)",
        &["name", "state"]
    )
    .unwrap();

    /// Total requests processed by circuit breaker
    pub static ref CIRCUIT_BREAKER_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_circuit_breaker_requests_total",
        "Total requests processed by circuit breaker",
        &["name", "result"]
    )
    .unwrap();

    /// Circuit breaker state transitions
    pub static ref CIRCUIT_BREAKER_TRANSITIONS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_circuit_breaker_transitions_total",
        "Circuit breaker state transitions",
        &["name", "from_state", "to_state"]
    )
    .unwrap();

    /// Current failure count per circuit breaker
    pub static ref CIRCUIT_BREAKER_FAILURE_COUNT: GaugeVec = register_gauge_vec!(
        "rustok_circuit_breaker_failure_count",
        "Current failure count per circuit breaker",
        &["name"]
    )
    .unwrap();

    /// Circuit breaker rejection rate (fail-fast rejections)
    pub static ref CIRCUIT_BREAKER_REJECTIONS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_circuit_breaker_rejections_total",
        "Total requests rejected by circuit breaker (fail-fast)",
        &["name"]
    )
    .unwrap();
}

/// Circuit breaker metrics handle
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics;

impl CircuitBreakerMetrics {
    /// Record circuit breaker state change
    pub fn record_state(&self, name: &str, state: &str) {
        // Set current state to 1, others to 0
        for s in &["closed", "open", "half_open"] {
            let value = if *s == state { 1.0 } else { 0.0 };
            CIRCUIT_BREAKER_STATE
                .with_label_values(&[name, s])
                .set(value);
        }
    }

    /// Record request result
    pub fn record_request(&self, name: &str, success: bool) {
        let result = if success { "success" } else { "failure" };
        CIRCUIT_BREAKER_REQUESTS_TOTAL
            .with_label_values(&[name, result])
            .inc();
    }

    /// Record state transition
    pub fn record_state_change(&self, name: &str, from: &str, to: &str) {
        CIRCUIT_BREAKER_TRANSITIONS_TOTAL
            .with_label_values(&[name, from, to])
            .inc();
        self.record_state(name, to);
    }

    /// Record rejection (circuit open)
    pub fn record_rejection(&self, name: &str) {
        CIRCUIT_BREAKER_REJECTIONS_TOTAL
            .with_label_values(&[name])
            .inc();
    }

    /// Set current failure count
    pub fn set_failure_count(&self, name: &str, count: f64) {
        CIRCUIT_BREAKER_FAILURE_COUNT
            .with_label_values(&[name])
            .set(count);
    }
}

/// Get circuit breaker metrics handle
pub fn circuit_breaker_metrics() -> CircuitBreakerMetrics {
    CircuitBreakerMetrics
}

// ============================================================================
// Cache Metrics
// ============================================================================

lazy_static! {
    /// Cache hit counter by cache name
    pub static ref CACHE_HITS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_cache_hits_total",
        "Total cache hits",
        &["cache_name"]
    )
    .unwrap();

    /// Cache miss counter by cache name
    pub static ref CACHE_MISSES_TOTAL: CounterVec = register_counter_vec!(
        "rustok_cache_misses_total",
        "Total cache misses",
        &["cache_name"]
    )
    .unwrap();

    /// Cache entries count by cache name
    pub static ref CACHE_ENTRIES: GaugeVec = register_gauge_vec!(
        "rustok_cache_entries",
        "Current number of entries in cache",
        &["cache_name"]
    )
    .unwrap();

    /// Cache eviction counter
    pub static ref CACHE_EVICTIONS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_cache_evictions_total",
        "Total cache evictions",
        &["cache_name", "reason"]
    )
    .unwrap();

    /// Cache operation duration
    pub static ref CACHE_OPERATION_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "rustok_cache_operation_duration_seconds",
        "Duration of cache operations",
        &["cache_name", "operation"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
    )
    .unwrap();
}

/// Cache metrics handle
#[derive(Debug, Clone)]
pub struct CacheMetrics;

impl CacheMetrics {
    /// Record cache hit
    pub fn record_hit(&self, cache_name: &str) {
        CACHE_HITS_TOTAL.with_label_values(&[cache_name]).inc();
    }

    /// Record cache miss
    pub fn record_miss(&self, cache_name: &str) {
        CACHE_MISSES_TOTAL.with_label_values(&[cache_name]).inc();
    }

    /// Set cache size
    pub fn set_entries(&self, cache_name: &str, count: f64) {
        CACHE_ENTRIES.with_label_values(&[cache_name]).set(count);
    }

    /// Record cache eviction
    pub fn record_eviction(&self, cache_name: &str, reason: &str) {
        CACHE_EVICTIONS_TOTAL
            .with_label_values(&[cache_name, reason])
            .inc();
    }

    /// Record cache operation duration
    pub fn record_operation(&self, cache_name: &str, operation: &str, duration_secs: f64) {
        CACHE_OPERATION_DURATION_SECONDS
            .with_label_values(&[cache_name, operation])
            .observe(duration_secs);
    }

    /// Get hit rate for a cache (returns 0.0 if no accesses)
    pub fn hit_rate(&self, cache_name: &str) -> f64 {
        let hits = CACHE_HITS_TOTAL.with_label_values(&[cache_name]).get();
        let misses = CACHE_MISSES_TOTAL.with_label_values(&[cache_name]).get();
        let total = hits + misses;

        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }
}

/// Get cache metrics handle
pub fn cache_metrics() -> CacheMetrics {
    CacheMetrics
}

// ============================================================================
// Error Metrics
// ============================================================================

lazy_static! {
    /// Error counter by module and error type
    pub static ref ERRORS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_errors_total",
        "Total errors by module and type",
        &["module", "error_type", "severity"]
    )
    .unwrap();

    /// Error rate by module (calculated from errors / total operations)
    pub static ref ERROR_RATE: GaugeVec = register_gauge_vec!(
        "rustok_error_rate",
        "Current error rate by module (0.0-1.0)",
        &["module"]
    )
    .unwrap();

    /// Panic counter
    pub static ref PANICS_TOTAL: IntCounter = register_int_counter!(
        "rustok_panics_total",
        "Total number of panics caught"
    )
    .unwrap();

    /// Retry attempts by module
    pub static ref RETRY_ATTEMPTS_TOTAL: CounterVec = register_counter_vec!(
        "rustok_retry_attempts_total",
        "Total retry attempts by module",
        &["module", "result"]
    )
    .unwrap();
}

/// Error metrics handle
#[derive(Debug, Clone)]
pub struct ErrorMetrics;

impl ErrorMetrics {
    /// Record an error occurrence
    pub fn record_error(&self, module: &str, error_type: &str, severity: &str) {
        ERRORS_TOTAL
            .with_label_values(&[module, error_type, severity])
            .inc();
    }

    /// Record a panic
    pub fn record_panic(&self) {
        PANICS_TOTAL.inc();
    }

    /// Record retry attempt
    pub fn record_retry(&self, module: &str, success: bool) {
        let result = if success { "success" } else { "failure" };
        RETRY_ATTEMPTS_TOTAL
            .with_label_values(&[module, result])
            .inc();
    }

    /// Update error rate gauge
    pub fn update_error_rate(&self, module: &str, rate: f64) {
        ERROR_RATE.with_label_values(&[module]).set(rate);
    }
}

/// Get error metrics handle
pub fn error_metrics() -> ErrorMetrics {
    ErrorMetrics
}

// ============================================================================
// Span Metrics (OpenTelemetry)
// ============================================================================

lazy_static! {
    /// Span count by operation name
    pub static ref SPAN_COUNT: CounterVec = register_counter_vec!(
        "rustok_span_count_total",
        "Total spans created by operation",
        &["operation", "span_kind"]
    )
    .unwrap();

    /// Span duration distribution
    pub static ref SPAN_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "rustok_span_duration_seconds",
        "Span duration distribution",
        &["operation"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap();

    /// Active spans gauge (approximate via exporter)
    pub static ref ACTIVE_SPANS: IntGauge = register_int_gauge!(
        "rustok_active_spans",
        "Approximate number of active spans"
    )
    .unwrap();
}

static ACTIVE_SPAN_COUNT: AtomicU64 = AtomicU64::new(0);

/// Span metrics handle
#[derive(Debug, Clone)]
pub struct SpanMetrics;

impl SpanMetrics {
    /// Record span creation
    pub fn record_span(&self, operation: &str, span_kind: &str) {
        SPAN_COUNT
            .with_label_values(&[operation, span_kind])
            .inc();
        ACTIVE_SPAN_COUNT.fetch_add(1, Ordering::Relaxed);
        self.update_active_spans();
    }

    /// Record span completion
    pub fn record_span_end(&self, operation: &str, duration_secs: f64) {
        SPAN_DURATION_SECONDS
            .with_label_values(&[operation])
            .observe(duration_secs);
        ACTIVE_SPAN_COUNT.fetch_sub(1, Ordering::Relaxed);
        self.update_active_spans();
    }

    fn update_active_spans(&self) {
        let count = ACTIVE_SPAN_COUNT.load(Ordering::Relaxed) as i64;
        ACTIVE_SPANS.set(count);
    }
}

/// Get span metrics handle
pub fn span_metrics() -> SpanMetrics {
    SpanMetrics
}

// ============================================================================
// Integration Functions
// ============================================================================

/// Initialize all custom metrics by registering them with the global registry
pub fn init_metrics() -> Result<(), prometheus::Error> {
    let registry = prometheus::default_registry();

    // EventBus metrics
    registry.register(Box::new(EVENTBUS_EVENTS_PUBLISHED_TOTAL.clone()))?;
    registry.register(Box::new(EVENTBUS_EVENTS_DROPPED_TOTAL.clone()))?;
    registry.register(Box::new(EVENTBUS_SUBSCRIBERS.clone()))?;
    registry.register(Box::new(EVENTBUS_EVENTS_BY_TYPE.clone()))?;
    registry.register(Box::new(EVENTBUS_PUBLISH_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(EVENTBUS_LAG.clone()))?;

    // Circuit breaker metrics
    registry.register(Box::new(CIRCUIT_BREAKER_STATE.clone()))?;
    registry.register(Box::new(CIRCUIT_BREAKER_REQUESTS_TOTAL.clone()))?;
    registry.register(Box::new(CIRCUIT_BREAKER_TRANSITIONS_TOTAL.clone()))?;
    registry.register(Box::new(CIRCUIT_BREAKER_FAILURE_COUNT.clone()))?;
    registry.register(Box::new(CIRCUIT_BREAKER_REJECTIONS_TOTAL.clone()))?;

    // Cache metrics
    registry.register(Box::new(CACHE_HITS_TOTAL.clone()))?;
    registry.register(Box::new(CACHE_MISSES_TOTAL.clone()))?;
    registry.register(Box::new(CACHE_ENTRIES.clone()))?;
    registry.register(Box::new(CACHE_EVICTIONS_TOTAL.clone()))?;
    registry.register(Box::new(CACHE_OPERATION_DURATION_SECONDS.clone()))?;

    // Error metrics
    registry.register(Box::new(ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(ERROR_RATE.clone()))?;
    registry.register(Box::new(PANICS_TOTAL.clone()))?;
    registry.register(Box::new(RETRY_ATTEMPTS_TOTAL.clone()))?;

    // Span metrics
    registry.register(Box::new(SPAN_COUNT.clone()))?;
    registry.register(Box::new(SPAN_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(ACTIVE_SPANS.clone()))?;

    Ok(())
}

/// Collect all metrics in a structured format for health checks
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MetricsSnapshot {
    pub eventbus: EventBusSnapshot,
    pub circuit_breakers: Vec<CircuitBreakerSnapshot>,
    pub caches: Vec<CacheSnapshot>,
    pub errors: ErrorSnapshot,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EventBusSnapshot {
    pub events_published: u64,
    pub events_dropped: u64,
    pub subscribers: i64,
    pub lag: i64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CircuitBreakerSnapshot {
    pub name: String,
    pub state: String,
    pub total_requests: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CacheSnapshot {
    pub name: String,
    pub entries: f64,
    pub hit_rate: f64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ErrorSnapshot {
    pub total_errors: f64,
    pub total_panics: u64,
}

impl MetricsSnapshot {
    /// Create a snapshot of current metrics
    pub fn capture() -> Self {
        Self {
            eventbus: EventBusSnapshot {
                events_published: EVENTBUS_EVENTS_PUBLISHED_TOTAL.get() as u64,
                events_dropped: EVENTBUS_EVENTS_DROPPED_TOTAL.get() as u64,
                subscribers: EVENTBUS_SUBSCRIBERS.get(),
                lag: EVENTBUS_LAG.get(),
            },
            circuit_breakers: vec![], // Populated by external systems
            caches: vec![],             // Populated by external systems
            errors: ErrorSnapshot {
                total_errors: ERRORS_TOTAL.get(),
                total_panics: PANICS_TOTAL.get(),
            },
        }
    }

    /// Check if any critical metrics are in alert state
    pub fn check_health(&self) -> Vec<String> {
        let mut alerts = vec![];

        // EventBus health checks
        if self.eventbus.events_dropped > 100 {
            alerts.push(format!(
                "EventBus dropped {} events - possible overflow",
                self.eventbus.events_dropped
            ));
        }

        if self.eventbus.lag > 1000 {
            alerts.push(format!(
                "EventBus lag is {} - processing behind",
                self.eventbus.lag
            ));
        }

        // Error checks
        if self.errors.total_panics > 0 {
            alerts.push(format!("{} panics detected", self.errors.total_panics));
        }

        alerts
    }
}
