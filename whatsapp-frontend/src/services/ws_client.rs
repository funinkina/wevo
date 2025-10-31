use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use tungstenite::{Message as WsMessage, connect};
use url::Url;

#[derive(Debug, Clone)]
pub enum WhatsAppEvent {
    QrCode(String),
    Connected,
    Message {
        jid: String,
        sender: String,
        content: String,
        timestamp: i64,
        is_from_me: bool,
    },
    ContactsUpdate(Vec<crate::models::Contact>),
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

        println!("WebSocket message type: {:?}", parsed["type"].as_str());

        match parsed["type"].as_str() {
            Some("qr") => {
                if let Some(qr) = parsed["qr"].as_str() {
                    println!("Received QR code");
                    tx.send(WhatsAppEvent::QrCode(qr.to_string()))?;
                }
            }
            Some("connected") => {
                println!("Received connected event");
                tx.send(WhatsAppEvent::Connected)?;
            }
            Some("message") => {
                let msg = &parsed["message"];
                let key = &msg["key"];
                let jid = key["remoteJid"].as_str().unwrap_or("").to_string();
                let is_from_me = key["fromMe"].as_bool().unwrap_or(false);

                // Extract message content
                let content = if let Some(conversation) = msg["message"]["conversation"].as_str() {
                    conversation.to_string()
                } else if let Some(text) = msg["message"]["extendedTextMessage"]["text"].as_str() {
                    text.to_string()
                } else {
                    "[Media]".to_string()
                };

                let timestamp = msg["messageTimestamp"].as_i64().unwrap_or(0);
                let sender = if is_from_me {
                    "me".to_string()
                } else {
                    key["participant"].as_str().unwrap_or(&jid).to_string()
                };

                println!("Received message from: {}", jid);

                tx.send(WhatsAppEvent::Message {
                    jid,
                    sender,
                    content,
                    timestamp,
                    is_from_me,
                })?;
            }
            Some("contacts") => {
                println!("Received contacts event");
                if let Some(contacts_array) = parsed["contacts"].as_array() {
                    println!("Contacts array length: {}", contacts_array.len());

                    let contacts: Vec<crate::models::Contact> = contacts_array
                        .iter()
                        .filter_map(|c| {
                            let contact = crate::models::Contact {
                                jid: c["jid"].as_str()?.to_string(),
                                name: c["name"].as_str()?.to_string(),
                                last_message: None,
                                last_message_time: None,
                                unread_count: c["unreadCount"].as_i64().unwrap_or(0) as i32,
                                conversation_timestamp: c["conversationTimestamp"]
                                    .as_i64()
                                    .unwrap_or(0),
                                is_group: c["isGroup"].as_bool().unwrap_or(false),
                                archived: c["archived"].as_bool().unwrap_or(false),
                                pinned: c["pinned"].as_i64().unwrap_or(0),
                                mute_end_time: c["muteEndTime"].as_i64().unwrap_or(0),
                                profile_picture_url: c["profilePictureUrl"]
                                    .as_str()
                                    .map(|s| s.to_string()),
                            };
                            println!("  - Contact: {} ({})", contact.name, contact.jid);
                            Some(contact)
                        })
                        .collect();

                    println!("Parsed {} contacts successfully", contacts.len());
                    tx.send(WhatsAppEvent::ContactsUpdate(contacts))?;
                } else {
                    println!("Contacts field is not an array");
                }
            }
            Some(other) => {
                println!("Unknown message type: {}", other);
            }
            None => {
                println!("Message without type field");
            }
        }

        Ok(())
    }
}
