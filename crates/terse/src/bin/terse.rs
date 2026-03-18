use clap::{Args, CommandFactory, Parser, Subcommand};

fn banner() -> String {
    // Visible inside width between the left/right border glyphs.
    const INNER_WIDTH: usize = 46;

    const RESET: &str = "\x1b[0m";
    const FRAME: &str = "\x1b[38;5;45m";
    const TITLE: &str = "\x1b[1;38;5;231m";
    const SUBTLE: &str = "\x1b[2;38;5;250m"; // faint/dim

    fn line(content_colored: &str, content_visible_len: usize) -> String {
        const INNER_WIDTH: usize = 46;
        const RESET: &str = "\x1b[0m";
        const FRAME: &str = "\x1b[38;5;45m";

        let pad = INNER_WIDTH.saturating_sub(content_visible_len);
        format!(
            "{FRAME}┃{RESET}{content_colored}{}{FRAME}┃{RESET}\n",
            " ".repeat(pad)
        )
    }

    let mut out = String::new();
    out.push_str(&format!(
        "{FRAME}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}\n"
    ));

    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    // Title row (keep your exact inner text).
    let title_plain = "            T  ·  E  ·  R  ·  S  ·  E";
    let title_visible_len = title_plain.chars().count();
    let title_colored = format!("{title_plain_prefix}{TITLE}{title}{RESET}{title_suffix}",
        title_plain_prefix = "            ",
        title = "T  ·  E  ·  R  ·  S  ·  E",
        title_suffix = ""
    );
    // We already included the 12 leading spaces in title_plain; keep those in colored output too.
    out.push_str(&line(&title_colored, title_visible_len));

    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    // Version row (dim "smaller" look) and keep right border aligned regardless of version length.
    let version = env!("CARGO_PKG_VERSION");
    let version_plain = format!("                      v{version}               ");
    let version_visible_len = version_plain.chars().count();
    let version_colored = format!("                    {SUBTLE}v{version}{RESET}                 ");
    out.push_str(&line(&version_colored, version_visible_len));

    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    out.push_str(&format!(
        "{FRAME}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}\n"
    ));
    out
}
use std::io::{self, Read, Write};
use std::path::PathBuf;

use terse::{
    compress, compress_history,
    types::{CompressConfig, Message, MessageContent, Tier, TokenMethod},
};

#[derive(Parser)]
#[command(name = "terse", version, about = "Compress LLM conversation history")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    // Default subcommand args (when no subcommand given, treat as `compress`)
    #[command(flatten)]
    args: CompressArgs,
}

#[derive(Subcommand)]
enum Command {
    /// Compress text or history (default)
    Compress(CompressArgs),
    /// Show compression stats without emitting output
    Stats(CommonArgs),
    /// Show before/after diff
    Diff(CommonArgs),
}

#[derive(Args, Clone)]
struct CommonArgs {
    /// File to read (stdin if omitted)
    file: Option<PathBuf>,

    /// Comma-separated tiers to apply (rules, nlp)
    #[arg(long, default_value = "rules")]
    tiers: String,

    /// Token counting method (chars or tiktoken)
    #[arg(long, default_value = "chars")]
    tokens: String,

    /// Role for text mode (user or assistant)
    #[arg(long, default_value = "assistant")]
    role: String,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Args, Clone)]
struct CompressArgs {
    /// File to read (stdin if omitted)
    file: Option<PathBuf>,

    /// Comma-separated tiers to apply (rules, nlp)
    #[arg(long, default_value = "rules")]
    tiers: String,

    /// Token counting method (chars or tiktoken)
    #[arg(long, default_value = "chars")]
    tokens: String,

    /// Role for text mode (user or assistant)
    #[arg(long, default_value = "assistant")]
    role: String,

    /// History mode: emit plain text instead of JSON (role: content per line)
    #[arg(long)]
    text: bool,

