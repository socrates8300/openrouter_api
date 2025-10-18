//! Retry utilities for HTTP requests with exponential backoff

use crate::client::RetryConfig;
use crate::error::{Error, Result};
use fastrand::Rng;
use reqwest::{
    header::{HeaderMap, RETRY_AFTER},
    RequestBuilder, Response,
};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::{sleep, timeout};

/// Operation names for retry logging and error context
pub mod operations {
    pub const TEXT_COMPLETION: &str = "text_completion";
    pub const WEB_SEARCH: &str = "web_search";
    pub const LIST_MODELS: &str = "list_models";
    pub const GET_BALANCE: &str = "get_balance";
    pub const GET_ACTIVITY: &str = "get_activity";
    pub const GET_PROVIDERS: &str = "get_providers";
    pub const GET_GENERATION: &str = "get_generation";
    pub const STRUCTURED_GENERATE: &str = "structured_generate";
    pub const CHAT_COMPLETION: &str = "chat_completion";
}

/// Executes an HTTP request with retry logic using a closure for request building
///
/// This version recreates the request for each retry attempt, avoiding request builder consumption.
/// Retries are performed on selected HTTP status codes **and** transient network errors/timeouts.
pub async fn execute_with_retry_builder<F>(
    config: &RetryConfig,
    operation_name: &str,
    mut request_builder: F,
) -> Result<Response>
where
    F: FnMut() -> RequestBuilder,
{
    let mut retry_count = 0usize;
    let mut backoff_ms = config.initial_backoff_ms;
    let mut rng = Rng::new();
    let start_time = Instant::now();

    loop {
        // Remaining time against the overall cap.
        let remaining = config.total_timeout.saturating_sub(start_time.elapsed());
        if remaining.is_zero() {
            // FIX: Use TimeoutError (more semantically accurate than ConfigError here).
            return Err(Error::TimeoutError(format!(
                "Retry timeout exceeded for {}: {}ms limit",
                operation_name,
                config.total_timeout.as_millis()
            )));
        }

        // Rebuild and send the request, bounded by the remaining overall time.
        let send_fut = request_builder().send();
        match timeout(remaining, send_fut).await {
            // Outer timeout (this single attempt took too long)
            Err(_) => {
                if retry_count < config.max_retries as usize {
                    retry_count += 1;

                    // Wait with jitter, but never sleep past the remaining overall time.
                    let sleep_ms =
                        jittered_backoff_ms(backoff_ms, config.max_backoff_ms, &mut rng, remaining);
                    eprintln!(
                        "Retrying {} request due to attempt timeout ({}/{}) in {} ms",
                        operation_name, retry_count, config.max_retries, sleep_ms
                    );
                    sleep(Duration::from_millis(sleep_ms)).await;

                    // Exponential step for next time.
                    backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
                    continue;
                } else {
                    return Err(Error::TimeoutError(format!(
                        "Request timeout for {} after {:?}",
                        operation_name, config.total_timeout
                    )));
                }
            }

            // The send completed; now check whether it succeeded or failed with a network error.
            Ok(Err(e)) => {
                // NEW: Treat transient network failures as retryable (connect/timeouts).
                if is_retryable_reqwest_error(&e) && retry_count < config.max_retries as usize {
                    retry_count += 1;

                    let sleep_ms =
                        jittered_backoff_ms(backoff_ms, config.max_backoff_ms, &mut rng, remaining);
                    eprintln!(
                        "Retrying {} due to transient error ({}/{}) in {} ms: {}",
                        operation_name, retry_count, config.max_retries, sleep_ms, e
                    );
                    sleep(Duration::from_millis(sleep_ms)).await;

                    backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
                    continue;
                }

                // Non-retryable or out of retries.
                return Err(e.into());
            }

            Ok(Ok(response)) => {
                let status = response.status();
                let status_code = status.as_u16();

                // HTTP status-based retries.
                if config.retry_on_status_codes.contains(&status_code)
                    && retry_count < config.max_retries as usize
                {
                    retry_count += 1;

                    // Parse Retry-After (delta-seconds or HTTP date), capped to 1 hour.
                    let retry_after_ms = parse_retry_after_ms(response.headers());

                    // Consume body to free the connection.
                    let _ = response.bytes().await;

                    // Decide sleep time: prefer Retry-After, else exponential.
                    let base_ms = retry_after_ms.unwrap_or(backoff_ms);
                    let sleep_ms =
                        jittered_backoff_ms(base_ms, config.max_backoff_ms, &mut rng, remaining);

                    eprintln!(
                        "Retrying {} request ({}/{}) after {} ms (status: {}, retry_after_ms={:?}, base_backoff_ms={})",
                        operation_name, retry_count, config.max_retries, sleep_ms, status_code, retry_after_ms, backoff_ms
                    );

                    sleep(Duration::from_millis(sleep_ms)).await;

                    // Only grow exponential backoff if we didn't use Retry-After.
                    if retry_after_ms.is_none() {
                        backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
                    }
                    continue;
                }

                // Either success, or a non-retryable status (return as-is).
                return Ok(response);
            }
        }
    }
}

