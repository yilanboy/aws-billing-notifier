use thiserror::Error;

pub mod aws;
pub mod telegram;

use aws::AwsError;
use telegram::TelegramError;

/// Application-specific error types
#[derive(Error, Debug)]
pub enum AppError {
    /// AWS-related errors
    #[error("AWS error: {0}")]
    Aws(#[from] AwsError),

    /// Telegram-related errors
    #[error("Telegram error: {0}")]
    Telegram(#[from] TelegramError),

    /// Environment variable errors
    #[error("Environment error: {0}")]
    Environment(String),

    /// Data processing errors
    #[error("Data processing error: {0}")]
    DataProcessing(String),

    /// Parse errors
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Helper function for amount formatting
///
/// Parses a string amount, rounds to 2 decimal places, and formats it
pub fn format_amount(amount: &str) -> Result<String, AppError> {
    let float_value: f64 = amount
        .parse()
        .map_err(|e| AppError::Parse(format!("Invalid float string: {}", e)))?;
    let rounded_value = (float_value * 100.0).round() / 100.0;

    Ok(format!("{:.2}", rounded_value))
}

/// Escapes special characters for Telegram MarkdownV2 format
pub fn escape_markdown(text: String) -> String {
    // Escape characters that have special meaning in MarkdownV2
    text.replace("-", "\\-")
        .replace(".", "\\.")
        .replace("!", "\\!")
        .replace("(", "\\(")
        .replace(")", "\\)")
        .replace("[", "\\[")
        .replace("]", "\\]")
}
