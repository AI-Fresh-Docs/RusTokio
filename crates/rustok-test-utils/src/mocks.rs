//! # Mock External Services
//!
//! Provides mock implementations of external services for testing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use thiserror::Error;

// ============================================================================
// Payment Gateway Mock
// ============================================================================

/// Mock payment gateway response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPaymentResponse {
    /// Payment ID
    pub payment_id: String,
    /// Order ID
    pub order_id: Uuid,
    /// Amount
    pub amount: rust_decimal::Decimal,
    /// Currency
    pub currency: String,
    /// Payment status
    pub status: PaymentStatus,
    /// Transaction ID from the gateway
    pub transaction_id: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Payment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Canceled,
    Refunded,
}

/// Payment gateway error
#[derive(Debug, Error)]
pub enum PaymentGatewayError {
    #[error("Invalid payment details: {0}")]
    InvalidDetails(String),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Gateway unavailable: {0}")]
    Unavailable(String),

    #[error("Rate limited")]
    RateLimited,
}

/// Mock payment gateway configuration
#[derive(Debug, Clone)]
pub struct MockPaymentGatewayConfig {
    /// Whether to simulate successful payments by default
    pub default_success: bool,
    /// Simulated delay in milliseconds
    pub delay_ms: u64,
    /// Simulated failure rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Rate limit in requests per minute
    pub rate_limit: Option<u32>,
}

impl Default for MockPaymentGatewayConfig {
    fn default() -> Self {
        Self {
            default_success: true,
            delay_ms: 100,
            failure_rate: 0.0,
            rate_limit: None,
        }
    }
}

/// Mock payment gateway for testing
pub struct MockPaymentGateway {
    config: MockPaymentGatewayConfig,
    payments: Arc<Mutex<HashMap<String, MockPaymentResponse>>>,
    request_count: Arc<Mutex<u32>>,
    last_request_time: Arc<Mutex<DateTime<Utc>>>,
}

impl MockPaymentGateway {
    /// Create a new mock payment gateway
    pub fn new(config: MockPaymentGatewayConfig) -> Self {
        Self {
            config,
            payments: Arc::new(Mutex::new(HashMap::new())),
            request_count: Arc::new(Mutex::new(0)),
            last_request_time: Arc::new(Mutex::new(Utc::now() - chrono::Duration::seconds(60))),
        }
    }

    /// Create a mock gateway with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MockPaymentGatewayConfig::default())
    }

    /// Create a mock gateway that always fails
    pub fn with_failures() -> Self {
        Self::new(MockPaymentGatewayConfig {
            default_success: false,
            failure_rate: 1.0,
            ..Default::default()
        })
    }

    /// Create a mock gateway with a specific failure rate
    pub fn with_failure_rate(rate: f64) -> Self {
        Self::new(MockPaymentGatewayConfig {
            failure_rate: rate,
            ..Default::default()
        })
    }

    /// Create a mock gateway with rate limiting
    pub fn with_rate_limit(limit: u32) -> Self {
        Self::new(MockPaymentGatewayConfig {
            rate_limit: Some(limit),
            ..Default::default()
        })
    }

    /// Process a payment
    pub async fn process_payment(
        &self,
        order_id: Uuid,
        amount: rust_decimal::Decimal,
        card_token: &str,
    ) -> Result<MockPaymentResponse, PaymentGatewayError> {
        // Check rate limit
        if let Some(limit) = self.config.rate_limit {
            let mut count = self.request_count.lock().unwrap();
            let mut last_time = self.last_request_time.lock().unwrap();

            let now = Utc::now();
            let elapsed = (now - *last_time).num_seconds() as u32;

            if elapsed >= 60 {
                *count = 0;
                *last_time = now;
            }

            if *count >= limit {
                return Err(PaymentGatewayError::RateLimited);
            }

            *count += 1;
        }

        // Simulate delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.config.delay_ms)).await;

        // Simulate failures based on failure rate
        if self.config.failure_rate > 0.0 {
            let rand: f64 = rand::random();
            if rand < self.config.failure_rate {
                return Err(PaymentGatewayError::PaymentFailed(
                    "Simulated payment failure".to_string(),
                ));
            }
        }

        // Validate card token
        if card_token.is_empty() {
            return Err(PaymentGatewayError::InvalidDetails(
                "Card token is required".to_string(),
            ));
        }

        // Check for known failing tokens
        if card_token == "fail_invalid_card" {
            return Err(PaymentGatewayError::InvalidDetails(
                "Invalid card details".to_string(),
            ));
        }

        if card_token == "fail_declined" {
            return Err(PaymentGatewayError::PaymentFailed(
                "Card declined".to_string(),
            ));
        }

        // Create payment response
        let payment_id = format!("pay_{}", Uuid::new_v4());
        let response = MockPaymentResponse {
            payment_id: payment_id.clone(),
            order_id,
            amount,
            currency: "USD".to_string(),
            status: if self.config.default_success {
                PaymentStatus::Succeeded
            } else {
                PaymentStatus::Failed
            },
            transaction_id: format!("txn_{}", Uuid::new_v4()),
            created_at: Utc::now(),
        };

        // Store payment
        self.payments.lock().unwrap().insert(payment_id.clone(), response.clone());

        Ok(response)
    }

    /// Get a payment by ID
    pub fn get_payment(&self, payment_id: &str) -> Option<MockPaymentResponse> {
        self.payments.lock().unwrap().get(payment_id).cloned()
    }

    /// Get all payments
    pub fn get_all_payments(&self) -> Vec<MockPaymentResponse> {
        self.payments.lock().unwrap().values().cloned().collect()
    }

    /// Clear all payments
    pub fn clear_payments(&self) {
        self.payments.lock().unwrap().clear();
    }

    /// Reset request count
    pub fn reset_request_count(&self) {
        *self.request_count.lock().unwrap() = 0;
    }
}

