# leptos-graphql

GraphQL transport layer для Leptos приложений с reactive hooks.

## Features

- ✅ **HTTP Transport** — POST requests к GraphQL endpoint
- ✅ **Reactive Hooks** — `use_query()`, `use_mutation()`, `use_lazy_query()`
- ✅ **Type-safe** — Generic types для requests/responses
- ✅ **Error Handling** — Network, GraphQL, HTTP, Unauthorized errors
- ✅ **Headers** — Автоматическая вставка Authorization, X-Tenant-Slug
- ✅ **Persisted Queries** — Support для persisted query hashes

## Installation

```toml
[dependencies]
leptos-graphql = { path = "../../crates/leptos-graphql" }
```

## Usage

### Basic Execute (Low-level API)

```rust
use leptos_graphql::{execute, GraphqlRequest};
use serde_json::json;

let request = GraphqlRequest::new(
    "query { users { id email } }",
    Some(json!({}))
);

let response: UsersResponse = execute(
    "/api/graphql",
    request,
    Some("token"),
    Some("tenant")
).await?;
```

### use_query() Hook

Reactive hook для GraphQL queries с автоматическим loading/error management.

```rust
use leptos::*;
use leptos_graphql::use_query;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Clone)]
struct UsersResponse {
    users: Vec<User>,
}

#[derive(Deserialize, Clone)]
struct User {
    id: String,
    email: String,
    name: Option<String>,
}

#[component]
fn UsersList() -> impl IntoView {
    let query = r#"
        query GetUsers($limit: Int!) {
            users(limit: $limit) {
                id
                email
                name
            }
        }
    "#;
    
    let variables = json!({ "limit": 10 });
    
    let result = use_query(
        "/api/graphql".to_string(),
        query.to_string(),
        Some(variables),
        Some(token.get()),
        Some(tenant.get()),
    );
    
    view! {
        <div>
            <Show when=move || result.loading.get()>
                <p>"Loading users..."</p>
            </Show>
            
            <Show when=move || result.error.get().is_some()>
                <p class="text-red-500">
                    {move || result.error.get().map(|e| e.to_string())}
                </p>
            </Show>
            
            <Show when=move || result.data.get().is_some()>
                <ul>
                    {move || result.data.get().map(|data| {
                        data.users.iter().map(|user| view! {
                            <li>{&user.email}</li>
                        }).collect_view()
                    })}
                </ul>
            </Show>
            
            <button on:click=move |_| result.refetch()>
                "Refetch"
            </button>
        </div>
    }
}
```

### use_mutation() Hook

Hook для GraphQL mutations.

```rust
use leptos::*;
use leptos_graphql::use_mutation;
use serde_json::json;

#[component]
fn CreateUserForm() -> impl IntoView {
    let mutation = r#"
        mutation CreateUser($input: CreateUserInput!) {
            createUser(input: $input) {
                id
                email
                name
            }
        }
    "#;
    
    let create_user = use_mutation(
        "/api/graphql".to_string(),
        mutation.to_string(),
        Some(token.get()),
        Some(tenant.get()),
    );
    
    let (email, set_email) = signal(String::new());
    let (name, set_name) = signal(String::new());
    
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        create_user.mutate(json!({
            "input": {
                "email": email.get(),
                "name": name.get(),
            }
        }));
    };
    
    view! {
        <form on:submit=on_submit>
            <input
                type="email"
                placeholder="Email"
                on:input=move |ev| set_email(event_target_value(&ev))
            />
            <input
                type="text"
                placeholder="Name"
                on:input=move |ev| set_name(event_target_value(&ev))
            />
            
            <button
                type="submit"
                disabled=create_user.loading.get()
            >
                {move || if create_user.loading.get() {
                    "Creating..."
                } else {
                    "Create User"
                }}
            </button>
            
            <Show when=move || create_user.error.get().is_some()>
                <p class="text-red-500">
                    {move || create_user.error.get().map(|e| e.to_string())}
                </p>
            </Show>
            
            <Show when=move || create_user.data.get().is_some()>
                <p class="text-green-500">
                    "User created successfully!"
                </p>
            </Show>
        </form>
    }
}
```

### use_lazy_query() Hook

Lazy query hook — query не выполняется автоматически при mount.

