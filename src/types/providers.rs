use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provider {
    /// The display name of the provider
    pub name: String,

    /// The unique slug identifier for the provider
    pub slug: String,

    /// URL to the provider's privacy policy (may be null)
    pub privacy_policy_url: Option<String>,

    /// URL to the provider's terms of service (may be null)
    pub terms_of_service_url: Option<String>,

    /// URL to the provider's status page (may be null)
    pub status_page_url: Option<String>,
}

impl Provider {
    /// Creates a new Provider instance
    pub fn new(
        name: String,
        slug: String,
        privacy_policy_url: Option<String>,
        terms_of_service_url: Option<String>,
        status_page_url: Option<String>,
    ) -> Self {
        Self {
            name,
            slug,
            privacy_policy_url,
            terms_of_service_url,
            status_page_url,
        }
    }

    /// Returns true if the provider has a privacy policy URL
    pub fn has_privacy_policy(&self) -> bool {
        self.privacy_policy_url
            .as_ref()
            .is_some_and(|url| !url.is_empty())
    }

    /// Returns true if the provider has a terms of service URL
    pub fn has_terms_of_service(&self) -> bool {
        self.terms_of_service_url
            .as_ref()
            .is_some_and(|url| !url.is_empty())
    }

    /// Returns true if the provider has a status page URL
    pub fn has_status_page(&self) -> bool {
        self.status_page_url
            .as_ref()
            .is_some_and(|url| !url.is_empty())
    }

    /// Gets the domain from the privacy policy URL if available
    pub fn privacy_policy_domain(&self) -> Option<String> {
        self.privacy_policy_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
        })
    }

    /// Gets the domain from the terms of service URL if available
    pub fn terms_of_service_domain(&self) -> Option<String> {
        self.terms_of_service_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
        })
    }

    /// Gets the domain from the status page URL if available
    pub fn status_page_domain(&self) -> Option<String> {
        self.status_page_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
        })
    }
}

/// Response from the providers endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProvidersResponse {
    /// List of available providers
    pub data: Vec<Provider>,
}

impl ProvidersResponse {
    /// Creates a new ProvidersResponse
    pub fn new(data: Vec<Provider>) -> Self {
        Self { data }
    }

    /// Returns the number of providers
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Finds a provider by slug
    pub fn find_by_slug(&self, slug: &str) -> Option<&Provider> {
        self.data.iter().find(|provider| provider.slug == slug)
    }

