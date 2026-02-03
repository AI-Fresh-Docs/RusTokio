use rustok_content::ContentError;
use sea_orm::DbErr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PageError {
    #[error("Content error: {0}")]
    Content(#[from] ContentError),

    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Page not found for slug '{slug}' and locale '{locale}'")]
    PageNotFound { slug: String, locale: String },

    #[error("Missing body for page {node_id} and locale {locale}")]
    BodyNotFound { node_id: Uuid, locale: String },
}

pub type PageResult<T> = Result<T, PageError>;
