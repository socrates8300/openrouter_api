//! Types for the OpenRouter Guardrails management API.

use serde::{Deserialize, Serialize};

/// Interval at which a guardrail spending limit resets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GuardrailResetInterval {
    Daily,
    Weekly,
    Monthly,
}

/// A configured guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Guardrail {
    /// Unique identifier for the guardrail.
    pub id: String,
    /// Human-readable guardrail name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional spending limit in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    /// Optional spending limit reset interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<GuardrailResetInterval>,
    /// Optional provider allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    /// Optional provider deny-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_providers: Option<Vec<String>>,
    /// Optional model allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    /// Whether zero data retention is enforced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Response wrapper for a single guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailResponse {
    /// Guardrail payload.
    pub data: Guardrail,
}

/// Response wrapper for a list of guardrails.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailsListResponse {
    /// Guardrails in the current page.
    pub data: Vec<Guardrail>,
    /// Total number of matching guardrails.
    pub total_count: u32,
}

impl GuardrailsListResponse {
    /// Returns the number of guardrails in this page.
    #[must_use]
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

/// Request body for creating a guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailCreateRequest {
    /// Guardrail name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional spending limit in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    /// Optional spending limit reset interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<GuardrailResetInterval>,
    /// Optional provider allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    /// Optional provider deny-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_providers: Option<Vec<String>>,
    /// Optional model allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    /// Whether zero data retention is enforced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
}

impl GuardrailCreateRequest {
    /// Creates a new guardrail create request.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            limit_usd: None,
            reset_interval: None,
            allowed_providers: None,
            ignored_providers: None,
            allowed_models: None,
            enforce_zdr: None,
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the spending limit.
    #[must_use]
    pub fn with_limit_usd(mut self, limit_usd: f64) -> Self {
        self.limit_usd = Some(limit_usd);
        self
    }

    /// Sets the reset interval.
    #[must_use]
    pub fn with_reset_interval(mut self, reset_interval: GuardrailResetInterval) -> Self {
        self.reset_interval = Some(reset_interval);
        self
    }

    /// Sets the allowed providers.
    #[must_use]
    pub fn with_allowed_providers(mut self, allowed_providers: Vec<String>) -> Self {
        self.allowed_providers = Some(allowed_providers);
        self
    }

    /// Sets the ignored providers.
    #[must_use]
    pub fn with_ignored_providers(mut self, ignored_providers: Vec<String>) -> Self {
        self.ignored_providers = Some(ignored_providers);
        self
    }

    /// Sets the allowed models.
    #[must_use]
    pub fn with_allowed_models(mut self, allowed_models: Vec<String>) -> Self {
        self.allowed_models = Some(allowed_models);
        self
    }

    /// Sets zero data retention enforcement.
    #[must_use]
    pub fn with_enforce_zdr(mut self, enforce_zdr: bool) -> Self {
        self.enforce_zdr = Some(enforce_zdr);
        self
    }
}

/// Request body for updating a guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GuardrailUpdateRequest {
    /// New guardrail name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New spending limit in USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_usd: Option<f64>,
    /// New spending limit reset interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<GuardrailResetInterval>,
    /// New provider allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_providers: Option<Vec<String>>,
    /// New provider deny-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_providers: Option<Vec<String>>,
    /// New model allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    /// Whether zero data retention is enforced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_zdr: Option<bool>,
}

impl GuardrailUpdateRequest {
    /// Creates an empty update request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when no fields have been set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.limit_usd.is_none()
            && self.reset_interval.is_none()
            && self.allowed_providers.is_none()
            && self.ignored_providers.is_none()
            && self.allowed_models.is_none()
            && self.enforce_zdr.is_none()
    }

    /// Sets the new name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the new description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the new spending limit.
    #[must_use]
    pub fn with_limit_usd(mut self, limit_usd: f64) -> Self {
        self.limit_usd = Some(limit_usd);
        self
    }

    /// Sets the new reset interval.
    #[must_use]
    pub fn with_reset_interval(mut self, reset_interval: GuardrailResetInterval) -> Self {
        self.reset_interval = Some(reset_interval);
        self
    }

    /// Sets the new allowed providers.
    #[must_use]
    pub fn with_allowed_providers(mut self, allowed_providers: Vec<String>) -> Self {
        self.allowed_providers = Some(allowed_providers);
        self
    }

    /// Sets the new ignored providers.
    #[must_use]
    pub fn with_ignored_providers(mut self, ignored_providers: Vec<String>) -> Self {
        self.ignored_providers = Some(ignored_providers);
        self
    }

    /// Sets the new allowed models.
    #[must_use]
    pub fn with_allowed_models(mut self, allowed_models: Vec<String>) -> Self {
        self.allowed_models = Some(allowed_models);
        self
    }

    /// Sets zero data retention enforcement.
    #[must_use]
    pub fn with_enforce_zdr(mut self, enforce_zdr: bool) -> Self {
        self.enforce_zdr = Some(enforce_zdr);
        self
    }
}

