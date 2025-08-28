use aws_billing_notifier::escape_markdown;

#[test]
fn test_escape_markdown_basic_characters() {
    let input = "Hello World".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, "Hello World");
}

#[test]
fn test_escape_markdown_special_characters() {
    let input = "Test-message with.special characters!".to_string();
    let expected = "Test\\-message with\\.special characters\\!".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, expected);
}

#[test]
fn test_escape_markdown_parentheses_and_brackets() {
    let input = "Cost (AWS) [EC2] service".to_string();
    let expected = "Cost \\(AWS\\) \\[EC2\\] service".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, expected);
}

#[test]
fn test_escape_markdown_complex_billing_message() {
    let input = "Your AWS costs: EC2-Instance $45.67 (us-east-1)".to_string();
    let expected = "Your AWS costs: EC2\\-Instance $45\\.67 \\(us\\-east\\-1\\)".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, expected);
}

#[test]
fn test_escape_markdown_empty_string() {
    let input = "".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, "");
}

#[test]
fn test_escape_markdown_only_special_characters() {
    let input = ".-!()[]".to_string();
    let expected = "\\.\\-\\!\\(\\)\\[\\]".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, expected);
}

#[test]
fn test_escape_markdown_mixed_content() {
    let input = "Total: $123.45 - EC2 (compute) [active]!".to_string();
    let expected = "Total: $123\\.45 \\- EC2 \\(compute\\) \\[active\\]\\!".to_string();
    let result = escape_markdown(input);
    assert_eq!(result, expected);
}
