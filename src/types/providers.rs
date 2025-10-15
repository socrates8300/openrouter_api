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
        self.privacy_policy_url.is_some()
    }

    /// Returns true if the provider has a terms of service URL
    pub fn has_terms_of_service(&self) -> bool {
        self.terms_of_service_url.is_some()
    }

    /// Returns true if the provider has a status page URL
    pub fn has_status_page(&self) -> bool {
        self.status_page_url.is_some()
    }

    /// Gets the domain from the privacy policy URL if available
    pub fn privacy_policy_domain(&self) -> Option<String> {
        self.privacy_policy_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .map(|parsed| parsed.host_str().unwrap_or("").to_string())
        })
    }

    /// Gets the domain from the terms of service URL if available
    pub fn terms_of_service_domain(&self) -> Option<String> {
        self.terms_of_service_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .map(|parsed| parsed.host_str().unwrap_or("").to_string())
        })
    }

    /// Gets the domain from the status page URL if available
    pub fn status_page_domain(&self) -> Option<String> {
        self.status_page_url.as_ref().and_then(|url| {
            url::Url::parse(url)
                .ok()
                .map(|parsed| parsed.host_str().unwrap_or("").to_string())
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

            let provider_list = groups.entry(domain).or_insert_with(Vec::new);
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
}
