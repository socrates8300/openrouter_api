// src/types/common.rs
use serde::{Deserialize, Serialize};

/// Token usage information returned by the API.
///
/// This structure is used across multiple endpoints to provide
/// token usage statistics for chat completions and text completions.
///
/// # Example
/// ```rust
/// use openrouter_api::types::common::Usage;
///
/// let usage = Usage::new(100, 50);
/// assert_eq!(usage.total_tokens(), 150);
/// assert_eq!(usage.completion_percentage(), 33);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    /// Number of tokens in the prompt sent to the model
    pub prompt_tokens: u32,
    /// Number of tokens in the completion returned by the model
    pub completion_tokens: u32,
    /// Total tokens used (prompt + completion)
    pub total_tokens: u32,
}

impl Usage {
    /// Creates a new Usage instance from prompt and completion tokens.
    ///
    /// Automatically calculates the total token count.
    ///
    /// # Arguments
    /// * `prompt_tokens` - Number of tokens in the prompt
    /// * `completion_tokens` - Number of tokens in the completion
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let usage = Usage::new(100, 50);
    /// assert_eq!(usage.prompt_tokens, 100);
    /// assert_eq!(usage.completion_tokens, 50);
    /// assert_eq!(usage.total_tokens, 150);
    /// ```
    #[must_use]
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }

    /// Returns the total tokens used (sum of prompt and completion).
    ///
    /// This is a convenience method that returns `self.total_tokens`.
    /// In most cases, this value matches the API response, but this method
    /// ensures consistency if you manually constructed the instance.
    #[must_use]
    pub fn total_tokens(&self) -> u32 {
        self.total_tokens
    }

    /// Calculates the percentage of tokens used for completion.
    ///
    /// Returns the percentage as an integer (0-100), representing
    /// what portion of the total tokens were for the completion.
    ///
    /// # Returns
    /// The completion percentage as an integer (0-100)
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let usage = Usage::new(100, 50);
    /// assert_eq!(usage.completion_percentage(), 33);
    /// ```
    #[must_use]
    pub fn completion_percentage(&self) -> u32 {
        if self.total_tokens == 0 {
            0
        } else {
            (self.completion_tokens * 100) / self.total_tokens
        }
    }

    /// Calculates the percentage of tokens used for the prompt.
    ///
    /// Returns the percentage as an integer (0-100), representing
    /// what portion of the total tokens were for the prompt.
    ///
    /// # Returns
    /// The prompt percentage as an integer (0-100)
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let usage = Usage::new(100, 50);
    /// assert_eq!(usage.prompt_percentage(), 66);
    /// ```
    #[must_use]
    pub fn prompt_percentage(&self) -> u32 {
        if self.total_tokens == 0 {
            0
        } else {
            (self.prompt_tokens * 100) / self.total_tokens
        }
    }

    /// Calculates the ratio of completion tokens to prompt tokens.
    ///
    /// Returns the ratio as a floating-point number. A value > 1.0
    /// indicates the completion was longer than the prompt.
    ///
    /// # Returns
    /// The completion-to-prompt ratio (0.0 or higher)
    /// Returns 0.0 if prompt_tokens is 0 to avoid division by zero
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let usage = Usage::new(100, 50);
    /// assert_eq!(usage.completion_prompt_ratio(), 0.5);
    /// ```
    #[must_use]
    pub fn completion_prompt_ratio(&self) -> f64 {
        if self.prompt_tokens == 0 {
            0.0
        } else {
            self.completion_tokens as f64 / self.prompt_tokens as f64
        }
    }

    /// Checks if the total tokens exceed a specified limit.
    ///
    /// # Arguments
    /// * `limit` - The maximum allowed tokens
    ///
    /// # Returns
    /// `true` if total_tokens exceeds the limit, `false` otherwise
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let usage = Usage::new(1000, 200);
    /// assert!(usage.exceeds_limit(1000));
    /// assert!(!usage.exceeds_limit(2000));
    /// ```
    #[must_use]
    pub fn exceeds_limit(&self, limit: u32) -> bool {
        self.total_tokens > limit
    }

    /// Returns `true` if this is a zero-usage response (no tokens used).
    ///
    /// # Example
    /// ```rust
    /// use openrouter_api::types::common::Usage;
    ///
    /// let empty = Usage::new(0, 0);
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = Usage::new(10, 5);
    /// assert!(!non_empty.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_tokens == 0
    }
}
