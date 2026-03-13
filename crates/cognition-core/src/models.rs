use crate::types::{Confidence, NodeId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    /// Semantic: Objective facts
    World,      
    /// Episodic: Past events
    Experience, 
    /// Opinion: Subjective judgments
    Opinion,    
    /// Pattern: Learned habits
    Pattern,    
    /// Intention: Future goals
    Intention,  
    /// Action: Causal models
    Action,     
}

/// The core unit of memory in Cognition.
/// Implements the 2-layer schema: Narrative Fact for retrieval, Raw Snippet for context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: NodeId,
    pub network_type: NetworkType,
    
    // Layer 1: Semantic representation (Lossy/Compressed)
    pub narrative_fact: String,
    pub embedding: Option<Vec<f32>>,
    
    // Layer 2: Original source (Lossless/Verbatim)
    pub raw_snippet: Option<String>,
    
    // Metadata
    pub confidence: Confidence,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl MemoryNode {
    pub fn new(network: NetworkType, fact: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            network_type: network,
            narrative_fact: fact,
            embedding: None,
            raw_snippet: None,
            confidence: Confidence::new(1.0),
            created_at: Utc::now(),
            expires_at: None,
        }
    }
}