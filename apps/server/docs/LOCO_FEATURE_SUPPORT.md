# RusToK Server — Loco.rs Feature Support Analysis

**Date:** 2026-02-18  
**Loco.rs Version:** `0.16` (workspace dependency)  
**Status:** Core Loco hooks and runtime are in use; some optional Loco subsystems are replaced by project-specific implementations.

---

## ✅ Implemented Loco.rs functionality in `apps/server`

### 1) App lifecycle hooks (`Hooks`)

| Hook / capability | Status | Evidence |
|---|---|---|
| `app_name`, `app_version` | ✅ | implemented in `App` |
| `boot` (`create_app`) | ✅ | default Loco boot via migrator |
| `routes` | ✅ | controllers registered through `AppRoutes` |
| `after_routes` | ✅ | Axum layers + tenant middleware + module registry |
| `truncate` | ✅ | ordered table cleanup (not stub) |
| `register_tasks` | ✅ | cleanup task registered |
| `initializers` | ✅ | telemetry initializer factory |
| `connect_workers` | ✅ | outbox relay worker startup hook |
| `seed` | ✅ | custom seed pipeline wired |

### 2) Config/runtime conventions

| Capability | Status | Evidence |
|---|---|---|
| Environment config files | ✅ | `development.yaml`, `test.yaml` |
| Loco `auth.jwt` usage | ✅ | secret/expiration in config |
| `settings` extension | ✅ | `settings.rustok.*` parsed into typed settings |

### 3) Tasks / maintenance

| Capability | Status | Evidence |
|---|---|---|
| Loco task registration | ✅ | `tasks::register(tasks)` |
| Loco `Task` implementation | ✅ | `CleanupTask` (`sessions`, `cache`, full cleanup) |
| CLI execution model | ✅ | `cargo loco task --name cleanup ...` notes in source/docs |

### 4) Initializers

| Capability | Status | Evidence |
|---|---|---|
| Loco Initializer integration | ✅ | `initializers::create` returns `Vec<Box<dyn Initializer>>` |
| Telemetry initializer | ✅ | `TelemetryInitializer` implements `before_run` |

---

## ⚠️ Loco features not implemented directly (or partially)

### 1) Loco job workers / queue subsystem

**In Loco:** generalized `Worker` jobs backed by queue integrations.  
**In RusToK now:** only domain-specific outbox relay is started from `connect_workers`; no generic `Worker` registrations / queue job processing.

### 2) Loco Mailer abstraction

**In Loco:** framework mailer trait/integration.

**In RusToK now:** no Loco Mailer integration, but email sending exists via a **separate SMTP service** (`lettre`) used in GraphQL password reset flow:
- `EmailService::{Disabled,Smtp}` abstraction
- SMTP sender implementation
- `forgot_password` mutation dispatching reset email

So mail capability exists, but implemented project-side, not through Loco mailer API.

### 3) Loco Storage abstraction for uploads/assets

**In Loco:** framework storage adapters (e.g. local/S3 style abstraction).  
**In RusToK now:** no Loco storage layer for file uploads/assets.

There is project-specific persistence for the Alloy scripting subsystem (`SeaOrmStorage`), but this is not equivalent to Loco's generic file storage feature.

---

## Что есть в Loco, но у нас реализовано отдельно

1. **Mail delivery flow** (у Loco это Mailer subsystem) → у нас отдельный SMTP-сервис (`services/email.rs`) с feature-toggle через `settings.rustok.email.*`.
2. **Background processing strategy** (у Loco обычно queue workers) → у нас event-driven outbox relay worker (`connect_workers` + event runtime factory).
3. **Framework-level storage uploads** (Loco storage adapters) → у нас пока нет общего storage для upload; отдельные подсистемные storage-решения используются точечно (Alloy scripting).
4. **Auth business logic** (Loco даёт базовые паттерны) → у нас собственная реализация auth/session/password-reset/RBAC поверх GraphQL и доменных моделей.

---

## Quick coverage summary

- **Core Loco app hooks:** implemented and actively used.
- **Tasks + Initializers:** implemented.
- **Generic Loco Workers / Mailer / Storage APIs:** not adopted as-is.
- **Equivalent project behavior:** implemented selectively via RusToK-specific services and event architecture.

---

## Sources (server-side)

- `apps/server/src/app.rs`
- `apps/server/src/tasks/mod.rs`
- `apps/server/src/tasks/cleanup.rs`
- `apps/server/src/initializers/mod.rs`
- `apps/server/src/initializers/telemetry.rs`
- `apps/server/src/services/email.rs`
- `apps/server/src/graphql/auth/mutation.rs`
- `apps/server/src/common/settings.rs`
- `apps/server/config/development.yaml`
- `apps/server/config/test.yaml`
- `apps/server/Cargo.toml`
- workspace `Cargo.toml`