    /// Print compression stats to stderr
    #[arg(long)]
    stats: bool,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,
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

fn guard_tty(file: &Option<PathBuf>, is_root: bool) {
    use std::io::IsTerminal;
    if file.is_none() && io::stdin().is_terminal() {
        if is_root {
            print!("{}", banner());
            println!();
            let _ = Cli::command().print_help();
            println!();
        } else {
            eprintln!("Pass a file path as an argument, or pipe input via stdin.");
            eprintln!("Run 'terse --help' for usage.");
        }
        std::process::exit(2);
    }
}

fn read_input(file: &Option<PathBuf>) -> Result<String, String> {
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

fn detect_history(input: &str) -> Option<Vec<Message>> {
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

fn fmt_stats(
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

fn exit_err(msg: &str, code: i32) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(code);
}

fn open_output(path: &Option<PathBuf>) -> Box<dyn Write> {
    match path {
        Some(p) => Box::new(
            std::fs::File::create(p)
                .unwrap_or_else(|e| exit_err(&format!("failed to open '{}': {}", p.display(), e), 1)),
        ),
        None => Box::new(io::stdout()),
    }
}

fn make_config(tiers: &str, tokens: &str) -> CompressConfig {
    let tiers = parse_tiers(tiers).unwrap_or_else(|e| exit_err(&e, 2));
    let token_method = parse_token_method(tokens).unwrap_or_else(|e| exit_err(&e, 2));
    CompressConfig { tiers, token_method }
}

// ── compress ─────────────────────────────────────────────────────────────────

fn run_compress(args: &CompressArgs, is_root: bool) {
    guard_tty(&args.file, is_root);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(&args.tiers, &args.tokens);
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
            serde_json::to_string_pretty(&output).unwrap_or_else(|e| exit_err(&format!("failed to serialize output: {}", e), 1))
        ).unwrap();
    }

    if stats {
        eprintln!("{}", fmt_stats(result.stats.total_original_tokens, result.stats.total_compressed_tokens, result.stats.total_saved_tokens, result.stats.total_saved_percent, Some(result.messages.len())));
    }
}

// ── stats ─────────────────────────────────────────────────────────────────────

fn run_stats(args: &CommonArgs) {
    guard_tty(&args.file, false);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(&args.tiers, &args.tokens);
    let mut out = open_output(&args.output);

    if let Some(messages) = detect_history(&input) {
        let result = compress_history(messages, &config).unwrap_or_else(|e| exit_err(&e, 1));
        writeln!(out, "{}", fmt_stats(result.stats.total_original_tokens, result.stats.total_compressed_tokens, result.stats.total_saved_tokens, result.stats.total_saved_percent, Some(result.messages.len()))).unwrap();
    } else {
        let text = input.trim_end_matches('\n');
        let result = compress(text, &args.role, &config).unwrap_or_else(|e| exit_err(&e, 1));
        writeln!(out, "{}", fmt_stats(result.original_tokens, result.compressed_tokens, result.saved_tokens, result.saved_percent, None)).unwrap();
    }
}

// ── diff ──────────────────────────────────────────────────────────────────────

fn run_diff(args: &CommonArgs) {
    guard_tty(&args.file, false);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(&args.tiers, &args.tokens);
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

        eprintln!("{}", fmt_stats(result.stats.total_original_tokens, result.stats.total_compressed_tokens, result.stats.total_saved_tokens, result.stats.total_saved_percent, Some(result.messages.len())));
    } else {
        let text = input.trim_end_matches('\n');
        let result = compress(text, &args.role, &config).unwrap_or_else(|e| exit_err(&e, 1));

        writeln!(out, "--- original ({} tokens)", result.original_tokens).unwrap();
        writeln!(out, "{}", text).unwrap();
        writeln!(out, "+++ compressed ({} tokens, -{}%)", result.compressed_tokens, result.saved_percent).unwrap();
        writeln!(out, "{}", result.text).unwrap();

        eprintln!("{}", fmt_stats(result.original_tokens, result.compressed_tokens, result.saved_tokens, result.saved_percent, None));
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Compress(args)) => run_compress(&args, false),
        Some(Command::Stats(args)) => run_stats(&args),
        Some(Command::Diff(args)) => run_diff(&args),
        None => run_compress(&cli.args, true),
    }
}
