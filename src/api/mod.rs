pub mod analytics;
pub mod chat;
pub mod completion;
pub mod credits;
pub mod embeddings;
pub mod generation;
pub mod key_info;
pub mod models;
pub mod providers;
pub mod request;
pub mod structured;
pub mod web_search;

// Re-export commonly used API types
pub use analytics::AnalyticsApi;
pub use chat::ChatApi;
pub use completion::CompletionApi;
pub use credits::CreditsApi;
pub use embeddings::EmbeddingsApi;
pub use generation::GenerationApi;
pub use key_info::KeyInfoApi;
pub use models::ModelsApi;
pub use providers::ProvidersApi;
pub use structured::StructuredApi;
pub use web_search::WebSearchApi;
