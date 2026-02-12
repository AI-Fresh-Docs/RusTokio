//! # Test Fixtures
//!
//! Provides factory methods for creating test data entities.

use chrono::Utc;
use rustok_content::entities::{node, body};
use rustok_commerce::entities::{product, order, order_item, customer};
use rustok_core::events::types::DomainEvent;
use sea_orm::{EntityTrait, ActiveModelTrait, Set, ActiveValue};
use uuid::Uuid;
use std::sync::Arc;

// ============================================================================
// ID Generators
// ============================================================================

/// Generate a test UUID
pub fn test_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Generate a deterministic test UUID (for reproducible tests)
pub fn test_deterministic_uuid(seed: u64) -> Uuid {
    // Simple deterministic UUID generation
    let bytes = seed.to_be_bytes();
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes[0..8].copy_from_slice(&bytes);
    uuid_bytes[8..16].copy_from_slice(&bytes);
    Uuid::from_bytes(uuid_bytes)
}

// ============================================================================
// Tenant Fixtures
// ============================================================================

/// Create a test tenant identifier
pub fn test_tenant_id() -> String {
    "test-tenant".to_string()
}

/// Create a test tenant with a specific identifier
pub fn test_tenant_identifier(identifier: &str) -> String {
    identifier.to_string()
}

// ============================================================================
// User/Actor Fixtures
// ============================================================================

/// Create a test user ID
pub fn test_user_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .unwrap_or_else(|_| Uuid::new_v4())
}

/// Create a test admin user ID
pub fn test_admin_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000002")
        .unwrap_or_else(|_| Uuid::new_v4())
}

/// Create a customer ID for testing
pub fn test_customer_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000003")
        .unwrap_or_else(|_| Uuid::new_v4())
}

// ============================================================================
// Content/Node Fixtures
// ============================================================================

/// Default node kind for testing
pub fn default_node_kind() -> String {
    "article".to_string()
}

/// Create a test node input
pub fn test_node_input() -> rustok_content::dto::CreateNodeInput {
    rustok_content::dto::CreateNodeInput {
        kind: default_node_kind(),
        title: "Test Article".to_string(),
        slug: Some("test-article".to_string()),
        status: None,
        published_at: None,
        body: Some(test_body_input()),
    }
}

/// Create a test node input with custom title
pub fn test_node_input_with_title(title: &str) -> rustok_content::dto::CreateNodeInput {
    rustok_content::dto::CreateNodeInput {
        kind: default_node_kind(),
        title: title.to_string(),
        slug: None,
        status: None,
        published_at: None,
        body: Some(test_body_input()),
    }
}

/// Create a test body input
pub fn test_body_input() -> rustok_content::dto::BodyInput {
    rustok_content::dto::BodyInput {
        format: rustok_content::dto::BodyFormat::Markdown,
        content: "# Test Article\n\nThis is test content.".to_string(),
    }
}

/// Create a test translation input
pub fn test_translation_input(locale: &str) -> rustok_content::dto::TranslationInput {
    rustok_content::dto::TranslationInput {
        title: "Test Article Translation".to_string(),
        slug: Some(format!("test-article-{}", locale)),
        body: "# Translated Article\n\nTranslated content.".to_string(),
    }
}

/// Create a test node active model
pub async fn test_node_active_model(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &str,
    author_id: Uuid,
) -> node::ActiveModel {
    let node_id = test_uuid();
    node::ActiveModel {
        id: Set(node_id),
        tenant_id: Set(tenant_id.to_string()),
        kind: Set(default_node_kind()),
        status: Set(rustok_content::entities::NodeStatus::Draft.to_string()),
        title: Set("Test Article".to_string()),
        slug: Set("test-article".to_string()),
        author_id: Set(author_id),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        published_at: Set(None),
        ..Default::default()
    }
}

