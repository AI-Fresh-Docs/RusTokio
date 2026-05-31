use crate::api::{self, ApiError};
use crate::model::StorefrontBlogData;

pub async fn fetch_blog(
    post_slug: String,
    locale: Option<String>,
) -> Result<StorefrontBlogData, ApiError> {
    api::fetch_storefront_blog_server(api::configured_tenant_slug(), post_slug, locale).await
}
