# RusToK Metrics Guide

Comprehensive guide for RusToK custom metrics, dashboards, and alerting.

## Overview

RusToK exposes detailed Prometheus metrics across multiple dimensions:

- **EventBus**: Throughput, lag, drops, subscriber counts
- **Circuit Breakers**: State transitions, success/failure rates
- **Cache**: Hit/miss rates, eviction counts, operation latency
- **Errors**: Module-specific error tracking, retry attempts
- **Spans**: OpenTelemetry span creation and duration

## Metric Reference

### EventBus Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rustok_eventbus_events_published_total` | Counter | Total events published | - |
| `rustok_eventbus_events_dropped_total` | Counter | Events dropped (channel full) | - |
| `rustok_eventbus_subscribers` | Gauge | Current subscriber count | - |
| `rustok_eventbus_events_by_type_total` | Counter | Events by type | `event_type` |
| `rustok_eventbus_publish_duration_seconds` | Histogram | Publish operation duration | `result` |
| `rustok_eventbus_lag` | Gauge | Estimated event lag | - |

**Example Queries:**

```promql
# Event throughput per second
sum(rate(rustok_eventbus_events_published_total[1m]))

# Drop rate (should be 0)
rate(rustok_eventbus_events_dropped_total[5m])

# Events by type
sum by (event_type) (rate(rustok_eventbus_events_by_type_total[5m]))

# P99 publish latency
histogram_quantile(0.99, 
  sum(rate(rustok_eventbus_publish_duration_seconds_bucket[5m])) by (le)
)
```

### Circuit Breaker Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rustok_circuit_breaker_state` | Gauge | Current state (1=active) | `name`, `state` |
| `rustok_circuit_breaker_requests_total` | Counter | Total requests | `name`, `result` |
| `rustok_circuit_breaker_transitions_total` | Counter | State transitions | `name`, `from_state`, `to_state` |
| `rustok_circuit_breaker_failure_count` | Gauge | Current failure count | `name` |
| `rustok_circuit_breaker_rejections_total` | Counter | Requests rejected | `name` |

**Example Queries:**

```promql
# Circuit breakers in OPEN state
sum by (name) (rustok_circuit_breaker_state{state="open"})

# Success rate by circuit breaker
sum by (name) (rate(rustok_circuit_breaker_requests_total{result="success"}[5m]))
/
sum by (name) (rate(rustok_circuit_breaker_requests_total[5m]))

# State transitions per minute
sum by (name, from_state, to_state) (
  rate(rustok_circuit_breaker_transitions_total[1m])
)
```

### Cache Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rustok_cache_hits_total` | Counter | Cache hits | `cache_name` |
| `rustok_cache_misses_total` | Counter | Cache misses | `cache_name` |
| `rustok_cache_entries` | Gauge | Current entry count | `cache_name` |
| `rustok_cache_evictions_total` | Counter | Evictions | `cache_name`, `reason` |
| `rustok_cache_operation_duration_seconds` | Histogram | Operation latency | `cache_name`, `operation` |

**Example Queries:**

```promql
# Hit rate by cache
rate(rustok_cache_hits_total[5m])
/
(
  rate(rustok_cache_hits_total[5m]) 
  + rate(rustok_cache_misses_total[5m])
)

# P99 cache operation latency
histogram_quantile(0.99,
  sum by (le, cache_name) (rate(rustok_cache_operation_duration_seconds_bucket[5m]))
)
```

### Error Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rustok_errors_total` | Counter | Error occurrences | `module`, `error_type`, `severity` |
| `rustok_error_rate` | Gauge | Current error rate | `module` |
| `rustok_panics_total` | Counter | Panic count | - |
| `rustok_retry_attempts_total` | Counter | Retry attempts | `module`, `result` |

**Example Queries:**

```promql
# Errors by module and type
sum by (module, error_type) (rate(rustok_errors_total[5m]))

# Overall error rate
sum(rate(rustok_errors_total[5m]))
/
sum(rate(rustok_http_requests_total[5m]))

# Retry success rate
sum by (module) (rate(rustok_retry_attempts_total{result="success"}[5m]))
/
sum by (module) (rate(rustok_retry_attempts_total[5m]))
```

### Span Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `rustok_span_count_total` | Counter | Spans created | `operation`, `span_kind` |
| `rustok_span_duration_seconds` | Histogram | Span duration | `operation` |
| `rustok_active_spans` | Gauge | Approximate active spans | - |

**Example Queries:**

```promql
# Span creation rate by operation
sum by (operation) (rate(rustok_span_count_total[1m]))

# P99 span duration
histogram_quantile(0.99,
  sum by (le, operation) (rate(rustok_span_duration_seconds_bucket[5m]))
)
```

## Alerting

### Critical Alerts (Page Immediately)

| Alert | Condition | Action |
|-------|-----------|--------|
| `HighErrorRate` | Error rate > 5% for 2m | Check logs, identify source |
| `PanicDetected` | Any panic | Check logs immediately |
| `EventBusDroppingEvents` | Events being dropped | Check EventBus subscribers |
| `EventBusHighLag` | Lag > 1000 events | Scale consumers |
| `MultipleCircuitBreakersOpen` | >2 breakers open | Check for cascading failure |
| `SLOAvailabilityBreached` | <99.9% availability | Emergency response |

