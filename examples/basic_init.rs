// cognition/examples/basic_init.rs
use cognition_core::config::AppConfig;
use cognition_core::init_tracing;

fn main() -> cognition_core::result::CognitionResult<()> {
    // 1. Load config (Giai đoạn bootstrap)
    let config = AppConfig::load()?;

    // 2. Khởi tạo logging
    // Lưu ý: phải giữ biến _guard này, nếu không log file sẽ trống rỗng
    let _guard = init_tracing(&config)?;

    // 3. Bây giờ mới dùng tracing macros
    tracing::info!("🚀 Cognition Framework initialized!");
    tracing::warn!("This is a warning log");
    tracing::error!("This is an error log");

    println!("Check your log directory: {:?}", config.core.log_dir);
    Ok(())
}