```rust
use leptos::*;
use leptos_graphql::use_lazy_query;

#[component]
fn SearchUsers() -> impl IntoView {
    let query = r#"
        query SearchUsers($search: String!) {
            users(search: $search) {
                id email name
            }
        }
    "#;
    
    let (result, fetch) = use_lazy_query(
        "/api/graphql".to_string(),
        query.to_string(),
        Some(token.get()),
        Some(tenant.get()),
    );
    
    let (search, set_search) = signal(String::new());
    
    let on_search = move |_| {
        fetch(Some(json!({ "search": search.get() })));
    };
    
    view! {
        <div>
            <input
                type="text"
                placeholder="Search users..."
                on:input=move |ev| set_search(event_target_value(&ev))
            />
            <button on:click=on_search>"Search"</button>
            
            <Show when=move || result.loading.get()>
                "Searching..."
            </Show>
            
            <Show when=move || result.data.get().is_some()>
                {move || result.data.get().map(|data| view! {
                    <ul>
                        {data.users.iter().map(|user| view! {
                            <li>{&user.email}</li>
                        }).collect_view()}
                    </ul>
                })}
            </Show>
        </div>
    }
}
```

## API Reference

### Hooks

#### `use_query<V, T>(endpoint, query, variables, token, tenant) -> QueryResult<T>`

Execute GraphQL query и return reactive result.

**Returns:**
- `QueryResult<T>`:
  - `data: ReadSignal<Option<T>>` — Query response data
  - `error: ReadSignal<Option<GraphqlHttpError>>` — Error if any
  - `loading: ReadSignal<bool>` — Loading state
  - `refetch()` — Refetch query

#### `use_mutation<T>(endpoint, mutation, token, tenant) -> MutationResult<T>`

Create mutation function.

**Returns:**
- `MutationResult<T>`:
  - `data: ReadSignal<Option<T>>` — Mutation response data
  - `error: ReadSignal<Option<GraphqlHttpError>>` — Error if any
  - `loading: ReadSignal<bool>` — Loading state
  - `mutate(variables: Value)` — Execute mutation

#### `use_lazy_query<V, T>(endpoint, query, token, tenant) -> (QueryResult<T>, Fetch)`

Create lazy query (не выполняется автоматически).

**Returns:**
- `(QueryResult<T>, fetch_fn)`:
  - `QueryResult<T>` — Query result (same as use_query)
  - `fetch_fn(variables: Option<V>)` — Function to execute query

### Low-level API

#### `execute<V, T>(endpoint, request, token, tenant) -> Result<T, GraphqlHttpError>`

Low-level function для выполнения GraphQL requests.

### Types

#### `GraphqlRequest<V>`

```rust
pub struct GraphqlRequest<V = Value> {
    pub query: String,
    pub variables: Option<V>,
    pub extensions: Option<Value>,
}
```

#### `GraphqlResponse<T>`

```rust
pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphqlError>>,
}
```

#### `GraphqlHttpError`

```rust
pub enum GraphqlHttpError {
    Network,
    Graphql(String),
    Http(String),
    Unauthorized,
}
```

## Error Handling

```rust
use leptos_graphql::GraphqlHttpError;

match result.error.get() {
    Some(GraphqlHttpError::Network) => {
        // Network error (e.g., no internet)
    }
    Some(GraphqlHttpError::Unauthorized) => {
        // 401 Unauthorized (redirect to login)
    }
    Some(GraphqlHttpError::Graphql(msg)) => {
        // GraphQL error (e.g., validation error)
    }
    Some(GraphqlHttpError::Http(status)) => {
        // HTTP error (e.g., 500 Internal Server Error)
    }
    None => {
        // No error
    }
}
```

## Persisted Queries

```rust
use leptos_graphql::{persisted_query_extension, GraphqlRequest};
use serde_json::json;

let query_hash = "abc123..."; // SHA-256 hash of query

let request = GraphqlRequest::new(
    "", // Empty query for persisted queries
    Some(variables),
).with_extensions(persisted_query_extension(query_hash));

let response = execute(endpoint, request, token, tenant).await?;
```

## Integration с leptos-auth

`leptos-auth` использует `leptos-graphql` под капотом для auth operations:

```rust
use leptos_auth::api;

// Sign in через GraphQL
let (user, session) = api::sign_in(email, password, tenant).await?;

// Fetch current user через GraphQL
let user = api::fetch_current_user(token, tenant).await?;
```

## Compatibility

- ✅ CSR (Client-Side Rendering)
- ✅ SSR (Server-Side Rendering)
- Leptos: 0.6+

## Roadmap

### Phase 1 (Complete) ✅
- [x] Basic HTTP transport
- [x] Reactive hooks (`use_query`, `use_mutation`, `use_lazy_query`)
- [x] Error handling
- [x] Persisted queries support

### Phase 2 (Planned)
- [ ] GraphQL client context provider
- [ ] Query caching (in-memory)
- [ ] Optimistic updates
- [ ] Polling support

### Phase 3 (Future)
- [ ] Type-safe query builder (macro)
- [ ] Code generation from GraphQL schema
- [ ] Cache persistence (localStorage)

## License

MIT OR Apache-2.0
