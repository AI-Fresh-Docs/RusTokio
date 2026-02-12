//! # Integration Benchmarks
//!
//! Performance benchmarks for key integration scenarios.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use rustok_test_utils::*;

/// Benchmark order creation
fn bench_order_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("order_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let _order = rt.block_on(async {
                let app = spawn_test_app().await.unwrap();
                app.create_order(test_order_input()).await
            });
        });
    });
}

/// Benchmark node creation
fn bench_node_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("node_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let _node = rt.block_on(async {
                let app = spawn_test_app().await.unwrap();
                app.create_node(test_node_input()).await
            });
        });
    });
}

/// Benchmark product creation
fn bench_product_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("product_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let _product = rt.block_on(async {
                let app = spawn_test_app().await.unwrap();
                app.create_product(test_product_input()).await
            });
        });
    });
}

/// Benchmark search operations
fn bench_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create some nodes to search
    rt.block_on(async {
        let app = spawn_test_app().await.unwrap();
        for i in 0..10 {
            let input = test_node_input_with_title(&format!("Test Article {}", i));
            let _ = app.create_node(input).await;
        }
    });

    c.bench_function("search", |b| {
        b.to_async(&rt).iter(|| async {
            let _results = rt.block_on(async {
                let app = spawn_test_app().await.unwrap();
                app.search_nodes("Test Article").await
            });
        });
    });
}

/// Benchmark event propagation
fn bench_event_propagation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("event_propagation", |b| {
        b.to_async(&rt).iter(|| async {
            let _events = rt.block_on(async {
                let app = spawn_test_app().await.unwrap();
                app.subscribe_to_events().await;
                let node = app.create_node(test_node_input()).await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                app.get_events_for_node(node.id).await
            });
        });
    });
}

/// Benchmark with varying batch sizes
fn bench_batch_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("batch_operations");
    for size in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let app = spawn_test_app().await.unwrap();
                for _ in 0..size {
                    let _ = app.create_node(test_node_input()).await;
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_order_creation,
    bench_node_creation,
    bench_product_creation,
    bench_search,
    bench_event_propagation,
    bench_batch_operations
);
criterion_main!(benches);
