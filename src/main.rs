use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::collections::HashMap;

mod aws;
mod telegram;

// Helper function for amount formatting
fn format_amount(amount: String) -> Result<String, Error> {
    let float_value: f64 = amount
        .parse()
        .map_err(|e| Error::from(format!("Invalid float string: {}", e)))?;
    let rounded_value = (float_value * 100.0).round() / 100.0;

    Ok(format!("{:.2}", rounded_value))
}

fn escape_markdown(text: String) -> String {
    text.replace("-", "\\-").replace(".", "\\.")
}

async fn send_service_costs(aws: &aws::Aws, telegram: &telegram::Telegram) -> Result<(), Error> {
    let account_id = aws.get_account_id().await;
    let mut total_amount: f64 = 0.0;
    let mut message = String::new();

    message.push_str("```text\n");

    message.push_str(&format!(
        "Your AWS Account {account_id} costs in this month\n\n"
    ));

    let results = aws.get_cost_by_service().await;

    let mut service_cost_info_list: Vec<HashMap<String, String>> = Vec::new();

    for result in results {
        for group in result.groups() {
            let service = group
                .keys()
                .first()
                .ok_or_else(|| Error::from("No service name found"))?;

            let metrics = group
                .metrics()
                .ok_or_else(|| Error::from("No metrics found"))?;

            if let Some(value) = metrics.get("UnblendedCost") {
                let amount = value
                    .amount()
                    .ok_or_else(|| Error::from("Error getting amount"))?;
                let formatted_amount = format_amount(amount.to_string())?;
                let unit = value.unit().unwrap();

                if formatted_amount.as_str() != "0.00" {
                    total_amount += formatted_amount.parse::<f64>().unwrap();

                    let mut service_cost_info = HashMap::new();

                    service_cost_info.insert("service".to_string(), service.clone());
                    service_cost_info.insert("formatted_amount".to_string(), formatted_amount);
                    service_cost_info.insert("unit".to_string(), unit.to_string());

                    service_cost_info_list.push(service_cost_info);
                }
            }
        }
    }

    service_cost_info_list.sort_by(|a, b| {
        let service_a = a.get("service").unwrap();
        let service_b = b.get("service").unwrap();

        service_a.len().cmp(&service_b.len())
    });

    let last_element = service_cost_info_list.last().unwrap();
    let last_element_service = last_element.get("service").unwrap();
    let last_element_formatted_amount = last_element.get("formatted_amount").unwrap();
    let last_element_unit = last_element.get("unit").unwrap();

    let last_service_cost_info_text = String::from(format!(
        "{last_element_service} ---- {last_element_formatted_amount} {last_element_unit}\n"
    ));

    let last_service_cost_info_text_length = last_service_cost_info_text.len();

    for service_cost_info in service_cost_info_list {
        let service = service_cost_info.get("service").unwrap();
        let formatted_amount = service_cost_info.get("formatted_amount").unwrap();
        let unit = service_cost_info.get("unit").unwrap();

        let service_cost_info_text = String::from(format!("{service} {formatted_amount} {unit}\n"));

        let service_cost_info_text_length = service_cost_info_text.len();

        let char_count_difference =
            last_service_cost_info_text_length - service_cost_info_text_length;

        let line = "-".repeat(char_count_difference);

        let service_cost_info_text =
            String::from(format!("{service} {line} {formatted_amount} {unit}\n"));

        message.push_str(&service_cost_info_text);
    }

    message.push_str(&format!("\nTotal: {:.2}", total_amount));

    message.push_str("```");

    telegram.send(escape_markdown(message)).await?;

    Ok(())
}

async fn handler(
    telegram_token: &str,
    chat_id: &str,
    _event: LambdaEvent<Value>,
) -> Result<(), Error> {
    let aws = aws::Aws::new();
    let telegram = telegram::Telegram::new(telegram_token.to_string(), chat_id.to_string());

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
