//! Strongly-typed identifiers for OpenRouter entities.
//!
//! These newtype wrappers prevent mixing up different entity IDs at compile time.
//! All IDs serialize as transparent strings for API compatibility.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

/// Strongly-typed identifier for AI models.
///
/// Prevents accidental mixing of model IDs with other entity IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(String);

impl ModelId {
    /// Creates a new ModelId from any string-like type.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the ID is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for ModelId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ModelId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<str> for ModelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ModelId> for String {
    fn from(val: ModelId) -> Self {
        val.0
    }
}

/// Strongly-typed identifier for generation requests.
///
/// Prevents accidental mixing of generation IDs with other entity IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenerationId(String);

impl GenerationId {
    /// Creates a new GenerationId from any string-like type.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the ID is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for GenerationId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for GenerationId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<str> for GenerationId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GenerationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<GenerationId> for String {
    fn from(val: GenerationId) -> Self {
        val.0
    }
}

/// Strongly-typed identifier for analytics activity entries.
///
/// Prevents accidental mixing of activity IDs with other entity IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivityId(String);

impl ActivityId {
    /// Creates a new ActivityId from any string-like type.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the ID is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for ActivityId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ActivityId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<str> for ActivityId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActivityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ActivityId> for String {
    fn from(val: ActivityId) -> Self {
        val.0
    }
}

/// Strongly-typed identifier for tool calls.
///
/// Prevents accidental mixing of tool call IDs with other entity IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolCallId(String);

impl ToolCallId {
    /// Creates a new ToolCallId from any string-like type.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the ID is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for ToolCallId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ToolCallId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<str> for ToolCallId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Strongly-typed price value for API costs and pricing.
///
/// Provides compile-time type safety and validation. The API may return negative
/// prices as special indicators (e.g., -1 for "free" or "unknown").
/// Prices serialize as transparent numbers for API compatibility.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct Price(f64);

/// Custom deserializer for Price that handles both string and number inputs.
/// The OpenRouter API returns prices as strings, but we want to use them as f64 internally.
/// Null values are treated as zero price.
impl<'de> Deserialize<'de> for Price {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PriceVisitor;

        impl<'de> serde::de::Visitor<'de> for PriceVisitor {
            type Value = Price;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a number or string representing a price (may be negative for API compatibility)")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.is_finite() {
                    Ok(Price(value))
                } else {
                    Err(serde::de::Error::custom("price must be finite"))
                }
            }

            fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.is_finite() {
                    Ok(Price(value as f64))
                } else {
                    Err(serde::de::Error::custom("price must be finite"))
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Price(value as f64))
            }

            fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value as f64 > 0.0 {
                    Ok(Price(value as f64))
                } else {
                    Ok(Price(0.0))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<f64>()
                    .map_err(|_| serde::de::Error::custom("invalid price string format"))
                    .and_then(|v| {
                        if v.is_finite() {
                            Ok(Price(v))
                        } else {
                            Err(serde::de::Error::custom("price must be finite"))
                        }
                    })
            }
        }

        deserializer.deserialize_any(PriceVisitor)
    }
}

impl Price {
    /// Creates a new Price from any numeric type.
    ///
    /// Returns None if the value is NaN or infinite.
    /// Note: Negative prices are accepted for API compatibility.
    pub fn new(value: impl Into<f64>) -> Option<Self> {
        let v = value.into();
        if v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    /// Creates a new Price, panicking on NaN/infinite values.
    ///
    /// # Panics
    /// Panics if the value is NaN or infinite.
    pub fn new_unchecked(value: impl Into<f64>) -> Self {
        let v = value.into();
        assert!(v.is_finite(), "Price must be finite");
        Self(v)
    }

    /// Creates a new Price, accepting negative values for API compatibility.
    pub fn new_allow_negative(value: impl Into<f64>) -> Option<Self> {
        let v = value.into();
        if v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    /// Returns a reference to the underlying f64 value.
    pub fn as_f64(&self) -> f64 {
        self.0
    }

    /// Returns the price as USD-formatted string with 6 decimal places.
    pub fn as_usd(&self) -> String {
        format!("{:.6}", self.0)
    }

    /// Returns true if price is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    /// Returns the absolute value of the price.
    pub fn abs(&self) -> Self {
        Price(self.0.abs())
    }

    /// Returns true if is price is positive.
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    /// Returns true if is price is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < 0.0
    }

    /// Validates that the price is non-negative (valid business logic).
    pub fn is_valid_business_logic(&self) -> bool {
        self.0 >= 0.0
    }
}

impl From<f64> for Price {
    fn from(value: f64) -> Self {
        Self::new_unchecked(value)
    }
}

impl From<f32> for Price {
    fn from(value: f32) -> Self {
        Self::new_unchecked(value as f64)
    }
}

impl AsRef<f64> for Price {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.0)
    }
}

impl From<Price> for f64 {
    fn from(val: Price) -> Self {
        val.0
    }
}

impl fmt::Display for ToolCallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ToolCallId> for String {
    fn from(val: ToolCallId) -> Self {
        val.0
    }
}
