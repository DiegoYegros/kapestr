use std::collections::HashMap;
use std::sync::Arc;

use crate::win::MainWindow;
use nostr_sdk::{Filter, Event, Kind, RelayPoolNotification, PublicKey, Client};
use serde_json;
use tokio::sync::{mpsc, Mutex};
use chrono::{DateTime, Utc};

pub struct PostFetchService {
    client: Arc<Client>,
    sender: mpsc::Sender<(Event, String)>,
    metadata_map: Arc<Mutex<HashMap<PublicKey, String>>>,
}

impl PostFetchService {
    pub fn new(client: Client) -> (Self, mpsc::Receiver<(Event, String)>) {
        let (sender, receiver) = mpsc::channel(100);
        let metadata_map = Arc::new(Mutex::new(HashMap::new()));
        (Self { client: Arc::new(client), sender, metadata_map }, receiver)
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("PostFetchService: Starting");
        self.client.add_relays(vec!["wss://relay.snort.social", "wss://relay.nostr.band"]).await?;
        self.client.connect().await;
        println!("PostFetchService: Connected to relays");
        let subscription = self
            .client
            .subscribe(vec![
                Filter::new()
                    .kinds(vec![Kind::TextNote, Kind::Metadata])
            ], None)
            .await;
        println!("PostFetchService: Subscribed");
        let mut events = self.client.notifications();
        loop {
            match events.recv().await {
                Ok(notification) => {
                    println!("PostFetchService: Received notification");
                    match notification {
                        RelayPoolNotification::Event { event, .. } => {
                            if let Err(e) = self.process_event(*event).await {
                                eprintln!("Error processing event: {:?}", e);
                            }
                        },
                        RelayPoolNotification::Message { message, .. } => {
                            println!("Received message: {:?}", message);
                        },
                        RelayPoolNotification::Shutdown => {
                            println!("PostFetchService: Received shutdown notification");
                            break;
                        },
                        _ => println!("Received other notification: {:?}", notification),
                    }
                },
                Err(e) => {
                    eprintln!("Error receiving notification: {:?}", e);
                    // Instead of breaking, we'll continue the loop
                    continue;
                }
            }
        }
        self.client.unsubscribe(subscription).await;
        println!("PostFetchService: Ended");
        Ok(())
    }

    async fn process_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        match event.kind {
            Kind::Metadata => {
                if let Some(display_name) = Self::handle_metadata_event(&event) {
                    self.metadata_map.lock().await.insert(event.pubkey, display_name);
                }
            },
            Kind::TextNote => {
                if let Some(display_name) = self.get_or_fetch_display_name(event.pubkey).await {
                    self.sender.send((event, display_name)).await.map_err(|_| "Failed to send event")?;
                }
            },
            _ => {}
        }
        Ok(())
    }

    async fn get_or_fetch_display_name(&self, pubkey: PublicKey) -> Option<String> {
        let metadata_map = self.metadata_map.lock().await;
        if let Some(name) = metadata_map.get(&pubkey) {
            Some(name.clone())
        } else {
            drop(metadata_map);
            self.fetch_metadata(pubkey).await;
            self.metadata_map.lock().await.get(&pubkey).cloned()
        }
    }

    async fn fetch_metadata(&self, pubkey: PublicKey) {
        let filter = Filter::new().authors(vec![pubkey]).kind(Kind::Metadata).limit(1);
        let subscription = self.client.subscribe(vec![filter], None).await;
        let mut events = self.client.notifications();
        
        while let Ok(RelayPoolNotification::Event { event, .. }) = events.recv().await {
            if event.kind == Kind::Metadata && event.pubkey == pubkey {
                if let Some(display_name) = Self::handle_metadata_event(&event) {
                    self.metadata_map.lock().await.insert(pubkey, display_name);
                    break;
                }
            }
        }
        
        self.client.unsubscribe(subscription).await;
    }

    fn handle_metadata_event(event: &Event) -> Option<String> {
        if event.kind == Kind::Metadata {
            serde_json::from_str::<serde_json::Value>(&event.content)
                .ok()
                .and_then(|v| v.get("display_name").or_else(|| v.get("name")).and_then(|n| n.as_str()).map(String::from))
        } else {
            None
        }
    }

    pub async fn add_post_to_ui(window: &MainWindow, event: &Event, display_name: &str) {
        let timestamp = DateTime::<Utc>::from_timestamp(event.created_at.as_u64().try_into().unwrap(), 0)
            .expect("Invalid timestamp");
        let formatted_time = timestamp.format("%H:%M").to_string();
        window.add_post(display_name, &formatted_time, &event.content);
    }
}