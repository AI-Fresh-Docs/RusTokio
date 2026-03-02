//! Platform Detection Logic
//!
//! Provides functionality to detect which frontend platform is making
//! an HTTP request based on headers and other request metadata.

use super::platform::Platform;
use once_cell::sync::Lazy;
use regex::Regex;

/// Known platform identifiers used in the X-Platform header.
mod platform_ids {
    pub const LEPTOS_ADMIN: &str = "leptos_admin";
    pub const LEPTOS_STOREFRONT: &str = "leptos_storefront";
    pub const NEXT_ADMIN: &str = "next_admin";
    pub const NEXT_STOREFRONT: &str = "next_storefront";
    pub const MOBILE: &str = "mobile";
    pub const API: &str = "api";
}

/// Known origin patterns for different platforms.
/// These are used when X-Platform header is not present.
static ORIGIN_PATTERNS: Lazy<Vec<(&'static str, Platform)>> = Lazy::new(|| {
    vec![
        // Leptos Admin (default port 8080)
        ("http://localhost:8080", Platform::LeptosAdmin),
        ("http://127.0.0.1:8080", Platform::LeptosAdmin),
        // Leptos Storefront (default port 3100)
        ("http://localhost:3100", Platform::LeptosStorefront),
        ("http://127.0.0.1:3100", Platform::LeptosStorefront),
        // Next.js Admin (default port 3001)
        ("http://localhost:3001", Platform::NextAdmin),
        ("http://127.0.0.1:3001", Platform::NextAdmin),
        // Next.js Storefront (default port 3000)
        ("http://localhost:3000", Platform::NextStorefront),
        ("http://127.0.0.1:3000", Platform::NextStorefront),
    ]
});

/// User-Agent patterns for platform detection.
static USER_AGENT_PATTERNS: Lazy<Vec<(Regex, Platform)>> = Lazy::new(|| {
    vec![
        // Mobile apps
        (
            Regex::new(r"(?i)(okhttp|retrofit| Alamofire| AFNetworking| Volley)").unwrap(),
            Platform::Mobile,
        ),
        // Leptos (WASM-based)
        (
            Regex::new(r"(?i)(leptos|wasm)").unwrap(),
            Platform::LeptosStorefront,
        ),
    ]
});

/// HTTP headers used for platform detection.
pub mod headers {
    /// Custom header for explicit platform identification.
    pub const X_PLATFORM: &str = "x-platform";
    /// Origin header for CORS-based detection.
    pub const ORIGIN: &str = "origin";
    /// User-Agent header for pattern-based detection.
    pub const USER_AGENT: &str = "user-agent";
}

/// Platform detection result with confidence level.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub platform: Platform,
    pub confidence: DetectionConfidence,
    pub method: DetectionMethod,
}

/// Confidence level of platform detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionConfidence {
    /// High confidence - explicit header provided.
    High,
    /// Medium confidence - derived from origin or other indicators.
    Medium,
    /// Low confidence - inferred from User-Agent or other signals.
    Low,
    /// Unknown platform.
    Unknown,
}

/// Method used to detect the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionMethod {
    /// Detected via X-Platform header.
    ExplicitHeader,
    /// Detected via Origin header matching.
    Origin,
    /// Detected via User-Agent pattern matching.
    UserAgent,
    /// Default/unknown detection.
    Default,
}

impl Default for DetectionResult {
    fn default() -> Self {
        Self {
            platform: Platform::Unknown,
            confidence: DetectionConfidence::Unknown,
            method: DetectionMethod::Default,
        }
    }
}

