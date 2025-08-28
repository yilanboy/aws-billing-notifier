use reqwest::StatusCode;
use std::collections::HashMap;
use thiserror::Error;

/// Error types for Telegram operations
#[derive(Error, Debug)]
pub enum TelegramError {
    /// Error when parsing URL
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Error when sending HTTP request
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Error when receiving non-OK status code
    #[error("Telegram API returned unexpected status: {0}")]
    ApiError(StatusCode),
}

/// Telegram client for sending messages
pub struct Message {
    telegram_token: String,
    chat_id: String,
    base_url: String,
}

impl Message {
    /// Creates a new Telegram client
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - The Telegram bot token
    /// * `chat_id` - The chat ID to send messages to
    pub fn new(telegram_token: String, chat_id: String) -> Self {
        Self {
            telegram_token,
            chat_id,
            base_url: "https://api.telegram.org".to_string(),
        }
    }

    /// Creates a new Telegram client with custom base URL (for testing)
    ///
    /// # Arguments
    ///
    /// * `telegram_token` - The Telegram bot token
    /// * `chat_id` - The chat ID to send messages to
    /// * `base_url` - Custom base URL for the Telegram API
    pub fn new_with_base_url(telegram_token: String, chat_id: String, base_url: String) -> Self {
        Self {
            telegram_token,
            chat_id,
            base_url,
        }
    }

    /// Sends a message via Telegram
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send, formatted according to MarkdownV2 syntax
    ///
    /// # Returns
    ///
    /// * `Result<(), TelegramError>` - Ok, if successful, Err otherwise
    pub async fn send(&self, message: String) -> Result<(), TelegramError> {
        let url: String = format!("{}/bot{}/sendMessage", self.base_url, self.telegram_token);

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("chat_id", self.chat_id.as_str());
        params.insert("parse_mode", "MarkdownV2");
        params.insert("text", message.as_str());

        let url = reqwest::Url::parse_with_params(&url, &params)?;
        let response = reqwest::get(url).await?;

        if response.status() == StatusCode::OK {
            println!("Telegram message sent successfully!");
            Ok(())
        } else {
            Err(TelegramError::ApiError(response.status()))
        }
    }
}
