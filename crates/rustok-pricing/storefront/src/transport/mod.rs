mod graphql_adapter;
mod native_server_adapter;

use crate::api::ApiError;
use crate::core::StorefrontPricingQuery;
use crate::model::StorefrontPricingData;

pub(crate) async fn fetch_storefront_pricing(
    query: StorefrontPricingQuery,
) -> Result<StorefrontPricingData, ApiError> {
    match native_server_adapter::fetch_storefront_pricing(query.clone()).await {
        Ok(data) => Ok(data),
        Err(_) => graphql_adapter::fetch_storefront_pricing(query).await,
    }
}
