# API Architecture for RusToK Admin Panel

**Дата:** 2026-02-13  
**Критическая информация:** API стратегия для frontends

---

## 🎯 Главное правило

> **Админки используют GraphQL для domain operations + REST API для аутентификации.**

**Разделение:**
- ✅ **Authentication (REST):** `/api/auth/*` — login, register, logout, refresh, etc.
- ✅ **Domain Operations (GraphQL):** `/api/graphql` — users, content, commerce, blog, forum

**Почему именно так:**
- REST для auth — стандартный подход (OAuth, JWT refresh, session management)
- GraphQL для domain — гибкость, типизация, single endpoint

---

## 📋 Текущая ситуация

### ✅ Правильная библиотека: `leptos-graphql`

**Местоположение:** `crates/leptos-graphql/`

**Назначение:**
- Тонкий transport/utils слой поверх `reqwest` + GraphQL
- Формирует стандартный GraphQL request shape (`query`, `variables`, `extensions`)
- Выполняет HTTP-запросы с заголовками авторизации и tenant-scope
- **НЕ** дублирует state-management (это зона `leptos::Resource`/actions)

**API:**
```rust
// crates/leptos-graphql/src/lib.rs

pub const GRAPHQL_ENDPOINT: &str = "/api/graphql";
pub const TENANT_HEADER: &str = "X-Tenant-Slug";
pub const AUTH_HEADER: &str = "Authorization";

pub struct GraphqlRequest<V = Value> {
    pub query: String,
    pub variables: Option<V>,
    pub extensions: Option<Value>,
}

pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphqlError>>,
}

pub async fn execute<V, T>(
    endpoint: &str,
    request: GraphqlRequest<V>,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, GraphqlHttpError>
where
    V: Serialize,
    T: DeserializeOwned;
```

**Пример использования:**
```rust
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};

async fn fetch_user(user_id: String, token: String, tenant: String) -> Result<User, GraphqlHttpError> {
    let query = r#"
        query GetUser($id: ID!) {
            user(id: $id) {
                id
                email
                name
            }
        }
    "#;
    
    let variables = serde_json::json!({ "id": user_id });
    let request = GraphqlRequest::new(query, Some(variables));
    
    execute(GRAPHQL_ENDPOINT, request, Some(token), Some(tenant)).await
}
```

---

### ✅ Правильная библиотека: `leptos-auth`

**Назначение:** Authentication через REST API

**Файл:** `crates/leptos-auth/src/api.rs`

**Текущая реализация (ПРАВИЛЬНО):**
```rust
// ✅ ПРАВИЛЬНО - использует REST API для auth
const API_BASE: &str = "/api/auth";

pub async fn sign_in(
    email: String,
    password: String,
    tenant: String,
) -> Result<(AuthUser, AuthSession), AuthError> {
    // REST endpoint для аутентификации
    fetch_json(&format!("{}/login", API_BASE), "POST", ...).await
}
```

**Endpoints (REST):**
- ✅ `POST /api/auth/login` — Вход в систему
- ✅ `POST /api/auth/register` — Регистрация
- ✅ `POST /api/auth/logout` — Выход
- ✅ `GET /api/auth/me` — Получить текущего пользователя
- ✅ `POST /api/auth/forgot-password` — Восстановление пароля
- ✅ `POST /api/auth/reset-password` — Сброс пароля
- ✅ `POST /api/auth/refresh` — Обновление токена

**Backend:** `apps/server/src/controllers/auth.rs`

---

## 🔧 API Endpoints Summary

### Authentication (REST API)

**Используется:** `leptos-auth` библиотека

| Endpoint | Method | Назначение | Body |
|----------|--------|------------|------|
| `/api/auth/login` | POST | Вход в систему | `{email, password}` |
| `/api/auth/register` | POST | Регистрация | `{email, password, name?}` |
| `/api/auth/logout` | POST | Выход | — |
| `/api/auth/me` | GET | Текущий пользователь | — |
| `/api/auth/refresh` | POST | Обновление токена | — |
| `/api/auth/forgot-password` | POST | Восстановление пароля | `{email}` |
| `/api/auth/reset-password` | POST | Сброс пароля | `{token, password}` |

