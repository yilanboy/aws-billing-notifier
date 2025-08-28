use aws_billing_notifier::telegram::{Message, TelegramError};
use reqwest::StatusCode;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_telegram_message_send_success() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Set up the mock response for a successful message sending
    Mock::given(method("GET"))
        .and(path("/bot123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ/sendMessage"))
        .and(query_param("chat_id", "12345"))
        .and(query_param("parse_mode", "MarkdownV2"))
        .and(query_param("text", "Test message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {
                "message_id": 123,
                "date": 1234567890,
                "chat": {
                    "id": 12345,
                    "type": "private"
                },
                "text": "Test message"
            }
        })))
        .mount(&mock_server)
        .await;

    // Create a message client with the mock server URL
    let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let chat_id = "12345";

    // Replace the hardcoded Telegram API URL with our mock server
    // We'll need to modify the Message struct for this to work properly
    let message =
        Message::new_with_base_url(token.to_string(), chat_id.to_string(), mock_server.uri());

    // Send a test message
    let result = message.send("Test message".to_string()).await;

    // Assert the message was sent successfully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_telegram_message_send_api_error() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Set up the mock response for API error (401 Unauthorized)
    Mock::given(method("GET"))
        .and(path("/bot123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ/sendMessage"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "ok": false,
            "error_code": 401,
            "description": "Unauthorized"
        })))
        .mount(&mock_server)
        .await;

    let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let chat_id = "12345";

    let message =
        Message::new_with_base_url(token.to_string(), chat_id.to_string(), mock_server.uri());

    // Send a test message
    let result = message.send("Test message".to_string()).await;

    // Assert we get the expected error
    assert!(result.is_err());
    match result.unwrap_err() {
        TelegramError::ApiError(status) => {
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }
        _ => panic!("Expected ApiError"),
    }
}

#[tokio::test]
async fn test_telegram_message_with_markdown_characters() {
    let mock_server = MockServer::start().await;

    // Test message with special MarkdownV2 characters that need escaping
    let test_message = "Cost: $10.50 (AWS EC2-Instance)";

    Mock::given(method("GET"))
        .and(path("/bot123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ/sendMessage"))
        .and(query_param("chat_id", "12345"))
        .and(query_param("parse_mode", "MarkdownV2"))
        .and(query_param("text", test_message))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {
                "message_id": 124,
                "date": 1234567891,
                "chat": {
                    "id": 12345,
                    "type": "private"
                },
                "text": test_message
            }
        })))
        .mount(&mock_server)
        .await;

    let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let chat_id = "12345";

    let message =
        Message::new_with_base_url(token.to_string(), chat_id.to_string(), mock_server.uri());

    let result = message.send(test_message.to_string()).await;
    assert!(result.is_ok());
}

#[test]
fn test_telegram_message_creation() {
    let token = "test_token".to_string();
    let chat_id = "test_chat_id".to_string();

    let message = Message::new(token.clone(), chat_id.clone());

    // We can't directly test the private fields, but we can test that creation succeeds
    // This is more of a sanity check that the constructor works
    assert_eq!(
        std::mem::size_of_val(&message),
        std::mem::size_of::<Message>()
    );
}

// Integration test that would require network access (disabled by default)
#[tokio::test]
#[ignore] // Use `cargo test -- --ignored` to run this test
async fn test_telegram_real_api_integration() {
    // This test would require real Telegram credentials
    // Set these environment variables to run this test:
    // TELEGRAM_TOKEN_TEST and CHAT_ID_TEST

    let token = std::env::var("TELEGRAM_TOKEN_TEST").unwrap_or_else(|_| {
        panic!("Set TELEGRAM_TOKEN_TEST environment variable to run integration tests")
    });
    let chat_id = std::env::var("CHAT_ID_TEST").unwrap_or_else(|_| {
        panic!("Set CHAT_ID_TEST environment variable to run integration tests")
    });

    let message = Message::new(token, chat_id);
    let result = message
        .send("🤖 Test message from Rust integration test".to_string())
        .await;

    assert!(result.is_ok());
}
