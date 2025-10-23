// Placeholder data for the application
use crate::config::ConfigManager;
use crate::models::{Chat, Config, Contact, Message, MessagesResponse};
use anyhow::Result;
use serde_json::json;

pub fn get_sample_contacts() -> Vec<Contact> {
    vec![
        Contact::new(
            "Alice Johnson".to_string(),
            "Hey! How are you doing?".to_string(),
            "10:23 AM".to_string(),
            "sample1@s.whatsapp.net".to_string(),
        ),
        Contact::new(
            "Bob Smith".to_string(),
            "Did you see the latest update?".to_string(),
            "Yesterday".to_string(),
            "sample2@s.whatsapp.net".to_string(),
        ),
        Contact::new(
            "Charlie Brown".to_string(),
            "Thanks for your help!".to_string(),
            "Monday".to_string(),
            "sample3@s.whatsapp.net".to_string(),
        ),
        Contact::new(
            "Diana Prince".to_string(),
            "Let's schedule a meeting".to_string(),
            "Sunday".to_string(),
            "sample4@s.whatsapp.net".to_string(),
        ),
        Contact::new(
            "Eve Wilson".to_string(),
            "Perfect! See you there.".to_string(),
            "Oct 20".to_string(),
            "sample5@s.whatsapp.net".to_string(),
        ),
    ]
}

pub fn get_sample_messages() -> Vec<Message> {
    vec![
        Message::new(
            "Hey! How are you doing?".to_string(),
            "10:20 AM".to_string(),
            false,
        ),
        Message::new(
            "I'm doing great! Thanks for asking.".to_string(),
            "10:21 AM".to_string(),
            true,
        ),
        Message::new(
            "That's wonderful to hear! I wanted to ask you about the project we discussed last week.".to_string(),
            "10:22 AM".to_string(),
            false,
        ),
        Message::new(
            "Sure! I've made some progress on it. What would you like to know?".to_string(),
            "10:23 AM".to_string(),
            true,
        ),
    ]
}

pub fn get_config() -> Config {
    // Try environment variables first, then fall back to keyring
    let url = std::env::var("WEVO_URL").unwrap_or_else(|_| ConfigManager::get_url());

    let api_key = std::env::var("WEVO_API_KEY").unwrap_or_else(|_| ConfigManager::get_api_key());

    Config::new(url, api_key)
}

pub fn fetch_chats() -> Result<Vec<Contact>> {
    let config = get_config();

    // Build the API endpoint
    let url = format!("{}/chat/findChats/main", config.url);

    // Make the API request
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).header("apikey", &config.api_key).send()?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({}): {}", status, error_text);
    }

    // Parse the response
    let chats: Vec<Chat> = response.json()?;

    // Convert chats to contacts
    let contacts = chats.iter().map(|chat| Contact::from_chat(chat)).collect();

    Ok(contacts)
}

pub fn fetch_chats_or_fallback() -> Vec<Contact> {
    match fetch_chats() {
        Ok(contacts) if !contacts.is_empty() => contacts,
        Ok(_) => {
            eprintln!("No chats found, using sample data");
            get_sample_contacts()
        }
        Err(e) => {
            eprintln!("Failed to fetch chats: {}. Using sample data", e);
            get_sample_contacts()
        }
    }
}

pub fn fetch_messages(remote_jid: &str) -> Result<Vec<Message>> {
    let config = get_config();

    // Build the API endpoint
    let url = format!("{}/chat/findMessages/main", config.url);

    // Build the request body
    let body = json!({
        "where": {
            "key": {
                "remoteJid": remote_jid
            }
        },
        "page": 1,
        "offset": 100
    });

    // Make the API request
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .header("apikey", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({}): {}", status, error_text);
    }

    // Parse the response
    let messages_response: MessagesResponse = response.json()?;

    // Convert API messages to UI messages and reverse the order
    // API returns newest first, but we want oldest first for display
    let mut messages: Vec<Message> = messages_response
        .messages
        .records
        .iter()
        .map(|msg| Message::from_api_message(msg))
        .collect();

    messages.reverse();

    Ok(messages)
}

pub fn fetch_messages_or_fallback(remote_jid: &str) -> Vec<Message> {
    match fetch_messages(remote_jid) {
        Ok(messages) if !messages.is_empty() => messages,
        Ok(_) => {
            eprintln!("No messages found, using sample data");
            get_sample_messages()
        }
        Err(e) => {
            eprintln!("Failed to fetch messages: {}. Using sample data", e);
            get_sample_messages()
        }
    }
}
