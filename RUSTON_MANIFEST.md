# RusToK — System Architecture Manifest v4.0

**Codename:** "The Highload Tank"  \
**Target:** AI Assistants (Cursor, Windsurf, Copilot, Claude)  \
**Role:** Senior Rust Architect & System Designer  \
**Philosophy:** "Write Optimized vs Read Optimized" / "Rust is ON. WordPress is OFF."

---

## CHANGELOG v3.0 → v4.0

- **Unified Core:** базовые таблицы контента в ядре.
- **CQRS-lite:** разделение write/read paths.
- **Index Module:** денормализованные индексы для поиска.
- **Partitioning Strategy:** масштабирование таблиц.
- **Event-Driven:** модули общаются через события.
- **Microservice-Ready:** Index Module выносится отдельно.

---

## 1. PROJECT IDENTITY

| Property | Value |
|----------|-------|
| **Name** | RusToK |
| **Type** | Event-Driven Enterprise Headless Platform |
| **Architecture** | Modular Monolith with CQRS & Event Sourcing elements |
| **Language** | Rust 100% |
| **License** | AGPL-3.0 (core) + MIT (modules) |
| **Version** | 4.0 (The Highload Tank) |
| **Repository** | https://github.com/RustokCMS/RusToK |

---

## 2. CORE PHILOSOPHY

### 2.1 The Tank Strategy

- **Stability First:** Мы строим "Танк", а не хрупкую экосистему плагинов.
- **Compile-Time Safety:** Если компилируется — работает.
- **Monorepo:** Backend, Admin и Storefront живут вместе.

### 2.2 Universal Core, Specific Modules

- **Unified Core:** Ядро содержит только то, что нужно всем (SEO, Tags, Users, Basic Nodes).
- **Specialized Modules:** Товары ≠ статьи, у каждого модуля своя бизнес-логика и таблицы.
- **Empty Tables Cost Zero:** Неиспользуемые таблицы не мешают.

### 2.3 CQRS (Write vs Read)

- **Write Model (Modules):** строгие реляционные таблицы (3NF), транзакции, валидация.
- **Read Model (Index/Catalog):** денормализованный JSONB/индексы, GIN, быстрый поиск.
- **Event-Driven Sync:** изменения propagate через события.

### 2.4 Highload by Default

- **Event-Driven Glue:** модули не знают друг о друге напрямую. Они общаются через EventBus.
- **No Heavy JOINs on Storefront:** данные "склеиваются" при записи, а не при чтении.

---

## 3. TECHNOLOGY STACK

| Layer | Technology | Details |
|-------|------------|---------|
| **Repository** | Cargo Workspace | Monorepo for all apps & crates |
| **Runtime** | Tokio | Async runtime |
| **Backend Framework** | Loco.rs | Axum-based, Rails-like MVC |
| **Admin UI** | Leptos CSR | Client-Side WASM |
| **Storefront** | Leptos SSR | Server-Side Rendering |
| **Database** | PostgreSQL 16+ | Partitioning, JSONB |
| **ORM** | SeaORM | Async, fully typed |
| **API** | async-graphql | Schema Federation |
| **IDs** | ULID | Generated via `ulid` crate, stored as `Uuid` |
| **Events** | tokio::broadcast | In-process pub/sub |
| **Search (optional)** | Meilisearch / Tantivy | Full-text search |

---

## 4. PROJECT STRUCTURE (Workspace)

```text
rustok/
├── apps/
│   ├── server/           # Loco.rs host application
│   ├── admin/            # Leptos CSR (Unified UI)
│   └── storefront/       # Leptos SSR (Reads from Index/Catalog)
├── crates/
│   ├── rustok-core/      # Universal Entities (User, Tag, SEO, Node)
│   ├── rustok-commerce/  # Medusa-style logic (Product, Order, Inventory)
│   ├── rustok-community/ # Social features
│   └── rustok-index/     # THE INDEXER (CQRS Read Model)
```

---

## 5. DATABASE ARCHITECTURE

### 5.1 ID Generation (ULID → UUID)

```rust
// crates/rustok-core/src/id.rs
use ulid::Ulid;
use uuid::Uuid;

pub fn generate_id() -> Uuid {
    Uuid::from(Ulid::new())
}

pub fn parse_id(s: &str) -> Result<Uuid, IdError> {
    s.parse::<Ulid>()
        .map(Uuid::from)
        .or_else(|_| s.parse::<Uuid>())
        .map_err(|_| IdError::InvalidFormat(s.to_string()))
}
```

