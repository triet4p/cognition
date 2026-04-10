/// The PromptRegistry centralizes all LLM instructions used by the framework.
/// It uses `include_str!` to embed the Markdown templates into the binary at compile time.
pub struct PromptRegistry;

impl PromptRegistry {
    /// Template for extracting structured facts from raw conversational data.
    pub const FACT_EXTRACTION: &'static str = include_str!("fact_extraction.md");
}