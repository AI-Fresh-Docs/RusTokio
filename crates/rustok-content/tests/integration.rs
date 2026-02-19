use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rustok_core::events::{
    DomainEvent, EventDispatcher, EventEnvelope, EventHandler, HandlerResult,
};
use rustok_core::EventBus;
use uuid::Uuid;

#[derive(Clone, Default)]
struct ContentIndexProjection {
    documents: Arc<Mutex<HashMap<Uuid, String>>>,
}

impl ContentIndexProjection {
    fn upsert(&self, node_id: Uuid, kind: &str) {
        self.documents
            .lock()
            .expect("content projection lock poisoned")
            .insert(node_id, kind.to_string());
    }

    fn get(&self, node_id: Uuid) -> Option<String> {
        self.documents
            .lock()
            .expect("content projection lock poisoned")
            .get(&node_id)
            .cloned()
    }

    fn len(&self) -> usize {
        self.documents
            .lock()
            .expect("content projection lock poisoned")
            .len()
    }
}

#[derive(Clone)]
struct NodeCreatedIndexHandler {
    projection: ContentIndexProjection,
    processed_count: Arc<AtomicUsize>,
}

impl NodeCreatedIndexHandler {
    fn new(projection: ContentIndexProjection, processed_count: Arc<AtomicUsize>) -> Self {
        Self {
            projection,
            processed_count,
        }
    }
}

#[async_trait]
impl EventHandler for NodeCreatedIndexHandler {
    fn name(&self) -> &'static str {
        "node_created_index_handler"
    }

    fn handles(&self, event: &DomainEvent) -> bool {
        matches!(event, DomainEvent::NodeCreated { .. })
    }

    async fn handle(&self, envelope: &EventEnvelope) -> HandlerResult {
        if let DomainEvent::NodeCreated { node_id, kind, .. } = &envelope.event {
            self.projection.upsert(*node_id, kind);
            self.processed_count.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_node_created_event_updates_index_projection() {
    let tenant_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();

    let bus = EventBus::new();
    let mut event_stream = bus.subscribe();

    let projection = ContentIndexProjection::default();
    let processed_count = Arc::new(AtomicUsize::new(0));

    let mut dispatcher = EventDispatcher::new(bus.clone());
    dispatcher.register(NodeCreatedIndexHandler::new(
        projection.clone(),
        Arc::clone(&processed_count),
    ));
    let running_dispatcher = dispatcher.start();

    bus.publish(
        tenant_id,
        None,
        DomainEvent::NodeCreated {
            node_id,
            kind: "post".to_string(),
            author_id: None,
        },
    )
    .expect("must publish NodeCreated event");

    let envelope = tokio::time::timeout(std::time::Duration::from_secs(1), event_stream.recv())
        .await
        .expect("must receive published event")
        .expect("event stream should stay open");

    assert!(matches!(
        envelope.event,
        DomainEvent::NodeCreated { node_id: event_node_id, .. } if event_node_id == node_id
    ));

    wait_until(|| processed_count.load(Ordering::Relaxed) == 1).await;

    assert_eq!(processed_count.load(Ordering::Relaxed), 1);
    assert_eq!(projection.get(node_id).as_deref(), Some("post"));
    assert_eq!(projection.len(), 1);

    running_dispatcher.stop();
}

#[tokio::test]
async fn test_node_created_event_repeat_is_idempotent_for_index_projection() {
    let tenant_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();

    let bus = EventBus::new();
    let projection = ContentIndexProjection::default();
    let processed_count = Arc::new(AtomicUsize::new(0));

    let mut dispatcher = EventDispatcher::new(bus.clone());
    dispatcher.register(NodeCreatedIndexHandler::new(
        projection.clone(),
        Arc::clone(&processed_count),
    ));
    let running_dispatcher = dispatcher.start();

    for _ in 0..2 {
        bus.publish(
            tenant_id,
            None,
            DomainEvent::NodeCreated {
                node_id,
                kind: "post".to_string(),
                author_id: None,
            },
        )
        .expect("NodeCreated publish must succeed");
    }

    wait_until(|| processed_count.load(Ordering::Relaxed) >= 2).await;

    assert_eq!(processed_count.load(Ordering::Relaxed), 2);
    assert_eq!(projection.get(node_id).as_deref(), Some("post"));
    assert_eq!(projection.len(), 1, "projection must stay deduplicated");

    running_dispatcher.stop();
}

async fn wait_until(condition: impl Fn() -> bool) {
    for _ in 0..40 {
        if condition() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    panic!("condition was not met within the expected time");
}
