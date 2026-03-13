use thiserror::Error;

/// Central error type for the Cognition framework.
/// Uses `thiserror` for automatic Display and From implementations.
#[derive(Error, Debug)]
pub enum CognitionError {
    /// Errors related to configuration loading and parsing.
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Errors related to file I/O operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors from LLM Providers
    #[error("LLM Provider error: {provider} - {message}")]
    LlmProvider {
        provider: String,
        message: String,
    },

    /// Errors related to cognitive logic processing.
    #[error("Cognitive logic error: {0}")]
    Logic(String),

    /// Errors related to memory storage and retrieval.
    #[error("Memory storage error: {0}")]
    Memory(String),

    /// Catch-all for any other errors that don't fit into the above categories.
    #[error("Unknown error: {0}")]
    Unknown(String),
}
