use reqwest::StatusCode;
use std::collections::HashMap;

pub struct Telegram {
    telegram_token: String,
    chat_id: String,
}

impl Telegram {
    pub fn new(telegram_token: String, chat_id: String) -> Self {
        Self {
            telegram_token,
            chat_id,
        }
    }

    pub async fn send(&self, message: String) -> Result<(), reqwest::Error> {
        let url: String = format!(
            "https://api.telegram.org/bot{0}/sendMessage",
            self.telegram_token.as_str()
        );

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("chat_id", self.chat_id.as_str());
        params.insert("parse_mode", "MarkdownV2");
        params.insert("text", message.as_str());

        let url = reqwest::Url::parse_with_params(&url, &params);

        let url = match url {
            Ok(url) => url,
            Err(error) => panic!("There was a error parsing the url: {:?}", error),
        };

        let body = reqwest::get(url).await?;

        if body.status() == StatusCode::OK {
            println!("Telegram message sent successfully!");
        } else {
            panic!(
                "Telegram message returned unexpected status: {:?}",
                body.status()
            );
        }

        Ok(())
    }
}
