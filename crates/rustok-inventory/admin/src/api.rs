use crate::core::{InventoryProductRequest, InventoryProductsRequest};
use crate::model::{InventoryAdminBootstrap, InventoryProductDetail, InventoryProductList};
use crate::transport::{
    CommerceGraphqlInventoryReadAdapter, InventoryReadTransport, InventoryTransportError,
};

pub type ApiError = InventoryTransportError;

fn read_transport() -> impl InventoryReadTransport {
    CommerceGraphqlInventoryReadAdapter
}

async fn fetch_bootstrap_with_transport<T>(
    transport: &T,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<InventoryAdminBootstrap, ApiError>
where
    T: InventoryReadTransport,
{
    transport.fetch_bootstrap(token, tenant_slug).await
}

async fn fetch_products_with_transport<T>(
    transport: &T,
    token: Option<String>,
    tenant_slug: Option<String>,
    tenant_id: String,
    locale: Option<String>,
    search: Option<String>,
    status: Option<String>,
) -> Result<InventoryProductList, ApiError>
where
    T: InventoryReadTransport,
{
    transport
        .fetch_products(InventoryProductsRequest {
            token,
            tenant_slug,
            tenant_id,
            locale,
            search,
            status,
        })
        .await
}

async fn fetch_product_with_transport<T>(
    transport: &T,
    token: Option<String>,
    tenant_slug: Option<String>,
    tenant_id: String,
    id: String,
    locale: Option<String>,
) -> Result<Option<InventoryProductDetail>, ApiError>
where
    T: InventoryReadTransport,
{
    transport
        .fetch_product(InventoryProductRequest {
            token,
            tenant_slug,
            tenant_id,
            id,
            locale,
        })
        .await
}

pub async fn fetch_bootstrap(
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<InventoryAdminBootstrap, ApiError> {
    let transport = read_transport();
    fetch_bootstrap_with_transport(&transport, token, tenant_slug).await
}

pub async fn fetch_products(
    token: Option<String>,
    tenant_slug: Option<String>,
    tenant_id: String,
    locale: Option<String>,
    search: Option<String>,
    status: Option<String>,
) -> Result<InventoryProductList, ApiError> {
    let transport = read_transport();
    fetch_products_with_transport(
        &transport,
        token,
        tenant_slug,
        tenant_id,
        locale,
        search,
        status,
    )
    .await
}

pub async fn fetch_product(
    token: Option<String>,
    tenant_slug: Option<String>,
    tenant_id: String,
    id: String,
    locale: Option<String>,
) -> Result<Option<InventoryProductDetail>, ApiError> {
    let transport = read_transport();
    fetch_product_with_transport(&transport, token, tenant_slug, tenant_id, id, locale).await
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::{
        fetch_bootstrap_with_transport, fetch_product_with_transport, fetch_products_with_transport,
    };
    use crate::core::{InventoryProductRequest, InventoryProductsRequest};
    use crate::model::{
        CurrentTenant, InventoryAdminBootstrap, InventoryProductDetail, InventoryProductList,
    };
    use crate::transport::{InventoryReadTransport, InventoryTransportError};

    #[derive(Default)]
    struct RecordingTransport {
        bootstrap_calls: RefCell<Vec<(Option<String>, Option<String>)>>,
        products_requests: RefCell<Vec<InventoryProductsRequest>>,
        product_requests: RefCell<Vec<InventoryProductRequest>>,
    }

    impl InventoryReadTransport for RecordingTransport {
        async fn fetch_bootstrap(
            &self,
            token: Option<String>,
            tenant_slug: Option<String>,
        ) -> Result<InventoryAdminBootstrap, InventoryTransportError> {
            self.bootstrap_calls.borrow_mut().push((token, tenant_slug));
            Ok(InventoryAdminBootstrap {
                current_tenant: CurrentTenant {
                    id: "tenant-id".to_string(),
                    slug: "tenant-slug".to_string(),
                    name: "Tenant".to_string(),
                },
            })
        }

        async fn fetch_products(
            &self,
            request: InventoryProductsRequest,
        ) -> Result<InventoryProductList, InventoryTransportError> {
            self.products_requests.borrow_mut().push(request);
            Ok(InventoryProductList {
                items: Vec::new(),
                total: 0,
                page: 1,
                per_page: 24,
                has_next: false,
            })
        }

        async fn fetch_product(
            &self,
            request: InventoryProductRequest,
        ) -> Result<Option<InventoryProductDetail>, InventoryTransportError> {
            self.product_requests.borrow_mut().push(request);
            Ok(None)
        }
    }

    #[tokio::test]
    async fn bootstrap_facade_delegates_token_and_tenant_slug_to_transport() {
        let transport = RecordingTransport::default();

        let bootstrap = fetch_bootstrap_with_transport(
            &transport,
            Some("token".to_string()),
            Some("tenant-slug".to_string()),
        )
        .await
        .expect("bootstrap should come from fake transport");

        assert_eq!(bootstrap.current_tenant.id, "tenant-id");
        assert_eq!(
            transport.bootstrap_calls.borrow().as_slice(),
            &[(Some("token".to_string()), Some("tenant-slug".to_string()))]
        );
    }

    #[tokio::test]
    async fn products_facade_builds_inventory_owned_request_for_transport() {
        let transport = RecordingTransport::default();

        let list = fetch_products_with_transport(
            &transport,
            Some("token".to_string()),
            Some("tenant-slug".to_string()),
            "tenant-id".to_string(),
            Some("ru".to_string()),
            Some("coat".to_string()),
            Some("active".to_string()),
        )
        .await
        .expect("product list should come from fake transport");

        assert_eq!(list.per_page, 24);
        let requests = transport.products_requests.borrow();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.token.as_deref(), Some("token"));
        assert_eq!(request.tenant_slug.as_deref(), Some("tenant-slug"));
        assert_eq!(request.tenant_id, "tenant-id");
        assert_eq!(request.locale.as_deref(), Some("ru"));
        assert_eq!(request.search.as_deref(), Some("coat"));
        assert_eq!(request.status.as_deref(), Some("active"));
    }

    #[tokio::test]
    async fn product_facade_builds_inventory_owned_detail_request_for_transport() {
        let transport = RecordingTransport::default();

        let detail = fetch_product_with_transport(
            &transport,
            Some("token".to_string()),
            Some("tenant-slug".to_string()),
            "tenant-id".to_string(),
            "product-id".to_string(),
            Some("en".to_string()),
        )
        .await
        .expect("product detail should come from fake transport");

        assert!(detail.is_none());
        let requests = transport.product_requests.borrow();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.token.as_deref(), Some("token"));
        assert_eq!(request.tenant_slug.as_deref(), Some("tenant-slug"));
        assert_eq!(request.tenant_id, "tenant-id");
        assert_eq!(request.id, "product-id");
        assert_eq!(request.locale.as_deref(), Some("en"));
    }
}
