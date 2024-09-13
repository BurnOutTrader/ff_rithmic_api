use std::sync::Arc;
use dashmap::DashMap;
use futures_util::SinkExt;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use prost::{Message as RithmicMessage};

pub type SubscriberName = String;
pub struct Broadcaster {
    subscribers: Arc<DashMap<SubscriberName, Arc<Sender<Vec<u8>>>>>,
}

impl Broadcaster {

    pub fn new() -> Self {
        Self {
            subscribers: Default::default(),
        }
    }
    /// Subscribe to the messages sent by this broadcaster
    pub fn subscribe(&self, subscriber_name: SubscriberName, buffer_size: usize) -> Receiver<Vec<u8>> {
        let (sender, mut receiver) = channel(buffer_size);
        self.subscribers.insert(subscriber_name, Arc::new(sender));
        receiver
    }

    /// Unsubscribe from further messages
    pub fn unsubscribe(&self, subscriber_name: &SubscriberName) {
        self.subscribers.remove(subscriber_name);
    }

    /// fwd messages to subscribers
    async fn broadcast(&mut self, message: Vec<u8>) {
        let subscribers = self.subscribers.clone();
        tokio::task::spawn(async move {
            for subscriber in subscribers.iter() {
                let message_clone = message.clone();
                let subscriber = subscriber.clone();
                tokio::task::spawn(async move {
                    let _ = subscriber.send(message_clone).await;
                });
            }
        });
    }
}