### 5.2 RusToK Core (Unified Foundation)

```sql
-- Tenants
CREATE TABLE tenants (
    id              UUID PRIMARY KEY,
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(64) NOT NULL UNIQUE,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Users
CREATE TABLE users (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email           VARCHAR(255) NOT NULL,
    password_hash   VARCHAR(255) NOT NULL,
    role            VARCHAR(32) NOT NULL DEFAULT 'customer',
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, email)
);

-- Nodes (универсальный контент)
CREATE TABLE nodes (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES nodes(id) ON DELETE CASCADE,
    author_id       UUID REFERENCES users(id) ON DELETE SET NULL,
    kind            VARCHAR(32) NOT NULL,       -- 'page', 'post', 'comment'
    title           VARCHAR(255),
    slug            VARCHAR(255),
    excerpt         TEXT,
    category_id     UUID REFERENCES categories(id) ON DELETE SET NULL,
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    position        INT DEFAULT 0,
    depth           INT DEFAULT 0,
    reply_count     INT DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,
    UNIQUE (tenant_id, kind, slug) WHERE slug IS NOT NULL
);

-- Bodies (тяжёлый контент отдельно)
CREATE TABLE bodies (
    node_id         UUID PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    body            TEXT,
    format          VARCHAR(16) NOT NULL DEFAULT 'markdown',
    search_vector   TSVECTOR,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Categories (контентные)
CREATE TABLE categories (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES categories(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL,
    description     TEXT,
    position        INT NOT NULL DEFAULT 0,
    depth           INT NOT NULL DEFAULT 0,
    node_count      INT NOT NULL DEFAULT 0,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, slug)
);

-- Tags (универсальные ярлыки)
CREATE TABLE tags (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL,
    use_count       INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, slug)
);

-- Taggables (полиморфная связь)
CREATE TABLE taggables (
    tag_id          UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    target_type     VARCHAR(32) NOT NULL,       -- 'node', 'product'
    target_id       UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tag_id, target_type, target_id)
);

-- Meta (SEO, универсальное)
CREATE TABLE meta (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    target_type     VARCHAR(32) NOT NULL,       -- 'node', 'product', 'category'
    target_id       UUID NOT NULL,
    title           VARCHAR(255),
    description     VARCHAR(500),
    keywords        VARCHAR(255),
    og_title        VARCHAR(255),
    og_description  VARCHAR(500),
    og_image        VARCHAR(500),
    og_type         VARCHAR(32),
    twitter_card    VARCHAR(32),
    no_index        BOOLEAN NOT NULL DEFAULT false,
    no_follow       BOOLEAN NOT NULL DEFAULT false,
    canonical_url   VARCHAR(500),
    structured_data JSONB,
    UNIQUE (target_type, target_id)
);

-- Media (файлы)
CREATE TABLE media (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    uploaded_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    filename        VARCHAR(255) NOT NULL,
    original_name   VARCHAR(255) NOT NULL,
    mime_type       VARCHAR(100) NOT NULL,
    size            BIGINT NOT NULL,
    storage_path    VARCHAR(500) NOT NULL,
    storage_driver  VARCHAR(32) NOT NULL DEFAULT 'local',
    width           INT,
    height          INT,
    alt_text        VARCHAR(255),
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Module Toggles
CREATE TABLE tenant_modules (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module_slug     VARCHAR(64) NOT NULL,
    enabled         BOOLEAN NOT NULL DEFAULT true,
    settings        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, module_slug)
);
```

### 5.3 RusToK Commerce (Module)

```sql
CREATE TABLE commerce_products (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    subtitle        VARCHAR(255),
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
    discountable    BOOLEAN NOT NULL DEFAULT true,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, handle)
);

CREATE TABLE commerce_variants (
    id              UUID PRIMARY KEY,
    product_id      UUID NOT NULL REFERENCES commerce_products(id) ON DELETE CASCADE,
    title           VARCHAR(255) NOT NULL,
    sku             VARCHAR(64),
    barcode         VARCHAR(64),
    manage_inventory BOOLEAN NOT NULL DEFAULT true,
    allow_backorder  BOOLEAN NOT NULL DEFAULT false,
    weight          INT,
    length          INT,
    height          INT,
    width           INT,
    position        INT NOT NULL DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE commerce_categories (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES commerce_categories(id) ON DELETE SET NULL,
    name            VARCHAR(255) NOT NULL,
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    is_internal     BOOLEAN NOT NULL DEFAULT false,
    rank            INT NOT NULL DEFAULT 0,
    metadata        JSONB NOT NULL DEFAULT '{}',
    UNIQUE (tenant_id, handle)
);
```

