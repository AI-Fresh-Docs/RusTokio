# leptos-graphql Enhancement Plan

**Дата:** 2026-02-14  
**Цель:** Расширить функционал leptos-graphql для более удобной работы с GraphQL в админках

---

## 📋 Текущее состояние

### Что есть (v0.1.0):

✅ **Core functionality:**
- `GraphqlRequest<V>` — структура запроса
- `GraphqlResponse<T>` — структура ответа
- `execute()` — async функция для выполнения запросов
- Error handling: `GraphqlHttpError` (Network, Graphql, Http, Unauthorized)
- Persisted queries support

✅ **Features:**
- Автоматическая вставка заголовков (Authorization, X-Tenant-Slug)
- Type-safe requests/responses через generics
- Serde serialization/deserialization

### Что используется:

```rust
use leptos_graphql::{execute, GraphqlRequest};

let request = GraphqlRequest::new(query, Some(variables));
let response: MyData = execute(endpoint, request, token, tenant).await?;
```

---

## 🎯 Что нужно добавить

### 1. Reactive Hooks для Queries

**Проблема:** Сейчас нужно вручную вызывать `execute()` в `spawn_local` и управлять состоянием загрузки/ошибок.

**Решение:** Добавить hooks `use_query()` и `use_mutation()` по аналогии с urql/Apollo.

#### `use_query()` hook

```rust
// crates/leptos-graphql/src/hooks.rs

use leptos::*;
use serde::{de::DeserializeOwned, Serialize};
use crate::{execute, GraphqlRequest, GraphqlHttpError};

pub struct QueryResult<T> {
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<GraphqlHttpError>>,
    pub loading: Signal<bool>,
    pub refetch: Box<dyn Fn() + 'static>,
}

/// Hook для выполнения GraphQL query с reactive state
pub fn use_query<V, T>(
    endpoint: String,
    query: String,
    variables: Option<V>,
    token: Option<String>,
    tenant: Option<String>,
) -> QueryResult<T>
where
    V: Serialize + Clone + 'static,
    T: DeserializeOwned + Clone + 'static,
{
    let (data, set_data) = create_signal(None);
    let (error, set_error) = create_signal(None);
    let (loading, set_loading) = create_signal(true);
    
    let fetch = move || {
        set_loading.set(true);
        set_error.set(None);
        
        let endpoint = endpoint.clone();
        let query = query.clone();
        let variables = variables.clone();
        let token = token.clone();
        let tenant = tenant.clone();
        
        spawn_local(async move {
            let request = GraphqlRequest::new(query, variables);
            
            match execute::<V, T>(&endpoint, request, token, tenant).await {
                Ok(response) => {
                    set_data.set(Some(response));
                    set_loading.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err));
                    set_loading.set(false);
                }
            }
        });
    };
    
    // Initial fetch
    create_effect(move |_| {
        fetch();
    });
    
    QueryResult {
        data,
        error,
        loading,
        refetch: Box::new(fetch),
    }
}
```

**Usage:**

```rust
use leptos_graphql::use_query;

#[component]
fn UsersList() -> impl IntoView {
    let query = r#"
        query GetUsers($limit: Int!) {
            users(limit: $limit) {
                nodes { id email name }
            }
        }
    "#;
    
    let variables = json!({ "limit": 10 });
    
    let result = use_query(
        "/api/graphql".to_string(),
        query.to_string(),
        Some(variables),
        Some(token),
        Some(tenant),
    );
    
    view! {
        <Show when=move || result.loading.get()>
            <p>"Loading..."</p>
        </Show>
        
        <Show when=move || result.error.get().is_some()>
            <p class="text-red-500">
                {move || result.error.get().unwrap().to_string()}
            </p>
        </Show>
        
        <Show when=move || result.data.get().is_some()>
            <ul>
                {move || result.data.get().map(|users| {
                    users.nodes.iter().map(|user| view! {
                        <li>{&user.email}</li>
                    }).collect_view()
                })}
            </ul>
        </Show>
        
        <button on:click=move |_| (result.refetch)()>
            "Refetch"
        </button>
    }
}
```

---

#### `use_mutation()` hook

