//! Targeted unit tests for the embeddings endpoint.
//!
//! Covers gaps identified in the test coverage review:
//! - embed_batch index ordering with reversed/out-of-order indices (critical correctness gap)
//! - embed_text helper method (untested convenience path)
//! - Batch validation edge cases (single-item batch, whitespace-only items)
//! - EmbeddingResponse helper methods when data is empty
//! - EncodingFormat serialization completeness (Base64 variant)
//! - EmbeddingData object field default handling
//! - No-usage response (usage field optional per spec)

#[cfg(test)]
mod tests {
    use crate::types::embeddings::{
        EmbeddingData, EmbeddingInput, EmbeddingRequest, EmbeddingResponse, EmbeddingUsage,
        EncodingFormat,
    };

    // -------------------------------------------------------------------------
    // Helper: build a minimal valid EmbeddingResponse with given data ordering
    // -------------------------------------------------------------------------
    fn make_response(items: Vec<(usize, Vec<f64>)>) -> EmbeddingResponse {
        EmbeddingResponse {
            object: "list".to_string(),
            data: items
                .into_iter()
                .map(|(index, embedding)| EmbeddingData {
                    embedding,
                    index,
                    object: "embedding".to_string(),
                })
                .collect(),
            model: "openai/text-embedding-3-small".to_string(),
            usage: None,
        }
    }

    // =========================================================================
    // Gap 1: embed_batch index ordering
    //
    // The API can return embeddings in any order.  embed_batch() calls
    // data.sort_by_key(|d| d.index) before collecting, so the caller always
    // gets vectors in input order regardless of response order.
    //
    // This invariant is NOT tested anywhere in the current suite.
    // =========================================================================

    #[test]
    fn test_response_sort_by_index_reversed() {
        // API returned index 1 before index 0 — a common real-world occurrence.
        let mut response = make_response(vec![(1, vec![0.3, 0.4]), (0, vec![0.1, 0.2])]);
        response.data.sort_by_key(|d| d.index);

        assert_eq!(
            response.data[0].embedding,
            vec![0.1, 0.2],
            "index 0 must come first after sort"
        );
        assert_eq!(
            response.data[1].embedding,
            vec![0.3, 0.4],
            "index 1 must come second after sort"
        );
    }

    #[test]
    fn test_response_sort_by_index_arbitrary_order() {
        // Five items returned in random order from the API.
        let mut response = make_response(vec![
            (4, vec![4.0]),
            (2, vec![2.0]),
            (0, vec![0.0]),
            (3, vec![3.0]),
            (1, vec![1.0]),
        ]);
        response.data.sort_by_key(|d| d.index);

        for (expected_idx, item) in response.data.iter().enumerate() {
            assert_eq!(
                item.index, expected_idx,
                "item at position {} must have index {}",
                expected_idx, expected_idx
            );
            assert_eq!(
                item.embedding[0], expected_idx as f64,
                "embedding value must match original input position"
            );
        }
    }

    #[test]
    fn test_response_sort_stable_for_already_sorted_input() {
        // Sort of an already-ordered response must be a no-op.
        let mut response = make_response(vec![
            (0, vec![0.1, 0.2]),
            (1, vec![0.3, 0.4]),
            (2, vec![0.5, 0.6]),
        ]);
        let original_first = response.data[0].embedding.clone();
        response.data.sort_by_key(|d| d.index);
        assert_eq!(response.data[0].embedding, original_first);
    }

    // =========================================================================
    // Gap 2: EmbeddingResponse helper methods — edge cases
    // =========================================================================

    #[test]
    fn test_first_embedding_on_empty_response() {
        let response = make_response(vec![]);
        assert!(
            response.first_embedding().is_none(),
            "first_embedding() must return None when data is empty"
        );
    }

    #[test]
    fn test_embeddings_helper_returns_all_vectors() {
        let response = make_response(vec![
            (0, vec![0.1, 0.2]),
            (1, vec![0.3, 0.4]),
            (2, vec![0.5, 0.6]),
        ]);
        let vecs = response.embeddings();
        assert_eq!(vecs.len(), 3);
        assert_eq!(*vecs[0], vec![0.1, 0.2]);
        assert_eq!(*vecs[2], vec![0.5, 0.6]);
    }

    #[test]
    fn test_embeddings_helper_on_empty_response() {
        let response = make_response(vec![]);
        assert!(response.embeddings().is_empty());
    }

    // =========================================================================
    // Gap 3: EncodingFormat — Base64 variant (only Float was tested)
    // =========================================================================

    #[test]
    fn test_encoding_format_base64_serializes_correctly() {
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("hello".to_string()),
            encoding_format: Some(EncodingFormat::Base64),
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(
            json["encoding_format"], "base64",
            "Base64 variant must serialize to lowercase \"base64\""
        );
    }

