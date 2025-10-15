//! Analytics API implementation for OpenRouter activity data

use crate::client::ClientConfig;
use crate::error::Error;
use crate::types::analytics::{ActivityRequest, ActivityResponse};
use reqwest::Client;

/// Analytics API for retrieving activity data
#[derive(Debug, Clone)]
pub struct AnalyticsApi {
    client: Client,
    config: ClientConfig,
}

impl AnalyticsApi {
    /// Create a new Analytics API instance
    pub(crate) fn new(client: Client, config: ClientConfig) -> Self {
        Self { client, config }
    }

    /// Get activity data for the last 30 days
    ///
    /// Returns daily user activity data grouped by model endpoint for the last 30 (completed) UTC days.
    ///
    /// If ingesting on a schedule, it is recommended to wait for ~30 minutes after the UTC boundary
    /// to request the previous day, because events are aggregated by request start time, and some
    /// reasoning models may take a few minutes to complete.
    ///
    /// Note that a provisioning key is required to access this endpoint, to ensure that your historic
    /// usage is not accessible to just anyone in your org with an inference API key.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request parameters for filtering the activity data
    ///
    /// # Returns
    ///
    /// Returns a `ActivityResponse` containing the activity data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails
    /// - The response cannot be parsed
    /// - Authentication fails (requires provisioning key)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    /// use openrouter_api::types::analytics::ActivityRequest;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OpenRouterClient::from_env()?;
    /// let analytics = client.analytics()?;
    ///
    /// // Get all activity data for the last 30 days
    /// let activity = analytics.get_activity(None).await?;
    ///
    /// // Get activity data for a specific date
    /// let request = ActivityRequest::new().date("2024-01-01");
    /// let activity = analytics.get_activity(Some(request)).await?;
    ///
    /// println!("Total usage: ${:.2}", activity.total_usage());
    /// println!("Total requests: {}", activity.total_requests());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_activity(
        &self,
        request: Option<ActivityRequest>,
    ) -> Result<ActivityResponse, Error> {
        let mut url = self
            .config
            .base_url
            .join("activity")
            .map_err(|e| Error::ConfigError(format!("Invalid URL: {}", e)))?;

        // Add query parameters if provided
        if let Some(req) = request {
            let mut query_params = Vec::new();

            if let Some(date) = req.date {
                query_params.push(("date".to_string(), date));
            }

            if !query_params.is_empty() {
                let query_string = query_params
                    .into_iter()
                    .map(|(key, value)| format!("{}={}", key, urlencoding::encode(&value)))
                    .collect::<Vec<_>>()
                    .join("&");
                url.set_query(Some(&query_string));
            }
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(Error::HttpError)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: format!(
                    "Analytics API request failed with status {}: {}",
                    status, error_text
                ),
                metadata: None,
            });
        }

        let activity_response: ActivityResponse =
            response.json().await.map_err(Error::HttpError)?;

        Ok(activity_response)
    }

    /// Get activity data for all available days (no filtering)
    ///
    /// This is a convenience method that calls `get_activity` with no request parameters.
    ///
    /// # Returns
    ///
    /// Returns a `ActivityResponse` containing all available activity data.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or response cannot be parsed.
    pub async fn get_all_activity(&self) -> Result<ActivityResponse, Error> {
        self.get_activity(None).await
    }

    /// Get activity data for a specific date
    ///
    /// This is a convenience method that creates an `ActivityRequest` with the specified date.
    ///
    /// # Arguments
    ///
    /// * `date` - Date in YYYY-MM-DD format (must be within the last 30 days)
    ///
    /// # Returns
    ///
    /// Returns a `ActivityResponse` containing activity data for the specified date.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The date is invalid
    /// - The API request fails
    /// - The response cannot be parsed
    pub async fn get_activity_for_date(
        &self,
        date: impl Into<String>,
    ) -> Result<ActivityResponse, Error> {
        let request = ActivityRequest::new().date(date);
        self.get_activity(Some(request)).await
    }

    /// Validate date format for activity requests
    ///
    /// # Arguments
    ///
    /// * `date` - Date string to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the date format is valid, `Err(Error)` otherwise.
    pub fn validate_date_format(&self, date: &str) -> Result<(), Error> {
        // Check if date matches YYYY-MM-DD format
        if date.len() != 10 {
            return Err(Error::SchemaValidationError(
                "Date must be in YYYY-MM-DD format".to_string(),
            ));
        }

        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            return Err(Error::SchemaValidationError(
                "Date must be in YYYY-MM-DD format".to_string(),
            ));
        }

        // Validate year (4 digits)
        if parts[0].len() != 4 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::SchemaValidationError(
                "Year must be 4 digits".to_string(),
            ));
        }

        // Validate month (2 digits, 01-12)
        if parts[1].len() != 2 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::SchemaValidationError(
                "Month must be 2 digits (01-12)".to_string(),
            ));
        }
        if let Ok(month) = parts[1].parse::<u32>() {
            if !(1..=12).contains(&month) {
                return Err(Error::SchemaValidationError(
                    "Month must be between 01 and 12".to_string(),
                ));
            }
        } else {
            return Err(Error::SchemaValidationError("Invalid month".to_string()));
        }

        // Validate day (2 digits, 01-31)
        if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::SchemaValidationError(
                "Day must be 2 digits (01-31)".to_string(),
            ));
        }
        if let Ok(day) = parts[2].parse::<u32>() {
            if !(1..=31).contains(&day) {
                return Err(Error::SchemaValidationError(
                    "Day must be between 01 and 31".to_string(),
                ));
            }
        } else {
            return Err(Error::SchemaValidationError("Invalid day".to_string()));
        }

        Ok(())
    }

    /// Check if a date is within the last 30 days (the retention period for activity data)
    ///
    /// # Arguments
    ///
    /// * `date` - Date string in YYYY-MM-DD format
    ///
    /// # Returns
    ///
    /// Returns `true` if the date is within the last 30 days, `false` otherwise.
    pub fn is_date_within_retention(&self, date: &str) -> bool {
        // Parse the date
        let parsed_date = match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_) => return false,
        };

        // Get current UTC date
        let now = chrono::Utc::now().date_naive();

        // Calculate the difference in days
        let days_diff = (now - parsed_date).num_days();

        // Activity data is available for the last 30 completed UTC days
        (0..=30).contains(&days_diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_format() {
        let api = create_test_api();

        // Valid dates
        assert!(api.validate_date_format("2024-01-01").is_ok());
        assert!(api.validate_date_format("2024-12-31").is_ok());

        // Invalid dates
        assert!(api.validate_date_format("24-01-01").is_err()); // Year not 4 digits
        assert!(api.validate_date_format("2024-1-01").is_err()); // Month not 2 digits
        assert!(api.validate_date_format("2024-01-1").is_err()); // Day not 2 digits
        assert!(api.validate_date_format("2024-13-01").is_err()); // Invalid month
        assert!(api.validate_date_format("2024-01-32").is_err()); // Invalid day
        assert!(api.validate_date_format("2024/01/01").is_err()); // Wrong separator
        assert!(api.validate_date_format("invalid").is_err()); // Completely invalid
    }

    #[test]
    fn test_is_date_within_retention() {
        let api = create_test_api();

        // Test with mock dates (this test will depend on current date)
        let today = chrono::Utc::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();

        // Today should be within retention
        assert!(api.is_date_within_retention(&today_str));

        // 30 days ago should be within retention
        let thirty_days_ago = today - chrono::Duration::days(30);
        let thirty_days_ago_str = thirty_days_ago.format("%Y-%m-%d").to_string();
        assert!(api.is_date_within_retention(&thirty_days_ago_str));

        // 31 days ago should be outside retention
        let thirty_one_days_ago = today - chrono::Duration::days(31);
        let thirty_one_days_ago_str = thirty_one_days_ago.format("%Y-%m-%d").to_string();
        assert!(!api.is_date_within_retention(&thirty_one_days_ago_str));

        // Future date should be outside retention
        let tomorrow = today + chrono::Duration::days(1);
        let tomorrow_str = tomorrow.format("%Y-%m-%d").to_string();
        assert!(!api.is_date_within_retention(&tomorrow_str));

        // Invalid date should be outside retention
        assert!(!api.is_date_within_retention("invalid-date"));
    }

    fn create_test_api() -> AnalyticsApi {
        let client = Client::new();
        let config = ClientConfig {
            api_key: None,
            base_url: "https://openrouter.ai/api/v1/".parse().unwrap(),
            http_referer: None,
            site_title: None,
            user_id: None,
            timeout: std::time::Duration::from_secs(30),
            retry_config: crate::client::RetryConfig::default(),
        };
        AnalyticsApi::new(client, config)
    }
}
