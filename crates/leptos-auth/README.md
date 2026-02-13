# leptos-auth

## Назначение

`crates/leptos-auth` — Leptos authentication library для RusToK, использующая **REST API** для auth operations.

## Архитектура

**Главное правило:** ✅ **Auth через REST API (`/api/auth/*`), Data через GraphQL (`/api/graphql`)**

Эта библиотека предоставляет:
- Компоненты для защищённых маршрутов (`ProtectedRoute`, `GuestRoute`)
- Hooks для работы с аутентификацией (`use_auth`, `use_token`, `use_tenant`)
- REST API client для auth operations (`sign_in`, `sign_up`, `sign_out`)
- LocalStorage helpers для сохранения сессии

## Взаимодействие

- `apps/admin` — использует для аутентификации
- `apps/storefront` — использует для аутентификации
- `apps/server` — REST endpoints (`/api/auth/*`) на backend
- `crates/leptos-graphql` — используется ТОЛЬКО для `me` query (fetch current user)

### Почему REST для auth?

**Best practice:** Auth operations (login, register, logout) через REST, а data queries через GraphQL.

**Причины:**
1. ✅ Industry standard (OAuth, JWT обычно через REST)
2. ✅ Проще отладка (curl, Postman)
3. ✅ Backend auth уже реализован через REST
4. ✅ Меньше дублирования кода

## Структура

```
src/
├── lib.rs          ← Public API, типы (AuthUser, AuthSession, AuthError)
├── api.rs          ← REST API client (sign_in, sign_up, sign_out via fetch())
├── context.rs      ← AuthProvider component, AuthContext
├── hooks.rs        ← use_auth(), use_token(), use_tenant(), etc.
├── storage.rs      ← LocalStorage helpers
└── components.rs   ← ProtectedRoute, GuestRoute, RequireAuth
```

## Использование

### 1. Обернуть приложение в AuthProvider

```rust
// apps/admin/src/app.rs
use leptos_auth::AuthProvider;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                {/* routes */}
            </Router>
        </AuthProvider>
    }
}
```

### 2. Login page

```rust
use leptos::*;
use leptos_auth::{api, use_auth};

#[component]
pub fn Login() -> impl IntoView {
    let auth = use_auth();
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    
    let login_action = create_action(|(email, password, tenant): &(String, String, String)| {
        let email = email.clone();
        let password = password.clone();
        let tenant = tenant.clone();
        
        async move {
            match auth.sign_in(email, password, tenant).await {
                Ok(_) => {
                    // Success - AuthContext updated automatically
                    use leptos_router::use_navigate;
                    let navigate = use_navigate();
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(format!("Login failed: {:?}", e)));
                }
            }
        }
    });
    
    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            login_action.dispatch((
                email.get(),
                password.get(),
                "demo".to_string(), // tenant slug
            ));
        }>
            <input
                type="email"
                placeholder="Email"
                prop:value=email
                on:input=move |ev| set_email.set(event_target_value(&ev))
            />
            <input
                type="password"
                placeholder="Password"
                prop:value=password
                on:input=move |ev| set_password.set(event_target_value(&ev))
            />
            <button type="submit" disabled=move || login_action.pending().get()>
                {move || if login_action.pending().get() { "Logging in..." } else { "Login" }}
            </button>
            
            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}
        </form>
    }
}
```

### 3. Protected routes

```rust
use leptos::*;
use leptos_router::*;
use leptos_auth::ProtectedRoute;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <Router>
                <Routes>
                    <Route path="/login" view=Login />
                    <Route path="/register" view=Register />
                    
                    {/* Protected routes */}
                    <ParentRoute path="" view=ProtectedRoute>
                        <Route path="/dashboard" view=Dashboard />
                        <Route path="/profile" view=Profile />
                    </ParentRoute>
                </Routes>
            </Router>
        </AuthProvider>
    }
}
```

### 4. Use auth hooks

