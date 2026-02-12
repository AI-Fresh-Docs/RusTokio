# Integration Testing Guide

> **Last Updated:** 2026-02-12
> **Sprint:** 4 (Testing & Quality)
> **Author:** RustoK Team

This guide covers integration testing practices for the RusToK platform, including setup, usage examples, and best practices.

---

## Table of Contents

1. [Overview](#overview)
2. [Setup](#setup)
3. [Test Database](#test-database)
4. [Mock Services](#mock-services)
5. [Writing Integration Tests](#writing-integration-tests)
6. [Test Fixtures](#test-fixtures)
7. [Running Tests](#running-tests)
8. [CI/CD Integration](#cicd-integration)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Overview

RustoK uses a comprehensive integration testing framework built around:

- **`rustok-test-utils`**: Shared testing utilities crate
- **Test Database**: Isolated PostgreSQL database for tests
- **Mock Services**: Mock implementations of external dependencies
- **Test Fixtures**: Factory methods for creating test data
- **Test App Wrapper**: HTTP client with helper methods

### Goals

- Test complete workflows (order flow, content flow, event flow)
- Verify API endpoints and business logic
- Ensure multi-tenant isolation
- Test event propagation and delivery
- Validate error handling and edge cases

---

## Setup

### Prerequisites

1. **Docker**: Required for running test PostgreSQL container
2. **Rust**: 1.75+ with tokio runtime
3. **Environment Variables**: Configure test database URL

### Environment Variables

```bash
# Test database URL (default: postgres://postgres:password@localhost:5432/rustok_test)
export TEST_DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"

# Test server URL (default: http://localhost:3000)
export TEST_SERVER_URL="http://localhost:3000"

# Test auth token (default: test_token)
export TEST_AUTH_TOKEN="test_token"

# Test tenant ID (default: test-tenant)
export TEST_TENANT_ID="test-tenant"

# Test user ID (auto-generated if not set)
export TEST_USER_ID="00000000-0000-0000-0000-000000000001"

# Clean database before tests (default: true)
export TEST_CLEAN_DB="true"

# Run migrations on setup (default: true)
export TEST_RUN_MIGRATIONS="true"
```

### Starting Test Database

```bash
# Using docker-compose
docker-compose up -d postgres

# Or using Docker directly
docker run -d \
  --name rustok-test-db \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=rustok_test \
  -p 5432:5432 \
  postgres:16
```

### Dependencies

Add to your test crate:

```toml
[dev-dependencies]
rustok-test-utils = { path = "../../crates/rustok-test-utils" }
tokio = { workspace = true, features = ["full"] }
sea-orm = { workspace = true }
serial_test = "3.1"
```

---

## Test Database

### Basic Setup

```rust
use rustok_test_utils::{TestDb, TestDbConfig, TestDbError};

#[tokio::test]
async fn test_with_database() -> TestDbError<()> {
    // Create test database with automatic cleanup
    let test_db = TestDb::new().await?;
    let db = test_db.conn();

    // Use the database connection
    let tenant_count = count_table_rows(db, "tenants").await?;
    assert_eq!(tenant_count, 0);

    Ok(())
}
```

### Custom Configuration

```rust
use rustok_test_utils::{TestDbConfig, setup_test_db};

#[tokio::test]
async fn test_with_custom_config() -> TestDbError<()> {
    let config = TestDbConfig {
        database_url: "postgres://user:pass@host:5432/db".to_string(),
        clean_on_start: true,
        run_migrations: true,
    };

    let db = setup_test_db(Some(config)).await?;
    // Use database...

    Ok(())
}
```

### Database Helpers

```rust
use rustok_test_utils::*;

// Check if table exists
let exists = table_exists(db, "tenants").await?;
assert!(exists);

// Count rows in table
let count = count_table_rows(db, "nodes").await?;

// Run migrations
run_migrations(db).await?;

// Reset database (clean + migrate)
reset_test_db(db).await?;

// Clean database (drop all tables)
clean_test_db(db).await?;
```

### Transaction Isolation

```rust
use rustok_test_utils::with_test_transaction;

#[tokio::test]
async fn test_transaction_commit() -> TestDbError<()> {
    with_test_transaction(db, |txn| {
        // Changes will be committed
        // Insert data, etc.
        Ok(())
    }).await?;

    Ok(())
}
```

```rust
use rustok_test_utils::with_test_rollback;

#[tokio::test]
async fn test_transaction_rollback() -> TestDbError<()> {
    with_test_rollback(db, |txn| {
        // Changes will be rolled back
        // Good for test isolation
        Ok(())
    }).await?;

    Ok(())
}
```

---

## Mock Services

### Payment Gateway Mock

```rust
use rustok_test_utils::MockPaymentGateway;
use rust_decimal::Decimal;

#[tokio::test]
async fn test_payment_success() {
    // Create gateway with default configuration (all payments succeed)
    let gateway = MockPaymentGateway::with_defaults();

    let result = gateway
        .process_payment(
            order_id,
            Decimal::new(1000, 2), // $10.00
            "tok_test_card",
        )
        .await;

    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.status, PaymentStatus::Succeeded);
}
```

```rust
#[tokio::test]
async fn test_payment_failure() {
    // Gateway that always fails
    let gateway = MockPaymentGateway::with_failures();

    let result = gateway
        .process_payment(order_id, Decimal::new(1000, 2), "tok_test_card")
        .await;

    assert!(result.is_err());
}
```

```rust
#[tokio::test]
async fn test_payment_with_failure_rate() {
    // 50% failure rate
    let gateway = MockPaymentGateway::with_failure_rate(0.5);

    // Test multiple times
    for _ in 0..10 {
        let result = gateway
            .process_payment(order_id, Decimal::new(1000, 2), "tok_test_card")
            .await;

        // Some will succeed, some will fail
        println!("Result: {:?}", result.is_ok());
    }
}
```

```rust
#[tokio::test]
async fn test_rate_limiting() {
    // Rate limit: 10 requests per minute
    let gateway = MockPaymentGateway::with_rate_limit(10);

    for i in 0..15 {
        let result = gateway
            .process_payment(
                Uuid::new_v4(),
                Decimal::new(1000, 2),
                "tok_test_card",
            )
            .await;

        if i < 10 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}
```

#### Known Failing Tokens

```rust
// Invalid card details
gateway.process_payment(order_id, amount, "fail_invalid_card").await?; // Error

// Card declined
gateway.process_payment(order_id, amount, "fail_declined").await?; // Error
```

### Email Service Mock

```rust
use rustok_test_utils::MockEmailService;

#[tokio::test]
async fn test_email_sending() {
    let email_service = MockEmailService::new();

    email_service
        .send_email(
            "user@example.com",
            "noreply@rustok.com",
            "Welcome to RustoK",
            "Thanks for signing up!",
        )
        .await
        .unwrap();

    // Verify email was sent
    assert!(email_service.was_email_sent("user@example.com", "Welcome"));

    // Get all emails
    let all_emails = email_service.get_emails();
    assert_eq!(all_emails.len(), 1);

    // Get emails to specific recipient
    let user_emails = email_service.get_emails_to("user@example.com");
    assert_eq!(user_emails.len(), 1);

    // Get emails with specific subject
    let welcome_emails = email_service.get_emails_with_subject("Welcome");
    assert_eq!(welcome_emails.len(), 1);

    // Clear emails
    email_service.clear();
}
```

### Cache Service Mock

```rust
use rustok_test_utils::MockCacheService;

#[test]
fn test_cache_operations() {
    let cache = MockCacheService::new();

    // Set value with TTL (60 seconds)
    cache.set("key1", "value1", 60);

    // Get value
    assert_eq!(cache.get("key1"), Some("value1".to_string()));

    // Check if key exists
    assert!(cache.exists("key1"));

    // Delete key
    cache.delete("key1");
    assert_eq!(cache.get("key1"), None);

    // Check cache size
    cache.set("key1", "value1", 60);
    cache.set("key2", "value2", 60);
    assert_eq!(cache.len(), 2);

    // Clear all keys
    cache.clear();
    assert_eq!(cache.len(), 0);
}
```

### File Storage Mock

```rust
use rustok_test_utils::MockFileStorage;

#[tokio::test]
async fn test_file_storage() {
    let storage = MockFileStorage::new("https://storage.example.com");

    // Store a file
    let url = storage
        .store_file(
            "uploads/avatar.jpg",
            b"fake_image_data".to_vec(),
            "image/jpeg",
        )
        .await
        .unwrap();

    assert_eq!(url, "https://storage.example.com/uploads/avatar.jpg");

    // Check if file exists
    assert!(storage.file_exists("uploads/avatar.jpg"));

    // Get file
    let file = storage.get_file("uploads/avatar.jpg").unwrap();
    assert_eq!(file.contents, b"fake_image_data");
    assert_eq!(file.content_type, "image/jpeg");
    assert_eq!(file.size, 15);

    // Delete file
    storage.delete_file("uploads/avatar.jpg");
    assert!(!storage.file_exists("uploads/avatar.jpg"));

    // List all files
    storage.store_file("file1.txt", b"data1".to_vec(), "text/plain").await.unwrap();
    storage.store_file("file2.txt", b"data2".to_vec(), "text/plain").await.unwrap();

    let files = storage.list_files();
    assert_eq!(files.len(), 2);
}
```

---

## Writing Integration Tests

### Test Structure

```
apps/server/tests/
├── integration/
│   ├── order_flow_test.rs     # Order lifecycle tests
│   ├── content_flow_test.rs   # Content lifecycle tests
│   └── event_flow_test.rs     # Event propagation tests
├── multi_tenant_isolation_test.rs
├── tenant_cache_test.rs
└── module_lifecycle_test.rs
```

### Basic Test Template

```rust
use rustok_test_utils::*;
use rustok_content::dto::CreateNodeInput;

#[tokio::test]
#[serial_test::serial]  // Run sequentially to avoid conflicts
async fn test_node_creation() -> Result<(), TestAppError> {
    // Setup
    let app = spawn_test_app().await;

    // Execute
    let input = CreateNodeInput {
        kind: "article".to_string(),
        title: "Test Article".to_string(),
        slug: Some("test-article".to_string()),
        status: None,
        published_at: None,
        body: Some(test_body_input()),
    };

    let node = app.create_node(input).await?;

    // Assert
    assert_eq!(node.title, "Test Article");
    assert_eq!(node.slug.unwrap(), "test-article");

    Ok(())
}
```

### Complete Order Flow Test

```rust
#[tokio::test]
#[serial_test::serial]
async fn test_complete_order_flow() -> Result<(), TestAppError> {
    let app = spawn_test_app().await;

    // 1. Create a product
    let product = app
        .create_product(test_product_input())
        .await?;

    // 2. Create an order
    let order_input = CreateOrderInput {
        items: vec![OrderItemInput {
            product_id: product.id,
            quantity: 2,
        }],
        customer_id: Some(test_customer_id()),
    };

    let order = app.create_order(order_input).await?;
    assert_eq!(order.status, "Draft");

    // 3. Submit order
    let order = app.submit_order(order.id).await?;
    assert_eq!(order.status, "PendingPayment");

    // 4. Process payment
    let payment_input = ProcessPaymentInput {
        amount: order.total,
        card_token: "tok_test_card".to_string(),
    };

    let payment = app.process_payment(order.id, payment_input).await?;
    assert_eq!(payment.status, "Succeeded");

    // 5. Verify order status
    let order = app.get_order(order.id).await?;
    assert_eq!(order.status, "Paid");

    // 6. Verify events
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderCreated { .. })));
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderPaid { .. })));

    Ok(())
}
```

### Content Flow Test

```rust
#[tokio::test]
#[serial_test::serial]
async fn test_node_lifecycle() -> Result<(), TestAppError> {
    let app = spawn_test_app().await;

    // 1. Create node
    let node = app
        .create_node(test_node_input())
        .await?;

    assert_eq!(node.status, "Draft");

    // 2. Add Russian translation
    let translation_input = TranslationInput {
        title: "Тестовая статья".to_string(),
        body: Some(test_body_input()),
    };

    let node = app
        .add_translation(node.id, "ru", translation_input)
        .await?;

    assert!(node.translations.contains_key("ru"));

    // 3. Publish node
    let node = app.publish_node(node.id).await?;

    assert_eq!(node.status, "Published");
    assert!(node.published_at.is_some());

    // 4. Search for node
    let results = app.search_nodes("Тестовая").await?;
    assert!(!results.is_empty());

    // 5. Verify events
    let events = app.get_events_for_node(node.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::NodeCreated { .. })));
    assert!(events.iter().any(|e| matches!(e, DomainEvent::NodePublished { .. })));

    Ok(())
}
```

### Event Flow Test

```rust
#[tokio::test]
#[serial_test::serial]
async fn test_event_propagation() -> Result<(), TestAppError> {
    let app = spawn_test_app().await;

    // Subscribe to events
    app.subscribe_to_events().await;

    // Create a node (generates NodeCreated event)
    let node = app.create_node(test_node_input()).await?;

    // Wait for event propagation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify event was captured
    let events = app.get_events_for_node(node.id).await;
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::NodeCreated { .. }));

    Ok(())
}
```

---

## Test Fixtures

### Content Fixtures

```rust
use rustok_test_utils::*;

// Default node input
let input = test_node_input();

// Custom title
let input = test_node_input_with_title("Custom Title");

// Default body input
let body = test_body_input();

// Custom body
let body = test_body_with_content("# Custom Markdown");
```

### Commerce Fixtures

```rust
// Product input
let product_input = test_product_input();

// Order input
let order_input = test_order_input();

// Payment input
let payment_input = test_payment_input();
```

### Tenant/User Fixtures

```rust
// Test tenant ID
let tenant_id = test_tenant_id(); // "test-tenant"

// Custom tenant
let tenant_id = test_tenant_identifier("custom-tenant");

// Test user ID
let user_id = test_user_id();
let admin_id = test_admin_id();
let customer_id = test_customer_id();
```

### ID Generators

```rust
// Random UUID
let id = test_uuid();

// Deterministic UUID (reproducible)
let id = test_deterministic_uuid(12345);
```

---

## Running Tests

### Run All Integration Tests

```bash
# Run all tests
cargo test --test '*' -- --ignored

# Run with verbose output
cargo test -- --nocapture --test-threads=1

# Run specific test file
cargo test --test integration/order_flow_test

# Run specific test
cargo test test_complete_order_flow
```

### Run with Test Database

```bash
# Start test database
docker-compose up -d postgres

# Set environment variables
export TEST_DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"

# Run tests
cargo test --test '*' -- --ignored
```

### Run with Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir ./coverage
```

### Run Tests in Docker

```bash
# Use docker-compose test command
docker-compose -f docker-compose.test.yml up --abort-on-container-exit
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: password
          POSTGRES_DB: rustok_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run integration tests
        env:
          TEST_DATABASE_URL: postgres://postgres:password@localhost:5432/rustok_test
          TEST_CLEAN_DB: "true"
          TEST_RUN_MIGRATIONS: "true"
        run: |
          cargo test --test '*' -- --ignored --test-threads=1

      - name: Generate coverage report
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Html --output-dir ./coverage

      - name: Upload coverage
        uses: actions/upload-artifact@v3
        with:
          name: coverage-report
          path: ./coverage/index.html
```

---

## Best Practices

### 1. Use Sequential Execution

```rust
#[tokio::test]
#[serial_test::serial]  // Always use for integration tests
async fn my_test() {
    // Test code
}
```

### 2. Clean Up After Tests

```rust
#[tokio::test]
async fn test_with_cleanup() {
    let app = spawn_test_app().await;

    // Test code...

    // Clean up (automatic with TestDb Drop)
    // Or use rollback for isolation
    with_test_rollback(app.conn(), |txn| {
        // Test operations
        Ok(())
    }).await.unwrap();
}
```

### 3. Use Meaningful Assertions

```rust
// Good
assert_eq!(order.status, "Paid");
assert_eq!(payment.status, PaymentStatus::Succeeded);

// Better
assert_eq!(order.status, "Paid", "Order should be paid after successful payment");
assert_eq!(
    payment.transaction_id.len(),
    36,
    "Transaction ID should be a UUID"
);
```

### 4. Test Edge Cases

```rust
// Test invalid inputs
#[tokio::test]
async fn test_order_with_invalid_product() {
    let result = app
        .create_order(CreateOrderInput {
            items: vec![OrderItemInput {
                product_id: Uuid::new_v4(),  // Non-existent
                quantity: 1,
            }],
            customer_id: None,
        })
        .await;

    assert!(result.is_err());
}

// Test error handling
#[tokio::test]
async fn test_payment_failure() {
    // Create gateway that fails
    let gateway = MockPaymentGateway::with_failures();

    let result = gateway
        .process_payment(order_id, amount, "tok_test_card")
        .await;

    assert!(result.is_err());
}
```

### 5. Use Deterministic Test Data

```rust
// Good - deterministic
let id = test_deterministic_uuid(12345);
let tenant = test_tenant_identifier("tenant-001");

// Avoid - random
let id = Uuid::new_v4();  // Makes debugging harder
```

### 6. Test Complete Workflows

Don't just test individual operations - test entire flows:

- **Order Flow**: Create → Submit → Pay → Verify
- **Content Flow**: Create → Translate → Publish → Search
- **Event Flow**: Emit → Persist → Relay → Consume

### 7. Verify Events

Always verify that the correct events are emitted:

```rust
let events = app.get_events_for_order(order.id).await;

assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderCreated { .. })));
assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderPaid { .. })));
```

### 8. Use Mock Services

Avoid calling real external services in tests:

```rust
// Bad - calls real payment gateway
let result = real_gateway.process_payment(...).await;

// Good - uses mock
let gateway = MockPaymentGateway::with_defaults();
let result = gateway.process_payment(...).await;
```

---

## Troubleshooting

### Database Connection Errors

**Problem**: `Connection refused` or `could not connect to server`

**Solution**:
```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Start PostgreSQL
docker-compose up -d postgres

# Check logs
docker logs rustok-postgres
```

### Migration Failures

**Problem**: Migration fails during test setup

**Solution**:
```bash
# Reset test database
docker-compose exec postgres psql -U postgres -c "DROP DATABASE IF EXISTS rustok_test;"
docker-compose exec postgres psql -U postgres -c "CREATE DATABASE rustok_test;"

# Run migrations manually
cargo run --bin migrate
```

### Test Isolation Issues

**Problem**: Tests interfere with each other

**Solution**:
```rust
// Use serial_test for sequential execution
#[serial_test::serial]

// Use test rollback for isolation
with_test_rollback(db, |txn| {
    // Test code
    Ok(())
}).await
```

### Slow Tests

**Problem**: Tests take too long to run

**Solution**:
```bash
# Run tests in parallel where possible (but use serial for DB tests)
cargo test -- --test-threads=4

# Skip ignored tests during development
cargo test

# Use mock services instead of real services
let gateway = MockPaymentGateway::with_defaults();
```

### Memory Leaks

**Problem**: Tests consume too much memory

**Solution**:
```rust
// Use TestDb with automatic cleanup
{
    let test_db = TestDb::new().await?;
    // Test code
} // Database cleaned up on Drop

// Clear event buffers
app.events.lock().await.clear();

// Clear mock service state
email_service.clear();
```

---

## Additional Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [SeaORM Testing](https://www.sea-ql.org/SeaORM/docs/intro/)
- [Integration Test Examples](../../apps/server/tests/integration/)

---

**Maintained by:** RustoK Team
**Questions?** Open an issue or contact the team
