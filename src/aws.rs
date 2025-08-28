use aws_sdk_costexplorer::types::{
    DateInterval, Granularity, GroupDefinition, GroupDefinitionType, ResultByTime,
};
use chrono::Datelike;
use thiserror::Error;

/// Error types for AWS operations
#[derive(Error, Debug)]
pub enum AwsError {
    /// Error when getting caller identity
    #[error("Failed to get caller identity: {0}")]
    CallerIdentityError(String),

    /// Error when a building date interval
    #[error("Failed to build date interval: {0}")]
    DateIntervalError(String),

    /// Error when getting cost and usage
    #[error("Failed to get cost and usage: {0}")]
    CostExplorerError(String),

    /// Error when account ID is missing
    #[error("Account ID is missing from response")]
    MissingAccountId,

    /// Parse errors
    #[error("Parse error: {0}")]
    Parse(String),
}

/// AWS client for interacting with AWS services
pub struct BillExplorer {}

impl BillExplorer {
    /// Creates a new AWS client
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the AWS account ID
    ///
    /// # Returns
    ///
    /// * `Result<String, AwsError>` - The account ID if successful, error otherwise
    pub async fn get_account_id(&self) -> Result<String, AwsError> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_sts::Client::new(&config);

        let response = client
            .get_caller_identity()
            .send()
            .await
            .map_err(|e| AwsError::CallerIdentityError(e.to_string()))?;

        response
            .account()
            .map(|id| id.to_string())
            .ok_or(AwsError::MissingAccountId)
    }

    /// Gets the cost breakdown by service for the current month
    ///
    /// # Returns
    ///
    /// * `Result<Vec<ResultByTime>, AwsError>` - Cost data if successful, error otherwise
    pub async fn get_cost_by_service(&self) -> Result<Vec<ResultByTime>, AwsError> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_costexplorer::Client::new(&config);

        let now = chrono::Utc::now().naive_utc();
        let start_of_month = now.with_day(1).ok_or_else(|| {
            AwsError::DateIntervalError("Invalid day for start of month".to_string())
        })?;

        let start_date = start_of_month.format("%Y-%m-%d").to_string();
        let end_date = now.format("%Y-%m-%d").to_string();

        let date_interval = DateInterval::builder()
            .start(start_date)
            .end(end_date)
            .build()
            .map_err(|e| AwsError::DateIntervalError(e.to_string()))?;

        let group_definition = GroupDefinition::builder()
            .r#type(GroupDefinitionType::Dimension)
            .key("SERVICE")
            .build();

        let response = client
            .get_cost_and_usage()
            .time_period(date_interval)
            .granularity(Granularity::Monthly)
            .metrics("UnblendedCost")
            .group_by(group_definition)
            .send()
            .await
            .map_err(|e| AwsError::CostExplorerError(e.to_string()))?;

        let mut result_by_time: Vec<ResultByTime> = Vec::new();

        for result in response.results_by_time() {
            result_by_time.push(result.clone());
        }

        Ok(result_by_time)
    }
}
