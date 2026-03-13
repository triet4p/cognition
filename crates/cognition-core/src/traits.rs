use crate::models::MemoryNode;
use crate::CognitionResult;
use async_trait::async_trait;

/// Interface for memory operations (to be implemented by cognition-memory/graph)
#[async_trait]
pub trait MemoryEngine: Send + Sync {
    async fn retain(&self, node: MemoryNode) -> CognitionResult<()>;
    async fn recall(&self, query: &str, limit: usize) -> CognitionResult<Vec<MemoryNode>>;
}

/// Interface for LLM operations (to be implemented by cognition-llm)
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> CognitionResult<String>;
}

/// Interface for Agent skills (to be implemented by cognition-skills)
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn manifest(&self) -> serde_json::Value;
    async fn execute(&self, args: serde_json::Value) -> CognitionResult<serde_json::Value>;
}