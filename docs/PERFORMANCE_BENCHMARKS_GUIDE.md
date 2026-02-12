# Performance Benchmarks Guide

> **Last Updated:** 2026-02-12
> **Sprint:** 4 (Testing & Quality)
> **Author:** RustoK Team

This guide covers performance benchmarking practices for the RusToK platform using Criterion.rs.

---

## Table of Contents

1. [Overview](#overview)
2. [Setup](#setup)
3. [Running Benchmarks](#running-benchmarks)
4. [Writing Benchmarks](#writing-benchmarks)
5. [Benchmark Scenarios](#benchmark-scenarios)
6. [Analyzing Results](#analyzing-results)
7. [CI/CD Integration](#cicd-integration)
8. [Best Practices](#best-practices)

---

## Overview

RustoK uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for benchmarking, which provides:

- **Statistical Analysis**: Automatic statistical analysis of benchmark results
- **Comparisons**: Easy comparison between different implementations
- **HTML Reports**: Interactive HTML reports with charts
- **Regression Detection**: Automatic detection of performance regressions
- **Plotters**: Support for generating plots

### Current Benchmarks

The following benchmarks are available in `apps/server/benches/integration_benchmarks.rs`:

| Benchmark | Description |
|-----------|-------------|
| `order_creation` | Time to create a new order |
| `node_creation` | Time to create a new content node |
| `product_creation` | Time to create a new product |
| `search` | Time to search for nodes |
| `event_propagation` | Time for events to propagate |
| `batch_operations` | Performance with varying batch sizes |

---

## Setup

### Prerequisites

1. **Rust**: 1.75+ with tokio runtime
2. **Criterion**: Added as dev-dependency
3. **Test Database**: PostgreSQL running for integration benchmarks

### Dependencies

Criterion is already configured in `apps/server/Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "integration_benchmarks"
harness = false
```

### Environment Variables

Set the same environment variables as integration tests:

```bash
export TEST_DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"
export TEST_SERVER_URL="http://localhost:3000"
export TEST_CLEAN_DB="true"
export TEST_RUN_MIGRATIONS="true"
```

---

## Running Benchmarks

### Run All Benchmarks

```bash
# Run all benchmarks
cargo bench --bench integration_benchmarks

# Run with verbose output
cargo bench --bench integration_benchmarks -- --verbose

# Run specific benchmark
cargo bench --bench integration_benchmarks order_creation

# Run with save baseline
cargo bench --bench integration_benchmarks -- --save-baseline main

# Compare against baseline
cargo bench --bench integration_benchmarks -- --baseline main
```

### Run with Different Profiles

```bash
# Run with release optimizations
cargo bench --release --bench integration_benchmarks

# Run with debug info
cargo bench --bench integration_benchmarks -- --profile-time 5
```

### Generate Reports

```bash
# Run benchmarks (automatically generates HTML report)
cargo bench --bench integration_benchmarks

# View HTML report
open target/criterion/report/index.html
```

---

## Writing Benchmarks

### Basic Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
            my_function(black_box(input_data))
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### Async Benchmark Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustok_test_utils::*;

fn bench_async_operation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("async_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let result = async_operation(black_box(input_data)).await;
            black_box(result)
        });
    });
}
```

### Benchmark with Setup

```rust
fn bench_with_setup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create test data
    let setup = || {
        rt.block_on(async {
            let app = spawn_test_app().await.unwrap();
            let node = app.create_node(test_node_input()).await.unwrap();
            (app, node)
        })
    };

    c.bench_function("operation", |b| {
        b.iter_batched(
            || setup(),
            |(app, node)| {
                rt.block_on(async {
                    // Benchmark operation
                    app.get_node(node.id).await.unwrap()
                })
            },
            criterion::BatchSize::SmallInput,
        )
    });
}
```

### Benchmark Multiple Inputs

```rust
fn bench_with_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_group");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                operation_with_size(black_box(size))
            });
        });
    }

    group.finish();
}
```

---

## Benchmark Scenarios

### 1. Order Creation Benchmark

```rust
fn bench_order_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("order_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            black_box(app.create_order(test_order_input()).await)
        });
    });
}
```

**What it measures**:
- API request time
- Database insert time
- Business logic execution time

### 2. Node Creation Benchmark

```rust
fn bench_node_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("node_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            black_box(app.create_node(test_node_input()).await)
        });
    });
}
```

**What it measures**:
- Content creation API performance
- Node validation time
- Database transaction time

### 3. Search Benchmark

```rust
fn bench_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create nodes
    rt.block_on(async {
        let app = spawn_test_app().await.unwrap();
        for i in 0..10 {
            let input = test_node_input_with_title(&format!("Test {}", i));
            app.create_node(input).await.unwrap();
        }
    });

    c.bench_function("search", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            black_box(app.search_nodes("Test").await)
        });
    });
}
```

**What it measures**:
- Search query performance
- Index lookup time
- Result serialization time

### 4. Event Propagation Benchmark

```rust
fn bench_event_propagation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("event_propagation", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            app.subscribe_to_events().await;
            let node = app.create_node(test_node_input()).await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;
            black_box(app.get_events_for_node(node.id).await)
        });
    });
}
```

**What it measures**:
- Event emission time
- Event persistence time
- Event relay time

### 5. Batch Operations Benchmark

```rust
fn bench_batch_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_operations");
    for size in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let app = spawn_test_app().await.unwrap();
                for _ in 0..size {
                    black_box(app.create_node(test_node_input()).await)
                }
            });
        });
    }
    group.finish();
}
```

**What it measures**:
- Performance scaling with batch size
- Database connection pool efficiency
- Transaction overhead

---

## Analyzing Results

### HTML Report

After running benchmarks, open the HTML report:

```bash
open target/criterion/report/index.html
```

The report includes:
- **Summary**: Overview of all benchmarks
- **Charts**: Performance over time
- **Statistics**: Mean, median, std dev
- **Comparisons**: Side-by-side comparisons

### Key Metrics

- **Mean Time**: Average execution time
- **Median Time**: 50th percentile
- **Std Dev**: Variability in execution time
- **Throughput**: Operations per second
- **Slope**: Performance trend over iterations

### Interpreting Results

| Metric | Good | Concerning |
|--------|------|------------|
| Mean Time | Consistent, low variance | Increasing, high variance |
| Std Dev | < 10% of mean | > 20% of mean |
| Slope | Flat or decreasing | Increasing |
| Comparison | Within 5% of baseline | > 10% regression |

### Detecting Regressions

Criterion automatically detects performance regressions:

```bash
# Save baseline before changes
cargo bench --bench integration_benchmarks -- --save-baseline main

