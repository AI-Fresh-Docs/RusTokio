# Документация по модулям RusToK

Этот документ описывает текущее состояние модульной архитектуры в репозитории:
- какие модульные crate'ы существуют;
- какие из них реально зарегистрированы в `rustok-server`;
- какие crate'ы относятся к инфраструктуре и приложениями.

## 1. Общая картина

RusToK — модульный монолит: модули компилируются в общий бинарник, но имеют
изолированную ответственность и общий контракт `RusToKModule`.

Ключевой момент: **наличие module crate не равно runtime-регистрации**. Модуль
должен быть явно добавлен в `build_registry()` сервера.

**Где смотреть в коде:**
- Runtime-регистрация: `apps/server/src/modules/mod.rs`
- Контракт модуля/реестр: `crates/rustok-core/src/module.rs`, `crates/rustok-core/src/registry.rs`
- Конфигурация workspace: `Cargo.toml`
- Манифест сборки: `modules.toml`

## 2. Разделение на Core и Optional

Начиная с [ADR 2026-02-19](../../DECISIONS/2026-02-19-module-kind-core-vs-optional.md),
модули разделены на два класса через `ModuleKind`:

| Класс | Описание |
|-------|----------|
| `ModuleKind::Core` | Всегда активен. Toggle запрещён. Регистрируется в `core_modules` bucket реестра. |
| `ModuleKind::Optional` | Управляется per-tenant через `ModuleLifecycleService`. |

Попытка отключить Core-модуль вернёт `ToggleModuleError::CoreModuleCannotBeDisabled`.

## 3. Что реально зарегистрировано в сервере

В текущей сборке в `ModuleRegistry` регистрируются:

### Core-модули (всегда активны, `required = true`)

| Slug | Crate | Назначение |
| --- | --- | --- |
| `index` | `rustok-index` | CQRS read-path; storefront читает из index-таблиц |
| `tenant` | `rustok-tenant` | Tenant resolution middleware; проходит каждый HTTP-запрос |
| `rbac` | `rustok-rbac` | RBAC enforcement; все CRUD-хендлеры проверяют права здесь |

### Optional-модули (per-tenant toggle)

| Slug | Crate | Зависимости | Назначение |
| --- | --- | --- | --- |
| `content` | `rustok-content` | — | Базовый CMS-контент |
| `commerce` | `rustok-commerce` | — | e-commerce домен |
| `blog` | `rustok-blog` | `content` | Блоговая надстройка |
| `forum` | `rustok-forum` | `content` | Форумный модуль |
| `pages` | `rustok-pages` | — | Страницы и меню |

## 4. Доменные модули и ответственность

### `rustok-index` (Core)
- Роль: read-model / индексный модуль (CQRS).
- Зарегистрирован как `ModuleKind::Core` — критичен для storefront.

### `rustok-tenant` (Core)
- Роль: tenant metadata/helpers и tenant lifecycle хуки.
- Зарегистрирован как `ModuleKind::Core` — обрабатывает каждый HTTP-запрос.

### `rustok-rbac` (Core)
- Роль: role-based access control helpers.
- Зарегистрирован как `ModuleKind::Core` — все CRUD-хендлеры проверяют права.

### `rustok-content` (Optional)
- Роль: базовый контентный модуль.
- Основные части: `entities/`, `services/`, `dto/`.

### `rustok-commerce` (Optional)
- Роль: commerce-домен (каталог, заказы, цены, склад).
- Основные части: `entities/`, `services/`, `dto/`.

### `rustok-blog` (Optional, depends on `content`)
- Роль: блоговая надстройка поверх контента.
- Объявляет `fn dependencies() -> &["content"]` — нельзя включить без `content`.

### `rustok-forum` (Optional, depends on `content`)
- Роль: форум (категории, темы, ответы, модерация).
- Объявляет `fn dependencies() -> &["content"]` — нельзя включить без `content`.

### `rustok-pages` (Optional)
- Роль: страницы и меню.

## 5. Инфраструктурные crates

Эти компоненты **не реализуют `RusToKModule`** и не регистрируются в `ModuleRegistry`:

- `rustok-core` — контракты модулей, registry, события, базовые типы.
- `rustok-outbox` — outbox-публикация событий; инициализируется через `build_event_runtime()`.
- `rustok-iggy` — L2 transport/replay.
- `rustok-iggy-connector` — connector-слой для Iggy.
- `rustok-telemetry` — tracing/metrics.
- `rustok-mcp` — MCP toolkit/integration crate.
- `rustok-test-utils` — исключительно `[dev-dependencies]`, в production binary не входит.
- `alloy-scripting` — скриптовый движок и orchestration.

## 6. Приложения

- `apps/server` (`rustok-server`) — API-сервер, поднимает `ModuleRegistry`.
- `apps/admin` (`rustok-admin`) — Leptos CSR админ-панель.
- `apps/storefront` (`rustok-storefront`) — Leptos SSR storefront.
- `apps/next-admin` — Next.js Admin Panel.
- `apps/next-frontend` — Next.js storefront skeleton.
- `crates/rustok-mcp` (bin `rustok-mcp-server`) — MCP stdio сервер и адаптер в одном crate.

## 7. Связанные документы

### Основная документация
- `docs/modules/registry.md` — актуальный реестр приложений и crate'ов.
- `docs/modules/manifest.md` — манифест и правила описания модулей.
- `docs/modules/_index.md` — индекс модульной документации.
- `DECISIONS/2026-02-19-module-kind-core-vs-optional.md` — ADR о разделении Core/Optional.

### Установка модулей с UI
- `docs/modules/UI_PACKAGES_INDEX.md` — Индекс документации по UI пакетам модулей (навигация)
- `docs/modules/UI_PACKAGES_QUICKSTART.md` — Быстрый старт: создание модулей с UI пакетами
- `docs/modules/MODULE_UI_PACKAGES_INSTALLATION.md` — Полное руководство по установке модулей с UI пакетами для админки и фронтенда
- `docs/modules/INSTALLATION_IMPLEMENTATION.md` — реализация системы установки модулей

### Технические спецификации
- `docs/modules/flex.md` — спецификация Flex модуля
- `docs/modules/ALLOY_MANIFEST.md` — манифест Alloy Scripting

## 8. Что делать при изменениях модульного состава

При добавлении/удалении модульных crate'ов или их регистрации в сервере:
1. Обновить `apps/server/src/modules/mod.rs` (если меняется runtime-регистрация).
2. Обновить `modules.toml` (добавить/удалить запись, указать `required = true` для Core).
3. Обновить `docs/modules/overview.md`, `docs/modules/registry.md` и `docs/modules/_index.md`.
4. Проверить consistency с `docs/index.md`.
5. При смене архитектуры — создать ADR в `DECISIONS/`.
