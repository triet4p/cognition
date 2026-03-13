use config::{Config, File};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use crate::CognitionResult;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest_dir)
}

fn resolve_config_path(raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);

    if path.is_absolute() || path.exists() {
        return path;
    }

    let workspace_candidate = workspace_root().join(&path);
    if workspace_candidate.exists() {
        return workspace_candidate;
    }

    path
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogType {
    Console,
    File,
    Both,
}

/// Core infrastructure settings (Logging, Agent Identity)
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct CoreSettings {
    pub agent_name: String,
    pub log_level: String,
    pub log_type: LogType,
    pub log_dir: String,
}

/// LLM Provider settings (Model selection, Hyperparameters)
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct LlmSettings {
    pub provider: String,
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// Cognitive Memory settings (Vector dimensions, Activation logic)
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MemorySettings {
    pub vector_dim: usize,
    pub activation_threshold: f32,
    pub consolidation_interval_secs: u64,
}

/// Root configuration structure for the Cognition framework.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub core: CoreSettings,
    pub llm: LlmSettings,
    pub memory: MemorySettings,
}

impl AppConfig {
    /// Entry point to load configuration.
    /// 1. Reads .env from project root to set environment variables.
    /// 2. Reads `COGNITION__CONFIG_PATH` for the YAML file location.
    /// 3. Parses the YAML file into the AppConfig struct.
    pub fn load() -> CognitionResult<Self> {
        // Load .env file if present
        if let Err(e) = dotenvy::dotenv() {
            eprintln!("(Bootstrap) Warning: .env file not loaded: {}", e);
        }

        // Get config path from env or use default
        let raw_config_path = env::var("COGNITION_CONFIG_PATH")
            .unwrap_or_else(|_| "config/default.yaml".into());
        let config_path = resolve_config_path(&raw_config_path);
        eprintln!("Loading configuration from: {}", config_path.display());

        Self::load_from(config_path)
    }

    /// Loads configuration from a specific PathBuf.
    /// Useful for testing and precise control.
    pub fn load_from(path: PathBuf) -> CognitionResult<Self> {
        let s = Config::builder()
            .add_source(File::from(path))
            .build()?;

        // Deserialization errors are automatically converted to CognitionError::Config
        Ok(s.try_deserialize()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_success() {
        let config = AppConfig::load();
        
        assert!(config.is_ok(), "Config should load successfully from default.yaml");
        let c = config.unwrap();
        assert_eq!(c.core.agent_name, "Cognition-Alpha");
        assert_eq!(c.llm.provider, "google");
    }

    #[test]
    fn test_invalid_path_fails() {
        let config = AppConfig::load_from(PathBuf::from("non_existent.yaml"));
        
        assert!(config.is_err(), "Loading from a non-existent file must return an Error");
    }
}