**Headers:**
- `Authorization: Bearer <token>` — для защищённых endpoints
- `X-Tenant-Slug: <tenant>` — для мультитенантности

### Domain Operations (GraphQL)

**Используется:** `leptos-graphql` библиотека

**Endpoint:** `/api/graphql` (POST)

**Примеры mutations/queries:**

#### User Management

```graphql
# Создать пользователя (admin only)
mutation CreateUser($input: CreateUserInput!) {
  createUser(input: $input) {
    id
    email
    name
    role
    status
  }
}

# Обновить пользователя
mutation UpdateUser($id: UUID!, $input: UpdateUserInput!) {
  updateUser(id: $id, input: $input) {
    id
    email
    name
    role
    status
  }
}

# Отключить пользователя
mutation DisableUser($id: UUID!) {
  disableUser(id: $id) {
    id
    status
  }
}

# Получить список пользователей
query GetUsers($limit: Int, $offset: Int) {
  users(limit: $limit, offset: $offset) {
    items {
      id
      email
      name
      role
      status
      createdAt
    }
    total
  }
}

# Получить пользователя по ID
query GetUser($id: UUID!) {
  user(id: $id) {
    id
    email
    name
    role
    status
    createdAt
    updatedAt
  }
}
```

#### Content Management

```graphql
# Создать страницу
mutation CreatePage($input: CreatePageInput!) {
  createPage(input: $input) {
    id
    title
    slug
    content
  }
}

# Получить страницы
query GetPages {
  pages {
    id
    title
    slug
    status
  }
}
```

#### Commerce

```graphql
# Создать продукт
mutation CreateProduct($input: CreateProductInput!) {
  createProduct(input: $input) {
    id
    name
    sku
    price
  }
}

# Получить продукты
query GetProducts {
  products {
    id
    name
    sku
    price
    stock
  }
}
```

#### Blog

```graphql
# Создать пост
mutation CreatePost($input: CreatePostInput!) {
  createPost(input: $input) {
    id
    title
    slug
    content
  }
}

# Получить посты
query GetPosts {
  posts {
    id
    title
    slug
    publishedAt
  }
}
```

**Headers:**
- `Authorization: Bearer <token>` — обязательно для всех запросов
- `X-Tenant-Slug: <tenant>` — обязательно для мультитенантности

---

## 🏗️ Архитектура слоёв

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (apps/admin)                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────────────┐         ┌────────────────────┐     │
│  │   leptos-auth      │         │  leptos-graphql    │     │
│  │  (REST for auth)   │         │  (GraphQL for      │     │
│  │                    │         │   domain ops)      │     │
│  └─────────┬──────────┘         └─────────┬──────────┘     │
│            │                              │                  │
└────────────┼──────────────────────────────┼─────────────────┘
             │                              │
             ▼                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend (apps/server)                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌────────────────────┐         ┌────────────────────┐     │
