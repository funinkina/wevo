// Configuration management with persistent storage
use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "wevo-chat";
const URL_KEY: &str = "api_url";
const API_KEY_KEY: &str = "api_key";

pub struct ConfigManager;

impl ConfigManager {
    /// Get the API URL from keyring or return default
    pub fn get_url() -> String {
        Self::get_from_keyring(URL_KEY).unwrap_or_else(|_| "http://localhost:8080".to_string())
    }

    /// Get the API key from keyring or return empty string
    pub fn get_api_key() -> String {
        Self::get_from_keyring(API_KEY_KEY).unwrap_or_else(|_| String::new())
    }

    /// Save the API URL to keyring
    pub fn set_url(url: &str) -> Result<()> {
        Self::save_to_keyring(URL_KEY, url)
    }

    /// Save the API key to keyring
    pub fn set_api_key(api_key: &str) -> Result<()> {
        Self::save_to_keyring(API_KEY_KEY, api_key)
    }

    /// Delete the API URL from keyring
    pub fn delete_url() -> Result<()> {
        Self::delete_from_keyring(URL_KEY)
    }

    /// Delete the API key from keyring
    pub fn delete_api_key() -> Result<()> {
        Self::delete_from_keyring(API_KEY_KEY)
    }

    // Private helper methods
    fn get_from_keyring(key: &str) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, key).context("Failed to create keyring entry")?;
        entry
            .get_password()
            .context("Failed to get password from keyring")
    }

    fn save_to_keyring(key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key).context("Failed to create keyring entry")?;
        entry
            .set_password(value)
            .context("Failed to save to keyring")
    }

    fn delete_from_keyring(key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key).context("Failed to create keyring entry")?;
        entry
            .delete_password()
            .context("Failed to delete from keyring")
    }
}
