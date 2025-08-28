# AWS Cost Notifier in Lambda

A Rust Lambda function that retrieves AWS costs for the current month and sends a formatted notification via Telegram.

## Features

- Fetches AWS cost data for the current month using AWS Cost Explorer API
- Groups costs by service
- Formats the data in a readable table
- Sends the formatted data to a Telegram chat
- Proper error handling throughout the application
- Well-documented code with rustdoc comments

## Installation

This project uses [cargo lambda](https://www.cargo-lambda.info/) to build the Lambda deployment package.

### Prerequisites

Install cargo lambda:

```bash
# MacOS
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda

# Other platforms
# See https://www.cargo-lambda.info/guide/installation.html
```

AWS credentials configured in your environment

### Building

Build the Lambda deployment package:

```bash
cargo lambda build --release --output-format zip --arm64
```

## Deployment

1. Create a Lambda function in AWS with the ARM64 architecture
2. Upload the generated zip file from the `target/lambda/aws_billing_notifier/` directory
3. Set the following environment variables in the Lambda configuration:
    - `TELEGRAM_TOKEN`: Your Telegram bot token
    - `CHAT_ID`: The Telegram chat ID to send messages to

## Configuration

The Lambda function requires two environment variables:

- `TELEGRAM_TOKEN`: The API token for your Telegram bot
- `CHAT_ID`: The chat ID where notifications should be sent

## Error Handling

The application uses proper error handling throughout:

- Custom error types for different categories of errors
- Detailed error messages
- No panics in production code
- Proper error propagation

## Development

### Project Structure

- `src/main.rs`: Main application logic and Lambda handler
- `src/aws.rs`: AWS service interactions
- `src/telegram.rs`: Telegram API interactions