/// API key assignment entry for a guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardrailKeyAssignment {
    /// Unique assignment ID.
    pub id: String,
    /// Hash of the assigned API key.
    pub key_hash: String,
    /// Guardrail ID.
    pub guardrail_id: String,
    /// API key display name.
    pub key_name: String,
    /// API key label.
    pub key_label: String,
    /// User that assigned the key, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_by: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// Organization member assignment entry for a guardrail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardrailMemberAssignment {
    /// Unique assignment ID.
    pub id: String,
    /// Assigned member user ID.
    pub user_id: String,
    /// Organization ID.
    pub organization_id: String,
    /// Guardrail ID.
    pub guardrail_id: String,
    /// User that assigned the member, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_by: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// Response wrapper for guardrail key assignments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailKeyAssignmentsResponse {
    /// Assignment page data.
    pub data: Vec<GuardrailKeyAssignment>,
    /// Total number of matching assignments.
    pub total_count: u32,
}

/// Response wrapper for guardrail member assignments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardrailMemberAssignmentsResponse {
    /// Assignment page data.
    pub data: Vec<GuardrailMemberAssignment>,
    /// Total number of matching assignments.
    pub total_count: u32,
}

/// Request body for bulk key assignment or unassignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkAssignKeysRequest {
    /// API key hashes to assign or unassign.
    pub key_hashes: Vec<String>,
}

impl BulkAssignKeysRequest {
    /// Creates a new bulk key assignment request.
    #[must_use]
    pub fn new(key_hashes: Vec<String>) -> Self {
        Self { key_hashes }
    }
}

/// Request body for bulk member assignment or unassignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkAssignMembersRequest {
    /// Member user IDs to assign or unassign.
    pub member_user_ids: Vec<String>,
}

impl BulkAssignMembersRequest {
    /// Creates a new bulk member assignment request.
    #[must_use]
    pub fn new(member_user_ids: Vec<String>) -> Self {
        Self { member_user_ids }
    }
}

/// Response for successful bulk assignment operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BulkAssignResponse {
    /// Number of entities assigned.
    pub assigned_count: u32,
}

/// Response for successful bulk unassignment operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BulkUnassignResponse {
    /// Number of entities unassigned.
    pub unassigned_count: u32,
}

/// Response for guardrail deletion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardrailDeleteResponse {
    /// Whether the guardrail was deleted.
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrail_reset_interval_roundtrip() {
        let json = serde_json::to_string(&GuardrailResetInterval::Weekly).unwrap();
        assert_eq!(json, "\"weekly\"");

        let interval: GuardrailResetInterval = serde_json::from_str(&json).unwrap();
        assert_eq!(interval, GuardrailResetInterval::Weekly);
    }

    #[test]
    fn test_guardrail_create_request_skips_none_fields() {
        let request = GuardrailCreateRequest::new("Production");
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "Production");
        assert!(json.get("description").is_none());
        assert!(json.get("limit_usd").is_none());
    }

    #[test]
    fn test_guardrail_update_request_is_empty() {
        let empty = GuardrailUpdateRequest::new();
        assert!(empty.is_empty());

        let non_empty = GuardrailUpdateRequest::new().with_enforce_zdr(true);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_guardrail_assignment_request_roundtrip() {
        let request =
            BulkAssignMembersRequest::new(vec!["user_123".to_string(), "user_456".to_string()]);
        let json = serde_json::to_string(&request).unwrap();
        let parsed: BulkAssignMembersRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, request);
    }
}