### Warning Alerts (Investigate Soon)

| Alert | Condition | Action |
|-------|-----------|--------|
| `ElevatedErrorRate` | Error rate > 1% for 5m | Monitor and investigate |
| `SlowRequestsP95Warning` | P95 latency > 250ms | Profile application |
| `EventBusModerateLag` | Lag > 500 events | Monitor consumers |
| `CircuitBreakerOpened` | Any breaker opened | Check service health |
| `LowCacheHitRate` | Hit rate < 50% | Review cache strategy |

### Info Alerts (Track Trends)

| Alert | Condition | Action |
|-------|-----------|--------|
| `HighCacheEvictions` | >100 evictions/sec | Consider cache sizing |
| `UnusualContentActivity` | >3 std devs from normal | Review traffic patterns |

## Dashboards

### Advanced Metrics Dashboard

Located at `grafana/dashboards/rustok-advanced.json`

**Sections:**

1. **Overview** - Key health indicators
2. **EventBus Metrics** - Throughput, latency, lag
3. **Circuit Breakers** - States, transitions, rejection rates
4. **Cache Performance** - Hit rates, operation latency
5. **Errors & Retries** - Error rates, retry attempts
6. **Tracing Metrics** - Span rates and durations

### Quick Start

```bash
# Start the observability stack
docker-compose -f docker-compose.observability.yml up -d

# Access Grafana at http://localhost:3001
# Default credentials: admin/admin
```

## Integration Guide

### Adding Metrics to Your Module

```rust
use rustok_telemetry::metrics::{
    error_metrics, 
    span_metrics,
    MetricsSnapshot
};

pub async fn handle_request() {
    // Record span
    span_metrics().record_span("handle_request", "server");
    let start = std::time::Instant::now();
    
    let result = process().await;
    
    // Record span completion
    span_metrics().record_span_end("handle_request", start.elapsed().as_secs_f64());
    
    // Record any errors
    if let Err(e) = &result {
        error_metrics().record_error("my_module", &error_type(e), "error");
    }
    
    result
}
```

### Creating Circuit Breaker with Metrics

```rust
use rustok_core::resilience::CircuitBreaker;

let breaker = CircuitBreaker::new_with_name(
    "payment_gateway",  // Name for metrics
    CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout: Duration::from_secs(60),
        ..Default::default()
    }
);
```

### Cache Metrics Integration

```rust
use rustok_telemetry::metrics::cache_metrics;

impl Cache for MyCache {
    async fn get(&self, key: &str) -> Option<Value> {
        let start = Instant::now();
        let result = self.inner.get(key).await;
        
        match &result {
            Some(_) => cache_metrics().record_hit("my_cache"),
            None => cache_metrics().record_miss("my_cache"),
        }
        
        cache_metrics().record_operation("my_cache", "get", start.elapsed().as_secs_f64());
        result
    }
}
```

## SLOs and Targets

| SLO | Target | Metric |
|-----|--------|--------|
| Availability | 99.9% | `rustok_http_requests_total` |
| Latency (P95) | < 500ms | `rustok_http_request_duration_seconds` |
| Latency (P99) | < 1s | `rustok_http_request_duration_seconds` |
| Error Rate | < 1% | `rustok_errors_total` |
| Event Lag | < 500 events | `rustok_eventbus_lag` |
| Cache Hit Rate | > 80% | `rustok_cache_hits_total / (hits + misses)` |

## Troubleshooting

### No Metrics Appearing

1. Check metrics endpoint: `curl http://localhost:3000/api/_health/metrics`
2. Verify Prometheus target is UP in Prometheus UI
3. Check for `#[cfg(feature = "metrics")]` compilation issues

### High EventBus Lag

1. Check consumer lag: Look at `rustok_eventbus_lag`
2. Verify all subscribers are running
3. Check for slow consumers in traces
4. Consider increasing channel capacity or scaling consumers

### Circuit Breaker Chatter

1. Check transition rate: High rate indicates flapping
2. Review timeout settings
3. Check downstream service health
4. Consider increasing failure threshold

### Low Cache Hit Rate

1. Check cache size: `rustok_cache_entries`
2. Review eviction rate: `rustok_cache_evictions_total`
3. Analyze access patterns
4. Consider cache warming strategy

## Best Practices

1. **Metric Cardinality**: Keep label values bounded. Don't use user IDs or timestamps as labels.

2. **Histogram Buckets**: Use appropriate bucket boundaries for your latency distribution.

3. **Alert Fatigue**: Tune alert thresholds based on your baseline. Start with warning alerts.

4. **Dashboard Organization**: Group related metrics. Use rows to separate concerns.

5. **Documentation**: Add runbook links to alert annotations.

## Further Reading

- [OpenTelemetry Guide](./DISTRIBUTED_TRACING_GUIDE.md)
- [Observability Quick Start](../OBSERVABILITY_QUICKSTART.md)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/)