/// FIX: Robust Retry-After parsing with single assignment and 1h cap.
/// Supports both `delta-seconds` and RFC 1123 HTTP-date.
fn parse_retry_after_ms(headers: &HeaderMap) -> Option<u64> {
    const MAX_SECONDS: u64 = 3600; // 1 hour cap

    let value = headers.get(RETRY_AFTER)?;
    let s = value.to_str().ok()?.trim();

    // 1) delta-seconds
    if let Ok(seconds) = s.parse::<u64>() {
        return Some(seconds.min(MAX_SECONDS) * 1000);
    }

    // 2) HTTP date
    if let Ok(http_date) = httpdate::parse_http_date(s) {
        let now = SystemTime::now();
        let dur = match http_date.duration_since(now) {
            Ok(d) => d,
            Err(_) => Duration::ZERO, // Past date → treat as 0
        };
        return Some(dur.min(Duration::from_secs(MAX_SECONDS)).as_millis() as u64);
    }

    None
}

/// NEW: Recognize transient reqwest errors worth retrying.
fn is_retryable_reqwest_error(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect()
    // You could add .is_request() if you want to retry malformed responses, but it's usually not transient.
}

/// NEW: Jittered backoff capped by both config.max_backoff_ms and remaining overall time.
/// Also safety-caps any single sleep to ≤5 minutes.
fn jittered_backoff_ms(
    base_ms: u64,
    max_backoff_ms: u64,
    rng: &mut Rng,
    remaining_overall: Duration,
) -> u64 {
    // Safety cap: never exceed 5 minutes for any single sleep.
    let safe_base = base_ms.min(300_000);

    // Respect configured max_backoff.
    let capped = safe_base.min(max_backoff_ms);

    // Jitter in [0.75, 1.25]
    let jitter = rng.f64() * 0.5 + 0.75;
    let jittered = (capped as f64 * jitter) as u64;

    // Never sleep longer than remaining overall time (minus a tiny buffer).
    let remaining_ms = remaining_overall.as_millis().saturating_sub(25) as u64;
    jittered.min(remaining_ms)
}

/// NEW: Next exponential backoff step with both config and safety cap.
fn next_backoff(current_ms: u64, max_backoff_ms: u64) -> u64 {
    let doubled = current_ms.saturating_mul(2);
    doubled.min(max_backoff_ms).min(300_000) // ≤ 5 minutes
}

/// Handles HTTP response with consistent error parsing
pub async fn handle_response_text(response: Response, operation_name: &str) -> Result<String> {
    let status = response.status();
    let status_code = status.as_u16();
    let body = response.text().await?;

    if !status.is_success() {
        // FIX: Avoid `?` inside `Err(...)` which could bubble an internal parse failure
        // instead of returning a best-effort API error. Fall back gracefully.
        let err =
            Error::from_response_text(status_code, &body).unwrap_or_else(|_| Error::ApiError {
                code: status_code,
                message: format!(
                    "HTTP {} for {} with non-JSON body: {}",
                    status_code,
                    operation_name,
                    elide(&body, 2_000)
                ),
                metadata: None,
            });
        return Err(err);
    }

    if body.trim().is_empty() {
        return Err(Error::ApiError {
            code: status_code,
            message: format!("Empty response body for {}", operation_name),
            metadata: None,
        });
    }

    Ok(body)
}

