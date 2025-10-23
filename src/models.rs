// Data models for the chat application
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Contact {
    pub name: String,
    pub last_message: String,
    pub time: String,
    pub avatar_color: String,
    pub remote_jid: String,
}

// API Response structures
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chat {
    pub id: Option<String>,
    #[serde(rename = "remoteJid")]
    pub remote_jid: String,
    #[serde(rename = "pushName")]
    pub push_name: Option<String>,
    #[serde(rename = "profilePicUrl")]
    pub profile_pic_url: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "windowStart")]
    pub window_start: String,
    #[serde(rename = "windowExpires")]
    pub window_expires: String,
    #[serde(rename = "windowActive")]
    pub window_active: bool,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<LastMessage>,
    #[serde(rename = "unreadCount")]
    pub unread_count: i32,
    #[serde(rename = "isSaved")]
    pub is_saved: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LastMessage {
    pub id: String,
    pub key: MessageKey,
    #[serde(rename = "pushName")]
    pub push_name: Option<String>,
    pub participant: Option<String>,
    #[serde(rename = "messageType")]
    pub message_type: String,
    pub message: MessageContent,
    #[serde(rename = "contextInfo")]
    pub context_info: Option<serde_json::Value>,
    pub source: String,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: i64,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageKey {
    pub id: String,
    #[serde(rename = "fromMe")]
    pub from_me: bool,
    #[serde(rename = "remoteJid")]
    pub remote_jid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageContent {
    pub conversation: Option<String>,
    #[serde(rename = "messageContextInfo")]
    pub message_context_info: Option<serde_json::Value>,
}

impl Contact {
    pub fn new(name: String, last_message: String, time: String, remote_jid: String) -> Self {
        let avatar_color = Self::generate_color(&name);
        Self {
            name,
            last_message,
            time,
            avatar_color,
            remote_jid,
        }
    }

    pub fn from_chat(chat: &Chat) -> Self {
        let name = chat
            .push_name
            .clone()
            .or_else(|| {
                // Extract phone number or group name from remoteJid
                chat.remote_jid.split('@').next().map(|s| s.to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let last_message = chat
            .last_message
            .as_ref()
            .and_then(|msg| msg.message.conversation.clone())
            .unwrap_or_else(|| "No messages".to_string());

        let time = Self::format_time(&chat.updated_at);

        Self::new(name, last_message, time, chat.remote_jid.clone())
    }

    fn format_time(timestamp: &str) -> String {
        // Simple time formatting - you can enhance this
        // For now, just extract date/time from ISO format
        if let Some(date_part) = timestamp.split('T').next() {
            date_part.to_string()
        } else {
            timestamp.to_string()
        }
    }

    pub fn initials(&self) -> String {
        self.name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }

    fn generate_color(name: &str) -> String {
        // Generate a consistent color based on the name
        let colors = vec![
            "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6", "#1abc9c", "#e67e22", "#34495e",
            "#16a085", "#c0392b",
        ];
        let index = name.chars().map(|c| c as usize).sum::<usize>() % colors.len();
        colors[index].to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
    pub time: String,
    pub is_own: bool,
}

impl Message {
    pub fn new(content: String, time: String, is_own: bool) -> Self {
        Self {
            content,
            time,
            is_own,
        }
    }

    pub fn from_api_message(msg: &ApiMessage) -> Self {
        let content = msg
            .message
            .conversation
            .clone()
            .unwrap_or_else(|| "[No text content]".to_string());

        let time = Self::format_timestamp(msg.message_timestamp);
        let is_own = msg.key.from_me;

        Self::new(content, time, is_own)
    }

    fn format_timestamp(timestamp: i64) -> String {
        use chrono::{Local, TimeZone};
        let dt = Local.timestamp_opt(timestamp, 0).unwrap();
        dt.format("%I:%M %p").to_string()
    }
}

// API Response for messages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessagesResponse {
    pub messages: MessagesData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessagesData {
    pub total: i32,
    pub pages: i32,
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    pub records: Vec<ApiMessage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApiMessage {
    pub id: String,
    pub key: MessageKey,
    #[serde(rename = "pushName")]
    pub push_name: Option<String>,
    #[serde(rename = "messageType")]
    pub message_type: String,
    pub message: MessageContent,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: i64,
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    pub source: String,
    #[serde(rename = "contextInfo")]
    pub context_info: Option<serde_json::Value>,
    #[serde(rename = "MessageUpdate")]
    pub message_update: Vec<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub url: String,
    pub api_key: String,
}

impl Config {
    pub fn new(url: String, api_key: String) -> Self {
        Self { url, api_key }
    }
}
