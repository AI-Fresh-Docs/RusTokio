use async_trait::async_trait;

use rustok_core::events::{EventEnvelope, EventTransport, ReliabilityLevel};
use rustok_core::Result;

use crate::backend::{EmbeddedBackend, IggyBackend, RemoteBackend};
use crate::config::{IggyConfig, IggyMode};
use crate::{producer, topology};

#[derive(Debug)]
pub struct IggyTransport {
    config: IggyConfig,
    backend: Box<dyn IggyBackend>,
}

impl IggyTransport {
    pub async fn new(config: IggyConfig) -> Result<Self> {
        let backend: Box<dyn IggyBackend> = match config.mode {
            IggyMode::Remote => Box::new(RemoteBackend::default()),
            IggyMode::Embedded => Box::new(EmbeddedBackend::default()),
        };

        backend.connect(&config).await?;
        topology::ensure_topology(&config).await?;

        Ok(Self { config, backend })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.backend.shutdown().await
    }
}

#[async_trait]
impl EventTransport for IggyTransport {
    async fn publish(&self, envelope: EventEnvelope) -> Result<()> {
        producer::publish(&self.config, envelope).await
    }

    fn reliability_level(&self) -> ReliabilityLevel {
        ReliabilityLevel::Streaming
    }
}
