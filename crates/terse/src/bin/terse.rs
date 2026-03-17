use clap::Parser;
use std::io::{self, Read};
use std::path::PathBuf;

use terse::{
    compress, compress_history,
    types::{CompressConfig, Message, MessageContent, Tier, TokenMethod},
};

/// Terse: compress LLM conversation history to reduce token usage.
#[derive(Parser, Debug)]
#[command(name = "terse", version, about)]
struct Args {
    /// Optional path to a JSON file containing a message history array.
    /// If omitted, reads text from stdin.
    file: Option<PathBuf>,

    /// Role for stdin mode (user or assistant).
    #[arg(long, default_value = "assistant")]
    role: String,

    /// Comma-separated list of tiers to apply (rules, nlp, llm).
    #[arg(long, default_value = "rules")]
    tiers: String,

    /// Token counting method (chars or tiktoken).
    #[arg(long, default_value = "chars")]
    tokens: String,

    /// Show a diff of original vs compressed text.
    #[arg(long)]
    diff: bool,
}

fn parse_tiers(s: &str) -> Result<Vec<Tier>, String> {
    s.split(',')
        .map(|t| match t.trim() {
            "rules" => Ok(Tier::Rules),
            "nlp" => Ok(Tier::Nlp),
            "llm" => Ok(Tier::Llm),
            other => Err(format!("Unknown tier: '{}'. Valid values: rules, nlp, llm", other)),
        })
        .collect()
}

fn parse_token_method(s: &str) -> Result<TokenMethod, String> {
    match s.trim() {
        "chars" => Ok(TokenMethod::Chars),
        "tiktoken" => Ok(TokenMethod::Tiktoken),
        other => Err(format!(
            "Unknown token method: '{}'. Valid values: chars, tiktoken",
            other
        )),
    }
}

fn main() {
    let args = Args::parse();

    let tiers = match parse_tiers(&args.tiers) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    let token_method = match parse_token_method(&args.tokens) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    let config = CompressConfig {
        tiers,
        token_method,
    };

    if let Some(path) = args.file {
        // History (JSON) mode
        run_history_mode(path, &config);
    } else {
        // Stdin mode
        run_stdin_mode(&args.role, &config, args.diff);
    }
}

fn run_stdin_mode(role: &str, config: &CompressConfig, diff: bool) {
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("error: failed to read stdin: {}", e);
        std::process::exit(1);
    }

    let text = input.trim_end_matches('\n');

    match compress(text, role, config) {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok(result) => {
            if diff {
                eprintln!("--- original ({} tokens)", result.original_tokens);
                eprintln!("{}", text);
                eprintln!("--- compressed ({} tokens, -{}%)", result.compressed_tokens, result.saved_percent);
                eprintln!("{}", result.text);
            } else {
                print!("{}", result.text);
                // Add trailing newline only if input had one
                if input.ends_with('\n') {
                    println!();
                }
            }

            eprintln!(
                "tokens: {} → {} (saved {} = {}%)",
                result.original_tokens,
                result.compressed_tokens,
                result.saved_tokens,
                result.saved_percent,
            );
        }
    }
}

fn run_history_mode(path: PathBuf, config: &CompressConfig) {
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", path.display(), e);
            std::process::exit(1);
        }
    };

    // Parse JSON — accept either a bare array or an object with a "messages" key
    let raw: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: invalid JSON in '{}': {}", path.display(), e);
            std::process::exit(1);
        }
    };

    let messages_value = if raw.is_array() {
        raw.clone()
    } else if let Some(msgs) = raw.get("messages") {
        msgs.clone()
    } else {
        eprintln!("error: JSON must be an array of messages or an object with a 'messages' key");
        std::process::exit(1);
    };

    let messages: Vec<Message> = match serde_json::from_value(messages_value) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: failed to parse messages: {}", e);
            std::process::exit(1);
        }
    };

    match compress_history(messages, config) {
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        Ok(result) => {
            // Output compressed JSON to stdout
            let output_messages: Vec<serde_json::Value> = result
                .messages
                .iter()
                .map(|m| {
                    let content_val = match &m.content {
                        Some(MessageContent::Text(s)) => serde_json::Value::String(s.clone()),
                        Some(MessageContent::Blocks(b)) => serde_json::Value::Array(b.clone()),
                        Some(MessageContent::Null) | None => serde_json::Value::Null,
                    };
                    let mut obj = serde_json::json!({
                        "role": m.role,
                        "content": content_val,
                        "_stats": {
                            "originalTokens":   m.stats.original_tokens,
                            "compressedTokens": m.stats.compressed_tokens,
                            "savedTokens":      m.stats.saved_tokens,
                            "savedPercent":     m.stats.saved_percent,
                        },
                    });
                    // Merge extra fields
                    if let serde_json::Value::Object(ref mut map) = obj {
                        for (k, v) in &m.extra {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                    obj
                })
                .collect();

            match serde_json::to_string_pretty(&output_messages) {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("error: failed to serialize output: {}", e);
                    std::process::exit(1);
                }
            }

            // Stats to stderr
            eprintln!(
                "tokens: {} → {} (saved {} = {}%)",
                result.stats.total_original_tokens,
                result.stats.total_compressed_tokens,
                result.stats.total_saved_tokens,
                result.stats.total_saved_percent,
            );
        }
    }
}