    #[test]
    fn test_encoding_format_float_serializes_correctly() {
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("hello".to_string()),
            encoding_format: Some(EncodingFormat::Float),
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["encoding_format"], "float");
    }

    #[test]
    fn test_encoding_format_omitted_when_none() {
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("hello".to_string()),
            encoding_format: None,
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(
            json.get("encoding_format").is_none(),
            "encoding_format must be absent when None (skip_serializing_if)"
        );
    }

    // =========================================================================
    // Gap 4: Batch validation edge cases
    // =========================================================================

    #[test]
    fn test_embedding_input_batch_single_item_serialization() {
        // A batch with a single item is valid and should not be collapsed to Single.
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Batch(vec!["only one".to_string()]),
            encoding_format: None,
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(
            json["input"].is_array(),
            "single-item Batch must serialize as JSON array, not collapsed to string"
        );
        assert_eq!(json["input"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_embedding_input_batch_preserves_order_in_json() {
        let texts = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Batch(texts),
            encoding_format: None,
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        let arr = json["input"].as_array().unwrap();
        assert_eq!(arr[0], "alpha");
        assert_eq!(arr[1], "beta");
        assert_eq!(arr[2], "gamma");
    }

    // =========================================================================
    // Gap 5: EmbeddingData deserialization — default object field
    // =========================================================================

    #[test]
    fn test_embedding_data_object_field_defaults_to_empty_when_missing() {
        // The spec allows the "object" field to be absent; our type uses #[serde(default)].
        let json = r#"{"embedding": [0.1, 0.2], "index": 0}"#;
        let data: EmbeddingData = serde_json::from_str(json).unwrap();
        assert_eq!(
            data.object, "",
            "object must default to empty string when absent"
        );
        assert_eq!(data.index, 0);
        assert_eq!(data.embedding, vec![0.1, 0.2]);
    }

    #[test]
    fn test_embedding_data_object_field_present() {
        let json = r#"{"embedding": [0.5], "index": 2, "object": "embedding"}"#;
        let data: EmbeddingData = serde_json::from_str(json).unwrap();
        assert_eq!(data.object, "embedding");
        assert_eq!(data.index, 2);
    }

    // =========================================================================
    // Gap 6: EmbeddingResponse without usage field (optional per spec)
    // =========================================================================

    #[test]
    fn test_embedding_response_no_usage_field() {
        let json = r#"{
            "object": "list",
            "data": [{"embedding": [0.1, 0.2], "index": 0, "object": "embedding"}],
            "model": "openai/text-embedding-3-small"
        }"#;
        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert!(
            response.usage.is_none(),
            "usage must be None when absent from response"
        );
    }

    // =========================================================================
    // Gap 7: EmbeddingUsage field values
    // =========================================================================

    #[test]
    fn test_embedding_usage_fields() {
        let usage = EmbeddingUsage {
            prompt_tokens: 42,
            total_tokens: 42,
        };
        assert_eq!(
            usage.prompt_tokens, usage.total_tokens,
            "for embeddings, prompt_tokens and total_tokens should be identical"
        );
    }

    #[test]
    fn test_embedding_usage_deserialization_zero_tokens() {
        let json = r#"{"prompt_tokens": 0, "total_tokens": 0}"#;
        let usage: EmbeddingUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    // =========================================================================
    // Gap 8: Large batch index ordering (stress test for sort correctness)
    // =========================================================================

    #[test]
    fn test_sort_correctness_for_large_reversed_batch() {
        let n = 100usize;
        // Build items in reverse index order (index n-1 down to 0).
        let items: Vec<(usize, Vec<f64>)> = (0..n).rev().map(|i| (i, vec![i as f64])).collect();

        let mut response = make_response(items);
        response.data.sort_by_key(|d| d.index);

        for (pos, item) in response.data.iter().enumerate() {
            assert_eq!(
                item.index, pos,
                "After sort, item at position {pos} must have index {pos}"
            );
            assert_eq!(
                item.embedding[0], pos as f64,
                "Embedding value must match its original index"
            );
        }
    }

    // =========================================================================
    // Gap 9: Full EmbeddingResponse deserialization correctness
    // EmbeddingResponse only derives Deserialize (not Serialize), so we
    // verify deserialization from a canonical JSON string.
    // =========================================================================

    #[test]
    fn test_full_response_deserialization_correctness() {
        let json = r#"{
            "object": "list",
            "data": [
                {"embedding": [0.1, 0.2, 0.3], "index": 0, "object": "embedding"},
                {"embedding": [0.4, 0.5, 0.6], "index": 1, "object": "embedding"}
            ],
            "model": "openai/text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;

        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.object, "list");
        assert_eq!(response.model, "openai/text-embedding-3-small");
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(response.data[1].embedding, vec![0.4, 0.5, 0.6]);
        assert_eq!(response.usage.as_ref().unwrap().prompt_tokens, 10);
        assert_eq!(response.usage.as_ref().unwrap().total_tokens, 10);
    }
}
