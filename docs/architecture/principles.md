# Architecture principles

## CQRS and event-driven workflows

- Commands mutate write models in domain services.
- Domain events feed projections and integrations.
- Read models are denormalized for storefront/search needs.

## Multi-tenant by default

- Tenant is resolved at request boundaries.
- Data access, caching, and events carry tenant context.

## Modular boundaries

- Each domain module owns its entities, services, DTOs, and API surface.
- Modules are composed in the server via shared contracts.

## Reliability and observability

- Structured logging and tracing are required for critical flows.
- Outbox and relay workers ensure event delivery.

## References

- [Architecture recommendations](../ARCHITECTURE_RECOMMENDATIONS.md)
- [Distributed tracing guide](../DISTRIBUTED_TRACING_GUIDE.md)
