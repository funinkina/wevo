use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct WAMessage {
    pub key: WAKey,
    #[serde(rename = "messageTimestamp")]
    pub timestamp: i64,
    pub message: Option<MessageContent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WAKey {
    #[serde(rename = "remoteJid")]
    pub jid: String,
    #[serde(rename = "fromMe")]
    pub from_me: bool,
    pub id: String,
    pub participant: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageContent {
    pub conversation: Option<String>,
    #[serde(rename = "extendedTextMessage")]
    pub extended_text_message: Option<ExtendedTextMessage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExtendedTextMessage {
    pub text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WAContact {
    pub id: String,
    pub name: Option<String>,
    pub notify: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WAChat {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "unreadCount")]
    pub unread_count: Option<i32>,
    #[serde(rename = "conversationTimestamp")]
    pub conversation_timestamp: Option<u64>,
    pub archived: Option<bool>,
    pub pinned: Option<i64>,
    #[serde(rename = "muteEndTime")]
    pub mute_end_time: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessagingHistorySet {
    pub chats: Vec<WAChat>,
    pub contacts: Vec<WAContact>,
    pub messages: Vec<WAMessage>,
    #[serde(rename = "isLatest")]
    pub is_latest: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionUpdate {
    pub qr: Option<String>,
    pub connection: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    ConnectionUpdate(ConnectionUpdate),
    MessagesUpsert { messages: Vec<WAMessage> },
    ChatsSet { chats: Vec<WAChat> },
    ChatsUpdate(Vec<WAChat>),
    ContactsSet { contacts: Vec<WAContact> },
    MessagingHistorySet(MessagingHistorySet),
    Other(serde_json::Value),
}
