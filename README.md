# AWS Cost Notifier in Lambda

A simple rust lambda function to get the aws cost for the month and notify the user using telegram.

## Installation

I use [cargo lambda](https://www.cargo-lambda.info/) to build the binary file.
You need to install cargo lambda first.

```bash
# MacOS
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

Use cargo lambda to generate the binary file.

```bash
cargo lambda build --release --output-format zip --arm64
```

This program need two environment variables `TELEGRAM_TOKEN` and `CHAT_ID`.
