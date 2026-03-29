//! Focused tests for the Guardrails management API.

#[cfg(test)]
mod tests {
    use crate::api::guardrails::GuardrailsApi;
    use crate::client::OpenRouterClient;
    use crate::error::Error;
    use crate::tests::test_helpers::{test_client_config, TEST_API_KEY};
    use crate::types::guardrails::{
        BulkAssignKeysRequest, BulkAssignMembersRequest, GuardrailCreateRequest,
        GuardrailResetInterval, GuardrailUpdateRequest,
    };
    use serde_json::json;
    use url::Url;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    fn guardrails_api_for(base_url: &str) -> GuardrailsApi {
        let mut config = test_client_config();
        config.base_url = Url::parse(&format!("{base_url}/")).unwrap();
        GuardrailsApi::new(reqwest::Client::new(), &config).unwrap()
    }

    fn sample_guardrail(id: &str) -> serde_json::Value {
        json!({
            "id": id,
            "name": "Production Guardrail",
            "description": "Protect production usage",
            "limit_usd": 100.0,
            "reset_interval": "monthly",
            "allowed_providers": ["openai", "anthropic"],
            "ignored_providers": ["azure"],
            "allowed_models": ["openai/gpt-5.2"],
            "enforce_zdr": true,
            "created_at": "2025-08-24T10:30:00Z",
            "updated_at": "2025-08-24T15:45:00Z"
        })
    }

    #[test]
    fn test_guardrails_api_client_integration() {
        let client = OpenRouterClient::new()
            .skip_url_configuration()
            .with_api_key(TEST_API_KEY)
            .unwrap();

        let api = client.guardrails().unwrap();
        assert!(api.config.headers.contains_key("authorization"));
    }

