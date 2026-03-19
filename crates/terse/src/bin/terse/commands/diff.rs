use std::io::Write;

use terse::{compress, compress_history, types::MessageContent};

use crate::cli::CommonArgs;
use crate::settings::TerseConfig;
use crate::utils::{detect_history, exit_err, fmt_stats, guard_tty, make_config, open_output, read_input};

pub fn run_diff(args: &CommonArgs, cfg: &TerseConfig) {
    guard_tty(&args.file);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
    let mut out = open_output(&args.output);

    if let Some(messages) = detect_history(&input) {
        let result = compress_history(messages, &config).unwrap_or_else(|e| exit_err(&e, 1));

        for (i, m) in result.messages.iter().enumerate() {
            if i > 0 {
                writeln!(out).unwrap();
            }
            let compressed = match &m.content {
                Some(MessageContent::Text(s)) => s.as_str(),
                _ => "",
            };
            writeln!(out, "--- [{}] {} ({} tokens)", i, m.role, m.stats.original_tokens).unwrap();
            writeln!(out, "+++ [{}] {} ({} tokens, -{}%)", i, m.role, m.stats.compressed_tokens, m.stats.saved_percent).unwrap();
            writeln!(out, "{}", compressed).unwrap();
        }

        eprintln!("{}", fmt_stats(
            result.stats.total_original_tokens,
            result.stats.total_compressed_tokens,
            result.stats.total_saved_tokens,
            result.stats.total_saved_percent,
            Some(result.messages.len()),
        ));
    } else {
        let text = input.trim_end_matches('\n');
        let result = compress(text, &args.role, &config).unwrap_or_else(|e| exit_err(&e, 1));

        writeln!(out, "--- original ({} tokens)", result.original_tokens).unwrap();
        writeln!(out, "{}", text).unwrap();
        writeln!(out, "+++ compressed ({} tokens, -{}%)", result.compressed_tokens, result.saved_percent).unwrap();
        writeln!(out, "{}", result.text).unwrap();

        eprintln!("{}", fmt_stats(
            result.original_tokens,
            result.compressed_tokens,
            result.saved_tokens,
            result.saved_percent,
            None,
        ));
    }
}