│  │  REST Controllers  │         │  GraphQL Resolvers │     │
│  │  /api/auth/*       │         │  /api/graphql      │     │
│  │                    │         │                    │     │
│  │  - login           │         │  - users           │     │
│  │  - register        │         │  - content         │     │
│  │  - logout          │         │  - commerce        │     │
│  │  - refresh         │         │  - blog            │     │
│  └────────────────────┘         │  - forum           │     │
│                                  └────────────────────┘     │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            Database (PostgreSQL)                     │   │
│  │  - users, sessions, tenants                          │   │
│  │  - nodes, pages, products, posts                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Разделение ответственности:**

1. **Authentication (REST)**
   - Login/Register/Logout
   - Token management (access + refresh)
   - Password reset
   - Session management
   - **Причина:** Стандартные паттерны OAuth/JWT, session cookies

2. **Domain Operations (GraphQL)**
   - CRUD для всех domain entities (users, pages, products, posts)
   - Сложные запросы с фильтрацией/сортировкой
   - Батчинг запросов
   - **Причина:** Гибкость, типизация, single endpoint

---

## 📦 Зависимости

### `leptos-auth` — REST API для аутентификации

**Текущий `Cargo.toml` (ПРАВИЛЬНЫЙ):**
```toml
[dependencies]
leptos = { workspace = true }
leptos_router = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
gloo-storage = { workspace = true }
thiserror = { workspace = true }
reqwest = { version = "0.13", default-features = false, features = ["json"] }  # ✅ Нужен для REST
```

**Не нужно менять!** `leptos-auth` использует `reqwest` напрямую для REST endpoints.

### `leptos-graphql` — GraphQL для domain operations

**`Cargo.toml`:**
```toml
[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
reqwest = { version = "0.13", default-features = false, features = ["json"] }
thiserror = { workspace = true }
```

**Использование в приложениях:**

```toml
# apps/admin/Cargo.toml
[dependencies]
leptos-auth = { workspace = true }     # ✅ для аутентификации
leptos-graphql = { workspace = true }  # ✅ для domain operations
```

---

## 🎯 Почему GraphQL, а не REST?

### 1. **Единая точка входа**
- ✅ Один endpoint: `/api/graphql`
- ❌ Много endpoints: `/api/auth/login`, `/api/auth/register`, ...

### 2. **Типобезопасность**
- ✅ GraphQL schema — single source of truth
- ✅ Можно генерировать типы с помощью `graphql-client`
- ❌ REST — типы нужно дублировать

### 3. **Flexible queries**
- ✅ Клиент запрашивает только нужные поля
- ❌ REST — фиксированные ответы

### 4. **Версионирование**
- ✅ GraphQL — эволюция schema без версий
- ❌ REST — `/api/v1/`, `/api/v2/`, ...

### 5. **Батчинг**
- ✅ Можно отправить несколько query/mutation в одном запросе
- ❌ REST — нужны дополнительные HTTP запросы

### 6. **Introspection**
- ✅ GraphQL schema можно запросить через introspection
- ✅ Автогенерация документации (GraphiQL, Playground)
- ❌ REST — нужен Swagger/OpenAPI

---

## 🚀 Использование API

### Authentication Flow (REST)

```rust
use leptos_auth::api;

// 1. Login
let (user, session) = api::sign_in(
    "admin@local".to_string(),
    "password123".to_string(),
    "demo".to_string(),  // tenant slug
).await?;

// session.token — JWT token для последующих запросов
// session.tenant — tenant slug

// 2. Использовать токен для GraphQL запросов
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};

let query = r#"
query GetUsers {
    users {
        id
        email
        name
    }
}
"#;

let request = GraphqlRequest::new(query, None);

let response = execute(
    GRAPHQL_ENDPOINT,
    request,
    Some(session.token.clone()),  // ✅ Токен из session
    Some(session.tenant.clone()), // ✅ Tenant из session
).await?;
```

---

### Domain Operations Flow (GraphQL)

**Пример 1: Получить список пользователей**

```rust
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct UsersData {
    users: UsersResponse,
}

#[derive(Deserialize)]
struct UsersResponse {
    items: Vec<User>,
    total: i32,
}

#[derive(Deserialize)]
struct User {
    id: String,
    email: String,
    name: Option<String>,
}

async fn fetch_users(token: String, tenant: String) -> Result<Vec<User>, GraphqlHttpError> {
    let query = r#"
        query GetUsers($limit: Int, $offset: Int) {
            users(limit: $limit, offset: $offset) {
                items {
                    id
                    email
                    name
                }
                total
            }
        }
    "#;
    
    let variables = serde_json::json!({
        "limit": 10,
        "offset": 0,
    });
    
    let request = GraphqlRequest::new(query, Some(variables));
    
    let response: UsersData = execute(
        GRAPHQL_ENDPOINT,
        request,
        Some(token),
        Some(tenant),
    ).await?;
    
    Ok(response.users.items)
}
```

**Пример 2: Создать пользователя**

```rust
async fn create_user(
    email: String,
    password: String,
    name: Option<String>,
    token: String,
    tenant: String,
) -> Result<User, GraphqlHttpError> {
    let query = r#"
        mutation CreateUser($input: CreateUserInput!) {
            createUser(input: $input) {
                id
                email
                name
                role
                status
            }
        }
    "#;
    
    let variables = serde_json::json!({
        "input": {
            "email": email,
            "password": password,
            "name": name,
        }
    });
    
    let request = GraphqlRequest::new(query, Some(variables));
    
    #[derive(Deserialize)]
    struct CreateUserData {
        #[serde(rename = "createUser")]
        create_user: User,
    }
    
    let response: CreateUserData = execute(
        GRAPHQL_ENDPOINT,
        request,
        Some(token),
        Some(tenant),
    ).await?;
    
    Ok(response.create_user)
}
```

---

### Интеграция с Leptos

**Пример: Использование в Leptos Resource**

```rust
use leptos::*;
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};
use leptos_auth::{use_token, use_tenant};

#[component]
pub fn UserList() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();
    
    // Resource для загрузки пользователей
    let users = create_resource(
        move || (token.get(), tenant.get()),
        |(token, tenant)| async move {
            if token.is_none() || tenant.is_none() {
                return Err("Not authenticated".to_string());
            }
            
            let query = r#"
                query GetUsers {
                    users {
                        items { id email name }
                    }
                }
            "#;
            
            let request = GraphqlRequest::new(query, None);
            
            execute(GRAPHQL_ENDPOINT, request, token, tenant)
                .await
                .map_err(|e| e.to_string())
        },
    );
    
    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || users.get().map(|result| match result {
                Ok(data) => view! { <ul>{/* render users */}</ul> },
                Err(e) => view! { <p class="error">{e}</p> },
            })}
        </Suspense>
    }
}
```

---

### Типизация с `graphql-client` (Optional)

Для compile-time типизации можно использовать `graphql-client`:

**1. Добавить зависимость:**

```toml
# apps/admin/Cargo.toml
[dependencies]
graphql_client = "0.14"
```

**2. Создать `.graphql` файлы:**

```graphql
# apps/admin/graphql/users.graphql