### 5.4 RusToK Index/Catalog (CQRS Read Model)

```sql
CREATE TABLE index_products (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    product_id      UUID NOT NULL,
    title           VARCHAR(255) NOT NULL,
    subtitle        VARCHAR(255),
    handle          VARCHAR(255) NOT NULL,
    description     TEXT,
    status          VARCHAR(32) NOT NULL,
    min_price       BIGINT,
    max_price       BIGINT,
    currencies      CHAR(3)[],
    total_stock     INT,
    has_stock       BOOLEAN,
    categories      JSONB,
    tags            TEXT[],
    meta_title      VARCHAR(255),
    meta_description VARCHAR(500),
    search_vector   TSVECTOR,
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (product_id)
);

CREATE TABLE index_content (
    id              UUID PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    node_id         UUID NOT NULL,
    kind            VARCHAR(32) NOT NULL,
    title           VARCHAR(255),
    slug            VARCHAR(255),
    excerpt         TEXT,
    body_preview    TEXT,
    status          VARCHAR(32) NOT NULL,
    author_id       UUID,
    author_name     VARCHAR(255),
    category_id     UUID,
    category_name   VARCHAR(255),
    category_slug   VARCHAR(255),
    tags            TEXT[],
    parent_id       UUID,
    reply_count     INT,
    meta_title      VARCHAR(255),
    meta_description VARCHAR(500),
    search_vector   TSVECTOR,
    published_at    TIMESTAMPTZ,
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (node_id)
);
```

### 5.5 Partitioning Strategy (Highload)

```sql
-- Orders by date
CREATE TABLE commerce_orders_partitioned (
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE TABLE commerce_orders_2025_q1 
    PARTITION OF commerce_orders_partitioned
    FOR VALUES FROM ('2025-01-01') TO ('2025-04-01');

CREATE TABLE commerce_orders_future 
    PARTITION OF commerce_orders_partitioned
    DEFAULT;

-- Nodes by tenant
CREATE TABLE nodes_partitioned (
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    PRIMARY KEY (id, tenant_id)
) PARTITION BY HASH (tenant_id);
```

---

## 6. TRAITS & INTERFACES (Rust Code)

### 6.1 Universal Traits (rustok-core)

```rust
// crates/rustok-core/src/traits.rs

#[async_trait]
pub trait RusToKEntity {
    fn id(&self) -> Uuid;
    fn tenant_id(&self) -> Uuid;
    fn entity_type(&self) -> &'static str;
}

#[async_trait]
pub trait SeoAware: RusToKEntity {
    async fn get_seo(&self, db: &DatabaseConnection) -> Result<Option<SeoModel>> {
        // Default implementation fetches from meta table
    }
}

#[async_trait]
pub trait Taggable: RusToKEntity {
    async fn sync_tags(&self, db: &DatabaseConnection, tags: Vec<String>) -> Result<()> {
        // Logic to update taggables
    }
}
```

---

## 7. EVENT BUS & INDEXING FLOW (CQRS)

### 7.1 Domain Events

```rust
// crates/rustok-core/src/events/types.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: DomainEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // Content Events
    NodeCreated { node_id: Uuid, kind: String, author_id: Option<Uuid> },
    NodeUpdated { node_id: Uuid },
    NodePublished { node_id: Uuid, kind: String },
    NodeDeleted { node_id: Uuid, kind: String },

    // Commerce Events
    ProductCreated { product_id: Uuid },
    ProductUpdated { product_id: Uuid },
    ProductPublished { product_id: Uuid },
    ProductDeleted { product_id: Uuid },

    VariantCreated { variant_id: Uuid, product_id: Uuid },
    VariantUpdated { variant_id: Uuid, product_id: Uuid },

    InventoryUpdated {
        variant_id: Uuid,
        location_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
    },

    OrderPlaced { order_id: Uuid, customer_id: Option<Uuid>, total: i64 },
    OrderStatusChanged { order_id: Uuid, old_status: String, new_status: String },
    OrderCompleted { order_id: Uuid },
    OrderCancelled { order_id: Uuid, reason: Option<String> },

    // User Events
    UserRegistered { user_id: Uuid, email: String },
    UserLoggedIn { user_id: Uuid },

    // Tag Events
    TagAttached { tag_id: Uuid, target_type: String, target_id: Uuid },
    TagDetached { tag_id: Uuid, target_type: String, target_id: Uuid },

    // Index Events
    ReindexRequested { target_type: String, target_id: Option<Uuid> },
}
```

