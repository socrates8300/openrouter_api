use crate::error::{Error, Result};
use crate::types::embeddings::{EmbeddingInput, EmbeddingRequest, EmbeddingResponse};
use crate::utils::retry::operations::GET_EMBEDDINGS;
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;

/// API endpoint for embeddings.
pub struct EmbeddingsApi {
    pub(crate) client: Client,
    pub(crate) config: crate::client::ApiConfig,
}

impl EmbeddingsApi {
    /// Creates a new EmbeddingsApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Creates embeddings for the given input.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    /// use openrouter_api::types::embeddings::{EmbeddingRequest, EmbeddingInput};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     let request = EmbeddingRequest {
    ///         model: "openai/text-embedding-3-small".to_string(),
    ///         input: EmbeddingInput::Single("Hello world".to_string()),
    ///         encoding_format: None,
    ///         provider: None,
    ///     };
    ///     let response = client.embeddings()?.create(request).await?;
    ///     println!("Embedding dimensions: {}", response.first_embedding().unwrap().len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        // Validate model ID
        crate::utils::validation::validate_model_id(&request.model)?;

        // Validate input is non-empty
        match &request.input {
            EmbeddingInput::Single(s) => {
                if s.trim().is_empty() {
                    return Err(Error::ValidationError(
                        "Embedding input cannot be empty".into(),
                    ));
                }
            }
            EmbeddingInput::Batch(v) => {
                if v.is_empty() {
                    return Err(Error::ValidationError(
                        "Embedding batch input cannot be empty".into(),
                    ));
                }
                if v.iter().any(|s| s.trim().is_empty()) {
                    return Err(Error::ValidationError(
                        "Embedding batch contains empty strings".into(),
                    ));
                }
            }
        }

        let url = self
            .config
            .base_url
            .join("embeddings")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for embeddings endpoint: {e}"),
                metadata: None,
            })?;

        let response =
            execute_with_retry_builder(&self.config.retry_config, GET_EMBEDDINGS, || {
                self.client
                    .post(url.clone())
                    .headers((*self.config.headers).clone())
                    .json(&request)
            })
            .await?;

        handle_response_json::<EmbeddingResponse>(response, GET_EMBEDDINGS).await
    }

    /// Convenience method to embed a single string.
    pub async fn embed_text(&self, model: &str, text: &str) -> Result<Vec<f64>> {
        let request = EmbeddingRequest {
            model: model.to_string(),
            input: EmbeddingInput::Single(text.to_string()),
            encoding_format: None,
            provider: None,
        };
        let response = self.create(request).await?;
        response
            .first_embedding()
            .cloned()
            .ok_or_else(|| Error::ApiError {
                code: 500,
                message: "No embedding returned in response".into(),
                metadata: None,
            })
    }

    /// Convenience method to embed a batch of strings.
    pub async fn embed_batch(&self, model: &str, texts: Vec<String>) -> Result<Vec<Vec<f64>>> {
        let expected_count = texts.len();
        let request = EmbeddingRequest {
            model: model.to_string(),
            input: EmbeddingInput::Batch(texts),
            encoding_format: None,
            provider: None,
        };
        let response = self.create(request).await?;
        if response.data.len() != expected_count {
            return Err(Error::ApiError {
                code: 500,
                message: format!(
                    "Embedding response count mismatch: expected {} embeddings, got {}",
                    expected_count,
                    response.data.len()
                ),
                metadata: None,
            });
        }
        let mut data = response.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_helpers::test_client_config;

    #[test]
    fn test_embeddings_api_new() {
        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();
        assert!(!api.config.headers.is_empty());
    }

    #[tokio::test]
    async fn test_create_validates_empty_model() {
        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        let request = EmbeddingRequest {
            model: "".to_string(),
            input: EmbeddingInput::Single("Hello".to_string()),
            encoding_format: None,
            provider: None,
        };

        let result = api.create(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_validates_empty_input() {
        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        let request = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("   ".to_string()),
            encoding_format: None,
            provider: None,
        };

        let result = api.create(request).await;
        assert!(result.is_err());
    }
}
