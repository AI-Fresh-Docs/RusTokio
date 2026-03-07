use async_graphql::{Context, FieldError, Object};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::graphql::common::PaginationInput;
use crate::graphql::errors::{GraphQLError, GraphQLResult};
use rustok_commerce::CatalogService;
use rustok_outbox::TransactionalEventBus;

use super::types::*;

#[derive(Default)]
pub struct CommerceQuery;

#[Object]
impl CommerceQuery {
    async fn product(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        id: Uuid,
        locale: Option<String>,
    ) -> GraphQLResult<Option<GqlProduct>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let locale = locale.unwrap_or_else(|| "en".to_string());

        let service = CatalogService::new(db.clone(), event_bus.clone());
        let product = match service.get_product(tenant_id, id).await {
            Ok(product) => product,
            Err(rustok_commerce::CommerceError::ProductNotFound(_)) => return Ok(None),
            Err(err) => {
                return Err(<FieldError as GraphQLError>::internal_error(&err.to_string()));
            }
        };

        let filtered_translations = product
            .translations
            .into_iter()
            .filter(|translation| translation.locale == locale)
            .collect::<Vec<_>>();

        let product = rustok_commerce::dto::ProductResponse {
            translations: filtered_translations,
            ..product
        };

        Ok(Some(GqlProduct::from_data(product.into())))
    }

    async fn products(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        locale: Option<String>,
        filter: Option<ProductsFilter>,
        #[graphql(default)] pagination: PaginationInput,
    ) -> GraphQLResult<GqlProductConnection> {
        let db = ctx.data::<DatabaseConnection>()?;
        let locale = locale.unwrap_or_else(|| "en".to_string());
        let filter = filter.unwrap_or(ProductsFilter {
            status: None,
            vendor: None,
            search: None,
        });

        use rustok_commerce::entities::{product, product_translation};
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};

        let (offset, limit) = pagination.normalize()?;

        let mut query = product::Entity::find().filter(product::Column::TenantId.eq(tenant_id));

        if let Some(status) = &filter.status {
            let status: rustok_commerce::entities::product::ProductStatus = (*status).into();
            query = query.filter(product::Column::Status.eq(status));
        }
        if let Some(vendor) = &filter.vendor {
            query = query.filter(product::Column::Vendor.eq(vendor));
        }

        if let Some(search) = &filter.search {
            let search_ids: Vec<Uuid> = product_translation::Entity::find()
                .filter(product_translation::Column::Locale.eq(&locale))
                .filter(product_translation::Column::Title.contains(search))
                .all(db)
                .await
                .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?
                .into_iter()
                .map(|translation| translation.product_id)
                .collect();

            if search_ids.is_empty() {
                return Ok(GqlProductConnection::new(Vec::new(), 0, offset, limit));
            }

            query = query.filter(product::Column::Id.is_in(search_ids));
        }

        let total = query
            .clone()
            .count(db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))? as i64;
        let products = query
            .order_by_desc(product::Column::CreatedAt)
            .offset(offset as u64)
            .limit(limit as u64)
            .all(db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        let product_ids: Vec<Uuid> = products.iter().map(|product| product.id).collect();
        let translations = product_translation::Entity::find()
            .filter(product_translation::Column::ProductId.is_in(product_ids))
            .filter(product_translation::Column::Locale.eq(&locale))
            .all(db)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        let translation_map: std::collections::HashMap<Uuid, _> = translations
            .into_iter()
            .map(|translation| (translation.product_id, translation))
            .collect();

        let items = products
            .into_iter()
            .map(|product| {
                let translation = translation_map.get(&product.id);
                GqlProductListItemData {
                    id: product.id,
                    status: product.status.into(),
                    title: translation
                        .map(|value| value.title.clone())
                        .unwrap_or_default(),
                    handle: translation
                        .map(|value| value.handle.clone())
                        .unwrap_or_default(),
                    vendor: product.vendor,
                    created_at: product.created_at.to_rfc3339(),
                }
            })
            .map(GqlProductListItem::from_data)
            .collect();

        Ok(GqlProductConnection::new(items, total, offset, limit))
    }
}
