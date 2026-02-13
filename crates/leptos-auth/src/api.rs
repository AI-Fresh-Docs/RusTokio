// REST API для аутентификации (leptos-auth)
// Использует fetch() для взаимодействия с /api/auth/* endpoints

use serde::{Deserialize, Serialize};
use crate::{AuthError, AuthSession, AuthUser};

// ============================================================================
// API Base URL
// ============================================================================

/// Get API base URL from environment or default to localhost
fn get_api_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        // В WASM получаем из window.location или env var
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("RUSTOK_API_URL")
            .unwrap_or_else(|_| "http://localhost:5150".to_string())
    }
}

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
    user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
    status: String,
}

// ============================================================================
// HTTP client helpers
// ============================================================================

#[cfg(target_arch = "wasm32")]
async fn fetch_json<T, R>(
    method: &str,
    url: &str,
    body: Option<&T>,
    token: Option<&str>,
    tenant: Option<&str>,
) -> Result<R, AuthError>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);

    let headers = web_sys::Headers::new().map_err(|_| AuthError::Network)?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|_| AuthError::Network)?;

    if let Some(token) = token {
        headers
            .set("Authorization", &format!("Bearer {}", token))
            .map_err(|_| AuthError::Network)?;
    }

    if let Some(tenant) = tenant {
        headers
            .set("X-Tenant-Slug", tenant)
            .map_err(|_| AuthError::Network)?;
    }

    opts.headers(&headers);

    if let Some(body) = body {
        let body_str = serde_json::to_string(body).map_err(|_| AuthError::Network)?;
        opts.body(Some(&JsValue::from_str(&body_str)));
    }

    let request = Request::new_with_str_and_init(url, &opts).map_err(|_| AuthError::Network)?;

    let window = web_sys::window().ok_or(AuthError::Network)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| AuthError::Network)?;

    let resp: Response = resp_value.dyn_into().map_err(|_| AuthError::Network)?;
    let status = resp.status();

    let text = JsFuture::from(resp.text().map_err(|_| AuthError::Network)?)
        .await
        .map_err(|_| AuthError::Network)?
        .as_string()
        .ok_or(AuthError::Network)?;

    if status == 200 || status == 201 {
        serde_json::from_str(&text).map_err(|_| AuthError::Network)
    } else if status == 401 {
        Err(AuthError::Unauthorized)
    } else {
        Err(AuthError::Http(status))
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_json<T, R>(
    _method: &str,
    _url: &str,
    _body: Option<&T>,
    _token: Option<&str>,
    _tenant: Option<&str>,
) -> Result<R, AuthError>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    // Non-WASM implementation (for SSR)
    // TODO: implement with reqwest or similar
    Err(AuthError::Network)
}

// ============================================================================
// Public API
// ============================================================================

/// Sign in with email and password
pub async fn sign_in(
    email: String,
    password: String,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let api_url = get_api_url();
    let url = format!("{}/api/auth/login", api_url);

    let request = SignInRequest { email, password };

    let response: AuthResponse = fetch_json("POST", &url, Some(&request), None, Some(&tenant)).await?;

    let user = AuthUser {
        id: response.user.id,
        email: response.user.email,
        name: response.user.name,
    };

    let session = AuthSession {
        token: response.access_token,
        tenant,
    };

    Ok((user, session))
}

/// Sign up with email and password
pub async fn sign_up(
    email: String,
    password: String,
    name: Option<String>,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let api_url = get_api_url();
    let url = format!("{}/api/auth/register", api_url);

    let request = SignUpRequest {
        email,
        password,
        name,
    };

    let response: AuthResponse = fetch_json("POST", &url, Some(&request), None, Some(&tenant)).await?;

    let user = AuthUser {
        id: response.user.id,
        email: response.user.email,
        name: response.user.name,
    };

    let session = AuthSession {
        token: response.access_token,
        tenant,
    };

    Ok((user, session))
}

/// Sign out (invalidate session)
pub async fn sign_out(token: String, tenant: String) -> Result<(), AuthError> {
    let api_url = get_api_url();
    let url = format!("{}/api/auth/logout", api_url);

    // Logout endpoint expects empty body
    fetch_json::<(), ()>("POST", &url, None, Some(&token), Some(&tenant)).await?;

    Ok(())
}

/// Refresh access token
pub async fn refresh_token(refresh_token: String, tenant: String) -> Result<AuthSession, AuthError> {
    let api_url = get_api_url();
    let url = format!("{}/api/auth/refresh", api_url);

    let request = RefreshRequest { refresh_token };

    let response: AuthResponse = fetch_json("POST", &url, Some(&request), None, Some(&tenant)).await?;

    let session = AuthSession {
        token: response.access_token,
        tenant,
    };

    Ok(session)
}

/// Get current user from GraphQL (uses leptos-graphql)
/// 
/// Note: Auth mutations (signIn, signUp, signOut) use REST API above,
/// but we still use GraphQL for fetching user data (query `me`)
pub async fn fetch_current_user(token: String, tenant: String) -> Result<Option<AuthUser>, AuthError> {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos_graphql::{GraphqlRequest, execute};

        const CURRENT_USER_QUERY: &str = r#"
        query CurrentUser {
            me {
                id
                email
                name
            }
        }
        "#;

        #[derive(Debug, Deserialize)]
        struct MeResponse {
            me: Option<AuthUser>,
        }

        let api_url = get_api_url();
        let graphql_url = format!("{}/api/graphql", api_url);

        let request = GraphqlRequest {
            query: CURRENT_USER_QUERY.to_string(),
            variables: serde_json::Value::Null,
        };

        let response: MeResponse = execute(&graphql_url, request, Some(&token), Some(&tenant))
            .await
            .map_err(|_| AuthError::Network)?;

        Ok(response.me)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Non-WASM fallback
        let _ = (token, tenant);
        Ok(None)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_api_url() {
        // In test environment, should default to localhost
        let url = get_api_url();
        assert!(url.contains("localhost") || url.contains("http"));
    }

    #[test]
    fn test_sign_in_request_serialization() {
        let request = SignInRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("password123"));
    }

    #[test]
    fn test_sign_up_request_serialization() {
        let request = SignUpRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: Some("Test User".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Test User"));
    }

    #[test]
    fn test_auth_user_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "email": "test@example.com",
            "name": "Test User",
            "role": "admin",
            "status": "active"
        }"#;

        let user: UserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, Some("Test User".to_string()));
    }
}
