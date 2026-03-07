use async_graphql::{
    dataloader::DataLoader, ComplexObject, Context, Enum, InputObject, Result, SimpleObject,
};
use rustok_core::{Permission, UserRole, UserStatus};
use std::str::FromStr;
use uuid::Uuid;

use crate::graphql::common::{encode_cursor, PageInfo};
use crate::graphql::connection::ListConnection;
use crate::graphql::loaders::TenantNameLoader;
use crate::models::users;
use crate::services::auth::AuthService;

#[derive(SimpleObject, Clone)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: String,
    #[graphql(skip)]
    pub tenant_id: Uuid,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GqlUserRole {
    SuperAdmin,
    Admin,
    Manager,
    Customer,
}

impl From<GqlUserRole> for UserRole {
    fn from(role: GqlUserRole) -> Self {
        match role {
            GqlUserRole::SuperAdmin => UserRole::SuperAdmin,
            GqlUserRole::Admin => UserRole::Admin,
            GqlUserRole::Manager => UserRole::Manager,
            GqlUserRole::Customer => UserRole::Customer,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum GqlUserStatus {
    Active,
    Inactive,
    Banned,
}

impl From<GqlUserStatus> for UserStatus {
    fn from(status: GqlUserStatus) -> Self {
        match status {
            GqlUserStatus::Active => UserStatus::Active,
            GqlUserStatus::Inactive => UserStatus::Inactive,
            GqlUserStatus::Banned => UserStatus::Banned,
        }
    }
}

#[derive(InputObject, Debug, Clone)]
pub struct UsersFilter {
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[derive(InputObject, Debug, Clone)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[derive(InputObject, Debug, Clone)]
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub role: Option<GqlUserRole>,
    pub status: Option<GqlUserStatus>,
}

#[ComplexObject]
impl User {
    async fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.email.clone())
    }

    async fn can(&self, ctx: &Context<'_>, action: String) -> Result<bool> {
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let permission = Permission::from_str(&action).map_err(|err| err.to_string())?;

        AuthService::has_permission(&app_ctx.db, &self.tenant_id, &self.id, &permission)
            .await
            .map_err(|err| err.to_string().into())
    }

    async fn tenant_name(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        let loader = ctx.data::<DataLoader<TenantNameLoader>>()?;
        loader.load_one(self.tenant_id).await
    }
}

impl From<&users::Model> for User {
    fn from(model: &users::Model) -> Self {
        Self {
            id: model.id,
            email: model.email.clone(),
            name: model.name.clone(),
            role: model.role.to_string(),
            status: model.status.to_string(),
            created_at: model.created_at.to_rfc3339(),
            tenant_id: model.tenant_id,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct TenantModule {
    pub module_slug: String,
    pub enabled: bool,
    pub settings: String,
}

#[derive(SimpleObject, Clone)]
pub struct EnabledModuleItem {
    pub module_slug: String,
}

#[derive(SimpleObject, Clone)]
pub struct DeleteUserPayload {
    pub success: bool,
}

#[derive(SimpleObject, Clone)]
pub struct ModuleRegistryItem {
    pub module_slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub kind: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
}

pub type EnabledModulesConnection = ListConnection<EnabledModuleItem>;
pub type ModuleRegistryConnection = ListConnection<ModuleRegistryItem>;
pub type TenantModuleConnection = ListConnection<TenantModule>;

#[derive(SimpleObject, Debug, Clone)]
pub struct UserEdge {
    pub node: User,
    pub cursor: String,
}

#[derive(SimpleObject, Debug, Clone)]
pub struct UserConnection {
    pub edges: Vec<UserEdge>,
    pub page_info: PageInfo,
}

impl UserConnection {
    pub fn from_users(users: &[users::Model], total: i64, offset: i64, limit: i64) -> Self {
        let edges = users
            .iter()
            .enumerate()
            .map(|(index, user)| UserEdge {
                node: User::from(user),
                cursor: encode_cursor(offset + index as i64),
            })
            .collect();

        Self {
            edges,
            page_info: PageInfo::new(total, offset, limit),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_posts: i64,
    pub total_orders: i64,
    pub total_revenue: i64,
    pub users_change: f64,
    pub posts_change: f64,
    pub orders_change: f64,
    pub revenue_change: f64,
}

impl DashboardStats {
    pub fn from_metrics(metrics: DashboardStatsMetrics) -> Self {
        Self {
            total_users: metrics.total_users,
            total_posts: metrics.total_posts,
            total_orders: metrics.total_orders,
            total_revenue: metrics.total_revenue,
            users_change: metrics.users_change,
            posts_change: metrics.posts_change,
            orders_change: metrics.orders_change,
            revenue_change: metrics.revenue_change,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DashboardStatsMetrics {
    pub total_users: i64,
    pub total_posts: i64,
    pub total_orders: i64,
    pub total_revenue: i64,
    pub users_change: f64,
    pub posts_change: f64,
    pub orders_change: f64,
    pub revenue_change: f64,
}

impl DashboardStatsMetrics {
    pub fn new(
        total_users: i64,
        total_posts: i64,
        total_orders: i64,
        total_revenue: i64,
        users_change: f64,
        posts_change: f64,
        orders_change: f64,
        revenue_change: f64,
    ) -> Self {
        Self {
            total_users,
            total_posts,
            total_orders,
            total_revenue,
            users_change,
            posts_change,
            orders_change,
            revenue_change,
        }
    }
}

pub type ActivityConnection = ListConnection<ActivityItem>;

impl ActivityConnection {
    pub fn from_users(users: Vec<users::Model>, total: i64, offset: i64, limit: i64) -> Self {
        let items = users.into_iter().map(ActivityItem::from).collect();
        Self::new(items, total, offset, limit)
    }
}

#[derive(SimpleObject, Clone)]
pub struct ActivityItem {
    pub id: String,
    pub r#type: String,
    pub description: String,
    pub timestamp: String,
    pub user: Option<ActivityUser>,
}

impl From<users::Model> for ActivityItem {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id.to_string(),
            r#type: "user.created".to_string(),
            description: format!("New user {} joined", user.email),
            timestamp: user.created_at.to_rfc3339(),
            user: Some(ActivityUser {
                id: user.id.to_string(),
                name: user.name,
            }),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ActivityUser {
    pub id: String,
    pub name: Option<String>,
}
