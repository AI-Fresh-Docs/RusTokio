pub mod graphql_adapter;
pub mod native_server_adapter;

use crate::api::ApiError;
use crate::model::StorefrontBlogData;

pub async fn fetch_blog(
    post_slug: String,
    locale: Option<String>,
) -> Result<StorefrontBlogData, ApiError> {
    match native_server_adapter::fetch_blog(post_slug.clone(), locale.clone()).await {
        Ok(data) => Ok(data),
        Err(_) => graphql_adapter::fetch_blog(post_slug, locale).await,
    }
}
