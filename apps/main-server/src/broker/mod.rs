//! Event broker for server-sent events.

use rpc_protocol::StreamEvent;
use tokio::sync::broadcast;

/// Capacity of the broadcast channel.
const CHANNEL_CAPACITY: usize = 256;

/// Broadcasts stream events to all subscribers.
#[derive(Debug, Clone)]
pub struct EventBroker {
    sender: broadcast::Sender<StreamEvent>,
}

impl EventBroker {
    /// Creates a new event broker.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Publishes an event to all subscribers.
    pub fn publish(&self, event: StreamEvent) {
        // Ignore errors — it's fine if there are no subscribers.
        let _ = self.sender.send(event);
    }

    /// Returns a new receiver for the broadcast channel.
    pub fn subscribe(&self) -> broadcast::Receiver<StreamEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBroker {
    fn default() -> Self {
        Self::new()
    }
}
