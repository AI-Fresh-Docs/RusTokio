# RusTok Mobile Workspace

Flutter workspace scaffold based on `docs/research/flutter.md`.

## Structure

- `apps/rustok_admin_mobile` — host Flutter app shell.
- `packages/app_core` — shared core primitives.
- `packages/app_ui_kit` — design tokens and presentational widgets.
- `packages/app_graphql` — GraphQL transport wiring.
- `packages/app_route_contracts` — typed route/query contracts.
- `packages/app_module_contracts` — interfaces for module-owned mobile packages.

## Implemented now

- Host app routing with `go_router` + `ShellRoute`.
- Generated-manifest style module registry adapter (`mobile_manifest.g.dart`).
- Shared route contracts with snake_case query key constraints.
- Shared GraphQL transport context/header builders.

## Next steps

1. Add real GraphQL client factory (HTTP + WebSocket split links).
2. Generate `mobile_manifest.g.dart` from RusTok module manifests.
3. Start first module package (`rustok_auth_mobile`) with real screens.
