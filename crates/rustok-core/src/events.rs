use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub event: Arc<DomainEvent>,
}

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ModuleEnabled {
        tenant_id: Uuid,
        module_slug: String,
    },
    ModuleDisabled {
        tenant_id: Uuid,
        module_slug: String,
    },
}

pub trait EventHandler: Send + Sync {
    fn handles(&self, event: &DomainEvent) -> bool;
    fn name(&self) -> &'static str;
    fn handle(&self, envelope: &EventEnvelope) -> crate::Result<()>;
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: DomainEvent) -> crate::Result<()> {
        let envelope = EventEnvelope {
            id: crate::generate_id(),
            occurred_at: Utc::now(),
            event: Arc::new(event),
        };
        let _ = self.sender.send(envelope);
        Ok(())
    }
}
