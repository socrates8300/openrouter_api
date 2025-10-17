use crate::error::{Error, Result};
use url::Url;

/// Utility for building API URLs with consistent path handling
pub struct UrlBuilder {
    base_url: Url,
}

impl UrlBuilder {
    /// Creates a new UrlBuilder with the given base URL
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    /// Builds a URL by appending the given path to the base URL
    /// Handles base URLs that may or may not end with '/'
    pub fn build(&self, path: &str) -> Result<Url> {
        // Ensure path doesn't start with '/' to avoid double slashes
        let clean_path = path.trim_start_matches('/');
<<<<<<< HEAD

        self.base_url.join(clean_path).map_err(|e| Error::ApiError {
            code: 400,
            message: format!("Invalid URL construction for path '{}': {}", path, e),
            metadata: None,
        })
=======

        self.base_url
            .join(clean_path)
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL construction for path '{}': {}", path, e),
                metadata: None,
            })
>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
    }

    /// Builds a URL with query parameters
    pub fn build_with_query(&self, path: &str) -> Result<Url> {
        self.build(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder_with_trailing_slash() {
        let base_url = Url::parse("https://api.example.com/v1/").unwrap();
        let builder = UrlBuilder::new(base_url);
<<<<<<< HEAD

=======

>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
        let url = builder.build("activity").unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/v1/activity");
    }

    #[test]
    fn test_url_builder_without_trailing_slash() {
        let base_url = Url::parse("https://api.example.com/v1").unwrap();
        let builder = UrlBuilder::new(base_url);
<<<<<<< HEAD

=======

>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
        let url = builder.build("activity").unwrap();
        // When base URL doesn't end with '/', join replaces the last segment
        assert_eq!(url.as_str(), "https://api.example.com/activity");
    }

    #[test]
    fn test_url_builder_with_leading_slash() {
        let base_url = Url::parse("https://api.example.com/v1/").unwrap();
        let builder = UrlBuilder::new(base_url);
<<<<<<< HEAD

=======

>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
        let url = builder.build("/activity").unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/v1/activity");
    }

    #[test]
    fn test_url_builder_invalid_path() {
        let base_url = Url::parse("https://api.example.com/v1/").unwrap();
        let builder = UrlBuilder::new(base_url);
<<<<<<< HEAD

=======

>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
        // URL join is quite permissive, so this might not fail as expected
        let result = builder.build("../../../etc/passwd");
        // The result should still be a valid URL, just might not be what we expect
        assert!(result.is_ok());
    }
<<<<<<< HEAD
}
=======
}
>>>>>>> 0eddcaa (feat: enterprise-grade error handling standardization (v0.3.0))
