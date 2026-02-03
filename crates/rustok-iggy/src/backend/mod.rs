use async_trait::async_trait;

use crate::config::IggyConfig;

#[async_trait]
pub trait IggyBackend: Send + Sync {
    async fn connect(&self, config: &IggyConfig) -> rustok_core::Result<()>;
    async fn shutdown(&self) -> rustok_core::Result<()>;
}

#[derive(Debug, Default)]
pub struct EmbeddedBackend;

#[derive(Debug, Default)]
pub struct RemoteBackend;

#[async_trait]
impl IggyBackend for EmbeddedBackend {
    async fn connect(&self, _config: &IggyConfig) -> rustok_core::Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> rustok_core::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl IggyBackend for RemoteBackend {
    async fn connect(&self, _config: &IggyConfig) -> rustok_core::Result<()> {
        Ok(())
    }

    async fn shutdown(&self) -> rustok_core::Result<()> {
        Ok(())
    }
}
