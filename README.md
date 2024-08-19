# AWS Cost Notifier

A simple project to get the AWS cost for the month and notify the user using Telegram!

## Installation

Use cargo to install the dependencies.

```bash
cargo build
```

Run the program with environment variables `TELEGRAM_TOKEN` and `CHAT_ID`.

```bash
export TELEGRAM_TOKEN="..."
# which chat you want to send the message
export CHAT_ID="..."

cargo run
```
