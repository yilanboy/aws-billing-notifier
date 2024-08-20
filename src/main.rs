use aws_sdk_costexplorer::types::{DateInterval, Granularity, ResultByTime};
use chrono::Datelike;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::HashMap;

async fn get_aws_account_id() -> String {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_sts::Client::new(&config);

    let response = client.get_caller_identity().send().await;

    let response = match response {
        Ok(response) => response,
        Err(error) =>
            panic!("There was a problem getting the caller_identity: {:?}", error),
    };

    if let Some(account_id) = response.account() {
        account_id.to_string()
    } else {
        panic!("There was a problem getting the caller_identity");
    }
}

async fn get_aws_cost_in_this_month() -> Vec<ResultByTime> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_costexplorer::Client::new(&config);

    let now = chrono::Utc::now().naive_utc();
    let start_of_month = now.with_day(1).unwrap();

    let start_date = start_of_month.format("%Y-%m-%d").to_string();
    let end_date = now.format("%Y-%m-%d").to_string();

    let date_interval = DateInterval::builder()
        .start(start_date)
        .end(end_date)
        .build()
        .unwrap();

    let response = client
        .get_cost_and_usage()
        .time_period(date_interval)
        .metrics("UnblendedCost")
        .granularity(Granularity::Monthly)
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(error) =>
            panic!("There was a error getting the cost and usage: {:?}", error),
    };

    let mut result_by_time: Vec<ResultByTime> = Vec::new();

    for result in response.results_by_time() {
        result_by_time.push(result.clone());
    }

    result_by_time
}


async fn send_telegram_message(
    telegram_token: &str,
    chat_id: &str,
    message: &str,
) -> Result<(), reqwest::Error> {
    let url: String = format!("https://api.telegram.org/bot{telegram_token}/sendMessage");

    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("chat_id", chat_id);
    params.insert("parse_mode", "MarkdownV2");
    params.insert("text", message);

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

async fn handler(telegram_token: &str, chat_id: &str, _event: LambdaEvent<Value>) -> Result<(), Error> {
    let account_id = get_aws_account_id().await;

    let results = get_aws_cost_in_this_month().await;

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

            send_telegram_message(telegram_token, chat_id, &message)
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