    #[tokio::test]
    async fn test_list_guardrails_wiremock_happy_path() {
        let mock_server = MockServer::start().await;
        let api = guardrails_api_for(&mock_server.uri());

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/guardrails"))
            .and(matchers::query_param("offset", "5"))
            .and(matchers::query_param("limit", "20"))
            .and(matchers::header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [sample_guardrail("gr-123")],
                "total_count": 1
            })))
            .mount(&mock_server)
            .await;

        let response = api.list_paginated(Some(5), Some(20)).await.unwrap();
        assert_eq!(response.total_count, 1);
        assert_eq!(response.count(), 1);
        assert_eq!(response.data[0].id, "gr-123");
        assert_eq!(
            response.data[0].reset_interval,
            Some(GuardrailResetInterval::Monthly)
        );
    }

    #[tokio::test]
    async fn test_guardrail_crud_wiremock_happy_path() {
        let mock_server = MockServer::start().await;
        let api = guardrails_api_for(&mock_server.uri());
        let guardrail_id = "gr-123";

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/guardrails"))
            .and(matchers::body_json(json!({
                "name": "Production Guardrail",
                "description": "Protect production usage",
                "limit_usd": 50.0,
                "reset_interval": "weekly",
                "allowed_providers": ["openai"],
                "enforce_zdr": true
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "data": sample_guardrail(guardrail_id)
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!("/guardrails/{guardrail_id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": sample_guardrail(guardrail_id)
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("PATCH"))
            .and(matchers::path(format!("/guardrails/{guardrail_id}")))
            .and(matchers::body_json(json!({
                "description": "Updated description",
                "enforce_zdr": false
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "id": guardrail_id,
                    "name": "Production Guardrail",
                    "description": "Updated description",
                    "limit_usd": 100.0,
                    "reset_interval": "monthly",
                    "allowed_providers": ["openai", "anthropic"],
                    "ignored_providers": ["azure"],
                    "allowed_models": ["openai/gpt-5.2"],
                    "enforce_zdr": false,
                    "created_at": "2025-08-24T10:30:00Z",
                    "updated_at": "2025-08-24T15:50:00Z"
                }
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("DELETE"))
            .and(matchers::path(format!("/guardrails/{guardrail_id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "deleted": true
            })))
            .mount(&mock_server)
            .await;

        let create_request = GuardrailCreateRequest::new("Production Guardrail")
            .with_description("Protect production usage")
            .with_limit_usd(50.0)
            .with_reset_interval(GuardrailResetInterval::Weekly)
            .with_allowed_providers(vec!["openai".to_string()])
            .with_enforce_zdr(true);
        let created = api.create(&create_request).await.unwrap();
        assert_eq!(created.data.id, guardrail_id);

        let fetched = api.get(guardrail_id).await.unwrap();
        assert_eq!(fetched.data.id, guardrail_id);

        let update_request = GuardrailUpdateRequest::new()
            .with_description("Updated description")
            .with_enforce_zdr(false);
        let updated = api.update(guardrail_id, &update_request).await.unwrap();
        assert_eq!(
            updated.data.description.as_deref(),
            Some("Updated description")
        );
        assert_eq!(updated.data.enforce_zdr, Some(false));

        let deleted = api.delete(guardrail_id).await.unwrap();
        assert!(deleted.deleted);
    }

    #[tokio::test]
    async fn test_guardrail_assignments_wiremock_happy_path() {
        let mock_server = MockServer::start().await;
        let api = guardrails_api_for(&mock_server.uri());
        let guardrail_id = "gr-123";

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/guardrails/assignments/keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{
                    "id": "assign-1",
                    "key_hash": "hash-1",
                    "guardrail_id": guardrail_id,
                    "key_name": "Production Key",
                    "key_label": "prod",
                    "assigned_by": "user_123",
                    "created_at": "2025-08-24T10:30:00Z"
                }],
                "total_count": 1
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/guardrails/{guardrail_id}/assignments/members"
            )))
            .and(matchers::query_param("offset", "0"))
            .and(matchers::query_param("limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{
                    "id": "member-1",
                    "user_id": "user_123",
                    "organization_id": "org_456",
                    "guardrail_id": guardrail_id,
                    "assigned_by": "user_admin",
                    "created_at": "2025-08-24T10:30:00Z"
                }],
                "total_count": 1
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(matchers::path(format!(
                "/guardrails/{guardrail_id}/assignments/keys"
            )))
            .and(matchers::body_json(json!({
                "key_hashes": ["hash-1", "hash-2"]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "assigned_count": 2
            })))
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(matchers::path(format!(
                "/guardrails/{guardrail_id}/assignments/members/remove"
            )))
            .and(matchers::body_json(json!({
                "member_user_ids": ["user_123"]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "unassigned_count": 1
            })))
            .mount(&mock_server)
            .await;

        let key_assignments = api.list_key_assignments().await.unwrap();
        assert_eq!(key_assignments.total_count, 1);
        assert_eq!(key_assignments.data[0].key_hash, "hash-1");

        let member_assignments = api
            .list_guardrail_member_assignments_paginated(guardrail_id, Some(0), Some(10))
            .await
            .unwrap();
        assert_eq!(member_assignments.total_count, 1);
        assert_eq!(member_assignments.data[0].user_id, "user_123");

        let assigned = api
            .bulk_assign_keys(
                guardrail_id,
                &BulkAssignKeysRequest::new(vec!["hash-1".to_string(), "hash-2".to_string()]),
            )
            .await
            .unwrap();
        assert_eq!(assigned.assigned_count, 2);

        let unassigned = api
            .bulk_unassign_members(
                guardrail_id,
                &BulkAssignMembersRequest::new(vec!["user_123".to_string()]),
            )
            .await
            .unwrap();
        assert_eq!(unassigned.unassigned_count, 1);
    }

    #[tokio::test]
    async fn test_guardrails_validation_errors_are_local() {
        let api = guardrails_api_for("https://openrouter.ai/api/v1");

        let create_error = api
            .create(&GuardrailCreateRequest::new(" "))
            .await
            .unwrap_err();
        assert!(matches!(
            create_error,
            Error::ValidationError(message) if message == "Guardrail name cannot be empty"
        ));

        let pagination_error = api.list_paginated(Some(0), Some(101)).await.unwrap_err();
        assert!(matches!(
            pagination_error,
            Error::ValidationError(message) if message == "Pagination limit cannot exceed 100"
        ));

        let assignment_error = api
            .bulk_assign_keys(" ", &BulkAssignKeysRequest::new(vec!["hash-1".to_string()]))
            .await
            .unwrap_err();
        assert!(matches!(
            assignment_error,
            Error::ValidationError(message) if message == "Guardrail ID cannot be empty"
        ));
    }
}
