# Phase 1 Status & Decision

**Дата:** 2026-02-13  
**Статус:** 🚧 В работе  
**Решение:** Auth через REST API (не GraphQL)

---

## 🎯 Текущая ситуация

### Что уже есть

#### Backend (✅ Готово)

1. **REST Auth Endpoints** (`apps/server/src/controllers/auth.rs`)
   - ✅ `/api/auth/register` — регистрация
   - ✅ `/api/auth/login` — логин
   - ✅ `/api/auth/logout` — logout
   - ✅ `/api/auth/refresh` — refresh token
   - ✅ `/api/auth/forgot-password` — запрос сброса
   - ✅ `/api/auth/reset-password` — сброс пароля
   - ✅ JWT + refresh tokens + sessions
   - ✅ Multi-tenant support
   - ✅ RBAC (roles & permissions)

2. **GraphQL Query** (`apps/server/src/graphql/queries.rs`)
   - ✅ `me` query (lines 95-111) — получить текущего пользователя
   - ✅ `user(id)` query — получить пользователя по ID
   - ✅ `users(pagination, filter)` query — список пользователей

#### Frontend (🚧 В работе)

1. **leptos-graphql** (✅ Готово)
   - HTTP transport для GraphQL
   - Работает корректно

2. **leptos-auth** (⚠️ Требует корректировки)
   - ✅ Context, hooks, components, storage — готовы
   - ❌ `api.rs` — написан под GraphQL mutations, но они НЕ реализованы на сервере
   - ❌ Ожидает `signIn`, `signUp`, `signOut` mutations (которых нет на сервере)

---

## ❓ Проблема

**Несоответствие между backend и frontend:**

```
┌──────────────────────────────────────────────────┐
│ Backend (apps/server)                             │
│  ✅ REST API для auth (/api/auth/*)              │
│  ❌ GraphQL mutations для auth (НЕ реализованы)  │
│  ✅ GraphQL query `me`                            │
└──────────────────────────────────────────────────┘
                      ▲
                      │ Ожидает GraphQL mutations
                      │
┌──────────────────────────────────────────────────┐
│ Frontend (crates/leptos-auth/src/api.rs)         │
│  ❌ Использует GraphQL mutations                 │
│  signIn, signUp, signOut — НЕ РАБОТАЮТ           │
└──────────────────────────────────────────────────┘
```

**Вопрос:** Что делать?

### Варианты решения

#### Вариант 1: ✅ **Auth через REST API** (РЕКОМЕНДУЕМ)

**Плюсы:**
- ✅ REST auth — industry best practice (OAuth, JWT)
- ✅ Backend УЖЕ реализован (работает)
- ✅ Проще отладка (curl, Postman)
- ✅ Меньше дублирования кода
- ✅ GraphQL для data, REST для auth — стандартный подход

**Минусы:**
- ⚠️ Нужно переписать `leptos-auth/src/api.rs` (30-50 строк)

**Действие:**
1. Переписать `leptos-auth/src/api.rs` — использовать `fetch()` вместо `leptos-graphql`
2. Обновить типы (REST response → `AuthUser`, `AuthSession`)
3. Тестирование
4. Обновить документацию

**Оценка:** 2-3 часа

---

#### Вариант 2: ❌ **Реализовать GraphQL mutations на сервере**

**Плюсы:**
- ✅ Единый API (все через GraphQL)
- ✅ `leptos-auth/src/api.rs` уже готов

**Минусы:**
- ❌ Дублирование кода (REST + GraphQL auth)
- ❌ Больше maintenance burden
- ❌ GraphQL auth — не best practice
- ❌ Сложнее отладка
- ❌ Нужно реализовать 6+ mutations на сервере

**Действие:**
1. Создать `apps/server/src/graphql/auth/mod.rs`
2. Реализовать mutations: `signIn`, `signUp`, `signOut`, `refreshToken`, `forgotPassword`, `resetPassword`
3. Интегрировать в schema.rs
4. Тестирование

**Оценка:** 6-8 часов

---

## ✅ Решение: Auth через REST API

**Причина:** Best practice + backend уже готов + меньше работы

### Архитектура (обновлённая)