// ============================================================================
// Email Service Mock
// ============================================================================

/// Mock email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockEmail {
    /// Recipient email
    pub to: String,
    /// Sender email
    pub from: String,
    /// Email subject
    pub subject: String,
    /// Email body
    pub body: String,
    /// Timestamp sent
    pub sent_at: DateTime<Utc>,
}

/// Mock email service for testing
pub struct MockEmailService {
    emails: Arc<Mutex<Vec<MockEmail>>>,
}

impl MockEmailService {
    /// Create a new mock email service
    pub fn new() -> Self {
        Self {
            emails: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Send an email
    pub async fn send_email(
        &self,
        to: &str,
        from: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let email = MockEmail {
            to: to.to_string(),
            from: from.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            sent_at: Utc::now(),
        };

        self.emails.lock().unwrap().push(email);
        Ok(())
    }

    /// Get all sent emails
    pub fn get_emails(&self) -> Vec<MockEmail> {
        self.emails.lock().unwrap().clone()
    }

    /// Get emails sent to a specific recipient
    pub fn get_emails_to(&self, to: &str) -> Vec<MockEmail> {
        self.emails
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.to == to)
            .cloned()
            .collect()
    }

    /// Get emails with a specific subject
    pub fn get_emails_with_subject(&self, subject: &str) -> Vec<MockEmail> {
        self.emails
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.subject.contains(subject))
            .cloned()
            .collect()
    }

    /// Clear all emails
    pub fn clear(&self) {
        self.emails.lock().unwrap().clear();
    }

    /// Check if an email was sent
    pub fn was_email_sent(&self, to: &str, subject: &str) -> bool {
        self.emails
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.to == to && e.subject.contains(subject))
    }
}

impl Default for MockEmailService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Cache Service Mock
// ============================================================================

/// Mock cache service for testing
pub struct MockCacheService {
    cache: Arc<Mutex<HashMap<String, (String, DateTime<Utc>)>>>,
}

impl MockCacheService {
    /// Create a new mock cache service
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        cache.get(key).and_then(|(value, expires)| {
            if *expires > Utc::now() {
                Some(value.clone())
            } else {
                None
            }
        })
    }

    /// Set a value in the cache
    pub fn set(&self, key: &str, value: &str, ttl_seconds: u64) {
        let expires = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        self.cache
            .lock()
            .unwrap()
            .insert(key.to_string(), (value.to_string(), expires));
    }

    /// Delete a value from the cache
    pub fn delete(&self, key: &str) -> bool {
        self.cache.lock().unwrap().remove(key).is_some()
    }

    /// Clear all values from the cache
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }

    /// Check if a key exists
    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get the number of items in the cache
    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }
}