/// Create a test body active model
pub async fn test_body_active_model(
    db: &sea_orm::DatabaseConnection,
    node_id: Uuid,
) -> body::ActiveModel {
    body::ActiveModel {
        id: Set(test_uuid()),
        node_id: Set(node_id),
        format: Set(rustok_content::dto::BodyFormat::Markdown.to_string()),
        content: Set("# Test Body\n\nContent".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
}

// ============================================================================
// Commerce/Product Fixtures
// ============================================================================

/// Create a test product input
pub fn test_product_input() -> rustok_commerce::dto::CreateProductInput {
    rustok_commerce::dto::CreateProductInput {
        sku: "TEST-001".to_string(),
        title: "Test Product".to_string(),
        description: Some("A test product".to_string()),
        price: 1000, // $10.00 in cents
        currency: "USD".to_string(),
        inventory: 100,
        status: None,
    }
}

/// Create a test product input with custom SKU
pub fn test_product_input_with_sku(sku: &str) -> rustok_commerce::dto::CreateProductInput {
    rustok_commerce::dto::CreateProductInput {
        sku: sku.to_string(),
        title: "Test Product".to_string(),
        description: Some("A test product".to_string()),
        price: 1000,
        currency: "USD".to_string(),
        inventory: 100,
        status: None,
    }
}

/// Create a test product active model
pub async fn test_product_active_model(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &str,
) -> product::ActiveModel {
    let product_id = test_uuid();
    product::ActiveModel {
        id: Set(product_id),
        tenant_id: Set(tenant_id.to_string()),
        sku: Set("TEST-001".to_string()),
        title: Set("Test Product".to_string()),
        description: Set(Some("A test product".to_string())),
        price: Set(1000),
        currency: Set("USD".to_string()),
        inventory: Set(100),
        status: Set(rustok_commerce::entities::ProductStatus::Active.to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
}

// ============================================================================
// Commerce/Order Fixtures
// ============================================================================

/// Create a test order input
pub fn test_order_input(customer_id: Uuid) -> rustok_commerce::dto::CreateOrderInput {
    rustok_commerce::dto::CreateOrderInput {
        customer_id,
        items: vec![test_order_item_input()],
    }
}

/// Create a test order input with custom items
pub fn test_order_input_with_items(
    customer_id: Uuid,
    items: Vec<rustok_commerce::dto::OrderItemInput>,
) -> rustok_commerce::dto::CreateOrderInput {
    rustok_commerce::dto::CreateOrderInput {
        customer_id,
        items,
    }
}

/// Create a test order item input
pub fn test_order_item_input() -> rustok_commerce::dto::OrderItemInput {
    rustok_commerce::dto::OrderItemInput {
        product_id: test_uuid(),
        quantity: 2,
        price: Some(1000),
    }
}

/// Create a test order item input with custom product
pub fn test_order_item_input_with_product(product_id: Uuid, quantity: i32) -> rustok_commerce::dto::OrderItemInput {
    rustok_commerce::dto::OrderItemInput {
        product_id,
        quantity,
        price: Some(1000),
    }
}

/// Create a test payment input
pub fn test_payment_input() -> rustok_commerce::dto::ProcessPaymentInput {
    rustok_commerce::dto::ProcessPaymentInput {
        method: rustok_commerce::dto::PaymentMethod::Card,
        amount: 2000,
        currency: "USD".to_string(),
        card_token: "tok_test_visa".to_string(),
        metadata: None,
    }
}

/// Create a test order active model
pub async fn test_order_active_model(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &str,
    customer_id: Uuid,
) -> order::ActiveModel {
    let order_id = test_uuid();
    order::ActiveModel {
        id: Set(order_id),
        tenant_id: Set(tenant_id.to_string()),
        customer_id: Set(customer_id),
        status: Set(rustok_commerce::entities::OrderStatus::Draft.to_string()),
        total: Set(2000),
        currency: Set("USD".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
}

// ============================================================================
// Event Fixtures
// ============================================================================

/// Create a test NodeCreated event
pub fn test_node_created_event() -> DomainEvent {
    DomainEvent::NodeCreated {
        node_id: test_uuid(),
        kind: "article".to_string(),
        author_id: Some(test_user_id()),
        tenant_id: test_tenant_id(),
    }
}

/// Create a test NodePublished event
pub fn test_node_published_event(node_id: Uuid) -> DomainEvent {
    DomainEvent::NodePublished {
        node_id,
        kind: "article".to_string(),
        author_id: Some(test_user_id()),
        tenant_id: test_tenant_id(),
        published_at: Utc::now(),
    }
}

/// Create a test ProductCreated event
pub fn test_product_created_event() -> DomainEvent {
    DomainEvent::ProductCreated {
        product_id: test_uuid(),
        sku: "TEST-001".to_string(),
        title: "Test Product".to_string(),
        price: 1000,
        currency: "USD".to_string(),
        tenant_id: test_tenant_id(),
    }
}

/// Create a test OrderCreated event
pub fn test_order_created_event() -> DomainEvent {
    DomainEvent::OrderCreated {
        order_id: test_uuid(),
        customer_id: test_customer_id(),
        total: 2000,
        currency: "USD".to_string(),
        tenant_id: test_tenant_id(),
    }
}

/// Create a test OrderPaid event
pub fn test_order_paid_event(order_id: Uuid) -> DomainEvent {
    DomainEvent::OrderPaid {
        order_id,
        payment_id: test_uuid(),
        amount: 2000,
        currency: "USD".to_string(),
        tenant_id: test_tenant_id(),
    }
}

// ============================================================================
// Database Fixtures
// ============================================================================

/// Create an in-memory test database connection
#[cfg(feature = "test-in-memory")]
pub async fn create_test_db() -> sea_orm::DatabaseConnection {
    use sea_orm::{Database, ConnectOptions};
    use std::time::Duration;
    
    let mut opt = ConnectOptions::new("sqlite::memory:".to_string());
    opt.max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    
    Database::connect(opt).await.expect("Failed to create test database")
}

/// Create a PostgreSQL test database connection
pub async fn create_test_postgres_db() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    use sea_orm::{Database, ConnectOptions};
    use std::time::Duration;
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/rustok_test".to_string());
    
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    
    Database::connect(opt).await
}

// ============================================================================
// HTTP Fixtures
// ============================================================================

/// Create a test HTTP client
pub fn create_test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create test client")
}

/// Create a test authorization header
pub fn test_auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

/// Create a test JWT token (placeholder - use real implementation)
pub fn test_jwt_token(user_id: Uuid, tenant_id: &str) -> String {
    // In a real implementation, this would create a signed JWT
    format!("test_token_{}_{}", user_id, tenant_id)
}

// ============================================================================
// Test Assertions
// ============================================================================

/// Assert that an event exists in a list
pub fn assert_event_exists(events: &[DomainEvent], expected_event: &DomainEvent) {
    assert!(
        events.iter().any(|e| std::mem::discriminant(e) == std::mem::discriminant(expected_event)),
        "Expected event {:?} not found in events: {:?}",
        expected_event,
        events
    );
}

/// Assert that an event with a specific ID exists
pub fn assert_event_with_id_exists(events: &[DomainEvent], event_id: Uuid) {
    assert!(
        events.iter().any(|e| {
            if let DomainEvent::NodeCreated { node_id, .. } = e {
                *node_id == event_id
            } else if let DomainEvent::OrderCreated { order_id, .. } = e {
                *order_id == event_id
            } else {
                false
            }
        }),
        "Event with ID {} not found in events: {:?}",
        event_id,
        events
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generators() {
        let uuid1 = test_uuid();
        let uuid2 = test_uuid();
        
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_deterministic_uuid() {
        let uuid1 = test_deterministic_uuid(42);
        let uuid2 = test_deterministic_uuid(42);
        
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_fixtures_create_valid_inputs() {
        let node_input = test_node_input();
        assert_eq!(node_input.kind, "article");
        assert_eq!(node_input.title, "Test Article");
        
        let product_input = test_product_input();
        assert_eq!(product_input.sku, "TEST-001");
        assert_eq!(product_input.price, 1000);
    }
}