```rust
pub struct MutationResult<T, V> {
    pub data: Signal<Option<T>>,
    pub error: Signal<Option<GraphqlHttpError>>,
    pub loading: Signal<bool>,
    pub mutate: Box<dyn Fn(V) + 'static>,
}

/// Hook для выполнения GraphQL mutation
pub fn use_mutation<V, T>(
    endpoint: String,
    mutation: String,
    token: Option<String>,
    tenant: Option<String>,
) -> MutationResult<T, V>
where
    V: Serialize + Clone + 'static,
    T: DeserializeOwned + Clone + 'static,
{
    let (data, set_data) = create_signal(None);
    let (error, set_error) = create_signal(None);
    let (loading, set_loading) = create_signal(false);
    
    let mutate = move |variables: V| {
        set_loading.set(true);
        set_error.set(None);
        
        let endpoint = endpoint.clone();
        let mutation = mutation.clone();
        let token = token.clone();
        let tenant = tenant.clone();
        
        spawn_local(async move {
            let request = GraphqlRequest::new(mutation, Some(variables));
            
            match execute::<V, T>(&endpoint, request, token, tenant).await {
                Ok(response) => {
                    set_data.set(Some(response));
                    set_loading.set(false);
                }
                Err(err) => {
                    set_error.set(Some(err));
                    set_loading.set(false);
                }
            }
        });
    };
    
    MutationResult {
        data,
        error,
        loading,
        mutate: Box::new(mutate),
    }
}
```

**Usage:**

```rust
use leptos_graphql::use_mutation;

#[component]
fn CreateUserForm() -> impl IntoView {
    let mutation = r#"
        mutation CreateUser($input: CreateUserInput!) {
            createUser(input: $input) {
                id email name
            }
        }
    "#;
    
    let result = use_mutation(
        "/api/graphql".to_string(),
        mutation.to_string(),
        Some(token),
        Some(tenant),
    );
    
    let on_submit = move |_| {
        let variables = json!({
            "input": {
                "email": email.get(),
                "name": name.get(),
            }
        });
        
        (result.mutate)(variables);
    };
    
    view! {
        <form on:submit=on_submit>
            <Field name="email" />
            <Field name="name" />
            
            <Button 
                loading=result.loading.get()
                disabled=result.loading.get()
            >
                "Create User"
            </Button>
            
            <Show when=move || result.error.get().is_some()>
                <p class="text-red-500">
                    {move || result.error.get().unwrap().to_string()}
                </p>
            </Show>
        </form>
    }
}
```

---

### 2. Query Builder (Type-safe)

**Проблема:** GraphQL queries как строки — нет compile-time проверки.

**Решение:** Macro для генерации type-safe queries (опционально, Phase 2).

```rust
// Будущее API (Phase 2+)
graphql_query! {
    GetUsers(limit: Int!) {
        users(limit: $limit) {
            nodes {
                id
                email
                name
            }
        }
    }
}

// Generates:
struct GetUsersQuery;
struct GetUsersVariables { limit: i32 }
struct GetUsersResponse { users: UserConnection }
```

---

### 3. GraphQL Client Context

**Проблема:** Передавать endpoint/token/tenant в каждый вызов неудобно.

**Решение:** Global context provider.

```rust
// crates/leptos-graphql/src/context.rs

use leptos::*;

#[derive(Clone)]
pub struct GraphqlClientConfig {
    pub endpoint: String,
    pub token: Signal<Option<String>>,
    pub tenant: Signal<Option<String>>,
}

pub fn provide_graphql_client(
    endpoint: String,
    token: Signal<Option<String>>,
    tenant: Signal<Option<String>>,
) {
    provide_context(GraphqlClientConfig {
        endpoint,
        token,
        tenant,
    });
}

pub fn use_graphql_client() -> GraphqlClientConfig {
    expect_context::<GraphqlClientConfig>()
}
```

**Usage:**

```rust
// In App component
#[component]
fn App() -> impl IntoView {
    let (token, set_token) = create_signal(None);
    let (tenant, set_tenant) = create_signal(Some("demo".to_string()));
    
    provide_graphql_client(
        "/api/graphql".to_string(),
        token,
        tenant,
    );
    
    view! {
        <Router>
            // ... routes
        </Router>
    }
}

// In child components
#[component]
fn UsersList() -> impl IntoView {
    let client = use_graphql_client();
    
    let result = use_query_with_client(
        client,
        USERS_QUERY.to_string(),
        Some(variables),
    );
    
    // ...
}
```

---

### 4. Cache & Optimistic Updates

**Проблема:** Каждый query заново fetches данные.

**Решение:** Простой in-memory cache (Phase 3).

