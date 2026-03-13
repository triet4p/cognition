use crate::config::{AppConfig, LogType};
use crate::CognitionResult;
use std::path::{PathBuf};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry};

/// Initializes the tracing system based on the provided configuration.
/// 
/// It sets up:
/// 1. A filter based on `log_level` (e.g., info, debug).
/// 2. A Console layer (if type is console or both).
/// 3. A File layer with hourly rotation (if type is file or both).
pub fn init_tracing(config: &AppConfig) -> CognitionResult<Option<WorkerGuard>> {
    // 1. Create the base filter from log_level string
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.core.log_level));

    // 2. Setup Subscriber with Registry
    let subscriber = Registry::default().with(env_filter);

    // 3. Handle Expand User Path (~)
    let raw_log_dir = &config.core.log_dir;
    let expanded_path = shellexpand::full(raw_log_dir)
        .map_err(|e| crate::CognitionError::Config(config::ConfigError::Message(e.to_string())))?;
    let log_path = PathBuf::from(expanded_path.into_owned());

    let mut guard: Option<WorkerGuard> = None;

    match config.core.log_type {
        LogType::Console => {
            subscriber.with(fmt::layer()).init();
        }
        LogType::File => {
            let (file_layer, g) = create_file_layer(log_path)?;
            guard = Some(g);
            subscriber.with(file_layer).init();
        }
        LogType::Both => {
            let (file_layer, g) = create_file_layer(log_path)?;
            guard = Some(g);
            subscriber
                .with(fmt::layer())
                .with(file_layer)
                .init();
        }
    }

    tracing::info!("Logging initialized (Mode: {:?}, Dir: {})", config.core.log_type, raw_log_dir);
    Ok(guard)
}

/// Creates a file layer and returns its guard.
/// Uses Box<dyn Layer<S> + Send + Sync> to avoid type mismatch errors.
fn create_file_layer<S>(
    log_dir: PathBuf,
) -> CognitionResult<(Box<dyn Layer<S> + Send + Sync>, WorkerGuard)>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    // Hourly rotation: cognition.log.2024-03-13-10
    let file_appender = tracing_appender::rolling::hourly(log_dir, "cognition.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false) // Files don't support ANSI colors well
        .boxed(); // Erasure of the specific type to Box<dyn Layer...>

    Ok((layer, guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CoreSettings, LlmSettings, MemorySettings};
    use std::fs;
    use std::time::Duration;

    #[test]
    fn test_basic_init_style_file_logging() {
        // Mirror the basic_init flow with a temp log directory to verify file output.
        let test_dir = std::env::temp_dir().join(format!("cognition-logs-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&test_dir).expect("Failed to create temporary log directory");

        let config = AppConfig {
            core: CoreSettings {
                agent_name: "Cognition-Test".to_string(),
                log_level: "info".to_string(),
                log_type: LogType::File,
                log_dir: test_dir.to_string_lossy().to_string(),
            },
            llm: LlmSettings {
                provider: "test".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.0,
                max_tokens: 32,
            },
            memory: MemorySettings {
                vector_dim: 8,
                activation_threshold: 0.5,
                consolidation_interval_secs: 60,
            },
        };

        let guard = init_tracing(&config)
            .expect("init_tracing should succeed")
            .expect("file logging should return a worker guard");

        tracing::info!("logging unit test message");
        drop(guard);
        std::thread::sleep(Duration::from_millis(50));

        let entries: Vec<_> = fs::read_dir(&test_dir)
            .expect("Failed to read log directory")
            .filter_map(Result::ok)
            .collect();
        assert!(!entries.is_empty(), "Expected at least one log file");

        let mut found_message = false;
        for entry in entries {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let content = fs::read_to_string(&path).unwrap_or_default();
            if content.contains("logging unit test message") {
                found_message = true;
                break;
            }
        }

        assert!(found_message, "Expected the test message to be written to a log file");

        let _ = fs::remove_dir_all(&test_dir);
    }
}