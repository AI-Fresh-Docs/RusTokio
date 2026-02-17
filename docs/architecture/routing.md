# Routing policy

The server exposes module APIs using a consistent `/api/{module}` routing convention.

## Guidelines

- Each module owns its REST controllers and OpenAPI documentation.
- GraphQL schema is composed from per-module query/mutation objects.
- Versioning and deprecation should be recorded in specs and release notes.

## References

- [API architecture](../api-architecture.md)
- [Architecture guide](../ARCHITECTURE_GUIDE.md)
