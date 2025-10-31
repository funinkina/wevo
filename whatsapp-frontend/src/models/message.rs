use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub jid: String,
    pub sender: String,
    pub content: String,
    pub timestamp: i64,
    pub is_from_me: bool,
}