impl Default for MockCacheService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// File Storage Mock
// ============================================================================

/// Mock file entry
#[derive(Debug, Clone)]
pub struct MockFile {
    /// File path/key
    pub path: String,
    /// File contents
    pub contents: Vec<u8>,
    /// Content type
    pub content_type: String,
    /// File size in bytes
    pub size: usize,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Mock file storage service for testing
pub struct MockFileStorage {
    files: Arc<Mutex<HashMap<String, MockFile>>>,
    base_url: String,
}

impl MockFileStorage {
    /// Create a new mock file storage service
    pub fn new(base_url: &str) -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            base_url: base_url.to_string(),
        }
    }

    /// Store a file
    pub async fn store_file(
        &self,
        path: &str,
        contents: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        let file = MockFile {
            path: path.to_string(),
            size: contents.len(),
            content_type: content_type.to_string(),
            created_at: Utc::now(),
            contents,
        };

        self.files.lock().unwrap().insert(path.to_string(), file);
        Ok(format!("{}/{}", self.base_url, path))
    }

    /// Get a file
    pub fn get_file(&self, path: &str) -> Option<MockFile> {
        self.files.lock().unwrap().get(path).cloned()
    }

    /// Delete a file
    pub fn delete_file(&self, path: &str) -> bool {
        self.files.lock().unwrap().remove(path).is_some()
    }

    /// Check if a file exists
    pub fn file_exists(&self, path: &str) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    /// Get all files
    pub fn list_files(&self) -> Vec<MockFile> {
        self.files.lock().unwrap().values().cloned().collect()
    }

    /// Clear all files
    pub fn clear(&self) {
        self.files.lock().unwrap().clear();
    }
}

impl Default for MockFileStorage {
    fn default() -> Self {
        Self::new("https://storage.example.com")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_payment_gateway_success() {
        let gateway = MockPaymentGateway::with_defaults();
        let result = gateway
            .process_payment(
                Uuid::new_v4(),
                Decimal::new(1000, 2),
                "tok_test_card",
            )
            .await;

        assert!(result.is_ok());
        let payment = result.unwrap();
        assert_eq!(payment.status, PaymentStatus::Succeeded);
    }

    #[tokio::test]
    async fn test_payment_gateway_failure() {
        let gateway = MockPaymentGateway::with_failures();
        let result = gateway
            .process_payment(Uuid::new_v4(), Decimal::new(1000, 2), "tok_test_card")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_payment_gateway_invalid_card() {
        let gateway = MockPaymentGateway::with_defaults();
        let result = gateway
            .process_payment(
                Uuid::new_v4(),
                Decimal::new(1000, 2),
                "fail_invalid_card",
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_service() {
        let email_service = MockEmailService::new();

        email_service
            .send_email(
                "test@example.com",
                "noreply@example.com",
                "Test Subject",
                "Test Body",
            )
            .await
            .unwrap();

        assert!(email_service.was_email_sent("test@example.com", "Test Subject"));
        assert_eq!(email_service.get_emails_to("test@example.com").len(), 1);
    }

    #[test]
    fn test_cache_service() {
        let cache = MockCacheService::new();

        cache.set("key1", "value1", 60);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert!(cache.exists("key1"));

        cache.delete("key1");
        assert_eq!(cache.get("key1"), None);
    }

    #[tokio::test]
    async fn test_file_storage() {
        let storage = MockFileStorage::new("https://storage.example.com");

        let url = storage
            .store_file("test.txt", b"Hello, World!".to_vec(), "text/plain")
            .await
            .unwrap();

        assert_eq!(url, "https://storage.example.com/test.txt");
        assert!(storage.file_exists("test.txt"));

        let file = storage.get_file("test.txt").unwrap();
        assert_eq!(file.contents, b"Hello, World!");
        assert_eq!(file.content_type, "text/plain");
    }
}