    /// Finds a provider by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Option<&Provider> {
        self.data
            .iter()
            .find(|provider| provider.name.to_lowercase() == name.to_lowercase())
    }

    /// Returns providers that have a privacy policy
    pub fn with_privacy_policy(&self) -> Vec<&Provider> {
        self.data
            .iter()
            .filter(|provider| provider.has_privacy_policy())
            .collect()
    }

    /// Returns providers that have terms of service
    pub fn with_terms_of_service(&self) -> Vec<&Provider> {
        self.data
            .iter()
            .filter(|provider| provider.has_terms_of_service())
            .collect()
    }

    /// Returns providers that have a status page
    pub fn with_status_page(&self) -> Vec<&Provider> {
        self.data
            .iter()
            .filter(|provider| provider.has_status_page())
            .collect()
    }

    /// Groups providers by domain (extracted from URLs)
    pub fn group_by_domain(&self) -> HashMap<String, Vec<&Provider>> {
        let mut groups: HashMap<String, Vec<&Provider>> = HashMap::new();

        for provider in &self.data {
            // Try to extract domain from any available URL
            let domain = provider
                .privacy_policy_domain()
                .or_else(|| provider.terms_of_service_domain())
                .or_else(|| provider.status_page_domain())
                .unwrap_or_else(|| "unknown".to_string());

            let provider_list = groups.entry(domain).or_default();
            provider_list.push(provider);
        }

        groups
    }

    /// Returns provider slugs sorted alphabetically
    pub fn sorted_slugs(&self) -> Vec<String> {
        let mut slugs: Vec<String> = self
            .data
            .iter()
            .map(|provider| provider.slug.clone())
            .collect();
        slugs.sort_unstable();
        slugs
    }

    /// Returns provider names sorted alphabetically
    pub fn sorted_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .data
            .iter()
            .map(|provider| provider.name.clone())
            .collect();
        names.sort_unstable();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = Provider::new(
            "OpenAI".to_string(),
            "openai".to_string(),
            Some("https://openai.com/policies".to_string()),
            Some("https://openai.com/terms".to_string()),
            Some("https://status.openai.com".to_string()),
        );

        assert_eq!(provider.name, "OpenAI");
        assert_eq!(provider.slug, "openai");
        assert!(provider.has_privacy_policy());
        assert!(provider.has_terms_of_service());
        assert!(provider.has_status_page());
    }

    #[test]
    fn test_provider_without_urls() {
        let provider = Provider::new(
            "Test Provider".to_string(),
            "test".to_string(),
            None,
            None,
            None,
        );

        assert!(!provider.has_privacy_policy());
        assert!(!provider.has_terms_of_service());
        assert!(!provider.has_status_page());
    }

    #[test]
    fn test_providers_response() {
        let providers = vec![
            Provider::new(
                "OpenAI".to_string(),
                "openai".to_string(),
                Some("https://openai.com/policies".to_string()),
                None,
                None,
            ),
            Provider::new(
                "Anthropic".to_string(),
                "anthropic".to_string(),
                Some("https://anthropic.com/policies".to_string()),
                Some("https://anthropic.com/terms".to_string()),
                None,
            ),
        ];

        let response = ProvidersResponse::new(providers);

        assert_eq!(response.count(), 2);
        assert_eq!(response.find_by_slug("openai").unwrap().name, "OpenAI");
        assert_eq!(
            response.find_by_name("anthropic").unwrap().slug,
            "anthropic"
        );
        assert!(response.find_by_slug("nonexistent").is_none());

        assert_eq!(response.with_privacy_policy().len(), 2);
        assert_eq!(response.with_terms_of_service().len(), 1);
        assert_eq!(response.with_status_page().len(), 0);
    }

    #[test]
    fn test_domain_extraction() {
        let provider = Provider::new(
            "OpenAI".to_string(),
            "openai".to_string(),
            Some("https://openai.com/policies".to_string()),
            None,
            None,
        );

        assert_eq!(
            provider.privacy_policy_domain(),
            Some("openai.com".to_string())
        );
        assert_eq!(provider.terms_of_service_domain(), None);
    }

    #[test]
    fn test_group_by_domain() {
        let providers = vec![
            Provider::new(
                "OpenAI".to_string(),
                "openai".to_string(),
                Some("https://openai.com/policies".to_string()),
                None,
                None,
            ),
            Provider::new(
                "Anthropic".to_string(),
                "anthropic".to_string(),
                Some("https://anthropic.com/policies".to_string()),
                None,
                None,
            ),
        ];

        let response = ProvidersResponse::new(providers);
        let groups = response.group_by_domain();

        assert_eq!(groups.get("openai.com").unwrap().len(), 1);
        assert_eq!(groups.get("anthropic.com").unwrap().len(), 1);
    }

    #[test]
    fn test_provider_edge_cases() {
        // Test provider with malformed URLs
        let provider_malformed = Provider::new(
            "Malformed Provider".to_string(),
            "malformed".to_string(),
            Some("not-a-valid-url".to_string()),
            Some("also-not-valid".to_string()),
            Some("https://valid.com/status".to_string()), // One valid URL
        );

        assert!(provider_malformed.has_privacy_policy());
        assert!(provider_malformed.has_terms_of_service());
        assert!(provider_malformed.has_status_page());

        // Domain extraction should return None for invalid URLs
        assert_eq!(provider_malformed.privacy_policy_domain(), None);
        assert_eq!(provider_malformed.terms_of_service_domain(), None);
        assert_eq!(
            provider_malformed.status_page_domain(),
            Some("valid.com".to_string())
        );

        // Test provider with empty URLs
        let provider_empty = Provider::new(
            "Empty Provider".to_string(),
            "empty".to_string(),
            Some("".to_string()),
            None,
            None,
        );

        assert!(!provider_empty.has_privacy_policy());
        assert_eq!(provider_empty.privacy_policy_domain(), None);
    }

    #[test]
    fn test_providers_response_edge_cases() {
        // Test empty response
        let empty_response = ProvidersResponse::new(vec![]);
        assert_eq!(empty_response.count(), 0);
        assert_eq!(empty_response.find_by_slug("anything"), None);
        assert_eq!(empty_response.find_by_name("anything"), None);
        assert_eq!(empty_response.with_privacy_policy().len(), 0);
        assert_eq!(empty_response.with_terms_of_service().len(), 0);
        assert_eq!(empty_response.with_status_page().len(), 0);
        assert_eq!(empty_response.sorted_slugs().len(), 0);
        assert_eq!(empty_response.sorted_names().len(), 0);

        // Test case-insensitive name search
        let providers = vec![
            Provider::new("OpenAI".to_string(), "openai".to_string(), None, None, None),
            Provider::new(
                "ANTHROPIC".to_string(),
                "anthropic".to_string(),
                None,
                None,
                None,
            ),
        ];

        let response = ProvidersResponse::new(providers);

        // Should find regardless of case
        assert!(response.find_by_name("openai").is_some());
        assert!(response.find_by_name("OPENAI").is_some());
        assert!(response.find_by_name("OpenAI").is_some());
        assert!(response.find_by_name("anthropic").is_some());
        assert!(response.find_by_name("ANTHROPIC").is_some());
        assert!(response.find_by_name("Anthropic").is_some());

        // Test partial name matching (should not match)
        assert!(response.find_by_name("Open").is_none());
        assert!(response.find_by_name("AI").is_none());
    }

    #[test]
    fn test_provider_url_validation() {
        // Test various URL formats
        let test_cases = vec![
            ("https://example.com", Some("example.com")),
            ("http://example.com", Some("example.com")),
            ("https://example.com/path", Some("example.com")),
            ("https://sub.example.com", Some("sub.example.com")),
            ("ftp://example.com", Some("example.com")), // URL::parse accepts this
            ("not-a-url", None),
            ("", None),
            ("javascript:alert('xss')", None), // Parses but has no host
        ];

        for (url_str, expected_domain) in test_cases {
            let provider = Provider::new(
                "Test".to_string(),
                "test".to_string(),
                Some(url_str.to_string()),
                None,
                None,
            );

            assert!(provider.privacy_policy_url.is_some());

            assert_eq!(
                provider.privacy_policy_domain(),
                expected_domain.map(|s| s.to_string())
            );
        }
    }
}
