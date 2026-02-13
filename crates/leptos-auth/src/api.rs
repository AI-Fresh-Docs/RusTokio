// GraphQL API для аутентификации (leptos-auth)
// Использует leptos-graphql как transport layer

use serde::{Deserialize, Serialize};

use crate::{AuthError, AuthSession, AuthUser};

// ============================================================================
// GraphQL Queries & Mutations (константы)
// ============================================================================

const SIGN_IN_MUTATION: &str = r#"
mutation SignIn($email: String!, $password: String!) {
    signIn(email: $email, password: $password) {
        token
        user {
            id
            email
            name
        }
    }
}
"#;

const SIGN_UP_MUTATION: &str = r#"
mutation SignUp($email: String!, $password: String!, $name: String) {
    signUp(email: $email, password: $password, name: $name) {
        token
        user {
            id
            email
            name
        }
    }
}
"#;

const SIGN_OUT_MUTATION: &str = r#"
mutation SignOut {
    signOut
}
"#;

const CURRENT_USER_QUERY: &str = r#"
query CurrentUser {
    currentUser {
        id
        email
        name
    }
}
"#;

const FORGOT_PASSWORD_MUTATION: &str = r#"
mutation ForgotPassword($email: String!) {
    forgotPassword(email: $email)
}
"#;

const RESET_PASSWORD_MUTATION: &str = r#"
mutation ResetPassword($token: String!, $newPassword: String!) {
    resetPassword(token: $token, newPassword: $newPassword)
}
"#;

const REFRESH_TOKEN_MUTATION: &str = r#"
mutation RefreshToken {
    refreshToken {
        token
    }
}
"#;

// ============================================================================
// Response Types (GraphQL data shapes)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignInData {
    #[serde(rename = "signIn")]
    sign_in: SignInPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignInPayload {
    token: String,
    user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignUpData {
    #[serde(rename = "signUp")]
    sign_up: SignUpPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignUpPayload {
    token: String,
    user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentUserData {
    #[serde(rename = "currentUser")]
    current_user: AuthUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshTokenData {
    #[serde(rename = "refreshToken")]
    refresh_token: RefreshTokenPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshTokenPayload {
    token: String,
}

// ============================================================================
// Error mapping: GraphqlHttpError → AuthError
// ============================================================================

impl From<leptos_graphql::GraphqlHttpError> for AuthError {
    fn from(err: leptos_graphql::GraphqlHttpError) -> Self {
        match err {
            leptos_graphql::GraphqlHttpError::Unauthorized => AuthError::Unauthorized,
            leptos_graphql::GraphqlHttpError::Graphql(msg) => {
                // Check for specific error types
                if msg.contains("Invalid credentials") || msg.contains("Invalid email or password")
                {
                    AuthError::InvalidCredentials
                } else if msg.contains("Unauthorized") {
                    AuthError::Unauthorized
                } else {
                    AuthError::Network
                }
            }
            leptos_graphql::GraphqlHttpError::Network => AuthError::Network,
            leptos_graphql::GraphqlHttpError::Http(status) => {
                if let Ok(code) = status.parse::<u16>() {
                    AuthError::Http(code)
                } else {
                    AuthError::Network
                }
            }
        }
    }
}

// ============================================================================
// Helper: execute_graphql с использованием leptos-graphql
// ============================================================================

async fn execute_graphql<V, T>(
    query: &str,
    variables: Option<V>,
    token: Option<String>,
    tenant: String,
) -> Result<T, AuthError>
where
    V: Serialize,
    T: serde::de::DeserializeOwned,
{
    // TODO: сделать endpoint конфигурируемым (из env или константы)
    let endpoint = "http://localhost:5150/api/graphql";

    let request = leptos_graphql::GraphqlRequest::new(query, variables);

    leptos_graphql::execute(endpoint, request, token, Some(tenant))
        .await
        .map_err(AuthError::from)
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Login via GraphQL mutation `signIn`
pub async fn sign_in(
    email: String,
    password: String,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let variables = serde_json::json!({
        "email": email,
        "password": password,
    });

    let response: SignInData =
        execute_graphql(SIGN_IN_MUTATION, Some(variables), None, tenant.clone()).await?;

    let session = AuthSession {
        token: response.sign_in.token,
        tenant,
    };

    Ok((response.sign_in.user, session))
}

/// Register via GraphQL mutation `signUp`
pub async fn sign_up(
    email: String,
    password: String,
    name: Option<String>,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    let variables = serde_json::json!({
        "email": email,
        "password": password,
        "name": name,
    });

    let response: SignUpData =
        execute_graphql(SIGN_UP_MUTATION, Some(variables), None, tenant.clone()).await?;

    let session = AuthSession {
        token: response.sign_up.token,
        tenant,
    };

    Ok((response.sign_up.user, session))
}

/// Logout via GraphQL mutation `signOut`
pub async fn sign_out(token: &str, tenant: &str) -> Result<(), AuthError> {
    let _: serde_json::Value = execute_graphql(
        SIGN_OUT_MUTATION,
        None::<serde_json::Value>,
        Some(token.to_string()),
        tenant.to_string(),
    )
    .await?;

    Ok(())
}

/// Get current user via GraphQL query `currentUser`
pub async fn get_current_user(token: &str, tenant: &str) -> Result<AuthUser, AuthError> {
    let response: CurrentUserData = execute_graphql(
        CURRENT_USER_QUERY,
        None::<serde_json::Value>,
        Some(token.to_string()),
        tenant.to_string(),
    )
    .await?;

    Ok(response.current_user)
}

/// Forgot password via GraphQL mutation `forgotPassword`
pub async fn forgot_password(email: String, tenant: String) -> Result<(), AuthError> {
    let variables = serde_json::json!({
        "email": email,
    });

    let _: serde_json::Value =
        execute_graphql(FORGOT_PASSWORD_MUTATION, Some(variables), None, tenant).await?;

    Ok(())
}

/// Reset password via GraphQL mutation `resetPassword`
pub async fn reset_password(
    token: String,
    new_password: String,
    tenant: String,
) -> Result<(), AuthError> {
    let variables = serde_json::json!({
        "token": token,
        "newPassword": new_password,
    });

    let _: serde_json::Value =
        execute_graphql(RESET_PASSWORD_MUTATION, Some(variables), None, tenant).await?;

    Ok(())
}

/// Refresh token via GraphQL mutation `refreshToken`
pub async fn refresh_token(token: &str, tenant: &str) -> Result<String, AuthError> {
    let response: RefreshTokenData = execute_graphql(
        REFRESH_TOKEN_MUTATION,
        None::<serde_json::Value>,
        Some(token.to_string()),
        tenant.to_string(),
    )
    .await?;

    Ok(response.refresh_token.token)
}
