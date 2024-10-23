use chrono::Datelike;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

mod telegram;
mod aws;

async fn handler(
    telegram_token: &str,
    chat_id: &str,
    _event: LambdaEvent<Value>,
) -> Result<(), Error> {
    let aws = aws::Aws::new();

    let account_id = aws.get_account_id().await;

    let results = aws.get_account_cost_in_this_month().await;

    for result in results {
        if let Some(total) = result.total() {
            let value = total.get("UnblendedCost");
            let value = match value {
                Some(value) => value,
                None => panic!("There was a error getting unblended cost."),
            };

            let amount = value.amount();
            let amount = match amount {
                Some(amount) => {
                    let float_value: f64 = amount.parse().expect("Invalid float string");
                    let rounded_value = (float_value * 100.0).round() / 100.0;
                    let rounded_str = format!("{:.2}", rounded_value);
                    rounded_str.replace(".", "\\.")
                }
                None => panic!("There was a error getting the total amount"),
            };

            let unit = value.unit().unwrap_or_else(|| "USD");

            let message = format!(r#"
Your AWS Account __{account_id}__
The cost in this month is: __{amount}__ {unit}
            "#);

            let telegram = telegram::Telegram::new(
                telegram_token.to_string(),
                chat_id.to_string(),
                message.to_string(),
            );

            telegram.send_message()
                .await
                .expect("There was a error sending telegram message.");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let telegram_token: String = std::env::var("TELEGRAM_TOKEN")
        .expect("A TELEGRAM_TOKEN must be set in this app's Lambda environment variables.");

    let chat_id: String = std::env::var("CHAT_ID")
        .expect("A CHAT_ID must be set in this app's Lambda environment variables.");

    lambda_runtime::run(service_fn(|event: LambdaEvent<Value>| async {
        handler(&telegram_token, &chat_id, event).await
    })).await
}
