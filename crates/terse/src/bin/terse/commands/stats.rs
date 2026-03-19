use terse::{compress, compress_history};

use crate::cli::CommonArgs;
use crate::settings::TerseConfig;
use crate::utils::{detect_history, exit_err, fmt_stats, guard_tty, make_config, open_output, read_input};

pub fn run_stats(args: &CommonArgs, cfg: &TerseConfig) {
    guard_tty(&args.file);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
    let mut out = open_output(&args.output);

    use std::io::Write;
    if let Some(messages) = detect_history(&input) {
        let result = compress_history(messages, &config).unwrap_or_else(|e| exit_err(&e, 1));
        writeln!(out, "{}", fmt_stats(
            result.stats.total_original_tokens,
            result.stats.total_compressed_tokens,
            result.stats.total_saved_tokens,
            result.stats.total_saved_percent,
            Some(result.messages.len()),
        )).unwrap();
    } else {
        let text = input.trim_end_matches('\n');
        let result = compress(text, &args.role, &config).unwrap_or_else(|e| exit_err(&e, 1));
        writeln!(out, "{}", fmt_stats(
            result.original_tokens,
            result.compressed_tokens,
            result.saved_tokens,
            result.saved_percent,
            None,
        )).unwrap();
    }
}
