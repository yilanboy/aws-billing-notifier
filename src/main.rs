use aws_billing_notifier::{aws, escape_markdown, format_amount, telegram, AppError};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

/// Service cost information
#[derive(Clone)]
struct ServiceCostInfo {
    service: String,
    amount: f64,
    formatted_amount: String,
    unit: String,
}

// AppError already implements std::error::Error via thiserror,
// so lambda_runtime::Error can automatically convert from it

/// Fetches and sends AWS service costs via Telegram
async fn send_service_costs(
    aws: &aws::BillExplorer,
    message: &telegram::Message,
) -> Result<(), AppError> {
    // Get AWS account ID
    let account_id = aws.get_account_id().await?;
    let mut total_amount: f64 = 0.0;
    let mut body = String::new();

    body.push_str("```text\n");
    body.push_str(&format!(
        "Your AWS Account {account_id} costs in this month\n\n"
    ));

    // Get cost data from AWS
    let results = aws.get_cost_by_service().await?;
    let mut service_costs: Vec<ServiceCostInfo> = Vec::new();

    // Process cost data
    for result in results {
        for group in result.groups() {
            let service = group
                .keys()
                .first()
                .ok_or_else(|| AppError::DataProcessing("No service name found".to_string()))?;

            let mut service_name = service.to_string();

            // if the service is over than 30 characters, replace it with "..."
            if service_name.len() > 30 {
                service_name.truncate(30);
                service_name.push_str("...");
            }

            let metrics = group
                .metrics()
                .ok_or_else(|| AppError::DataProcessing("No metrics found".to_string()))?;

            if let Some(value) = metrics.get("UnblendedCost") {
                let amount = value
                    .amount()
                    .ok_or_else(|| AppError::DataProcessing("Error getting amount".to_string()))?;

                let formatted_amount = format_amount(amount)?;
                let unit = value.unit().unwrap_or("USD");

                // Skip zero-cost services
                if formatted_amount != "0.00" {
                    let amount_value = formatted_amount
                        .parse::<f64>()
                        .map_err(|e| AppError::Parse(format!("Error parsing amount: {}", e)))?;

                    total_amount += amount_value;

                    service_costs.push(ServiceCostInfo {
                        service: service_name,
                        amount: amount_value,
                        formatted_amount,
                        unit: unit.to_string(),
                    });
                }
            }
        }
    }

    // Handle empty results
    if service_costs.is_empty() {
        body.push_str("No costs found for this month.\n");
        body.push_str("```");
        message.send(escape_markdown(body)).await?;

        return Ok(());
    }

    // Find the service with the longest display text for formatting
    let mut max_text_length = 0;
    for cost in &service_costs {
        let text_length = format!("{} {} {}", cost.service, cost.formatted_amount, cost.unit).len();
        if text_length > max_text_length {
            max_text_length = text_length;
        }
    }

    // Sort by cost (descending)
    service_costs.sort_by(|a, b| {
        b.amount
            .partial_cmp(&a.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Format and add each service cost to the message
    for cost in service_costs {
        let service_text = format!("{} {} {}", cost.service, cost.formatted_amount, cost.unit);
        let padding = max_text_length - service_text.len();

        // Create a line with dashes between the service name and amount
        let service_line = format!(
            "{} {} {}\n",
            cost.service,
            "-".repeat(padding + 2),
            format!("{} {}", cost.formatted_amount, cost.unit)
        );

        body.push_str(&service_line);
    }

    body.push_str(&format!("\nTotal: {:.2} USD", total_amount));
    body.push_str("```");

    // Send the message via Telegram
    message.send(escape_markdown(body)).await?;

    Ok(())
}

/// Lambda handler function
async fn handler(
    telegram_token: &str,
    chat_id: &str,
    _event: LambdaEvent<Value>,
) -> Result<(), Error> {
    let aws = aws::BillExplorer::new();
    let telegram = telegram::Message::new(telegram_token.to_string(), chat_id.to_string());

    send_service_costs(&aws, &telegram).await?;

    Ok(())
}

/// Main function - entry point for the Lambda
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Get environment variables
    let telegram_token: String = std::env::var("TELEGRAM_TOKEN").map_err(|_| {
        AppError::Environment(
            "TELEGRAM_TOKEN must be set in Lambda environment variables".to_string(),
        )
    })?;

    let chat_id: String = std::env::var("CHAT_ID").map_err(|_| {
        AppError::Environment("CHAT_ID must be set in Lambda environment variables".to_string())
    })?;

    // Run the Lambda handler
    lambda_runtime::run(service_fn(|event: LambdaEvent<Value>| async {
        handler(&telegram_token, &chat_id, event).await
    }))
    .await
}