/// Platform detector for HTTP requests.
///
/// # Detection Priority
///
/// 1. **X-Platform header** (highest priority) - Explicit platform identification
/// 2. **Origin header** - CORS-based detection using known origin patterns
/// 3. **User-Agent** (lowest priority) - Pattern matching for known clients
///
/// # Usage
///
/// ```ignore
/// use rustok_core::platform::{PlatformDetector, Platform};
///
/// let detector = PlatformDetector::new();
/// let result = detector.detect_from_headers([
///     ("x-platform", "leptos_admin"),
///     ("origin", "http://localhost:8080"),
/// ]);
/// assert_eq!(result.platform, Platform::LeptosAdmin);
/// ```
#[derive(Debug, Clone)]
pub struct PlatformDetector {
    /// Custom origin patterns (can be extended at runtime).
    origin_patterns: Vec<(String, Platform)>,
}

impl Default for PlatformDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformDetector {
    /// Creates a new platform detector with default patterns.
    pub fn new() -> Self {
        Self {
            origin_patterns: ORIGIN_PATTERNS
                .iter()
                .map(|(origin, platform)| ((*origin).to_string(), *platform))
                .collect(),
        }
    }

    /// Creates a detector with custom origin patterns.
    ///
    /// # Arguments
    /// * `patterns` - Vector of (origin_pattern, platform) tuples
    ///
    /// # Example
    ///
    /// ```ignore
    /// let detector = PlatformDetector::with_custom_origins(vec![
    ///     ("https://admin.example.com".to_string(), Platform::NextAdmin),
    ///     ("https://shop.example.com".to_string(), Platform::NextStorefront),
    /// ]);
    /// ```
    pub fn with_custom_origins(patterns: Vec<(String, Platform)>) -> Self {
        let mut detector = Self::new();
        detector.origin_patterns.extend(patterns);
        detector
    }

    /// Detects platform from a set of headers.
    ///
    /// # Arguments
    /// * `headers` - An iterator of (header_name, header_value) tuples
    ///
    /// # Returns
    /// A `DetectionResult` containing the detected platform and metadata.
    pub fn detect_from_headers<'a, I>(&self, headers: I) -> DetectionResult
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let headers_map: std::collections::HashMap<&str, &str> =
            headers.into_iter().map(|(k, v)| (k, v)).collect();

        // Priority 1: X-Platform header (explicit)
        if let Some(platform) = self.detect_from_explicit_header(&headers_map) {
            return platform;
        }

        // Priority 2: Origin header
        if let Some(platform) = self.detect_from_origin(&headers_map) {
            return platform;
        }

        // Priority 3: User-Agent
        if let Some(platform) = self.detect_from_user_agent(&headers_map) {
            return platform;
        }

