use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{warn, debug};

use cognition_core::{
    CognitionError, CognitionResult, LlmProvider, MemoryNode, NetworkType, 
    Confidence, IntentionStatus
};
use crate::prompts::PromptRegistry;

/// Internal Structure to map the LLM's JSON response
#[derive(Debug, Deserialize)]
struct RawExtractedFact {
    network_type: String, 
    narrative: String,
    confidence: f32,
    deadline: Option<DateTime<Utc>>,
    precondition: Option<String>,
    action: Option<String>,
    outcome: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlmExtractionResponse {
    facts: Vec<RawExtractedFact>,
}


/// MemoryExtractor handles the transformation of raw text into structured nodes.
/// It uses the LlmProvider trait defined in core.
pub struct MemoryExtractor {
    llm_provider: Arc<dyn LlmProvider>,
}


impl MemoryExtractor {
    pub fn new(llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self { llm_provider }
    }

    pub async fn extract(&self, raw_text: &str) -> CognitionResult<Vec<MemoryNode>> {
        debug!("Extracting facts from text length: {}", raw_text.len());

        let prompt = PromptRegistry::FACT_EXTRACTION.replace("{{INPUT_TEXT}}", raw_text);
        let response_text = self.llm_provider.generate(&prompt).await?;

        let parsed: LlmExtractionResponse = serde_json::from_str(&response_text) 
            .map_err(|e| CognitionError::Logic(format!("LLM JSON Parse Error: {}. Response was: {}", e, response_text)))?;

        let mut nodes = Vec::new();
        let now = Utc::now();

        for fact in parsed.facts {
            let network = self.map_network_type(&fact.network_type);
            let narrative = if network == NetworkType::ActionEffect {
                format!(
                    "IF [{}] DO [{}] LEADS TO [{}]",
                    fact.precondition.as_deref().unwrap_or("Any"),
                    fact.action.as_deref().unwrap_or("Action"),
                    fact.outcome.as_deref().unwrap_or("Effect")
                )
            } else {
                fact.narrative
            };

            let mut node = MemoryNode::new(network, narrative);
            
            node.raw_snippet = Some(raw_text.to_string());
            node.confidence = Confidence::new(fact.confidence);
            node.created_at = now;
            node.expires_at = fact.deadline;

            if network == NetworkType::Intention {
                node.intention_status = Some(IntentionStatus::Planning);
            }

            nodes.push(node);
        }

        debug!("Extracted {} facts from LLM", nodes.len());
        Ok(nodes)
    }

    fn map_network_type(&self, label: &str) -> NetworkType {
        match label.to_lowercase().trim() {
            "world" => NetworkType::World,
            "experience" => NetworkType::Experience,
            "opinion" => NetworkType::Opinion,
            "habit" => NetworkType::Habit,
            "intention" => NetworkType::Intention,
            "action_effect" => NetworkType::ActionEffect,
            _ => {
                warn!("LLM returned unknown network type '{}', defaulting to World", label);
                NetworkType::World
            }
        }
    }
}
