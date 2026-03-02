use serde::{Deserialize, Serialize};

/// Represents the different frontend platforms that can make requests to the backend.
///
/// Each variant represents a distinct frontend application:
/// - Leptos-based frontends (CSR/SSR)
/// - Next.js-based frontends
/// - Mobile apps
/// - Third-party API consumers
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Leptos-based Admin Panel (CSR)
    LeptosAdmin,
    /// Leptos-based Storefront (SSR)
    LeptosStorefront,
    /// Next.js-based Admin Panel
    NextAdmin,
    /// Next.js-based Storefront
    NextStorefront,
    /// Mobile application (React Native / Expo / etc.)
    Mobile,
    /// Third-party API consumer / webhook
    Api,
    /// Unknown or unrecognized platform
    Unknown,
}

impl Platform {
    /// Returns a human-readable name for the platform.
    pub fn name(&self) -> &'static str {
        match self {
            Platform::LeptosAdmin => "leptos_admin",
            Platform::LeptosStorefront => "leptos_storefront",
            Platform::NextAdmin => "next_admin",
            Platform::NextStorefront => "next_storefront",
            Platform::Mobile => "mobile",
            Platform::Api => "api",
            Platform::Unknown => "unknown",
        }
    }

    /// Returns whether this is an admin platform.
    pub fn is_admin(&self) -> bool {
        matches!(self, Platform::LeptosAdmin | Platform::NextAdmin)
    }

    /// Returns whether this is a storefront/platform.
    pub fn is_storefront(&self) -> bool {
        matches!(self, Platform::LeptosStorefront | Platform::NextStorefront)
    }

    /// Returns whether this is a browser-based platform.
    pub fn is_browser(&self) -> bool {
        matches!(
            self,
            Platform::LeptosAdmin
                | Platform::LeptosStorefront
                | Platform::NextAdmin
                | Platform::NextStorefront
        )
    }

    /// Returns the technology stack for this platform.
    pub fn technology(&self) -> &'static str {
        match self {
            Platform::LeptosAdmin | Platform::LeptosStorefront => "leptos",
            Platform::NextAdmin | Platform::NextStorefront => "nextjs",
            Platform::Mobile => "mobile",
            Platform::Api => "api",
            Platform::Unknown => "unknown",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Unknown
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TryFrom<&str> for Platform {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "leptos_admin" | "leptos-admin" => Ok(Platform::LeptosAdmin),
            "leptos_storefront" | "leptos-storefront" => Ok(Platform::LeptosStorefront),
            "next_admin" | "next-admin" | "nextadmin" => Ok(Platform::NextAdmin),
            "next_storefront" | "next-storefront" | "nextstorefront" => Ok(Platform::NextStorefront),
            "mobile" => Ok(Platform::Mobile),
            "api" => Ok(Platform::Api),
            "unknown" => Ok(Platform::Unknown),
            _ => Err(format!("Unknown platform: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::LeptosAdmin.name(), "leptos_admin");
        assert_eq!(Platform::NextStorefront.name(), "next_storefront");
    }

    #[test]
    fn test_platform_is_admin() {
        assert!(Platform::LeptosAdmin.is_admin());
        assert!(Platform::NextAdmin.is_admin());
        assert!(!Platform::LeptosStorefront.is_admin());
    }

    #[test]
    fn test_platform_is_storefront() {
        assert!(Platform::LeptosStorefront.is_storefront());
        assert!(Platform::NextStorefront.is_storefront());
        assert!(!Platform::LeptosAdmin.is_storefront());
    }

    #[test]
    fn test_platform_is_browser() {
        assert!(Platform::LeptosAdmin.is_browser());
        assert!(Platform::NextStorefront.is_browser());
        assert!(!Platform::Mobile.is_browser());
    }

    #[test]
    fn test_platform_technology() {
        assert_eq!(Platform::LeptosAdmin.technology(), "leptos");
        assert_eq!(Platform::NextStorefront.technology(), "nextjs");
    }

    #[test]
    fn test_platform_try_from() {
        assert_eq!(Platform::try_from("leptos_admin").unwrap(), Platform::LeptosAdmin);
        assert_eq!(Platform::try_from("next-storefront").unwrap(), Platform::NextStorefront);
        assert_eq!(Platform::try_from("mobile").unwrap(), Platform::Mobile);
        assert!(Platform::try_from("invalid").is_err());
    }

    #[test]
    fn test_platform_default() {
        assert_eq!(Platform::default(), Platform::Unknown);
    }
}
