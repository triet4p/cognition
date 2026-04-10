pub mod error;
pub mod result;
pub mod config;
pub mod logging;
pub mod types;
pub mod models;
pub mod traits;

use serde::{Deserialize, Serialize};

pub use error::CognitionError;
pub use result::CognitionResult;
pub use config::AppConfig;
pub use logging::init_tracing;
pub use models::{IntentionStatus, MemoryNode, NetworkType};
pub use types::*;
pub use traits::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
	Entity,      // Bidirectional
	Temporal,    // Directed
	Semantic,    // Undirected
	Causal,      // Directed (Opinion -> Action)
	SrLink,      // Habit -> Observation
	AoCausal,    // Precondition -> Action -> Outcome
	Transition,  // Lifecycle (Intention -> Experience)
}