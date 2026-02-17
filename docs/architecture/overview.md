# Architecture overview

RusToK is a modular, event-driven headless CMS + commerce platform built on Rust, Tokio, and Loco.rs (Axum).

## Key characteristics

- Modular domain crates (`crates/rustok-*`) composed by `apps/server`.
- CQRS-style separation with write models in domain services and read projections maintained by `rustok-index`.
- Event-driven integration via in-memory bus, outbox relay, or streaming transports.
- Multi-tenant request context enforced by middleware and tenant-aware caching.

## References

- [Architecture guide](../ARCHITECTURE_GUIDE.md)
- [Architecture diagram](../ARCHITECTURE_DIAGRAM.md)
- [Technical article](../../TECHNICAL_ARTICLE.md)
- [API architecture](../api-architecture.md)
