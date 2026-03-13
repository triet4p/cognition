use serde::{Deserialize, Serialize};

/// Represents a value constrained between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CognitiveScore(f32);

impl CognitiveScore {
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

pub type Confidence = CognitiveScore;
pub type Salience = CognitiveScore;

/// Unique identifiers for various entities
pub type NodeId = uuid::Uuid;
pub type AgentId = String;