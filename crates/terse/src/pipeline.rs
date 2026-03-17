use once_cell::sync::Lazy;
use regex::Regex;

use crate::tiers::llm::apply_llm;
use crate::tiers::nlp::apply_nlp;
use crate::tiers::rules::apply_rules;
use crate::tokens::estimate_tokens;
use crate::types::{
    CompressConfig, CompressResult, HistoryResult, HistoryStats, Message, MessageContent,
    MessageStats, MessageWithStats, Tier,
};

// Fenced code blocks: ```...``` (dot matches newline via (?s))
static FENCED_CODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)```[\s\S]*?```").unwrap());
// Inline code: `...` (no newline inside)
static INLINE_CODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"`[^`\n]+`").unwrap());
// URLs
static URL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://\S+").unwrap());

static PLACEHOLDER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\x00B(\d+)\x00").unwrap());

pub struct MaskedText {
    pub masked: String,
    pub blocks: Vec<String>,
}

pub fn mask_protected(text: &str) -> MaskedText {
    let mut blocks: Vec<String> = Vec::new();
    let mut masked = text.to_string();

    // Fenced code blocks first (multiline)
    masked = FENCED_CODE
        .replace_all(&masked, |caps: &regex::Captures| {
            let idx = blocks.len();
            blocks.push(caps[0].to_string());
            format!("\x00B{}\x00", idx)
        })
        .into_owned();

    // Inline code
    masked = INLINE_CODE
        .replace_all(&masked, |caps: &regex::Captures| {
            let idx = blocks.len();
            blocks.push(caps[0].to_string());
            format!("\x00B{}\x00", idx)
        })
        .into_owned();

    // URLs
    masked = URL
        .replace_all(&masked, |caps: &regex::Captures| {
            let idx = blocks.len();
            blocks.push(caps[0].to_string());
            format!("\x00B{}\x00", idx)
        })
        .into_owned();

    MaskedText { masked, blocks }
}

pub fn unmask_protected(text: &str, blocks: &[String]) -> String {
    PLACEHOLDER
        .replace_all(text, |caps: &regex::Captures| {
            let idx: usize = caps[1].parse().unwrap_or(0);
            blocks.get(idx).cloned().unwrap_or_default()
        })
        .into_owned()
}

fn apply_tier(text: &str, role: &str, tier: &Tier) -> Result<String, String> {
    match tier {
        Tier::Rules => Ok(apply_rules(text, role)),
        Tier::Nlp => Ok(apply_nlp(text, role)),
        Tier::Llm => apply_llm(text, role),
    }
}

pub fn run_compress(text: &str, role: &str, config: &CompressConfig) -> Result<CompressResult, String> {
    let original_tokens = estimate_tokens(text, &config.token_method)?;

    // Fast bail-out: no Latin characters → not English, nothing to compress
    if !text.chars().any(|c| c.is_ascii_alphabetic()) {
        return Ok(CompressResult { text: text.to_string(), original_tokens, compressed_tokens: original_tokens, saved_tokens: 0, saved_percent: 0 });
    }

    let MaskedText { masked, blocks } = mask_protected(text);
    let mut result = masked;

    for tier in &config.tiers {
        result = apply_tier(&result, role, tier)?;
    }

    result = unmask_protected(&result, &blocks);

    let compressed_tokens = estimate_tokens(&result, &config.token_method)?;
    let saved_tokens = original_tokens.saturating_sub(compressed_tokens);
    let saved_percent = if original_tokens > 0 {
        ((saved_tokens as f64 / original_tokens as f64) * 100.0).round() as i64
    } else {
        0
    };

    Ok(CompressResult {
        text: result,
        original_tokens,
        compressed_tokens,
        saved_tokens,
        saved_percent,
    })
}

/// Extract the text content from a message for compression.
/// Returns None if content is null or a blocks array.
fn extract_text(content: &Option<MessageContent>) -> Option<&str> {
    match content {
        Some(MessageContent::Text(s)) => Some(s.as_str()),
        Some(MessageContent::Null) | Some(MessageContent::Blocks(_)) | None => None,
    }
}

pub fn run_compress_history(
    messages: Vec<Message>,
    config: &CompressConfig,
) -> Result<HistoryResult, String> {
    let mut output: Vec<MessageWithStats> = Vec::with_capacity(messages.len());
    let mut total_original: usize = 0;
    let mut total_compressed: usize = 0;

    for msg in messages {
        let role = msg.role.clone();

        // Only compress user/assistant messages with string content
        let result = if role == "user" || role == "assistant" {
            if let Some(text) = extract_text(&msg.content) {
                let cr = run_compress(text, &role, config)?;
                Some(cr)
            } else {
                None
            }
        } else {
            None
        };

        let (new_content, stats) = if let Some(cr) = result {
            total_original += cr.original_tokens;
            total_compressed += cr.compressed_tokens;
            let stats = MessageStats {
                original_tokens: cr.original_tokens,
                compressed_tokens: cr.compressed_tokens,
                saved_tokens: cr.saved_tokens,
                saved_percent: cr.saved_percent,
            };
            (Some(MessageContent::Text(cr.text)), stats)
        } else {
            // Pass through: count tokens for stats
            let text = extract_text(&msg.content).unwrap_or("");
            let tok = estimate_tokens(text, &config.token_method).unwrap_or(0);
            total_original += tok;
            total_compressed += tok;
            let stats = MessageStats {
                original_tokens: tok,
                compressed_tokens: tok,
                saved_tokens: 0,
                saved_percent: 0,
            };
            (msg.content.clone(), stats)
        };

        output.push(MessageWithStats {
            role,
            content: new_content,
            stats,
            extra: msg.extra,
        });
    }

    let total_saved = total_original.saturating_sub(total_compressed);
    let total_saved_percent = if total_original > 0 {
        ((total_saved as f64 / total_original as f64) * 100.0).round() as i64
    } else {
        0
    };

    Ok(HistoryResult {
        messages: output,
        stats: HistoryStats {
            total_original_tokens: total_original,
            total_compressed_tokens: total_compressed,
            total_saved_tokens: total_saved,
            total_saved_percent,
        },
    })
}
