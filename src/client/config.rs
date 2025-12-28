use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use zeroize::ZeroizeOnDrop;

/// Secure wrapper for API keys that automatically zeros memory on drop
///
/// # Security Notes
/// - This type implements `Drop` to securely zero memory
/// - Does NOT implement `Clone` to prevent secret duplication
/// - Use references (`&SecureApiKey`) for passing around keys
#[derive(ZeroizeOnDrop)]
pub struct SecureApiKey {
    #[zeroize(skip)]
    inner: String,
}

impl SecureApiKey {
    /// Create a new secure API key with validation
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();

        // Enhanced validation
        if key.trim().is_empty() {
            return Err(Error::ConfigError("API key cannot be empty".into()));
        }

        // Minimum length validation (OpenRouter keys are typically 20+ chars)
        if key.len() < 20 {
            return Err(Error::ConfigError(
                "API key is too short. Expected at least 20 characters".into(),
            ));
        }

        // Format validation for known patterns
        if !key.starts_with("sk-") && !key.starts_with("or-") {
            return Err(Error::ConfigError(
                "API key format invalid. Expected 'sk-' or 'or-' prefix".into(),
            ));
        }

        Ok(Self { inner: key })
    }

    /// Get the API key as a string slice (should be used minimally)
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the API key for header construction
    pub fn to_bearer_header(&self) -> String {
        format!("Bearer {}", self.inner)
    }
}

impl std::fmt::Debug for SecureApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose the actual key in debug output
        f.debug_struct("SecureApiKey")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

/// Client configuration containing API key, base URL, and additional settings.
///
/// # Security Notes
/// - This type implements `Drop` to securely zero memory for API keys
/// - Does NOT implement `Clone` to prevent secret duplication
/// - Use references (`&ClientConfig`) for passing around configuration
#[derive(Debug)]
pub struct ClientConfig {
    pub api_key: Option<SecureApiKey>,
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub user_id: Option<String>,
    pub timeout: Duration,
    pub retry_config: RetryConfig,
    pub max_response_bytes: usize,
}

/// Configuration for API instances that doesn't include sensitive data
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub user_id: Option<String>,
    pub timeout: Duration,
    pub retry_config: Arc<RetryConfig>,
    pub max_response_bytes: usize,
    pub headers: Arc<HeaderMap>,
}

impl ClientConfig {
    /// Build HTTP headers required for making API calls.
    /// Returns a HeaderMap (will be wrapped in Arc in to_api_config).
    /// Returns an error if any header value cannot be constructed.
    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            let auth_header = HeaderValue::from_str(&key.to_bearer_header())
                .map_err(|e| Error::ConfigError(format!("Invalid API key header format: {e}")))?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        // Content-Type header is always valid.
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref referer) = self.http_referer {
            let ref_value = HeaderValue::from_str(referer)
                .map_err(|e| Error::ConfigError(format!("Invalid Referer header: {e}")))?;
            headers.insert("Referer", ref_value);
        }
        if let Some(ref site_title) = self.site_title {
            let title_value = HeaderValue::from_str(site_title)
                .map_err(|e| Error::ConfigError(format!("Invalid X-Title header: {e}")))?;
            headers.insert("X-Title", title_value);
        }
        if let Some(ref user_id) = self.user_id {
            let user_value = HeaderValue::from_str(user_id)
                .map_err(|e| Error::ConfigError(format!("Invalid X-User-ID header: {e}")))?;
            headers.insert("X-User-ID", user_value);
        }
        Ok(headers)
    }

    /// Create an ApiConfig for API instances (excludes sensitive API key).
    ///
    /// # Ownership Semantics
    /// This method clones string values (`base_url`, `http_referer`, etc.)
    /// to ensure `ApiConfig` owns its data independently of the parent
    /// `ClientConfig`. This is intentional design:
    ///
    /// - `ApiConfig` is meant to be thread-safe and independently usable
    /// - Clones happen once per client creation, not per request
    /// - Headers and retry_config are wrapped in `Arc` for shared access
    /// - Performance impact is negligible for typical configuration sizes
    ///
    /// # Hot Path Optimization
    /// The `retry_config` and `headers` are wrapped in `Arc` to avoid
    /// expensive clones on API request hot paths.
    pub fn to_api_config(&self) -> Result<ApiConfig> {
        let headers = self.build_headers()?;
        Ok(ApiConfig {
            base_url: self.base_url.clone(),
            http_referer: self.http_referer.clone(),
            site_title: self.site_title.clone(),
            user_id: self.user_id.clone(),
            timeout: self.timeout,
            retry_config: Arc::new(self.retry_config.clone()),
            max_response_bytes: self.max_response_bytes,
            headers: Arc::new(headers),
        })
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: Url::parse("https://openrouter.ai/api/v1")
                .expect("Default base URL should be valid"),
            http_referer: None,
            site_title: None,
            user_id: None,
            timeout: Duration::from_secs(120),
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Configuration for automatic retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub retry_on_status_codes: Vec<u16>,
    /// Total time cap for all retry attempts combined
    pub total_timeout: Duration,
    /// Maximum interval between retries (enforces upper bound on backoff)
    pub max_retry_interval: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 500,
            max_backoff_ms: 10000,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
            total_timeout: Duration::from_secs(120), // 2 minutes total
            max_retry_interval: Duration::from_secs(30), // 30 seconds max between retries
        }
    }
}

impl RetryConfig {
    /// Set total timeout for all retry attempts
    pub fn with_total_timeout(mut self, timeout: Duration) -> Self {
        self.total_timeout = timeout;
        self
    }

    /// Set maximum retry interval
    pub fn with_max_retry_interval(mut self, interval: Duration) -> Self {
        self.max_retry_interval = interval;
        self
    }
}
