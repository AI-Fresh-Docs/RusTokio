# Events and outbox

RusToK uses domain events to decouple write operations from projections and integrations.

## Key concepts

- Event envelopes include tenant context and versioning metadata.
- Outbox relay ensures reliable delivery for asynchronous transports.
- Projection workers (e.g., `rustok-index`) maintain read models.

## References

- [Transactional event publishing](../transactional_event_publishing.md)
- [Eventbus consistency audit](../EVENTBUS_CONSISTENCY_AUDIT.md)
- [State machine guide](../STATE_MACHINE_GUIDE.md)
