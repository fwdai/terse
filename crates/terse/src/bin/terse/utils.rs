use std::io::{self, Read, Write};
use std::path::PathBuf;

use terse::types::{CompressConfig, Message, Tier, TokenMethod};

use crate::settings::TerseConfig;

pub fn exit_err(msg: &str, code: i32) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(code);
}

pub fn guard_tty(file: &Option<PathBuf>) {
    use std::io::IsTerminal;
    if file.is_none() && io::stdin().is_terminal() {
        eprintln!("Pass a file path as an argument, or pipe input via stdin.");
        eprintln!("Run 'terse --help' for usage.");
        std::process::exit(2);
    }
}

pub fn read_input(file: &Option<PathBuf>) -> Result<String, String> {
    match file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read '{}': {}", path.display(), e)),
        None => {
            let mut s = String::new();
            io::stdin()
                .read_to_string(&mut s)
                .map_err(|e| format!("failed to read stdin: {}", e))?;
            Ok(s)
        }
    }
}

pub fn detect_history(input: &str) -> Option<Vec<Message>> {
    let raw: serde_json::Value = serde_json::from_str(input).ok()?;

    let messages_value = if raw.is_array() {
        raw
    } else if let Some(msgs) = raw.get("messages") {
        msgs.clone()
    } else {
        return None;
    };

    serde_json::from_value(messages_value).ok()
}

pub fn fmt_stats(
    original: impl std::fmt::Display,
    compressed: impl std::fmt::Display,
    saved: impl std::fmt::Display,
    percent: impl std::fmt::Display,
    messages: Option<usize>,
) -> String {
    let prefix = match messages {
        Some(n) => format!("{} messages: ", n),
        None => String::new(),
    };
    format!("{}{}  →  {} tokens  (saved {}, {}%)", prefix, original, compressed, saved, percent)
}

pub fn open_output(path: &Option<PathBuf>) -> Box<dyn Write> {
    match path {
        Some(p) => Box::new(
            std::fs::File::create(p)
                .unwrap_or_else(|e| exit_err(&format!("failed to open '{}': {}", p.display(), e), 1)),
        ),
        None => Box::new(io::stdout()),
    }
}

pub fn default_pid_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".terse").join("terse.pid")
}

pub fn make_config(mode: Option<&str>, tokenizer: Option<&str>, cfg: &TerseConfig) -> CompressConfig {
    let mode_str = resolve(mode, cfg.mode.as_deref(), "trim");
    let tokens_str = resolve(tokenizer, cfg.tokenizer.as_deref(), "tiktoken");
    let tiers = parse_mode(mode_str).unwrap_or_else(|e| exit_err(&e, 2));
    let token_method = parse_token_method(tokens_str).unwrap_or_else(|e| exit_err(&e, 2));
    CompressConfig { tiers, token_method }
}

fn resolve<'a>(cli: Option<&'a str>, config: Option<&'a str>, default: &'a str) -> &'a str {
    cli.or(config).unwrap_or(default)
}

fn parse_mode(s: &str) -> Result<Vec<Tier>, String> {
    match s.trim() {
        "trim"     => Ok(vec![Tier::Rules]),
        "compress" => Ok(vec![Tier::Rules, Tier::Nlp]),
        "rewrite"  => Ok(vec![Tier::Rules, Tier::Nlp, Tier::Llm]),
        other => Err(format!(
            "Unknown mode: '{}'. Valid values: trim, compress, rewrite",
            other
        )),
    }
}

fn parse_token_method(s: &str) -> Result<TokenMethod, String> {
    match s.trim() {
        "approximation" => Ok(TokenMethod::Chars),
        "tiktoken"      => Ok(TokenMethod::Tiktoken),
        other => Err(format!(
            "Unknown tokenizer: '{}'. Valid values: tiktoken, approximation",
            other
        )),
    }
}
