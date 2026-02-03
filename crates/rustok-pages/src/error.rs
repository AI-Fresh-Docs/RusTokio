use thiserror::Error;

#[derive(Debug, Error)]
pub enum PagesError {
    #[error("page not found")]
    NotFound,
}
