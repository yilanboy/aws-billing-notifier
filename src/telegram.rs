use std::collections::HashMap;
use reqwest::StatusCode;

pub struct Telegram {
    telegram_token: String,
    chat_id: String,
    message: String,
}

impl Telegram {
    pub fn new(telegram_token: String, chat_id: String, message: String) -> Self {
        Self {
            telegram_token,
            chat_id,
            message,
        }
    }

    pub async fn send_message(&self) -> Result<(), reqwest::Error> {
        let url: String = format!("https://api.telegram.org/bot{0}/sendMessage", self.telegram_token);

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("chat_id", &*self.chat_id);
        params.insert("parse_mode", "MarkdownV2");
        params.insert("text", &*self.message);

        let url = reqwest::Url::parse_with_params(&url, &params);

        let url = match url {
            Ok(url) => url,
            Err(error) => panic!("There was a error parsing the url: {:?}", error),
        };

        let body = reqwest::get(url).await?;

        if body.status() == StatusCode::OK {
            println!("Telegram message sent successfully!");
        } else {
            panic!("Telegram message returned unexpected status: {:?}", body.status());
        }

        Ok(())
    }
}