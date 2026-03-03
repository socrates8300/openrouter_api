//! HTTPS enforcement utilities

use url::Url;

/// Enforces that the given URL uses HTTPS.
///
/// Allows `http://` only for localhost development:
/// - `http://localhost`
/// - `http://127.0.0.1`
///
/// When the `allow-http` feature is enabled, all HTTP URLs are permitted
/// (useful for testing with mock servers).
#[cfg(feature = "allow-http")]
pub fn enforce_https(_url: &Url) -> crate::error::Result<()> {
    Ok(())
}

#[cfg(not(feature = "allow-http"))]
pub fn enforce_https(url: &Url) -> crate::error::Result<()> {
    if url.scheme() == "https" {
        return Ok(());
    }

    if url.scheme() == "http" {
        if let Some(host) = url.host_str() {
            if host == "localhost" || host == "127.0.0.1" {
                return Ok(());
            }
        }
    }

    Err(crate::error::Error::ConfigError(format!(
        "URL must use HTTPS: '{}'. HTTP is only allowed for localhost/127.0.0.1",
        url
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_url_allowed() {
        let url = Url::parse("https://openrouter.ai/api/v1/").unwrap();
        assert!(enforce_https(&url).is_ok());
    }

    #[test]
    fn test_http_localhost_allowed() {
        let url = Url::parse("http://localhost:8080/api/").unwrap();
        assert!(enforce_https(&url).is_ok());
    }

    #[test]
    fn test_http_127_0_0_1_allowed() {
        let url = Url::parse("http://127.0.0.1:3000/").unwrap();
        assert!(enforce_https(&url).is_ok());
    }

    #[test]
    #[cfg(not(feature = "allow-http"))]
    fn test_http_remote_rejected() {
        let url = Url::parse("http://api.example.com/v1/").unwrap();
        let result = enforce_https(&url);
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::ConfigError(msg) => {
                assert!(msg.contains("HTTPS"));
            }
            _ => panic!("Expected ConfigError"),
        }
    }
}