        // Default: Unknown
        DetectionResult::default()
    }

    /// Detects platform from X-Platform header.
    fn detect_from_explicit_header(
        &self,
        headers: &std::collections::HashMap<&str, &str>,
    ) -> Option<DetectionResult> {
        headers
            .get(headers::X_PLATFORM)
            .and_then(|value| Platform::try_from(*value).ok())
            .map(|platform| DetectionResult {
                platform,
                confidence: DetectionConfidence::High,
                method: DetectionMethod::ExplicitHeader,
            })
    }

    /// Detects platform from Origin header.
    fn detect_from_origin(
        &self,
        headers: &std::collections::HashMap<&str, &str>,
    ) -> Option<DetectionResult> {
        let origin = headers.get(headers::ORIGIN)?;

        // Check against known patterns
        for (pattern, platform) in &self.origin_patterns {
            if origin.starts_with(pattern) || origin == pattern {
                return Some(DetectionResult {
                    platform: *platform,
                    confidence: DetectionConfidence::Medium,
                    method: DetectionMethod::Origin,
                });
            }
        }

        None
    }

    /// Detects platform from User-Agent header.
    fn detect_from_user_agent(
        &self,
        headers: &std::collections::HashMap<&str, &str>,
    ) -> Option<DetectionResult> {
        let user_agent = headers.get(headers::USER_AGENT)?;

        for (pattern, platform) in USER_AGENT_PATTERNS.iter() {
            if pattern.is_match(user_agent) {
                return Some(DetectionResult {
                    platform: *platform,
                    confidence: DetectionConfidence::Low,
                    method: DetectionMethod::UserAgent,
                });
            }
        }

        None
    }

    /// Adds a custom origin pattern.
    ///
    /// # Arguments
    /// * `origin` - Origin pattern to match (e.g., "https://admin.example.com")
    /// * `platform` - Platform to detect for this origin
    pub fn add_origin_pattern(&mut self, origin: String, platform: Platform) {
        self.origin_patterns.push((origin, platform));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_explicit_header() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([
            (headers::X_PLATFORM, "leptos_admin"),
        ]);

        assert_eq!(result.platform, Platform::LeptosAdmin);
        assert_eq!(result.confidence, DetectionConfidence::High);
        assert_eq!(result.method, DetectionMethod::ExplicitHeader);
    }

    #[test]
    fn test_detect_from_explicit_header_invalid() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([
            (headers::X_PLATFORM, "invalid_platform"),
        ]);

        // Falls through to default when header value is invalid
        assert_eq!(result.platform, Platform::Unknown);
    }

    #[test]
    fn test_detect_from_origin() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([
            (headers::ORIGIN, "http://localhost:8080"),
        ]);

        assert_eq!(result.platform, Platform::LeptosAdmin);
        assert_eq!(result.confidence, DetectionConfidence::Medium);
        assert_eq!(result.method, DetectionMethod::Origin);
    }

    #[test]
    fn test_detect_from_origin_no_match() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([
            (headers::ORIGIN, "http://unknown:9999"),
        ]);

        assert_eq!(result.platform, Platform::Unknown);
    }

    #[test]
    fn test_priority_explicit_over_origin() {
        let detector = PlatformDetector::new();

        // X-Platform should take priority over Origin
        let result = detector.detect_from_headers([
            (headers::X_PLATFORM, "next_admin"),
            (headers::ORIGIN, "http://localhost:8080"),
        ]);

        assert_eq!(result.platform, Platform::NextAdmin);
        assert_eq!(result.method, DetectionMethod::ExplicitHeader);
    }

    #[test]
    fn test_priority_origin_over_user_agent() {
        let detector = PlatformDetector::new();

        // Origin should take priority over User-Agent
        let result = detector.detect_from_headers([
            (headers::ORIGIN, "http://localhost:3100"),
            (headers::USER_AGENT, "Mozilla/5.0"),
        ]);

        assert_eq!(result.platform, Platform::LeptosStorefront);
        assert_eq!(result.method, DetectionMethod::Origin);
    }

    #[test]
    fn test_detect_from_user_agent_mobile() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([
            (headers::USER_AGENT, "okhttp/4.9.0"),
        ]);

        assert_eq!(result.platform, Platform::Mobile);
        assert_eq!(result.confidence, DetectionConfidence::Low);
        assert_eq!(result.method, DetectionMethod::UserAgent);
    }

    #[test]
    fn test_custom_origin_patterns() {
        let mut detector = PlatformDetector::new();
        detector.add_origin_pattern(
            "https://admin.custom-domain.com".to_string(),
            Platform::NextAdmin,
        );

        let result = detector.detect_from_headers([
            (headers::ORIGIN, "https://admin.custom-domain.com"),
        ]);

        assert_eq!(result.platform, Platform::NextAdmin);
    }

    #[test]
    fn test_with_custom_origins() {
        let detector = PlatformDetector::with_custom_origins(vec![
            ("https://shop.example.com".to_string(), Platform::NextStorefront),
        ]);

        let result = detector.detect_from_headers([
            (headers::ORIGIN, "https://shop.example.com"),
        ]);

        assert_eq!(result.platform, Platform::NextStorefront);
    }

    #[test]
    fn test_default_detection() {
        let detector = PlatformDetector::new();

        let result = detector.detect_from_headers([]);

        assert_eq!(result.platform, Platform::Unknown);
        assert_eq!(result.confidence, DetectionConfidence::Unknown);
        assert_eq!(result.method, DetectionMethod::Default);
    }
}