# Make changes

# Compare against baseline
cargo bench --bench integration_benchmarks -- --baseline main
```

Look for:
- **Red indicators**: Performance regression detected
- **Green indicators**: No significant change
- **Yellow indicators**: Inconclusive results

---

## CI/CD Integration

### GitHub Actions Benchmark Job

Add to `.github/workflows/benchmarks.yml`:

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: rustok_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    env:
      TEST_DATABASE_URL: postgres://postgres:postgres@localhost:5432/rustok_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run benchmarks
        run: |
          cd apps/server
          cargo bench --bench integration_benchmarks -- --output-format bencher
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### Continuous Benchmarking with Bencher

For continuous benchmark tracking, use [Bencher](https://bencher.dev/):

```bash
# Install bencher-cli
cargo install bencher

# Run benchmarks with bencher
cargo bench --bench integration_benchmarks --bencher
```

---

## Best Practices

### 1. Use `black_box`

Always use `black_box` to prevent compiler optimizations:

```rust
// Bad
b.iter(|| my_function(input_data))

// Good
b.iter(|| my_function(black_box(input_data)))
```

### 2. Benchmark Realistic Scenarios

Don't benchmark just the happy path:

```rust
// Good - benchmarks typical workflow
fn bench_complete_order_flow(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("complete_order_flow", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            let order = app.create_order(test_order_input()).await.unwrap();
            let order = app.submit_order(order.id).await.unwrap();
            black_box(app.process_payment(order.id, test_payment_input()).await)
        });
    });
}
```

### 3. Measure End-to-End Performance

Benchmark from user perspective:

```rust
// Good - measures full API call
fn bench_api_call(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("create_node_api", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            black_box(app.create_node(test_node_input()).await)
        });
    });
}
```

### 4. Use Appropriate Batch Sizes

Choose batch size based on input:

```rust
use criterion::BatchSize;