```rust
use leptos::*;
use leptos_auth::{use_auth, use_current_user, use_is_authenticated};

#[component]
pub fn Dashboard() -> impl IntoView {
    let is_authenticated = use_is_authenticated();
    let current_user = use_current_user();
    
    view! {
        <div>
            <h1>"Dashboard"</h1>
            {move || {
                if is_authenticated.get() {
                    if let Some(user) = current_user.get() {
                        view! { <p>"Welcome, " {user.email} "!"</p> }.into_view()
                    } else {
                        view! { <p>"Loading user..."</p> }.into_view()
                    }
                } else {
                    view! { <p>"Not authenticated"</p> }.into_view()
                }
            }}
        </div>
    }
}
```

### 5. Logout

```rust
use leptos::*;
use leptos_auth::use_auth;

#[component]
pub fn LogoutButton() -> impl IntoView {
    let auth = use_auth();
    
    let logout_action = create_action(|_| async move {
        match auth.sign_out().await {
            Ok(_) => {
                use leptos_router::use_navigate;
                let navigate = use_navigate();
                navigate("/login", Default::default());
            }
            Err(e) => {
                log::error!("Logout failed: {:?}", e);
            }
        }
    });
    
    view! {
        <button on:click=move |_| logout_action.dispatch(())>
            "Logout"
        </button>
    }
}
```

## API Reference

### `api` module

#### `sign_in(email, password, tenant) -> Result<(AuthUser, AuthSession), AuthError>`

Login with email and password.

**Endpoint:** `POST /api/auth/login`

**Example:**
```rust
use leptos_auth::api;

let (user, session) = api::sign_in(
    "admin@local".to_string(),
    "admin12345".to_string(),
    "demo".to_string(),
).await?;
```

---

#### `sign_up(email, password, name, tenant) -> Result<(AuthUser, AuthSession), AuthError>`

Register new user.

**Endpoint:** `POST /api/auth/register`

**Example:**
```rust
use leptos_auth::api;

let (user, session) = api::sign_up(
    "user@example.com".to_string(),
    "password123".to_string(),
    Some("John Doe".to_string()),
    "demo".to_string(),
).await?;
```

---

#### `sign_out(token, tenant) -> Result<(), AuthError>`

Logout (invalidate session).

**Endpoint:** `POST /api/auth/logout`

**Example:**
```rust
use leptos_auth::api;

api::sign_out(
    session.token.clone(),
    "demo".to_string(),
).await?;
```

---

#### `refresh_token(refresh_token, tenant) -> Result<AuthSession, AuthError>`

Refresh access token.

**Endpoint:** `POST /api/auth/refresh`

**Example:**
```rust
use leptos_auth::api;

let new_session = api::refresh_token(
    old_refresh_token,
    "demo".to_string(),
).await?;
```

---

#### `fetch_current_user(token, tenant) -> Result<Option<AuthUser>, AuthError>`

Fetch current user (uses GraphQL `me` query).

**Endpoint:** `POST /api/graphql` (query `me`)

**Example:**
```rust
use leptos_auth::api;

let user = api::fetch_current_user(
    session.token.clone(),
    "demo".to_string(),
).await?;
```

---

### Hooks

#### `use_auth() -> AuthContext`

Get auth context (includes all methods and signals).

**Example:**
```rust
let auth = use_auth();
auth.sign_in(email, password, tenant).await?;
auth.sign_out().await?;
```

---

#### `use_current_user() -> Signal<Option<AuthUser>>`

Get current user signal.

---

#### `use_is_authenticated() -> Signal<bool>`

Check if user is authenticated.

---

#### `use_is_loading() -> Signal<bool>`

Check if auth is loading (initial check).

---

#### `use_token() -> Signal<Option<String>>`

Get current access token.

---

#### `use_tenant() -> Signal<Option<String>>`

Get current tenant slug.

---

### Components

#### `<ProtectedRoute />`

Wraps routes that require authentication. Redirects to `/login` if not authenticated.

**Props:**
- `children: Children` — child routes/components
- `redirect_path: Option<String>` — redirect path if not authenticated (default: `/login`)

