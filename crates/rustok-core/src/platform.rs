//! Platform Detection Module
//!
//! Provides utilities for detecting which frontend platform (admin, storefront, etc.)
//! is making requests to the backend. This enables:
//! - Analytics on platform usage
//! - Platform-specific rate limiting
//! - Different API responses based on platform
//! - Platform-specific feature flags
//!
//! # Detection Methods
//!
//! The detection uses multiple methods in order of priority:
//! 1. `X-Platform` header (explicit, highest priority)
//! 2. `Origin` header (for CORS-based detection)
//! 3. `User-Agent` header (fallback, pattern matching)
//!
//! # Usage
//!
//! ```ignore
//! use rustok_core::platform::{PlatformDetector, Platform};
//!
//! let detector = PlatformDetector::new();
//! let result = detector.detect_from_headers([
//!     ("x-platform", "leptos_admin"),
//!     ("origin", "http://localhost:8080"),
//! ]);
//! assert_eq!(result.platform, Platform::LeptosAdmin);
//! ```

pub mod detection;
pub mod platform;

pub use detection::{DetectionConfidence, DetectionMethod, DetectionResult, PlatformDetector};
pub use platform::Platform;