### 7.2 Event Bus

```rust
// crates/rustok-core/src/events/bus.rs

use tokio::sync::broadcast;
use std::sync::Arc;

pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    capacity: usize,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, capacity }
    }

    pub fn publish(&self, tenant_id: Uuid, event: DomainEvent) {
        let envelope = EventEnvelope {
            id: generate_id(),
            tenant_id,
            timestamp: Utc::now(),
            event,
        };

        if self.sender.receiver_count() == 0 {
            tracing::debug!("No event subscribers for {:?}", envelope.event);
        }

        let _ = self.sender.send(envelope);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            capacity: self.capacity,
        }
    }
}
```

### 7.3 Indexer Flow (CQRS)

```rust
// crates/rustok-index/src/indexers/product_indexer.rs

pub async fn handle_event(event: DomainEvent, db: &DatabaseConnection) -> Result<()> {
    match event {
        DomainEvent::ProductUpdated { product_id } => {
            let product = commerce::find_product(product_id).await?;
            let seo = core::find_meta(product_id).await?;
            let tags = core::find_tags(product_id).await?;

            let catalog_item = CatalogItem {
                id: product_id,
                kind: "product".to_string(),
                title: product.title,
                price: product.variants[0].price,
                seo_title: seo.title.unwrap_or(product.title),
                tags,
            };

            catalog::upsert(catalog_item).await?;
        }
        _ => {}
    }
    Ok(())
}
```

---

## 8. MODULE REGISTRATION

```rust
pub trait RusToKModule {
    fn name(&self) -> &str;
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;
    fn event_listeners(&self) -> Vec<Box<dyn EventListener>>;
}
```

---

## 9. DEPLOYMENT ARCHITECTURE

### 9.1 Monolith (Default)

```yaml
services:
  rustok:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db:5432/rustok
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis

  db:
    image: postgres:16
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_USER=rustok
      - POSTGRES_PASSWORD=rustok
      - POSTGRES_DB=rustok

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### 9.2 Microservices (Scale)

```yaml
services:
  api:
    build:
      context: .
      dockerfile: apps/server/Dockerfile
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db-primary:5432/rustok
      - INDEX_SERVICE_URL=http://index:3001
    deploy:
      replicas: 3

  index:
    build:
      context: .
      dockerfile: crates/rustok-index/Dockerfile
    environment:
      - DATABASE_URL=postgres://rustok:rustok@db-replica:5432/rustok
      - MEILISEARCH_URL=http://meilisearch:7700
    deploy:
      replicas: 2

  db-primary:
    image: postgres:16

  db-replica:
    image: postgres:16

  meilisearch:
    image: getmeili/meilisearch:v1.6
    volumes:
      - meilisearch_data:/meili_data

volumes:
  meilisearch_data:
```

---

## 10. SUMMARY: WHY THIS ROCKS

1. **Independent Scaling:** Index tables можно вынести отдельно или заменить Elasticsearch.
2. **Zero-Bloat Core:** Нет ненужных таблиц, если модуль не используется.
3. **Fast Storefront:** Один запрос к индекс-таблицам вместо тяжёлых JOIN-ов.
4. **Admin DX:** Админка выглядит монолитной, но под капотом разрозненные сервисы.

---

## 11. CHECKLIST (Updated)

Before implementing any feature, verify:

- Uses Uuid for all IDs (generated from ULID)
- Includes tenant_id for multi-tenant entities
- Implements proper error handling with RusToKError
- Has SeaORM entity with relations
- Has service layer (not direct DB access in handlers)
- Publishes events for state changes
- GraphQL resolvers check tenant context
- Admin resource registered with proper permissions
- Index updated via event handler (if searchable)
- Core tables used for universal features (tags, meta)
- Module-specific tables for domain logic

---

END OF MANIFEST v4.0
