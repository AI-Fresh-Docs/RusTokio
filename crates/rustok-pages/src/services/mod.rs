use rustok_content::{BodyInput, CreateNodeInput, NodeService, NodeTranslationInput};
use rustok_core::{EventBus, SecurityContext};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use uuid::Uuid;

use crate::dto::{CreatePageInput, PageResponse};
use crate::error::{PageError, PageResult};
use rustok_content::entities::{body, node, node_translation};

pub struct PageService {
    db: DatabaseConnection,
    node_service: NodeService,
}

impl PageService {
    pub fn new(db: DatabaseConnection, event_bus: EventBus) -> Self {
        Self {
            node_service: NodeService::new(db.clone(), event_bus),
            db,
        }
    }

    pub async fn create_page(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreatePageInput,
    ) -> PageResult<Uuid> {
        let layout = input.layout.unwrap_or_else(|| "default".to_string());
        let mut metadata = input.metadata.unwrap_or_else(|| serde_json::json!({}));
        if let Value::Object(map) = &mut metadata {
            map.insert("layout".to_string(), serde_json::json!(layout));
        } else {
            metadata = serde_json::json!({
                "layout": layout,
                "meta": metadata,
            });
        }

        let locale = input.locale.clone();
        let node = self
            .node_service
            .create_node(
                tenant_id,
                security.clone(),
                CreateNodeInput {
                    kind: "page".to_string(),
                    status: Some(if input.publish {
                        rustok_content::entities::node::ContentStatus::Published
                    } else {
                        rustok_content::entities::node::ContentStatus::Draft
                    }),
                    parent_id: input.parent_id,
                    author_id: security.user_id,
                    category_id: None,
                    position: None,
                    depth: None,
                    reply_count: None,
                    metadata,
                    translations: vec![NodeTranslationInput {
                        locale: locale.clone(),
                        title: Some(input.title),
                        slug: Some(input.slug),
                        excerpt: None,
                    }],
                    bodies: vec![BodyInput {
                        locale,
                        body: Some(input.content),
                        format: input.content_format,
                    }],
                },
            )
            .await?;

        Ok(node.id)
    }

    pub async fn get_page_by_slug(
        &self,
        tenant_id: Uuid,
        locale: &str,
        slug: &str,
    ) -> PageResult<PageResponse> {
        let translation = node_translation::Entity::find()
            .filter(node_translation::Column::Slug.eq(slug))
            .filter(node_translation::Column::Locale.eq(locale))
            .one(&self.db)
            .await?;

        let translation = translation.ok_or_else(|| PageError::PageNotFound {
            slug: slug.to_string(),
            locale: locale.to_string(),
        })?;

        let node = node::Entity::find_by_id(translation.node_id)
            .filter(node::Column::TenantId.eq(tenant_id))
            .filter(node::Column::Kind.eq("page"))
            .filter(node::Column::Status.eq(rustok_content::entities::node::ContentStatus::Published))
            .one(&self.db)
            .await?
            .ok_or_else(|| PageError::PageNotFound {
                slug: slug.to_string(),
                locale: locale.to_string(),
            })?;

        let body = body::Entity::find()
            .filter(body::Column::NodeId.eq(node.id))
            .filter(body::Column::Locale.eq(locale))
            .one(&self.db)
            .await?;

        let content = match body {
            Some(body) => body.body.unwrap_or_default(),
            None => {
                return Err(PageError::BodyNotFound {
                    node_id: node.id,
                    locale: locale.to_string(),
                });
            }
        };

        let layout = node
            .metadata
            .get("layout")
            .and_then(|value| value.as_str())
            .unwrap_or("default")
            .to_string();

        Ok(PageResponse {
            id: node.id,
            title: translation.title,
            slug: translation.slug,
            content,
            layout,
            locale: locale.to_string(),
            parent_id: node.parent_id,
            metadata: node.metadata,
        })
    }
}
