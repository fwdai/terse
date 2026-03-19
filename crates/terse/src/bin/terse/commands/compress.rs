use std::io::Write;

use terse::{compress, compress_history, types::{CompressConfig, Message, MessageContent}};

use crate::cli::CompressArgs;
use crate::settings::TerseConfig;
use crate::utils::{detect_history, exit_err, fmt_stats, guard_tty, make_config, open_output, read_input};

pub fn run_compress(args: &CompressArgs, cfg: &TerseConfig) {
    guard_tty(&args.file);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
    let mut out = open_output(&args.output);

    if let Some(messages) = detect_history(&input) {
        run_compress_history(messages, &config, args.text, args.stats, &mut out);
    } else {
        run_compress_text(input.trim_end_matches('\n'), &input, &args.role, &config, args.stats, &mut out);
    }
}

fn run_compress_text(text: &str, raw_input: &str, role: &str, config: &CompressConfig, stats: bool, out: &mut dyn Write) {
    let result = compress(text, role, config).unwrap_or_else(|e| exit_err(&e, 1));

    write!(out, "{}", result.text).unwrap();
    if raw_input.ends_with('\n') {
        writeln!(out).unwrap();
    }

    if stats {
        eprintln!("{}", fmt_stats(result.original_tokens, result.compressed_tokens, result.saved_tokens, result.saved_percent, None));
    }
}

fn run_compress_history(messages: Vec<Message>, config: &CompressConfig, text_mode: bool, stats: bool, out: &mut dyn Write) {
    let result = compress_history(messages, config).unwrap_or_else(|e| exit_err(&e, 1));

    if text_mode {
        for m in &result.messages {
            let content = match &m.content {
                Some(MessageContent::Text(s)) => s.as_str(),
                _ => "",
            };
            writeln!(out, "{}: {}", m.role, content).unwrap();
        }
    } else {
        let output: Vec<serde_json::Value> = result
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
                        "original": m.stats.original_tokens,
                        "compressed": m.stats.compressed_tokens,
                        "saved": m.stats.saved_tokens,
                        "percent": m.stats.saved_percent,
                    },
                });
                if let serde_json::Value::Object(ref mut map) = obj {
                    for (k, v) in &m.extra {
                        map.insert(k.clone(), v.clone());
                    }
                }
                obj
            })
            .collect();

        writeln!(
            out, "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|e| exit_err(&format!("failed to serialize output: {}", e), 1))
        ).unwrap();
    }

    if stats {
        eprintln!("{}", fmt_stats(
            result.stats.total_original_tokens,
            result.stats.total_compressed_tokens,
            result.stats.total_saved_tokens,
            result.stats.total_saved_percent,
            Some(result.messages.len()),
        ));
    }
}
