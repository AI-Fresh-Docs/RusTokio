//! Tests for custom metrics module

use rustok_telemetry::metrics::{
    cache_metrics, circuit_breaker_metrics, error_metrics, eventbus_metrics, span_metrics,
    MetricsSnapshot,
};

#[test]
fn test_eventbus_metrics() {
    let metrics = eventbus_metrics();

    // Record some events
    metrics.record_publish("UserCreated", true, 0.001);
    metrics.record_publish("OrderUpdated", true, 0.002);
    metrics.record_publish("PaymentFailed", false, 0.003);
    metrics.record_drop();

    // Set subscriber count
    metrics.set_subscribers(5);

    // Set lag
    metrics.set_lag(100);

    // Verify we can access values (they should be non-zero)
    let snapshot = MetricsSnapshot::capture();
    assert!(snapshot.eventbus.events_published > 0);
    assert_eq!(snapshot.eventbus.events_dropped, 1);
    assert_eq!(snapshot.eventbus.subscribers, 5);
    assert_eq!(snapshot.eventbus.lag, 100);
}

#[test]
fn test_circuit_breaker_metrics() {
    let metrics = circuit_breaker_metrics();

    // Record state
    metrics.record_state("tenant_cache", "closed");
    metrics.record_state("payment_gateway", "open");

    // Record requests
    metrics.record_request("tenant_cache", true);
    metrics.record_request("tenant_cache", true);
    metrics.record_request("tenant_cache", false);

    // Record rejection
    metrics.record_rejection("payment_gateway");

    // Record state change
    metrics.record_state_change("tenant_cache", "closed", "open");

    // Set failure count
    metrics.set_failure_count("tenant_cache", 3.0);
}

#[test]
fn test_cache_metrics() {
    let metrics = cache_metrics();

    // Record hits and misses
    metrics.record_hit("tenant_cache");
    metrics.record_hit("tenant_cache");
    metrics.record_miss("tenant_cache");

    // Set entries
    metrics.set_entries("tenant_cache", 150.0);

    // Record eviction
    metrics.record_eviction("tenant_cache", "expired");

    // Record operation duration
    metrics.record_operation("tenant_cache", "get", 0.0001);
    metrics.record_operation("tenant_cache", "set", 0.0002);

    // Check hit rate
    let hit_rate = metrics.hit_rate("tenant_cache");
    assert!((hit_rate - 0.666).abs() < 0.01, "Hit rate should be ~66.6%");
}

#[test]
fn test_error_metrics() {
    let metrics = error_metrics();

    // Record errors
    metrics.record_error("content", "ValidationError", "warning");
    metrics.record_error("content", "NotFound", "info");
    metrics.record_error("commerce", "PaymentError", "error");

    // Record retry
    metrics.record_retry("content", true);
    metrics.record_retry("commerce", false);

    // Update error rate
    metrics.update_error_rate("content", 0.05);

    // Note: We don't test record_panic as it should only be called from panic hooks
}

#[test]
fn test_span_metrics() {
    let metrics = span_metrics();

    // Record span creation
    metrics.record_span("http_request", "server");
    metrics.record_span("db_query", "client");

    // Record span completion
    metrics.record_span_end("http_request", 0.045);
    metrics.record_span_end("db_query", 0.012);
}

#[test]
fn test_metrics_snapshot_health_check() {
    let snapshot = MetricsSnapshot::capture();

    // Should not have any alerts initially
    let alerts = snapshot.check_health();
    assert!(alerts.is_empty());
}

#[test]
fn test_metrics_snapshot_with_alerts() {
    let eventbus = eventbus_metrics();

    // Simulate unhealthy state
    for _ in 0..150 {
        eventbus.record_drop();
    }
    eventbus.set_lag(1500);

    let snapshot = MetricsSnapshot::capture();
    let alerts = snapshot.check_health();

    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.contains("dropped")));
    assert!(alerts.iter().any(|a| a.contains("lag")));
}

#[test]
fn test_cache_hit_rate_no_accesses() {
    let metrics = cache_metrics();

    // Hit rate should be 0 when no accesses
    let hit_rate = metrics.hit_rate("empty_cache");
    assert_eq!(hit_rate, 0.0);
}

#[test]
fn test_circuit_breaker_state_tracking() {
    let metrics = circuit_breaker_metrics();

    // Initial state: closed
    metrics.record_state("test_breaker", "closed");

    // Transition to open
    metrics.record_state_change("test_breaker", "closed", "open");

    // Transition to half-open
    metrics.record_state_change("test_breaker", "open", "half_open");

    // Back to closed
    metrics.record_state_change("test_breaker", "half_open", "closed");

    // Verify no panic occurred (state transitions work)
}

#[test]
fn test_concurrent_metrics_access() {
    use std::sync::Arc;
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let metrics = eventbus_metrics();
                for _ in 0..100 {
                    metrics.record_publish("TestEvent", true, 0.001);
                }
                i
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let snapshot = MetricsSnapshot::capture();
    // Each of 10 threads recorded 100 events
    assert!(snapshot.eventbus.events_published >= 1000);
}
