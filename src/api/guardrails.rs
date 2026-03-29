use crate::error::{Error, Result};
use crate::types::guardrails::{
    BulkAssignKeysRequest, BulkAssignMembersRequest, BulkAssignResponse, BulkUnassignResponse,
    GuardrailCreateRequest, GuardrailDeleteResponse, GuardrailKeyAssignmentsResponse,
    GuardrailMemberAssignmentsResponse, GuardrailResponse, GuardrailUpdateRequest,
    GuardrailsListResponse,
};
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;
use serde::Serialize;
use url::Url;

const LIST_GUARDRAILS: &str = "list_guardrails";
const CREATE_GUARDRAIL: &str = "create_guardrail";
const GET_GUARDRAIL: &str = "get_guardrail";
const UPDATE_GUARDRAIL: &str = "update_guardrail";
const DELETE_GUARDRAIL: &str = "delete_guardrail";
const LIST_KEY_ASSIGNMENTS: &str = "list_guardrail_key_assignments";
const LIST_MEMBER_ASSIGNMENTS: &str = "list_guardrail_member_assignments";
const BULK_ASSIGN_KEYS: &str = "bulk_assign_guardrail_keys";
const BULK_ASSIGN_MEMBERS: &str = "bulk_assign_guardrail_members";
const BULK_UNASSIGN_KEYS: &str = "bulk_unassign_guardrail_keys";
const BULK_UNASSIGN_MEMBERS: &str = "bulk_unassign_guardrail_members";

/// API client for OpenRouter Guardrails management endpoints.
pub struct GuardrailsApi {
    pub(crate) client: Client,
    pub(crate) config: crate::client::ApiConfig,
}

impl GuardrailsApi {
    /// Creates a new GuardrailsApi with the given reqwest client and configuration.
    #[must_use = "returns an API client that should be used for API calls"]
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Lists all guardrails for the authenticated user.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list(&self) -> Result<GuardrailsListResponse> {
        self.list_paginated(None, None).await
    }

