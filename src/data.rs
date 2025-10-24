// Data fetching from Evolution API
use crate::config::ConfigManager;
use crate::models::{Chat, Config, Contact, Instance, Message, MessagesResponse};
use anyhow::Result;
use serde_json::json;

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

pub fn fetch_instance() -> Result<Instance> {
    let config = get_config();

    // Build the API endpoint
    let url = format!("{}/instance/fetchInstances?instanceName=main", config.url);

    // Make the API request
    let client = reqwest::blocking::Client::new();
    let response = client.get(&url).header("apikey", &config.api_key).send()?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("API error ({}): {}", status, error_text);
    }

    // Parse the response (it's an array, so we take the first element)
    let instances: Vec<Instance> = response.json()?;

    instances
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No instance found"))
}

pub fn fetch_instance_or_none() -> Option<String> {
    match fetch_instance() {
        Ok(instance) => {
            println!("Fetched instance profile: {}", instance.profile_name);
            instance.profile_pic_url
        }
        Err(e) => {
            eprintln!("Failed to fetch instance: {}", e);
            None
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

pub fn send_message(remote_jid: &str, text: &str) -> Result<()> {
    let config = get_config();

    // Build the API endpoint
    let url = format!("{}/message/sendText/main", config.url);

    // Build the request body
    let body = json!({
        "number": remote_jid,
        "text": text
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

    Ok(())
}
