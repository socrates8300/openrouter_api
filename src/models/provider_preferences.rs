//! Provider Preferences module
//!
//! This module defines strongly‑typed provider preference settings that allow
//! users to configure routing options including provider ordering, fallback behavior,
//! parameter requirements, data collection settings, quantizations and sorting.

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// Defines the data collection policy when selecting providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataCollection {
    Allow,
    Deny,
}

/// Defines provider sort preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderSort {
    Price,
    Throughput,
}

/// Defines quantization filtering options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quantization {
    Int4,
    Int8,
    Fp6,
    Fp8,
    Fp16,
    Bf16,
    Fp32,
    Unknown,
}

/// Strongly‑typed provider preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderPreferences {
    /// Ordered list of provider names to prefer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Whether fallback providers are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Whether to require providers to support all parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Controls data collection for providers ("allow" or "deny").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<DataCollection>,
    /// List of provider names to ignore.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    /// List of quantization levels to filter providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<Quantization>>,
    /// Sorting strategy to use when no explicit order is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<ProviderSort>,
}

impl Default for ProviderPreferences {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderPreferences {
    /// Creates a new empty ProviderPreferences.
    ///
    /// All fields are initialized to None.
    pub fn new() -> Self {
        Self {
            order: None,
            allow_fallbacks: None,
            require_parameters: None,
            data_collection: None,
            ignore: None,
            quantizations: None,
            sort: None,
        }
    }

    /// Validates the provider preferences.
    ///
    /// Performs validation checks to ensure the provider preferences are valid.
    /// Returns an error with a descriptive message if any validation fails.
    pub fn validate(&self) -> Result<(), Error> {
        // Validate order if present
        if let Some(ref order) = self.order {
            if order.is_empty() {
                return Err(Error::ConfigError(
                    "Provider order list cannot be empty".to_string(),
                ));
            }

            // Check for duplicates
            let mut seen = std::collections::HashSet::new();
            for provider in order {
                if !seen.insert(provider) {
                    return Err(Error::ConfigError(format!(
                        "Duplicate provider in order list: {provider}"
                    )));
                }
            }
        }

        // Validation passed
        Ok(())
    }

    /// Sets the order of providers.
    pub fn with_order(mut self, order: Vec<String>) -> Self {
        self.order = Some(order);
        self
    }

    /// Sets whether fallbacks are allowed.
    pub fn with_allow_fallbacks(mut self, allow_fallbacks: bool) -> Self {
        self.allow_fallbacks = Some(allow_fallbacks);
        self
    }

    /// Sets whether parameters are required.
    pub fn with_require_parameters(mut self, require_parameters: bool) -> Self {
        self.require_parameters = Some(require_parameters);
        self
    }

    /// Sets the data collection policy.
    pub fn with_data_collection(mut self, data_collection: DataCollection) -> Self {
        self.data_collection = Some(data_collection);
        self
    }

    /// Sets the providers to ignore.
    pub fn with_ignore(mut self, ignore: Vec<String>) -> Self {
        self.ignore = Some(ignore);
        self
    }

    /// Sets the quantizations to use.
    pub fn with_quantizations(mut self, quantizations: Vec<Quantization>) -> Self {
        self.quantizations = Some(quantizations);
        self
    }

    /// Sets the sort preference.
    pub fn with_sort(mut self, sort: ProviderSort) -> Self {
        self.sort = Some(sort);
        self
    }
}