query GetUsers($limit: Int, $offset: Int) {
    users(limit: $limit, offset: $offset) {
        items {
            id
            email
            name
            role
            status
        }
        total
    }
}
```

**3. Сгенерировать типы:**

```rust
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/users.graphql",
)]
pub struct GetUsers;

// Использование
let variables = get_users::Variables {
    limit: Some(10),
    offset: Some(0),
};

let request_body = GetUsers::build_query(variables);

// Отправить через leptos-graphql
let request = GraphqlRequest::new(
    request_body.query,
    Some(request_body.variables),
);
```

---

## 📖 Best Practices

### 1. Используйте правильную библиотеку для каждой задачи

```rust
// ✅ ПРАВИЛЬНО - Authentication через REST
use leptos_auth::api;

let (user, session) = api::sign_in(email, password, tenant).await?;

// ✅ ПРАВИЛЬНО - Domain operations через GraphQL
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};

let response = execute(GRAPHQL_ENDPOINT, request, token, tenant).await?;
```

```rust
// ❌ НЕПРАВИЛЬНО - Прямое использование reqwest
use reqwest::Client;

let client = Client::new();
let response = client.post("/api/graphql").json(&query).send().await?;
```

### 2. Используйте константы для GraphQL queries

```rust
// ✅ ПРАВИЛЬНО
const GET_USERS_QUERY: &str = r#"
query GetUsers($limit: Int, $offset: Int) {
    users(limit: $limit, offset: $offset) {
        items { id email name }
        total
    }
}
"#;

let request = GraphqlRequest::new(GET_USERS_QUERY, Some(variables));
```

```rust
// ❌ НЕПРАВИЛЬНО - строковая интерполяция
let query = format!("query {{ users {{ id email }} }}");
```

### 3. Обрабатывайте ошибки правильно

```rust
// ✅ ПРАВИЛЬНО - GraphQL errors
let response = execute(endpoint, request, token, tenant)
    .await
    .map_err(|e| match e {
        GraphqlHttpError::Unauthorized => "Unauthorized",
        GraphqlHttpError::Graphql(msg) => &msg,
        _ => "Network error",
    })?;

// ✅ ПРАВИЛЬНО - Auth errors
let (user, session) = api::sign_in(email, password, tenant)
    .await
    .map_err(|e| match e {
        AuthError::InvalidCredentials => "Invalid email or password",
        AuthError::Unauthorized => "Unauthorized",
        _ => "Network error",
    })?;
