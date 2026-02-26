use crate::{
    resolve_permissions_with_cache, PermissionCache, PermissionResolution, PermissionResolver,
    RelationPermissionStore,
};
use async_trait::async_trait;
use rustok_core::UserRole;

#[derive(Clone)]
pub struct RuntimePermissionResolver<S, C, A>
where
    S: RelationPermissionStore,
    C: PermissionCache,
    A: RoleAssignmentStore<Error = S::Error>,
{
    store: S,
    cache: C,
    assignment_store: A,
}

impl<S, C, A> RuntimePermissionResolver<S, C, A>
where
    S: RelationPermissionStore,
    C: PermissionCache,
    A: RoleAssignmentStore<Error = S::Error>,
{
    pub fn new(store: S, cache: C, assignment_store: A) -> Self {
        Self {
            store,
            cache,
            assignment_store,
        }
    }
}

#[async_trait]
pub trait RoleAssignmentStore {
    type Error;

    async fn assign_role_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;

    async fn replace_user_role(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<S, C, A> PermissionResolver for RuntimePermissionResolver<S, C, A>
where
    S: RelationPermissionStore + Send + Sync,
    C: PermissionCache + Send + Sync,
    A: RoleAssignmentStore<Error = S::Error> + Send + Sync,
    S::Error: Send + Sync,
{
    type Error = S::Error;

    async fn resolve_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<PermissionResolution, Self::Error> {
        resolve_permissions_with_cache(&self.store, &self.cache, tenant_id, user_id).await
    }

    async fn assign_role_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error> {
        self.assignment_store
            .assign_role_permissions(tenant_id, user_id, role)
            .await
    }

    async fn replace_user_role(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error> {
        self.assignment_store
            .replace_user_role(tenant_id, user_id, role)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::{RoleAssignmentStore, RuntimePermissionResolver};
    use crate::{PermissionCache, PermissionResolver, RelationPermissionStore};
    use async_trait::async_trait;
    use rustok_core::{Permission, UserRole};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct StubStore {
        role_ids: Vec<uuid::Uuid>,
        tenant_role_ids: Vec<uuid::Uuid>,
        permissions: Vec<Permission>,
    }

    #[derive(Default)]
    struct StubCache {
        values: Arc<Mutex<HashMap<(uuid::Uuid, uuid::Uuid), Vec<Permission>>>>,
    }

    #[derive(Default)]
    struct StubAssignmentStore {
        assigned: Arc<Mutex<Vec<(uuid::Uuid, uuid::Uuid, UserRole)>>>,
        replaced: Arc<Mutex<Vec<(uuid::Uuid, uuid::Uuid, UserRole)>>>,
    }

    #[async_trait]
    impl PermissionCache for StubCache {
        async fn get(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
        ) -> Option<Vec<Permission>> {
            self.values
                .lock()
                .await
                .get(&(*tenant_id, *user_id))
                .cloned()
        }

        async fn insert(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            permissions: Vec<Permission>,
        ) {
            self.values
                .lock()
                .await
                .insert((*tenant_id, *user_id), permissions);
        }

        async fn invalidate(&self, tenant_id: &uuid::Uuid, user_id: &uuid::Uuid) {
            self.values.lock().await.remove(&(*tenant_id, *user_id));
        }
    }

    #[async_trait]
    impl RelationPermissionStore for StubStore {
        type Error = String;

        async fn load_user_role_ids(
            &self,
            _user_id: &uuid::Uuid,
        ) -> Result<Vec<uuid::Uuid>, Self::Error> {
            Ok(self.role_ids.clone())
        }

        async fn load_tenant_role_ids(
            &self,
            _tenant_id: &uuid::Uuid,
            _role_ids: &[uuid::Uuid],
        ) -> Result<Vec<uuid::Uuid>, Self::Error> {
            Ok(self.tenant_role_ids.clone())
        }

        async fn load_permissions_for_roles(
            &self,
            _tenant_id: &uuid::Uuid,
            _role_ids: &[uuid::Uuid],
        ) -> Result<Vec<Permission>, Self::Error> {
            Ok(self.permissions.clone())
        }
    }

    #[async_trait]
    impl RoleAssignmentStore for StubAssignmentStore {
        type Error = String;

        async fn assign_role_permissions(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            role: UserRole,
        ) -> Result<(), Self::Error> {
            self.assigned
                .lock()
                .await
                .push((*tenant_id, *user_id, role));
            Ok(())
        }

        async fn replace_user_role(
            &self,
            tenant_id: &uuid::Uuid,
            user_id: &uuid::Uuid,
            role: UserRole,
        ) -> Result<(), Self::Error> {
            self.replaced
                .lock()
                .await
                .push((*tenant_id, *user_id, role));
            Ok(())
        }
    }

    #[tokio::test]
    async fn resolve_permissions_delegates_to_relation_and_cache_layer() {
        let role_id = uuid::Uuid::new_v4();
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let resolver = RuntimePermissionResolver::new(
            StubStore {
                role_ids: vec![role_id],
                tenant_role_ids: vec![role_id],
                permissions: vec![Permission::USERS_READ],
            },
            StubCache::default(),
            StubAssignmentStore::default(),
        );

        let first = resolver
            .resolve_permissions(&tenant_id, &user_id)
            .await
            .unwrap();
        let second = resolver
            .resolve_permissions(&tenant_id, &user_id)
            .await
            .unwrap();

        assert!(!first.cache_hit);
        assert!(second.cache_hit);
        assert_eq!(second.permissions, vec![Permission::USERS_READ]);
    }

    #[tokio::test]
    async fn role_assignment_use_cases_delegate_to_assignment_store() {
        let assignment_store = StubAssignmentStore::default();
        let resolver = RuntimePermissionResolver::new(
            StubStore {
                role_ids: vec![],
                tenant_role_ids: vec![],
                permissions: vec![],
            },
            StubCache::default(),
            assignment_store,
        );
        let tenant_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();

        resolver
            .assign_role_permissions(&tenant_id, &user_id, UserRole::Editor)
            .await
            .unwrap();
        resolver
            .replace_user_role(&tenant_id, &user_id, UserRole::Admin)
            .await
            .unwrap();

        let assigned = resolver.assignment_store.assigned.lock().await.clone();
        let replaced = resolver.assignment_store.replaced.lock().await.clone();

        assert_eq!(assigned, vec![(tenant_id, user_id, UserRole::Editor)]);
        assert_eq!(replaced, vec![(tenant_id, user_id, UserRole::Admin)]);
    }
}