b.iter_batched(
    || setup(),
    |input| operation(input),
    criterion::BatchSize::SmallInput,  // or MediumInput, LargeInput, PerIteration
)
```

### 5. Isolate Benchmarks

Run each benchmark in isolation:

```bash
# Run specific benchmark only
cargo bench --bench integration_benchmarks order_creation

# Skip other benchmarks
cargo bench --bench integration_benchmarks -- --skip search
```

### 6. Compare Against Baselines

Always compare against a known baseline:

```bash
# Save baseline before optimization work
cargo bench --bench integration_benchmarks -- --save-baseline before_optimization

# After changes, compare
cargo bench --bench integration_benchmarks -- --baseline before_optimization
```

### 7. Document Performance Goals

Document expected performance in benchmarks:

```rust
// Benchmark should complete in < 10ms
fn bench_fast_operation(c: &mut Criterion) {
    c.bench_function("fast_operation", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            fast_operation(black_box(input));
            let elapsed = start.elapsed();
            assert!(elapsed.as_millis() < 10, "Operation took too long");
        });
    });
}
```

### 8. Warm Up Cold Starts

For databases/caches, warm up first:

```rust
fn bench_cache(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Warm up
    rt.block_on(async {
        let app = spawn_test_app().await.unwrap();
        for _ in 0..10 {
            app.get_node(test_uuid()).await.ok();
        }
    });

    c.bench_function("cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let app = spawn_test_app().await.unwrap();
            black_box(app.get_node(test_uuid()).await)
        });
    });
}
```

### 9. Measure Memory Usage

Use tools alongside benchmarks:

```bash
# Run with memory profiler
cargo bench --bench integration_benchmarks 2>&1 | tee bench.log

# Analyze with heaptrack
heaptrack cargo bench --bench integration_benchmarks
```

### 10. Test on Realistic Hardware

Run benchmarks on production-like hardware:

```bash
# Run on remote server
ssh production-server
cargo bench --bench integration_benchmarks

# Compare results locally
cargo bench --bench integration_benchmarks -- --baseline production
```

---

## Troubleshooting

### Benchmarks Too Noisy

**Problem**: High variance in results

**Solutions**:
- Increase measurement time: `--measurement-time 10`
- Reduce system load
- Disable CPU frequency scaling
- Use `--warm-up-time 5` to warm up

### Benchmarks Too Slow

**Problem**: Benchmarks take too long

**Solutions**:
- Reduce sample size: `--sample-size 50`
- Reduce warm-up time: `--warm-up-time 1`
- Reduce measurement time: `--measurement-time 3`
- Use `--profile-time 5` to limit

### No Results Generated

**Problem**: No HTML report generated

**Solutions**:
- Check `target/criterion/` directory
- Ensure benchmarks complete successfully
- Check file permissions
- Run with `--verbose` to debug

### Comparison Fails

**Problem**: Cannot compare against baseline

**Solutions**:
- Ensure baseline was saved: `--save-baseline main`
- Check baseline directory exists: `target/criterion/main/`
- Verify same benchmark names
- Use `--baseline main` to compare

---

## Additional Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/index.html)
- [Criterion.rs GitHub](https://github.com/bheisler/criterion.rs)
- [Bencher - Continuous Benchmarking](https://bencher.dev/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

---

**Maintained by:** RustoK Team
**Questions?** Open an issue or contact the team
