use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePageInput {
    pub locale: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub content_format: Option<String>,
    pub layout: Option<String>,
    pub parent_id: Option<Uuid>,
    pub publish: bool,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub content: String,
    pub layout: String,
    pub locale: String,
    pub parent_id: Option<Uuid>,
    pub metadata: Value,
}