```
┌───────────────────────────────────────────────────┐
│  apps/admin (Leptos UI)                            │
│  ┌─────────────────────────────────────────────┐  │
│  │ Login, Register, Dashboard pages            │  │
│  │ Uses: use_auth() hook                       │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────┬─────────────────────────────────┘
                  │
                  │ use_auth() → api::sign_in()
                  ▼
┌───────────────────────────────────────────────────┐
│  crates/leptos-auth (Auth Logic)                   │
│  ┌─────────────────────────────────────────────┐  │
│  │ api.rs: sign_in(), sign_up(), sign_out()    │  │
│  │   → fetch() REST API (/api/auth/*)          │  │  ← CHANGE
│  │ context.rs: AuthProvider, AuthContext       │  │
│  │ hooks.rs: use_auth(), use_token()           │  │
│  │ storage.rs: LocalStorage helpers            │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────┬─────────────────────────────────┘
                  │
                  │ fetch() → REST API
                  ▼
┌───────────────────────────────────────────────────┐
│  apps/server (Backend)                             │
│  ┌─────────────────────────────────────────────┐  │
│  │ REST Auth: /api/auth/login, /register, etc. │  │  ← EXISTING
│  │ GraphQL Data: /api/graphql                  │  │
│  │   queries: me, user, users                  │  │
│  │   mutations: createUser, updateUser         │  │
│  └─────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

**Принцип:**
- **Auth flow** → REST API (`/api/auth/*`)
- **Data queries** → GraphQL API (`/api/graphql`)

---

## 📋 Скорректированный план Phase 1

### ✅ 1.1. Backend (УЖЕ ГОТОВО — ПРОПУСКАЕМ)

- ✅ REST Auth endpoints
- ✅ GraphQL query `me`
- ✅ JWT + sessions + multi-tenant
- ✅ RBAC

**Статус:** ✅ **Завершено** (ничего не нужно делать)

---

### 🚧 1.2. Update leptos-auth (ТЕКУЩАЯ ЗАДАЧА)

**Задача:** Переписать `crates/leptos-auth/src/api.rs` для использования REST API

#### Шаги:

1. **Обновить `api.rs`:**
   - Убрать GraphQL mutations
   - Использовать `fetch()` для REST calls
   - Endpoints:
     * `POST /api/auth/login` → `sign_in()`
     * `POST /api/auth/register` → `sign_up()`
     * `POST /api/auth/logout` → `sign_out()`
     * `POST /api/auth/refresh` → `refresh_token()`
   
2. **Обновить типы:**
   - REST response mapping → `AuthUser`, `AuthSession`
   - Error handling
   
3. **Обновить `context.rs`:**
   - Если нужно (вероятно не требуется)
   
4. **Testing:**
   - Unit tests для `api.rs`
   - Integration test (mock server)

**Приоритет:** **P0** (критично)  
**Оценка:** 2-3 часа  
**Блокирует:** Login/Register pages

---

### ⏳ 1.3. Custom Library: leptos-forms

**Статус:** ⏳ TODO  
**Приоритет:** P0  
**Блокирует:** Login, Register forms

**Задачи:**
- [ ] Core: `Form`, `Field`, `use_form()`
- [ ] Validators: `required()`, `email()`, `min_length()`, etc.
- [ ] Features: per-field errors, submit handling, reactive validation
- [ ] Documentation: README, examples, API reference

**Оценка:** 6-8 часов

---

### ⏳ 1.4. Custom Library: leptos-ui (Phase 1)

**Статус:** ⏳ TODO  
**Приоритет:** P0  
**Блокирует:** Все UI

**Phase 1 Components:**
- [ ] Button (variants, sizes, loading, disabled)
- [ ] Input (types, validation states, icons)
- [ ] Card (header, body, footer)
- [ ] Label
- [ ] Alert (variants, dismissible)

**Оценка:** 8-10 часов

---

### ⏳ 1.5. Leptos Admin: Login & Register

**Статус:** ⏳ TODO  
**Зависит от:** 1.2 (leptos-auth), 1.3 (leptos-forms), 1.4 (leptos-ui)

**Задачи:**
- [ ] Login page (`apps/admin/src/pages/login.rs`)
- [ ] Register page (`apps/admin/src/pages/register.rs`)
- [ ] Integration с `leptos-auth`
- [ ] Validation (email, password)
- [ ] Error handling
- [ ] Loading states

**Оценка:** 4-6 часов

---

### ⏳ 1.6. Leptos Admin: App Shell & Dashboard

**Статус:** ⏳ TODO  
**Зависит от:** 1.5

**Задачи:**
- [ ] App shell layout (`apps/admin/src/layouts/app_shell.rs`)
  - [ ] Sidebar (navigation, user menu)
  - [ ] Header (breadcrumbs, user dropdown)
  - [ ] Main content area
- [ ] Dashboard page (`apps/admin/src/pages/dashboard.rs`)
  - [ ] Stats cards
  - [ ] Charts (placeholder)
  - [ ] Recent activity
- [ ] Protected routes

**Оценка:** 6-8 часов

---

### ⏳ 1.7. Next.js Admin: Login & Register (Parallel)

**Статус:** ⏳ TODO  
**Приоритет:** P1 (после Leptos)

**Задачи:**
- [ ] Login page (`apps/next-admin/app/login/page.tsx`)
- [ ] Register page (`apps/next-admin/app/register/page.tsx`)
- [ ] Auth provider (Context API)
- [ ] Integration с REST API
- [ ] Validation (zod/react-hook-form)

**Оценка:** 4-6 часов

---

### ⏳ 1.8. Next.js Admin: App Shell & Dashboard (Parallel)

**Статус:** ⏳ TODO  
**Зависит от:** 1.7

**Задачи:**
- [ ] App shell layout (`apps/next-admin/components/layout/AppShell.tsx`)
- [ ] Dashboard page (`apps/next-admin/app/dashboard/page.tsx`)
- [ ] Protected routes (middleware)

**Оценка:** 6-8 часов

---

### ⏳ 1.9. Testing & QA

**Задачи:**
- [ ] E2E tests (Playwright)
  - [ ] Login flow
  - [ ] Register flow
  - [ ] Protected routes
  - [ ] Logout
- [ ] Manual testing
- [ ] Smoke tests в CI

**Оценка:** 4-6 часов

---

## 📊 Timeline (обновлённый)

| Задача | Оценка | Зависимости | Приоритет |
|--------|--------|-------------|-----------|
| 1.1. Backend | ✅ 0h | — | P0 |
| 1.2. Update leptos-auth | 2-3h | — | P0 |
| 1.3. leptos-forms | 6-8h | — | P0 |
| 1.4. leptos-ui (Phase 1) | 8-10h | — | P0 |
| 1.5. Leptos Login/Register | 4-6h | 1.2, 1.3, 1.4 | P0 |
| 1.6. Leptos App Shell | 6-8h | 1.5 | P0 |
| 1.7. Next.js Login/Register | 4-6h | 1.2 | P1 |
| 1.8. Next.js App Shell | 6-8h | 1.7 | P1 |
| 1.9. Testing & QA | 4-6h | 1.6, 1.8 | P1 |

**Total:** ~40-55 часов (5-7 рабочих дней)

**Critical Path:**
```
1.2 (2-3h) → 1.3 (6-8h) + 1.4 (8-10h) → 1.5 (4-6h) → 1.6 (6-8h) → 1.9 (4-6h)
              ↓ (parallel)
            1.7 (4-6h) → 1.8 (6-8h) →┘
```

**ETA:** 2026-02-20 (если работать full-time)

---

## 🎯 Следующие шаги

### Немедленно (сегодня)

1. ✅ **Update leptos-auth** (2-3h)
   - Переписать `api.rs` для REST
   - Testing

### Затем (следующие 2-3 дня)

2. **leptos-forms** (6-8h)
3. **leptos-ui Phase 1** (8-10h) — можно параллельно

### Потом (день 4-5)

4. **Leptos Login/Register** (4-6h)
5. **Leptos App Shell** (6-8h)

### Параллельно (день 6-7)

6. **Next.js Login/Register** (4-6h)
7. **Next.js App Shell** (6-8h)

### Финал (день 7)

8. **Testing & QA** (4-6h)

---

## 📚 Обновлённая документация

После завершения обновить:

1. ✅ `docs/UI/PHASE_1_STATUS.md` (этот документ)
2. ⏳ `docs/UI/MASTER_IMPLEMENTATION_PLAN.md`
   - Отметить 1.1 как ✅ (REST already exists)
   - Обновить 1.2 (leptos-auth update)
3. ⏳ `docs/UI/CUSTOM_LIBRARIES_STATUS.md`
   - Обновить статус leptos-auth
   - Добавить leptos-forms, leptos-ui
4. ⏳ `docs/UI/GRAPHQL_ARCHITECTURE.md`
   - Уточнить: Auth через REST, Data через GraphQL
5. ⏳ `crates/leptos-auth/README.md`
   - Обновить примеры (REST вместо GraphQL)

---

## 💡 Summary

**Решение:** ✅ Auth через REST API

**Причины:**
1. ✅ Backend REST auth УЖЕ готов и работает
2. ✅ Industry best practice (Auth → REST, Data → GraphQL)
3. ✅ Меньше работы (2-3h vs 6-8h)
4. ✅ Проще maintenance

**Действие:** Обновить `leptos-auth/src/api.rs` для использования REST API

**Статус:** 🚀 Готовы начать!

---

**Дата обновления:** 2026-02-13  
**Автор:** CTO Agent  
**Следующая задача:** Update `crates/leptos-auth/src/api.rs`
