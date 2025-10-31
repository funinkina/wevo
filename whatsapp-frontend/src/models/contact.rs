use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: String,
    pub name: String,
    #[serde(default)]
    pub last_message: Option<String>,
    #[serde(default)]
    pub last_message_time: Option<i64>,
    #[serde(rename = "unreadCount", default)]
    pub unread_count: i32,
    #[serde(rename = "conversationTimestamp", default)]
    pub conversation_timestamp: i64,
    #[serde(rename = "isGroup", default)]
    pub is_group: bool,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub pinned: i64,
    #[serde(rename = "muteEndTime", default)]
    pub mute_end_time: i64,
    #[serde(default)]
    pub profile_picture_url: Option<String>,
}

impl Contact {
    pub fn display_name(&self) -> String {
        self.name.clone()
    }

    pub fn is_muted(&self) -> bool {
        if self.mute_end_time == 0 {
            return false;
        }
        // Check if mute is still active (comparing with current timestamp in milliseconds)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        self.mute_end_time > now
    }
}
