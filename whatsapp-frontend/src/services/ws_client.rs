use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use tungstenite::{Message as WsMessage, connect};
use url::Url;

use super::events::{self, ConnectionUpdate};

#[derive(Debug, Clone)]
pub enum WhatsAppEvent {
    QrCode(String),
    Connected,
    Message(events::WAMessage),
    Contact(events::WAContact),
    Chat(events::WAChat),
}

pub struct WebSocketClient {
    tx: mpsc::Sender<WhatsAppEvent>,
}

impl WebSocketClient {
    pub fn new(url: &str) -> (Self, mpsc::Receiver<WhatsAppEvent>) {
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        let url = url.to_string();

        thread::spawn(move || {
            loop {
                match connect(Url::parse(&url).unwrap()) {
                    Ok((mut socket, _)) => {
                        println!("WebSocket connected");

                        loop {
                            match socket.read() {
                                Ok(msg) => {
                                    if let WsMessage::Text(text) = msg {
                                        if let Err(e) = Self::handle_message(&text, &tx_clone) {
                                            eprintln!("Error handling message: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to WebSocket: {}", e);
                    }
                }

                // Reconnect after 5 seconds
                thread::sleep(std::time::Duration::from_secs(5));
                println!("Attempting to reconnect...");
            }
        });

        (Self { tx }, rx)
    }

    fn handle_message(
        text: &str,
        tx: &mpsc::Sender<WhatsAppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let parsed: Value = serde_json::from_str(text)?;
        let event_type = parsed["type"].as_str();
        let payload = &parsed["payload"];

        println!("WebSocket event type: {:?}", event_type);

        match event_type {
            Some("connection.update") => {
                let update: ConnectionUpdate = serde_json::from_value(payload.clone())?;
                if let Some(qr) = update.qr {
                    tx.send(WhatsAppEvent::QrCode(qr))?;
                }
                if let Some(conn_status) = update.connection {
                    if conn_status == "open" {
                        tx.send(WhatsAppEvent::Connected)?;
                    }
                }
            }
            Some("messages.upsert") => {
                if let Ok(data) = serde_json::from_value::<events::EventPayload>(payload.clone()) {
                    if let events::EventPayload::MessagesUpsert { messages } = data {
                        for msg in messages {
                            tx.send(WhatsAppEvent::Message(msg))?;
                        }
                    }
                }
            }
            Some("chats.set") => {
                if let Ok(data) = serde_json::from_value::<events::EventPayload>(payload.clone()) {
                    if let events::EventPayload::ChatsSet { chats } = data {
                        for chat in chats {
                            tx.send(WhatsAppEvent::Chat(chat))?;
                        }
                    }
                }
            }
            Some("chats.update") => {
                if let Ok(data) = serde_json::from_value::<events::EventPayload>(payload.clone()) {
                    if let events::EventPayload::ChatsUpdate(chats) = data {
                        for chat in chats {
                            tx.send(WhatsAppEvent::Chat(chat))?;
                        }
                    }
                }
            }
            Some("contacts.set") => {
                if let Ok(data) = serde_json::from_value::<events::EventPayload>(payload.clone()) {
                    if let events::EventPayload::ContactsSet { contacts } = data {
                        for contact in contacts {
                            tx.send(WhatsAppEvent::Contact(contact))?;
                        }
                    }
                }
            }
            Some("messaging-history.set") => {
                if let Ok(data) = serde_json::from_value::<events::EventPayload>(payload.clone()) {
                    if let events::EventPayload::MessagingHistorySet(history) = data {
                        println!(
                            "📚 Received messaging history: {} chats, {} contacts, {} messages",
                            history.chats.len(),
                            history.contacts.len(),
                            history.messages.len()
                        );

                        // Send all chats
                        for chat in history.chats {
                            tx.send(WhatsAppEvent::Chat(chat))?;
                        }

                        // Send all contacts
                        for contact in history.contacts {
                            tx.send(WhatsAppEvent::Contact(contact))?;
                        }

                        // Send all messages
                        for msg in history.messages {
                            tx.send(WhatsAppEvent::Message(msg))?;
                        }
                    }
                }
            }
            Some(other) => {
                println!("Unhandled event type: {}", other);
            }
            None => {
                println!("Message without type field");
            }
        }

        Ok(())
    }
}
