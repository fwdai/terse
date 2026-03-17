/// LLM tier — stubbed. Throws a clear error when called.
pub fn apply_llm(_text: &str, _role: &str) -> Result<String, String> {
    Err(
        "LLM tier is not yet implemented. \
         Planned: local model via llama-cpp or candle (Rust rewrite). \
         Use tiers 'rules' or 'nlp' instead."
            .to_string(),
    )
}