---

#### `<GuestRoute />`

Wraps routes for guests only (e.g., login, register). Redirects to `/dashboard` if authenticated.

**Props:**
- `children: Children` — child routes/components
- `redirect_path: Option<String>` — redirect path if authenticated (default: `/dashboard`)

---

#### `<RequireAuth />`

Conditionally render content if authenticated.

**Props:**
- `children: Children` — content to show if authenticated
- `fallback: Option<View>` — fallback content if not authenticated

**Example:**
```rust
<RequireAuth fallback=move || view! { <p>"Please sign in"</p> }>
    <p>"Secret content"</p>
</RequireAuth>
```

---

## Types

### `AuthUser`

```rust
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}
```

### `AuthSession`

```rust
pub struct AuthSession {
    pub token: String,
    pub tenant: String,
}
```

### `AuthError`

```rust
pub enum AuthError {
    Unauthorized,
    InvalidCredentials,
    Network,
    Http(u16),
}
```

---

## Environment Variables

### WASM (Browser)

**`window.location.origin`** — используется как API base URL (auto-detected)

### SSR (Server)

**`RUSTOK_API_URL`** — API base URL (default: `http://localhost:5150`)

**Example:**
```bash
RUSTOK_API_URL=http://localhost:5150
```

---

## Testing

Run tests:
```bash
cargo test -p leptos-auth
```

---

## Dependencies

- `leptos` — reactive framework
- `leptos_router` — routing
- `leptos-graphql` — GraphQL transport (for `me` query only)
- `serde`, `serde_json` — serialization
- `gloo-storage` — LocalStorage wrapper
- `web-sys` — WASM fetch API
- `wasm-bindgen`, `wasm-bindgen-futures` — WASM bindings

---

## Implementation Notes

### Why REST + GraphQL?

**REST API** (`/api/auth/*`):
- ✅ Login, Register, Logout, Refresh token
- ✅ JWT auth flow
- ✅ Industry standard

**GraphQL API** (`/api/graphql`):
- ✅ Data queries (`me`, `users`, `posts`, etc.)
- ✅ Mutations (CRUD operations)
- ✅ Efficient data fetching

**This is a common pattern:**
- Auth operations → REST (OAuth, JWT standards)
- Data operations → GraphQL (flexibility, efficiency)

---

## Migration from GraphQL Auth

**Old approach (deprecated):**
```rust
// ❌ GraphQL mutations (not implemented on server)
signIn(email, password) -> SignInPayload
signUp(email, password, name) -> SignUpPayload
signOut -> Boolean
```

**New approach (current):**
```rust
// ✅ REST endpoints (working)
POST /api/auth/login
POST /api/auth/register
POST /api/auth/logout
POST /api/auth/refresh
```

**If you see GraphQL mutation errors**, update to latest `leptos-auth` which uses REST API.

---

## Troubleshooting

### "Network error" on login

**Check:**
1. API is running: `curl http://localhost:5150/api/health`
2. CORS headers are set
3. Tenant header is correct: `X-Tenant-Slug: demo`

### "Unauthorized" error

**Check:**
1. Credentials are correct
2. Token is not expired
3. Token is in `Authorization: Bearer <token>` header

### "User not found" after login

**Check:**
1. Seed data created: `admin@local` / `admin12345`
2. Tenant exists: `demo`
3. GraphQL `me` query works: `curl -X POST http://localhost:5150/api/graphql -H "Authorization: Bearer <token>" -d '{"query":"query{me{id email}}"}'`

---

## Roadmap

- [x] REST API client (`sign_in`, `sign_up`, `sign_out`)
- [x] Auth context & hooks
- [x] Protected routes
- [x] LocalStorage persistence
- [ ] Token refresh on expiry (auto-retry)
- [ ] Password reset flow
- [ ] Email verification flow
- [ ] 2FA support
- [ ] SSR support (server-side auth)

---

## Status

✅ **Ready to use**

**Last updated:** 2026-02-13  
**Version:** 0.1.0
