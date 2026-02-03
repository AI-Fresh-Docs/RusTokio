use rustok_core::Result;

use crate::config::IggyConfig;

pub async fn ensure_topology(config: &IggyConfig) -> Result<()> {
    tracing::debug!(
        stream = %config.stream,
        domain_partitions = config.topology.domain_partitions,
        replication_factor = config.topology.replication_factor,
        "Ensuring iggy topology"
    );
    Ok(())
}