```

### 4. Всегда добавляйте headers

```rust
// ✅ ПРАВИЛЬНО - с token и tenant
execute(GRAPHQL_ENDPOINT, request, Some(token), Some(tenant)).await

// ✅ ПРАВИЛЬНО - только tenant (для public queries)
execute(GRAPHQL_ENDPOINT, request, None, Some(tenant)).await
```

```rust
// ❌ НЕПРАВИЛЬНО - без tenant
execute(GRAPHQL_ENDPOINT, request, Some(token), None).await
```

### 5. Типизируйте ответы

```rust
// ✅ ПРАВИЛЬНО
#[derive(Deserialize)]
struct UsersData {
    users: UsersResponse,
}

#[derive(Deserialize)]
struct UsersResponse {
    items: Vec<User>,
    total: i32,
}

let response: UsersData = execute(endpoint, request, token, tenant).await?;
```

```rust
// ❌ НЕПРАВИЛЬНО - serde_json::Value
let response: serde_json::Value = execute(endpoint, request, token, tenant).await?;
let users = response["users"]["items"].as_array().unwrap();  // может паниковать!
```

---

## 🔍 Checklist перед использованием API

### Для Authentication (REST)

- [ ] **Используется `leptos-auth::api`?** (а не прямой `reqwest`)
- [ ] **Endpoint правильный?** (`/api/auth/login`, `/api/auth/register`, etc.)
- [ ] **Добавлен `X-Tenant-Slug` header?** (в параметре `tenant`)
- [ ] **Обработаны auth errors?** (`InvalidCredentials`, `Unauthorized`)
- [ ] **Токен сохранён в storage?** (для последующих GraphQL запросов)

### Для Domain Operations (GraphQL)

- [ ] **Используется `leptos-graphql::execute`?** (а не `reqwest` напрямую)
- [ ] **Endpoint = `/api/graphql`?**
- [ ] **Query/Mutation написан правильно?** (синтаксис GraphQL)
- [ ] **Добавлен `Authorization: Bearer <token>` header?** (из `leptos-auth` session)
- [ ] **Добавлен `X-Tenant-Slug` header?** (из `leptos-auth` session)
- [ ] **Обработаны GraphQL errors?** (`Unauthorized`, `Graphql`)
- [ ] **Типы ответов соответствуют schema?** (используйте struct с `Deserialize`)

---

## 📚 Дополнительные ресурсы

### Документация

- **`leptos-graphql`:** `/crates/leptos-graphql/README.md`
- **GraphQL spec:** https://graphql.org/learn/
- **`graphql-client`:** https://github.com/graphql-rust/graphql-client

### Примеры

- **Backend GraphQL schema:** `apps/server/src/graphql/schema.rs`
- **Использование в admin:** `apps/admin/src/` (после миграции)

---

## ✅ Summary

| Компонент | Назначение | Статус | Использование |
|-----------|------------|--------|---------------|
| `leptos-auth` | REST API для authentication | ✅ Готов | Login, Register, Logout, Refresh |
| `leptos-graphql` | GraphQL для domain operations | ✅ Готов | Users, Content, Commerce, Blog, Forum |
| `apps/server/controllers/auth.rs` | REST endpoints | ✅ Есть | `/api/auth/*` |
| `apps/server/graphql/*` | GraphQL resolvers | ✅ Есть | `/api/graphql` |
| `apps/admin` | Frontend | ✅ Готов | Использует обе библиотеки |

**Архитектура:**
- ✅ **Authentication:** REST API (`/api/auth/*`) через `leptos-auth`
- ✅ **Domain Operations:** GraphQL (`/api/graphql`) через `leptos-graphql`

**Workflow:**
1. Login через `leptos-auth::api::sign_in()` → получить `token` + `tenant`
2. Использовать `token` + `tenant` для GraphQL запросов через `leptos-graphql::execute()`

---

**Статус:** ✅ Архитектура задокументирована и работает корректно  
**Критичность:** 📘 ИНФОРМАЦИОННАЯ (всё правильно реализовано)  
**Блокирует:** Ничего (архитектура верная)  