    /// Lists guardrails with explicit pagination.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_paginated(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<GuardrailsListResponse> {
        self.get_paginated("guardrails", LIST_GUARDRAILS, offset, limit)
            .await
    }

    /// Creates a new guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn create(&self, request: &GuardrailCreateRequest) -> Result<GuardrailResponse> {
        Self::validate_create_request(request)?;
        self.post_json("guardrails", CREATE_GUARDRAIL, request)
            .await
    }

    /// Retrieves a single guardrail by ID.
    ///
    /// Management API key required by OpenRouter.
    pub async fn get(&self, id: &str) -> Result<GuardrailResponse> {
        let path = Self::guardrail_path(id)?;
        self.get_json(&path, GET_GUARDRAIL).await
    }

    /// Updates an existing guardrail by ID.
    ///
    /// Management API key required by OpenRouter.
    pub async fn update(
        &self,
        id: &str,
        request: &GuardrailUpdateRequest,
    ) -> Result<GuardrailResponse> {
        Self::validate_update_request(request)?;
        let path = Self::guardrail_path(id)?;
        self.patch_json(&path, UPDATE_GUARDRAIL, request).await
    }

    /// Deletes an existing guardrail by ID.
    ///
    /// Management API key required by OpenRouter.
    pub async fn delete(&self, id: &str) -> Result<GuardrailDeleteResponse> {
        let path = Self::guardrail_path(id)?;
        self.delete_json(&path, DELETE_GUARDRAIL).await
    }

    /// Lists all API key assignments across the authenticated user's guardrails.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_key_assignments(&self) -> Result<GuardrailKeyAssignmentsResponse> {
        self.list_key_assignments_paginated(None, None).await
    }

    /// Lists API key assignments across all guardrails with explicit pagination.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_key_assignments_paginated(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<GuardrailKeyAssignmentsResponse> {
        self.get_paginated(
            "guardrails/assignments/keys",
            LIST_KEY_ASSIGNMENTS,
            offset,
            limit,
        )
        .await
    }

    /// Lists all organization member assignments across the authenticated user's guardrails.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_member_assignments(&self) -> Result<GuardrailMemberAssignmentsResponse> {
        self.list_member_assignments_paginated(None, None).await
    }

    /// Lists member assignments across all guardrails with explicit pagination.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_member_assignments_paginated(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<GuardrailMemberAssignmentsResponse> {
        self.get_paginated(
            "guardrails/assignments/members",
            LIST_MEMBER_ASSIGNMENTS,
            offset,
            limit,
        )
        .await
    }

    /// Lists API key assignments for a specific guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_guardrail_key_assignments(
        &self,
        id: &str,
    ) -> Result<GuardrailKeyAssignmentsResponse> {
        self.list_guardrail_key_assignments_paginated(id, None, None)
            .await
    }

    /// Lists API key assignments for a specific guardrail with explicit pagination.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_guardrail_key_assignments_paginated(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<GuardrailKeyAssignmentsResponse> {
        let path = format!("{}/assignments/keys", Self::guardrail_path(id)?);
        self.get_paginated(&path, LIST_KEY_ASSIGNMENTS, offset, limit)
            .await
    }

    /// Assigns multiple API keys to a guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn bulk_assign_keys(
        &self,
        id: &str,
        request: &BulkAssignKeysRequest,
    ) -> Result<BulkAssignResponse> {
        Self::validate_key_assignment_request(request)?;
        let path = format!("{}/assignments/keys", Self::guardrail_path(id)?);
        self.post_json(&path, BULK_ASSIGN_KEYS, request).await
    }

    /// Lists member assignments for a specific guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_guardrail_member_assignments(
        &self,
        id: &str,
    ) -> Result<GuardrailMemberAssignmentsResponse> {
        self.list_guardrail_member_assignments_paginated(id, None, None)
            .await
    }

    /// Lists member assignments for a specific guardrail with explicit pagination.
    ///
    /// Management API key required by OpenRouter.
    pub async fn list_guardrail_member_assignments_paginated(
        &self,
        id: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<GuardrailMemberAssignmentsResponse> {
        let path = format!("{}/assignments/members", Self::guardrail_path(id)?);
        self.get_paginated(&path, LIST_MEMBER_ASSIGNMENTS, offset, limit)
            .await
    }

    /// Assigns multiple organization members to a guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn bulk_assign_members(
        &self,
        id: &str,
        request: &BulkAssignMembersRequest,
    ) -> Result<BulkAssignResponse> {
        Self::validate_member_assignment_request(request)?;
        let path = format!("{}/assignments/members", Self::guardrail_path(id)?);
        self.post_json(&path, BULK_ASSIGN_MEMBERS, request).await
    }

    /// Unassigns multiple API keys from a guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn bulk_unassign_keys(
        &self,
        id: &str,
        request: &BulkAssignKeysRequest,
    ) -> Result<BulkUnassignResponse> {
        Self::validate_key_assignment_request(request)?;
        let path = format!("{}/assignments/keys/remove", Self::guardrail_path(id)?);
        self.post_json(&path, BULK_UNASSIGN_KEYS, request).await
    }

    /// Unassigns multiple organization members from a guardrail.
    ///
    /// Management API key required by OpenRouter.
    pub async fn bulk_unassign_members(
        &self,
        id: &str,
        request: &BulkAssignMembersRequest,
    ) -> Result<BulkUnassignResponse> {
        Self::validate_member_assignment_request(request)?;
        let path = format!("{}/assignments/members/remove", Self::guardrail_path(id)?);
        self.post_json(&path, BULK_UNASSIGN_MEMBERS, request).await
    }

    async fn get_json<T>(&self, path: &str, operation: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        let response = execute_with_retry_builder(&self.config.retry_config, operation, || {
            self.client
                .get(url.clone())
                .headers((*self.config.headers).clone())
        })
        .await?;

        handle_response_json::<T>(response, operation).await
    }

    async fn get_paginated<T>(
        &self,
        path: &str,
        operation: &str,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        let query = Self::pagination_query(offset, limit)?;

        let response = execute_with_retry_builder(&self.config.retry_config, operation, || {
            let mut request = self
                .client
                .get(url.clone())
                .headers((*self.config.headers).clone());
            if !query.is_empty() {
                request = request.query(&query);
            }
            request
        })
        .await?;

        handle_response_json::<T>(response, operation).await
    }

    async fn post_json<B, T>(&self, path: &str, operation: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        let response = execute_with_retry_builder(&self.config.retry_config, operation, || {
            self.client
                .post(url.clone())
                .headers((*self.config.headers).clone())
                .json(body)
        })
        .await?;

        handle_response_json::<T>(response, operation).await
    }

    async fn patch_json<B, T>(&self, path: &str, operation: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        let response = execute_with_retry_builder(&self.config.retry_config, operation, || {
            self.client
                .patch(url.clone())
                .headers((*self.config.headers).clone())
                .json(body)
        })
        .await?;

        handle_response_json::<T>(response, operation).await
    }

    async fn delete_json<T>(&self, path: &str, operation: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path)?;
        let response = execute_with_retry_builder(&self.config.retry_config, operation, || {
            self.client
                .delete(url.clone())
                .headers((*self.config.headers).clone())
        })
        .await?;

        handle_response_json::<T>(response, operation).await
    }

    fn endpoint(&self, path: &str) -> Result<Url> {
        self.config
            .base_url
            .join(path)
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for guardrails endpoint '{path}': {e}"),
                metadata: None,
            })
    }

    fn guardrail_path(id: &str) -> Result<String> {
        let id = Self::validate_guardrail_id(id)?;
        Ok(format!("guardrails/{id}"))
    }

    fn validate_guardrail_id(id: &str) -> Result<&str> {
        let id = id.trim();
        if id.is_empty() {
            return Err(Error::ValidationError(
                "Guardrail ID cannot be empty".to_string(),
            ));
        }
        Ok(id)
    }

    fn pagination_query(
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<(&'static str, String)>> {
        if let Some(limit) = limit {
            if limit > 100 {
                return Err(Error::ValidationError(
                    "Pagination limit cannot exceed 100".to_string(),
                ));
            }
        }

        let mut query = Vec::new();
        if let Some(offset) = offset {
            query.push(("offset", offset.to_string()));
        }
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }

        Ok(query)
    }

    fn validate_create_request(request: &GuardrailCreateRequest) -> Result<()> {
        if request.name.trim().is_empty() {
            return Err(Error::ValidationError(
                "Guardrail name cannot be empty".to_string(),
            ));
        }

        Self::validate_guardrail_rules(
            request.limit_usd,
            request.allowed_providers.as_deref(),
            request.ignored_providers.as_deref(),
            request.allowed_models.as_deref(),
        )
    }

    fn validate_update_request(request: &GuardrailUpdateRequest) -> Result<()> {
        if request.is_empty() {
            return Err(Error::ValidationError(
                "Guardrail update request must include at least one field".to_string(),
            ));
        }

        if request
            .name
            .as_ref()
            .is_some_and(|name| name.trim().is_empty())
        {
            return Err(Error::ValidationError(
                "Guardrail name cannot be empty".to_string(),
            ));
        }

        Self::validate_guardrail_rules(
            request.limit_usd,
            request.allowed_providers.as_deref(),
            request.ignored_providers.as_deref(),
            request.allowed_models.as_deref(),
        )
    }

    fn validate_guardrail_rules(
        limit_usd: Option<f64>,
        allowed_providers: Option<&[String]>,
        ignored_providers: Option<&[String]>,
        allowed_models: Option<&[String]>,
    ) -> Result<()> {
        if let Some(limit_usd) = limit_usd {
            if !limit_usd.is_finite() || limit_usd <= 0.0 {
                return Err(Error::ValidationError(
                    "Guardrail limit_usd must be a positive number".to_string(),
                ));
            }
        }

        Self::validate_string_list("allowed_providers", allowed_providers)?;
        Self::validate_string_list("ignored_providers", ignored_providers)?;
        Self::validate_string_list("allowed_models", allowed_models)?;

        Ok(())
    }

    fn validate_key_assignment_request(request: &BulkAssignKeysRequest) -> Result<()> {
        Self::validate_string_list("key_hashes", Some(&request.key_hashes))
    }

    fn validate_member_assignment_request(request: &BulkAssignMembersRequest) -> Result<()> {
        Self::validate_string_list("member_user_ids", Some(&request.member_user_ids))
    }

    fn validate_string_list(field_name: &str, values: Option<&[String]>) -> Result<()> {
        let Some(values) = values else {
            return Ok(());
        };

        if values.is_empty() {
            return Err(Error::ValidationError(format!(
                "{field_name} cannot be empty"
            )));
        }

        if values.iter().any(|value| value.trim().is_empty()) {
            return Err(Error::ValidationError(format!(
                "{field_name} cannot contain empty values"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardrails_api_new() {
        use crate::tests::test_helpers::test_client_config;

        let config = test_client_config();
        let client = Client::new();
        let api = GuardrailsApi::new(client, &config).unwrap();

        assert!(!api.config.headers.is_empty());
        assert!(api.config.headers.contains_key("authorization"));
    }

    #[test]
    fn test_guardrails_base_url_resolves_correct_path() {
        use crate::tests::test_helpers::test_client_config;

        let config = test_client_config();
        let client = Client::new();
        let api = GuardrailsApi::new(client, &config).unwrap();
        let url = api.endpoint("guardrails").unwrap();

        assert!(
            url.path().ends_with("/guardrails"),
            "Expected path ending with /guardrails, got: {}",
            url.path()
        );
    }

    #[test]
    fn test_guardrails_validation_rejects_bad_inputs() {
        assert!(GuardrailsApi::validate_guardrail_id(" ").is_err());
        assert!(GuardrailsApi::pagination_query(Some(0), Some(101)).is_err());
        assert!(
            GuardrailsApi::validate_key_assignment_request(&BulkAssignKeysRequest {
                key_hashes: vec![],
            })
            .is_err()
        );
        assert!(
            GuardrailsApi::validate_member_assignment_request(&BulkAssignMembersRequest {
                member_user_ids: vec![" ".to_string()],
            })
            .is_err()
        );
        assert!(GuardrailsApi::validate_create_request(&GuardrailCreateRequest::new(" ")).is_err());
        assert!(GuardrailsApi::validate_update_request(&GuardrailUpdateRequest::new()).is_err());
    }
}
