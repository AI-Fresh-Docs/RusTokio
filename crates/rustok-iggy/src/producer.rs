use rustok_core::events::EventEnvelope;
use rustok_core::Result;

use crate::config::IggyConfig;

pub async fn publish(config: &IggyConfig, envelope: EventEnvelope) -> Result<()> {
    let topic = match envelope.event.event_type() {
        event_type if event_type.starts_with("system.") => "system",
        _ => "domain",
    };
    let partition_key = envelope.tenant_id.to_string();

    tracing::debug!(
        stream = %config.stream,
        topic,
        partition_key,
        event_id = %envelope.id,
        "Publishing event to iggy"
    );

    Ok(())
}
