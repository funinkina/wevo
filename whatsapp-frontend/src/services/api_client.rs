use reqwest::blocking::Client;
use serde_json::Value;

pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub fn send_message(&self, jid: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&format!("{}/send", self.base_url))
            .json(&serde_json::json!({
                "jid": jid,
                "text": text
            }))
            .send()?;

        let result: Value = response.json()?;

        if result["ok"].as_bool().unwrap_or(false) {
            Ok(())
        } else {
            Err(format!("Failed to send message: {:?}", result["error"]).into())
        }
    }

    pub fn get_contacts(&self) -> Result<Vec<crate::models::Contact>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/contacts", self.base_url))
            .send()?;

        let result: Value = response.json()?;

        // Backend returns { success: bool, contacts: [...], count: number }
        let contacts_array = result["contacts"]
            .as_array()
            .ok_or("Expected 'contacts' array in response")?;

        Ok(contacts_array
            .iter()
            .filter_map(|c| {
                Some(crate::models::Contact {
                    jid: c["jid"].as_str()?.to_string(),
                    name: c["name"].as_str()?.to_string(),
                    last_message: None,
                    last_message_time: None,
                    unread_count: c["unreadCount"].as_i64().unwrap_or(0) as i32,
                    conversation_timestamp: c["conversationTimestamp"].as_i64().unwrap_or(0),
                    is_group: c["isGroup"].as_bool().unwrap_or(false),
                    archived: c["archived"].as_bool().unwrap_or(false),
                    pinned: c["pinned"].as_i64().unwrap_or(0),
                    mute_end_time: c["muteEndTime"].as_i64().unwrap_or(0),
                    profile_picture_url: None, // Will be fetched separately if needed
                })
            })
            .collect())
    }

    pub fn get_profile_picture(
        &self,
        jid: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Based on Baileys documentation: profilePictureUrl(jid, 'image')
        let response = self
            .client
            .get(&format!("{}/profile-picture", self.base_url))
            .query(&[("jid", jid)])
            .send()?;

        if response.status().is_success() {
            let result: Value = response.json()?;
            Ok(result["url"].as_str().map(|s| s.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn get_messages(
        &self,
        jid: &str,
    ) -> Result<Vec<crate::models::Message>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/messages/{}", self.base_url, jid))
            .send()?;

        let messages: Vec<Value> = response.json()?;

        Ok(messages
            .iter()
            .filter_map(|m| {
                Some(crate::models::Message {
                    id: None,
                    message_id: m["message_id"].as_str()?.to_string(),
                    jid: m["jid"].as_str()?.to_string(),
                    sender: m["sender"].as_str()?.to_string(),
                    content: m["content"].as_str()?.to_string(),
                    timestamp: m["timestamp"].as_i64()?,
                    is_from_me: m["is_from_me"].as_bool()?,
                    message_type: m["message_type"].as_str().unwrap_or("text").to_string(),
                    raw_data: m["raw_data"].as_str().map(|s| s.to_string()),
                    quoted_message_id: m["quoted_message_id"].as_str().map(|s| s.to_string()),
                    media_url: m["media_url"].as_str().map(|s| s.to_string()),
                    caption: m["caption"].as_str().map(|s| s.to_string()),
                })
            })
            .collect())
    }

    pub fn request_qr(&self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&format!("{}/auth/request-qr", self.base_url))
            .send()?;

        let result: Value = response.json()?;

        if result["success"].as_bool().unwrap_or(false) {
            println!(
                "QR request successful: {}",
                result["message"].as_str().unwrap_or("")
            );
            Ok(())
        } else {
            Err(format!("Failed to request QR: {:?}", result["message"]).into())
        }
    }

    pub fn get_auth_status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&format!("{}/auth/status", self.base_url))
            .send()?;

        Ok(response.json()?)
    }
}
