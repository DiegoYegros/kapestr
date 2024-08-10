use crate::win::MainWindow;
use nostr_sdk::prelude::*;
use nostr_sdk::Client;
use tokio::sync::mpsc;

pub struct PostFetchService {
    client: Client,
    sender: mpsc::Sender<Event>,
}

impl PostFetchService {
    pub fn new(client: Client) -> (Self, mpsc::Receiver<Event>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { client, sender }, receiver)
    }

    pub async fn start(&self) {
        self.client.add_relay("wss://relay.damus.io").await.unwrap();
        self.client.connect().await;
        let subscription = self
            .client
            .subscribe(vec![Filter::new().kind(Kind::TextNote).limit(20)], None)
            .await;
        let mut events = self.client.notifications();
        while let Ok(notification) = events.recv().await {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if let Err(_) = self.sender.send(*event).await {
                    println!("Failed to send event, receiver might have been dropped");
                    break;
                }
            }
        }
        self.client.unsubscribe(subscription).await;
    }
    pub fn add_post_to_ui(window: &MainWindow, event: &Event) {
        println!("Adding new post to UI!");
        //let _post = Post::new_from_event(event);
        window.add_post(&event.author().to_string(), "", event.content());
    }
}