```rust
// crates/leptos-graphql/src/cache.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct QueryCache {
    cache: Arc<RwLock<HashMap<String, CachedQuery>>>,
}

struct CachedQuery {
    data: serde_json::Value,
    timestamp: u64,
}

impl QueryCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let cache = self.cache.read().unwrap();
        cache.get(key).map(|entry| entry.data.clone())
    }
    
    pub fn set(&self, key: String, data: serde_json::Value) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, CachedQuery {
            data,
            timestamp: now(),
        });
    }
    
    pub fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
    }
}
```

---

## 📊 Implementation Plan

### Phase 1 (Current Sprint) — Hooks

**Priority:** P0 (блокирует удобную работу с GraphQL)

- [ ] Создать `crates/leptos-graphql/src/hooks.rs`
- [ ] Реализовать `use_query()` hook
- [ ] Реализовать `use_mutation()` hook
- [ ] Добавить examples в `crates/leptos-graphql/examples/`
- [ ] Обновить README с новыми API
- [ ] Unit tests для hooks

**ETA:** 1-2 дня

---

### Phase 2 — Context Provider

**Priority:** P1

- [ ] Создать `crates/leptos-graphql/src/context.rs`
- [ ] Реализовать `provide_graphql_client()`
- [ ] Реализовать `use_graphql_client()`
- [ ] Обновить hooks для работы с context
- [ ] Examples + tests

**ETA:** 1 день

---

### Phase 3 — Cache (Optional)

**Priority:** P2

- [ ] Создать `crates/leptos-graphql/src/cache.rs`
- [ ] Реализовать in-memory cache
- [ ] Интегрировать cache в hooks
- [ ] Cache invalidation strategies
- [ ] Examples + tests

**ETA:** 2-3 дня

---

### Phase 4 — Type-safe Queries (Optional)

**Priority:** P3

- [ ] Macro для генерации type-safe queries
- [ ] Code generation из GraphQL schema
- [ ] Integration с rust-analyzer (IDE support)

**ETA:** 5-7 дней

---

## 🔗 Integration с leptos-auth

`leptos-auth` уже использует `leptos-graphql` под капотом. После добавления hooks, мы можем упростить API:

```rust
// Old (current)
use leptos_auth::api;

let result = api::sign_in(email, password, tenant).await?;

// New (with hooks)
use leptos_graphql::use_mutation;
use leptos_auth::mutations::SIGN_IN_MUTATION;

let sign_in = use_mutation(SIGN_IN_MUTATION, token, tenant);

let on_submit = move |_| {
    sign_in.mutate(json!({ "email": email, "password": password }));
};
```

---

## 💡 Benefits

1. **Less boilerplate** — не нужно вручную управлять loading/error states
2. **Reactive** — автоматические re-renders при изменении данных
3. **Type-safe** — compile-time проверка типов
4. **Reusable** — hooks можно использовать в любых компонентах
5. **Consistent** — единый API для queries/mutations
6. **Cache** — опционально, для оптимизации производительности

---

## 🚀 Quick Start (Phase 1 Complete)

После реализации Phase 1:

```rust
use leptos_graphql::{use_query, use_mutation};

#[component]
fn UsersPage() -> impl IntoView {
    // Query users
    let users = use_query(
        "/api/graphql".into(),
        USERS_QUERY.into(),
        Some(json!({ "limit": 10 })),
        token,
        tenant,
    );
    
    // Create user mutation
    let create_user = use_mutation(
        "/api/graphql".into(),
        CREATE_USER_MUTATION.into(),
        token,
        tenant,
    );
    
    view! {
        <div>
            <Show when=move || users.loading.get()>
                "Loading..."
            </Show>
            
            <For
                each=move || users.data.get().unwrap_or_default()
                key=|user| user.id.clone()
                children=move |user| view! {
                    <UserCard user=user />
                }
            />
            
            <button on:click=move |_| {
                create_user.mutate(json!({ "input": { "email": "new@user.com" } }));
            }>
                "Add User"
            </button>
        </div>
    }
}
```

---

## 📚 Related Docs

- [CUSTOM_LIBRARIES_STATUS.md](./CUSTOM_LIBRARIES_STATUS.md) — Status of all libraries
- [PHASE_1_IMPLEMENTATION_GUIDE.md](./PHASE_1_IMPLEMENTATION_GUIDE.md) — Phase 1 guide
- [GRAPHQL_ARCHITECTURE.md](./GRAPHQL_ARCHITECTURE.md) — GraphQL architecture overview

---

**Status:** 📝 Plan (Pending Implementation)  
**Next Step:** Implement Phase 1 (hooks)

---

**Last Updated:** 2026-02-14  
**Maintainer:** CTO Agent
