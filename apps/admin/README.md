# RusToK Admin (Leptos)

`apps/admin` — Leptos CSR админка RusToK, развиваемая параллельно с `apps/next-admin` для функционального паритета.

## Роль в платформе

- управление контентом, пользователями, безопасностью и настройками;
- Rust/Leptos реализация admin-панели;
- эталон для внутреннего UI и crate-контрактов в Rust фронтенде.

## FSD структура

Приложение следует FSD-слоям:

- `app/` — app shell, роутер, провайдеры;
- `pages/` — страницы;
- `widgets/` — агрегированные UI-блоки;
- `features/` — сценарные модули;
- `entities/` — сущности домена UI-уровня;
- `shared/` — инфраструктурные утилиты, API, UI primitives.

## Библиотеки и контракты

- `leptos`, `leptos_router` — UI и маршрутизация;
- `tailwindcss` + shadcn token model;
- `leptos-graphql` — GraphQL transport/контракты;
- `leptos-auth` — auth/session контракты;
- `leptos-hook-form`, `leptos-zod`, `leptos-table`, `leptos-zustand`, `leptos-shadcn-pagination` — формы/валидация/таблицы/состояние.

## Взаимодействие

- `apps/server` (HTTP/GraphQL API)
- `crates/rustok-rbac` и другие доменные модули через backend
- общий UI контракт с `apps/next-admin` и storefront приложениями

## Документация

- Локальная: `apps/admin/docs/README.md`
- Платформенная: `docs/UI/fsd-restructuring-plan.md`, `docs/UI/rust-ui-component-catalog.md`, `docs/index.md`
