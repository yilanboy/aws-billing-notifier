use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

mod aws;
mod telegram;

const UNBLENDED_COST_KEY: &str = "UnblendedCost";
const DEFAULT_CURRENCY: &str = "USD";

// Helper function for amount formatting
fn format_amount(amount: &str) -> Result<String, Error> {
    let float_value: f64 = amount
        .parse()
        .map_err(|e| Error::from(format!("Invalid float string: {}", e)))?;
    let rounded_value = (float_value * 100.0).round() / 100.0;

    Ok(format!("{:.2}", rounded_value).replace(".", "\\."))
}

async fn send_total_cost(aws: &aws::Aws, telegram: &telegram::Telegram) -> Result<(), Error> {
    let account_id = aws.get_account_id().await;
    let results = aws.get_account_cost_in_this_month().await;

    for result in results {
        if let Some(total) = result.total() {
            let value = total
                .get(UNBLENDED_COST_KEY)
                .ok_or_else(|| Error::from("Error getting unblended cost"))?;

            let amount = value
                .amount()
                .ok_or_else(|| Error::from("Error getting total amount"))?;
            let formatted_amount = format_amount(&amount)?;
            let unit = value.unit().unwrap_or(DEFAULT_CURRENCY);

            let message = format!(
                "Your AWS Account __{account_id}__\nThe cost in this month is: __{formatted_amount}__ {unit}"
            );

            telegram.send(message).await?;
        }
    }

    Ok(())
}

async fn send_service_costs(aws: &aws::Aws, telegram: &telegram::Telegram) -> Result<(), Error> {
    let results = aws.get_cost_by_service().await;

    for result in results {
        let mut message = String::new();

        for group in result.groups() {
            let service = group
                .keys()
                .first()
                .ok_or_else(|| Error::from("No service name found"))?;

            let metrics = group
                .metrics()
                .ok_or_else(|| Error::from("No metrics found"))?;

            if let Some(value) = metrics.get(UNBLENDED_COST_KEY) {
                let amount = value
                    .amount()
                    .ok_or_else(|| Error::from("Error getting amount"))?;
                let formatted_amount = format_amount(&amount)?;
                let unit = value.unit().unwrap_or(DEFAULT_CURRENCY);

                if formatted_amount.as_str() != "0\\.00" {
                    message.push_str(&format!("{service}: __{formatted_amount}__ {unit}\n"));
                }
            }
        }

        telegram.send(message).await?;
    }

    Ok(())
}

async fn handler(
    telegram_token: &str,
    chat_id: &str,
    _event: LambdaEvent<Value>,
) -> Result<(), Error> {
    let aws = aws::Aws::new();
    let telegram = telegram::Telegram::new(telegram_token.to_string(), chat_id.to_string());

    send_total_cost(&aws, &telegram).await?;
    send_service_costs(&aws, &telegram).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let telegram_token: String = std::env::var("TELEGRAM_TOKEN")
        .map_err(|_| Error::from("TELEGRAM_TOKEN must be set in Lambda environment variables"))?;

    let chat_id: String = std::env::var("CHAT_ID")
        .map_err(|_| Error::from("CHAT_ID must be set in Lambda environment variables"))?;

    lambda_runtime::run(service_fn(|event: LambdaEvent<Value>| async {
        handler(&telegram_token, &chat_id, event).await
    }))
    .await
}
