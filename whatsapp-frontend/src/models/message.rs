use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub message_id: String, // WhatsApp message ID
    pub jid: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
    pub is_from_me: bool,
    pub message_type: String,     // text, image, video, reaction, etc.
    pub raw_data: Option<String>, // Store full JSON for rich data
    pub quoted_message_id: Option<String>, // For replies
    pub media_url: Option<String>, // For media messages
    pub caption: Option<String>,  // For media captions
}
