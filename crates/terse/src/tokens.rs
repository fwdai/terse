use crate::types::TokenMethod;
use once_cell::sync::Lazy;
use tiktoken_rs::CoreBPE;

// cl100k_base: GPT-4 / text-embedding-ada-002.
// Good proxy for Claude: token density is comparable for English prose (typically ±5-10%).
static CL100K: Lazy<CoreBPE> =
    Lazy::new(|| tiktoken_rs::cl100k_base().expect("failed to load cl100k_base tokenizer"));

pub fn estimate_tokens(text: &str, method: &TokenMethod) -> Result<usize, String> {
    match method {
        TokenMethod::Chars => Ok((text.len() + 3) / 4),
        TokenMethod::Tiktoken => Ok(CL100K.encode_with_special_tokens(text).len()),
    }
}
