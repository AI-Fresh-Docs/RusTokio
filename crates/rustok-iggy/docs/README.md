# rustok-iggy docs

Documentation for the `crates/rustok-iggy` module.

## Documents

- [Implementation Plan](./implementation-plan.md) - Delivery phases and component status

## Module Overview

`rustok-iggy` provides event streaming transport using Iggy.rs. It implements the
`EventTransport` trait and supports both embedded and remote modes.

## Key Types

| Type | Description |
|------|-------------|
| `IggyTransport` | Main transport implementing EventTransport |
| `IggyConfig` | Configuration for transport setup |
| `TopologyManager` | Stream/topic management |
| `ConsumerGroupManager` | Consumer group coordination |
| `DlqManager` | Dead letter queue handling |
| `ReplayManager` | Event replay orchestration |

## Quick Reference

```rust
use rustok_iggy::{IggyConfig, IggyTransport, SerializationFormat};
use rustok_core::events::EventTransport;

// Create transport
let config = IggyConfig::default();
let transport = IggyTransport::new(config).await?;

// Use as EventTransport
transport.publish(envelope).await?;

// Cleanup
transport.shutdown().await?;
```

## Configuration

See the main [README](../README.md) for configuration options.
