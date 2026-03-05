use openrouter_api::{OpenRouterClient, utils, ModelGroups, PredefinedModelCoverageProfile};
use openrouter_api::types::chat::{ChatRole, Message};
use openrouter_api::models::structured::JsonSchemaConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct MovieRecommendation {
    title: String,
    year: u32,
    director: String,
    genre: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), openrouter_api::Error> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;

    // Initialize client with Model Coverage Profile
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_http_referer("https://github.com/your-org/your-repo")
        .with_timeout(Duration::from_secs(60))
        .with_model_coverage_profile(PredefinedModelCoverageProfile::Custom(
            ModelGroups::general()
        ))
        .with_api_key(api_key)?;

    // Create a simple chat message
    let messages = vec![Message::text(ChatRole::User, "Recommend a sci-fi movie from the 1980s")];

    // Define JSON schema for structured output
    let schema = JsonSchemaConfig {
        name: "MovieRecommendation".to_string(),
        strict: true,
        schema: serde_json::from_value(serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "year": {"type": "integer", "minimum": 1980, "maximum": 1989},
                "director": {"type": "string"},
                "genre": {"type": "string"},
                "description": {"type": "string"}
            },
            "required": ["title", "year", "director", "genre", "description"]
        })).unwrap(),
    };

    // Get a structured response
    let structured_api = client.structured()?;
    let movie: MovieRecommendation = structured_api.generate(
        "openai/gpt-4o",
        messages,
        schema
    ).await?;

    println!("Recommended movie: {} ({})", movie.title, movie.year);
    println!("Director: {}", movie.director);
    println!("Genre: {}", movie.genre);
    println!("Description: {}", movie.description);

    Ok(())
}

