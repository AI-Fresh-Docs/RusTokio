# Platform Detection

The platform detection module provides utilities for identifying which frontend platform is making requests to the RusToK backend.

## Overview

RusToK supports multiple frontend platforms:
- **Leptos Admin** - Leptos-based admin panel (CSR)
- **Leptos Storefront** - Leptos-based storefront (SSR)
- **Next Admin** - Next.js-based admin panel
- **Next Storefront** - Next.js-based storefront
- **Mobile** - Mobile applications
- **API** - Third-party API consumers

## Usage

### Basic Platform Detection

```rust
use rustok_core::platform::{PlatformDetector, Platform};

let detector = PlatformDetector::new();

// Detect from HTTP headers
let result = detector.detect_from_headers([
    ("x-platform", "leptos_admin"),
    ("origin", "http://localhost:8080"),
]);

assert_eq!(result.platform, Platform::LeptosAdmin);
assert_eq!(result.confidence, DetectionConfidence::High);
```

### Custom Origin Patterns

```rust
use rustok_core::platform::{PlatformDetector, Platform};

let detector = PlatformDetector::with_custom_origins(vec![
    ("https://admin.example.com".to_string(), Platform::NextAdmin),
    ("https://shop.example.com".to_string(), Platform::NextStorefront),
]);
```

## Detection Methods

The detector uses multiple methods in order of priority:

1. **X-Platform header** (highest priority) - Explicit platform identification
2. **Origin header** - CORS-based detection using known origin patterns
3. **User-Agent** (lowest priority) - Pattern matching for known clients

## Integration with HTTP Requests

For use with Axum or other HTTP frameworks:

```rust
use axum::{
    extract::Request,
    body::Body,
};
use rustok_core::platform::{PlatformDetector, Platform};

async fn handler(request: Request<Body>) {
    let detector = PlatformDetector::new();
    
    let headers: Vec<(&str, &str)> = request
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
        .collect();
    
    let result = detector.detect_from_headers(headers);
    
    match result.platform {
        Platform::LeptosAdmin => { /* admin-specific logic */ }
        Platform::LeptosStorefront => { /* storefront-specific logic */ }
        // ...
    }
}
```

## Platform Properties

The `Platform` enum provides useful methods:

```rust
use rustok_core::platform::Platform;

let platform = Platform::LeptosAdmin;

// Check if admin platform
assert!(platform.is_admin());

// Check if storefront platform
assert!(!platform.is_storefront());

// Get technology stack
assert_eq!(platform.technology(), "leptos");
```
