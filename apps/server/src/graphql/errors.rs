use async_graphql::{ErrorExtensions, FieldError};

pub type GraphQLResult<T> = Result<T, FieldError>;

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Unauthenticated,
    PermissionDenied,
    InternalError,
    NotFound,
    BadRequest,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::NotFound => "NOT_FOUND",
            Self::BadRequest => "BAD_REQUEST",
        }
    }
}

pub trait GraphQLError {
    fn unauthenticated() -> FieldError;
    fn permission_denied(message: &str) -> FieldError;
    fn internal_error(message: &str) -> FieldError;
    fn not_found(message: &str) -> FieldError;
    fn bad_request(message: &str) -> FieldError;
}

impl GraphQLError for FieldError {
    fn unauthenticated() -> FieldError {
        FieldError::new("Authentication required").extend_with(|_, e| {
            e.set("code", ErrorCode::Unauthenticated.as_str());
        })
    }

    fn permission_denied(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::PermissionDenied.as_str());
        })
    }

    fn internal_error(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::InternalError.as_str());
        })
    }

    fn not_found(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::NotFound.as_str());
        })
    }

    fn bad_request(message: &str) -> FieldError {
        FieldError::new(message).extend_with(|_, e| {
            e.set("code", ErrorCode::BadRequest.as_str());
        })
    }
}
