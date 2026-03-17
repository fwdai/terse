use crate::types::{CompressConfig, Tier, TokenMethod};

pub const TIER_RULES: &str = "rules";
pub const TIER_NLP: &str = "nlp";
pub const TIER_LLM: &str = "llm";

pub fn default_config() -> CompressConfig {
    CompressConfig {
        tiers: vec![Tier::Rules],
        token_method: TokenMethod::Chars,
    }
}
