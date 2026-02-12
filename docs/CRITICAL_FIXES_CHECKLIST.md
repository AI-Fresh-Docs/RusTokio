# RusToK — Чеклист критических исправлений

> **Для немедленного исполнения**

---

## 🔴 P0: Критические (безопасность/надежность)

### 1. TransactionalEventBus во всех модулях

- [ ] **rustok-commerce/src/services/catalog.rs**
  ```rust
  // Заменить:
  pub struct CatalogService {
      db: DatabaseConnection,
      event_bus: EventBus,  // ❌
  }
  
  // На:
  pub struct CatalogService {
      db: DatabaseConnection,
      event_bus: TransactionalEventBus,  // ✅
  }
  ```

- [ ] **rustok-commerce/src/services/inventory.rs**
  - Проверить и обновить

- [ ] **rustok-commerce/src/services/pricing.rs**
  - Проверить и обновить

- [ ] **rustok-forum/src/services/*.rs**
  - Проверить все сервисы

- [ ] **rustok-blog/src/services/*.rs**
  - Проверить все сервисы

- [ ] **rustok-pages/src/services/*.rs**
  - Проверить все сервисы

### 2. Убрать `let _ =` для событий

- [ ] **crates/rustok-commerce/src/services/catalog.rs**
  ```rust
  // Заменить (строка ~196):
  let _ = self.event_bus.publish(...);  // ❌
  
  // На:
  self.event_bus
      .publish_in_tx(&txn, tenant_id, Some(actor_id), DomainEvent::ProductCreated { product_id })
      .await?;  // ✅
  ```

---

## 🟡 P1: Важные (стабильность/производительность)

### 3. Добавить rate limiting в EventDispatcher

- [ ] **crates/rustok-core/src/events/handler.rs**
  ```rust
  pub struct DispatcherConfig {
      pub fail_fast: bool,
      pub max_concurrent: usize,
      pub retry_count: usize,
      pub retry_delay_ms: u64,
      pub max_queue_depth: usize,  // 🆕 Добавить
  }
  ```

### 4. Graceful shutdown

- [ ] **apps/server/src/app.rs**
  ```rust
  impl Hooks for App {
      async fn shutdown(&self, ctx: &AppContext) {
          // Добавить cleanup
      }
  }
  ```

### 5. Упрощение tenant cache

- [ ] **apps/server/src/middleware/tenant.rs**
  - Рассмотреть переход на `moka::future::Cache`

---

## 🟢 P2: Качество кода

### 6. Стандартизация slugify

- [ ] **crates/rustok-commerce/src/services/catalog.rs**
  ```rust
  // Добавить в Cargo.toml:
  // slug = "0.1"
  
  // Заменить ручную реализацию на:
  use slug::slugify;
  ```

### 7. Валидация событий

- [ ] **crates/rustok-core/src/events/types.rs**
  ```rust
  impl DomainEvent {
      pub fn validate(&self) -> Result<(), ValidationError> {
          // Добавить валидацию
      }
  }
  ```

---

## 📋 Порядок исправления

```
День 1-2: P0 (TransactionalEventBus)
День 3:   P1 (Graceful shutdown)
День 4-5: P1 (Rate limiting)
День 6+:  P2 (Качество кода)
```

---

## ✅ Проверка после исправлений

```bash
# Сборка
cargo build --release

# Тесты
cargo test --workspace

# Проверка безопасности
cargo audit

# Форматирование и линт
cargo fmt --check
cargo clippy -- -D warnings
```
