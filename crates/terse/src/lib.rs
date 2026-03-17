pub mod config;
pub mod pipeline;
pub mod tiers;
pub mod tokens;
pub mod types;

pub use config::default_config;
pub use pipeline::{run_compress, run_compress_history};
pub use types::{
    CompressConfig, CompressResult, HistoryResult, Message, MessageContent, MessageWithStats, Tier,
    TokenMethod,
};

/// Compress a single text string.
pub fn compress(
    text: &str,
    role: &str,
    config: &CompressConfig,
) -> Result<CompressResult, String> {
    pipeline::run_compress(text, role, config)
}

/// Compress a history of messages.
pub fn compress_history(
    messages: Vec<Message>,
    config: &CompressConfig,
) -> Result<HistoryResult, String> {
    pipeline::run_compress_history(messages, config)
}