/// Handles HTTP response with JSON deserialization
pub async fn handle_response_json<T: serde::de::DeserializeOwned>(
    response: Response,
    operation_name: &str,
) -> Result<T> {
    let status = response.status();
    let status_code = status.as_u16();
    let body = response.text().await?;

    if !status.is_success() {
        // FIX: Same `?` issue; use safe fallback.
        let err =
            Error::from_response_text(status_code, &body).unwrap_or_else(|_| Error::ApiError {
                code: status_code,
                message: format!(
                    "HTTP {} for {} with body: {}",
                    status_code,
                    operation_name,
                    elide(&body, 2_000)
                ),
                metadata: None,
            });
        return Err(err);
    }

    if body.trim().is_empty() {
        return Err(Error::ApiError {
            code: status_code,
            message: format!("Empty response body for {}", operation_name),
            metadata: None,
        });
    }

    // Decode JSON with a safe error message.
    serde_json::from_str::<T>(&body).map_err(|e| Error::ApiError {
        // NOTE: status may be 2xx; we keep it for visibility but consider using 0 or a dedicated DecodeError in your type.
        code: status_code,
        message: crate::utils::security::create_safe_error_message(
            &format!(
                "Failed to decode JSON response for {}: {}. Body (elided) was: {}",
                operation_name,
                e,
                elide(&body, 2_000)
            ),
            &format!("{} JSON parsing error", operation_name),
        ),
        metadata: None,
    })
}

/// NEW: Small helper to keep logs/errors short but useful.
fn elide(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}… ({} bytes total)", &s[..max], s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::RetryConfig;
    use reqwest::header::HeaderValue;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 500);
        assert_eq!(config.max_backoff_ms, 10000);
        assert!(config.retry_on_status_codes.contains(&429));
        assert!(config.retry_on_status_codes.contains(&500));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig::default();
        let mut backoff_ms = config.initial_backoff_ms;

        assert_eq!(backoff_ms, 500);

        backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
        assert_eq!(backoff_ms, 1000);

        backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
        assert_eq!(backoff_ms, 2000);

        for _ in 0..10 {
            backoff_ms = next_backoff(backoff_ms, config.max_backoff_ms);
        }
        assert_eq!(backoff_ms, config.max_backoff_ms.min(300_000));
    }

    #[test]
    fn test_parse_retry_after_delta_seconds() {
        let mut h = HeaderMap::new();
        h.insert(RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(parse_retry_after_ms(&h), Some(120_000));
    }

    #[test]
    fn test_parse_retry_after_http_date_future() {
        let mut h = HeaderMap::new();
        let future = SystemTime::now() + Duration::from_secs(5);
        let s = httpdate::fmt_http_date(future);
        h.insert(RETRY_AFTER, HeaderValue::from_str(&s).unwrap());
        let ms = parse_retry_after_ms(&h).unwrap();
        assert!(ms <= 5000 && ms > 0);
    }

    #[test]
    fn test_parse_retry_after_http_date_past() {
        let mut h = HeaderMap::new();
        let past = SystemTime::now() - Duration::from_secs(5);
        let s = httpdate::fmt_http_date(past);
        h.insert(RETRY_AFTER, HeaderValue::from_str(&s).unwrap());
        assert_eq!(parse_retry_after_ms(&h), Some(0));
    }

    #[tokio::test]
    async fn test_retry_config_status_codes() {
        let config = RetryConfig::default();

        assert!(config.retry_on_status_codes.contains(&429)); // Rate limited
        assert!(config.retry_on_status_codes.contains(&500)); // Internal server error
        assert!(config.retry_on_status_codes.contains(&502)); // Bad gateway
        assert!(config.retry_on_status_codes.contains(&503)); // Service unavailable
        assert!(config.retry_on_status_codes.contains(&504)); // Gateway timeout

        assert!(!config.retry_on_status_codes.contains(&200));
        assert!(!config.retry_on_status_codes.contains(&201));

        assert!(!config.retry_on_status_codes.contains(&400));
        assert!(!config.retry_on_status_codes.contains(&401));
        assert!(!config.retry_on_status_codes.contains(&404));
    }
}
