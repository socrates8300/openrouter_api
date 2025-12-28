use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProviderPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,

    /// Provider-specific options for fine-tuned control
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<HashMap<String, serde_json::Value>>,

    /// Routing optimizations for specific parameter requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_optimizations: Option<Vec<String>>,
}

impl Default for ProviderPreferences {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for ProviderPreferences
impl ProviderPreferences {
    pub fn new() -> Self {
        Self {
            order: None,
            allow: None,
            allow_fallbacks: None,
            require_parameters: None,
            data_collection: None,
            ignore: None,
            quantizations: None,
            sort: None,
            provider_options: None,
            route_optimizations: None,
        }
    }

    pub fn with_order(mut self, order: Vec<String>) -> Self {
        self.order = Some(order);
        self
    }

    pub fn with_allow(mut self, allow: Vec<String>) -> Self {
        self.allow = Some(allow);
        self
    }

    pub fn with_allow_fallbacks(mut self, allow: bool) -> Self {
        self.allow_fallbacks = Some(allow);
        self
    }

    pub fn with_require_parameters(mut self, require: bool) -> Self {
        self.require_parameters = Some(require);
        self
    }

    pub fn with_data_collection(mut self, collection: impl Into<String>) -> Self {
        self.data_collection = Some(collection.into());
        self
    }

    pub fn with_ignored_providers(mut self, ignore: Vec<String>) -> Self {
        self.ignore = Some(ignore);
        self
    }

    pub fn with_quantizations(mut self, quantizations: Vec<String>) -> Self {
        self.quantizations = Some(quantizations);
        self
    }

    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    pub fn with_provider_option(
        mut self,
        provider: impl Into<String>,
        options: serde_json::Value,
    ) -> Self {
        let provider_options = self.provider_options.get_or_insert_with(HashMap::new);
        provider_options.insert(provider.into(), options);
        self
    }

    pub fn with_route_optimizations(mut self, optimizations: Vec<String>) -> Self {
        self.route_optimizations = Some(optimizations);
        self
    }
}
