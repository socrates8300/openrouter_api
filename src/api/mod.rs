pub mod chat;
pub mod completion;
pub mod models;
pub mod request;
pub mod structured;
pub mod web_search;

// Re-export commonly used API types
pub use chat::ChatApi;
pub use completion::CompletionApi;
pub use models::ModelsApi;
pub use structured::StructuredApi;
pub use web_search::WebSearchApi;
