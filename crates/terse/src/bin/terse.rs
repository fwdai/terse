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
}

#[derive(Subcommand)]
enum Command {
    /// Install shell hook to auto-start proxy when running Claude Code
    Install(InstallArgs),
    /// Remove shell hook installed by terse install
    Uninstall(InstallArgs),
    /// Run a local proxy that compresses history before forwarding to Anthropic
    Proxy(ProxyArgs),
    /// Compress text or history JSON
    Compress(CompressArgs),
    /// Show compression stats without modified output
    Stats(CommonArgs),
    /// Show before/after diff of compression
    Diff(CommonArgs),
    /// Show effective configuration
    Config,
}

#[derive(Args, Clone)]
struct CommonArgs {
    /// File to read (stdin if omitted)
    file: Option<PathBuf>,

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    tokenizer: Option<String>,

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

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    tokenizer: Option<String>,

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

#[derive(Args, Clone)]
struct ProxyArgs {
    #[command(subcommand)]
    action: Option<ProxyCommand>,

    /// Port to listen on [config: proxy.port]
    #[arg(long)]
    port: Option<u16>,

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    tokenizer: Option<String>,

    /// PID file path [default: ~/.terse/terse.pid]
    #[arg(long)]
    pid_file: Option<PathBuf>,
}

#[derive(Subcommand, Clone)]
enum ProxyCommand {
    /// Stop a running terse proxy
    Stop {
        /// PID file path [default: ~/.terse/terse.pid]
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
}

#[derive(Args, Clone)]
struct InstallArgs {
    /// Shell RC file to write to [default: ~/.zshrc or ~/.bashrc based on $SHELL]
    #[arg(long)]
    rc_file: Option<PathBuf>,

    /// Proxy port to use in the shell hook [config: proxy.port]
    #[arg(long)]
    port: Option<u16>,
}

// ── config ────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
struct TerseConfig {
    /// Compression mode: "trim", "compress", or "rewrite"
    mode: Option<String>,
    /// Tokenizer to use: "tiktoken" or "approximation"
    tokenizer: Option<String>,
    /// Proxy-specific overrides
    #[serde(default)]
    proxy: ProxyConfig,
}

#[derive(serde::Deserialize, Default)]
struct ProxyConfig {
    port: Option<u16>,
}

const DEFAULT_CONFIG_JSON: &str = r#"{
  "mode": "trim",
  "tokenizer": "tiktoken",
  "proxy": {
    "port": 3847
  }
}
"#;

fn load_config() -> TerseConfig {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(home).join(".terse").join("config.json");

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, DEFAULT_CONFIG_JSON);
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn default_pid_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".terse").join("terse.pid")
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
        "tiktoken" => Ok(TokenMethod::Tiktoken),
        other => Err(format!(
            "Unknown tokenizer: '{}'. Valid values: tiktoken, approximation",
            other
        )),
    }
}

fn guard_tty(file: &Option<PathBuf>) {
    use std::io::IsTerminal;
    if file.is_none() && io::stdin().is_terminal() {
        eprintln!("Pass a file path as an argument, or pipe input via stdin.");
        eprintln!("Run 'terse --help' for usage.");
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

fn resolve<'a>(cli: Option<&'a str>, config: Option<&'a str>, default: &'a str) -> &'a str {
    cli.or(config).unwrap_or(default)
}

fn make_config(mode: Option<&str>, tokenizer: Option<&str>, cfg: &TerseConfig) -> CompressConfig {
    let mode_str = resolve(mode, cfg.mode.as_deref(), "trim");
    let tokens_str = resolve(tokenizer, cfg.tokenizer.as_deref(), "tiktoken");
    let tiers = parse_mode(mode_str).unwrap_or_else(|e| exit_err(&e, 2));
    let token_method = parse_token_method(tokens_str).unwrap_or_else(|e| exit_err(&e, 2));
    CompressConfig { tiers, token_method }
}


// ── compress ─────────────────────────────────────────────────────────────────

fn run_compress(args: &CompressArgs, cfg: &TerseConfig) {
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
            serde_json::to_string_pretty(&output).unwrap_or_else(|e| exit_err(&format!("failed to serialize output: {}", e), 1))
        ).unwrap();
    }

    if stats {
        eprintln!("{}", fmt_stats(result.stats.total_original_tokens, result.stats.total_compressed_tokens, result.stats.total_saved_tokens, result.stats.total_saved_percent, Some(result.messages.len())));
    }
}

// ── stats ─────────────────────────────────────────────────────────────────────

fn run_stats(args: &CommonArgs, cfg: &TerseConfig) {
    guard_tty(&args.file);
    let input = read_input(&args.file).unwrap_or_else(|e| exit_err(&e, 1));
    let config = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
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

fn run_diff(args: &CommonArgs, cfg: &TerseConfig) {
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

// ── install ───────────────────────────────────────────────────────────────────

const INSTALL_MARKER: &str = "# terse-install";

fn run_install(args: &InstallArgs, cfg: &TerseConfig) {
    let rc_file = args.rc_file.clone().unwrap_or_else(default_rc_file);

    // Check if already installed
    if let Ok(existing) = std::fs::read_to_string(&rc_file) {
        if existing.contains(INSTALL_MARKER) {
            eprintln!("terse shell hook already installed in {}", rc_file.display());
            eprintln!("To update it, remove the block between '{}' lines and re-run.", INSTALL_MARKER);
            return;
        }
    }

    let port = args.port.or(cfg.proxy.port).unwrap_or(3847);
    let snippet = format!(r#"
{INSTALL_MARKER}
claude() {{
    local port
    port=$(terse config 2>/dev/null | awk '/^proxy\.port:/ {{print $2}}')
    port=${{port:-{port}}}
    local pid_file="${{HOME}}/.terse/terse.pid"
    if [ ! -f "$pid_file" ] || ! kill -0 "$(cat "$pid_file")" 2>/dev/null; then
        terse proxy >/dev/null 2>&1 &
        local i=0
        while [ $i -lt 20 ]; do
            sleep 0.05
            nc -z 127.0.0.1 "$port" 2>/dev/null && break
            i=$((i + 1))
        done
    fi
    ANTHROPIC_BASE_URL="http://localhost:$port" command claude "$@"
}}
{INSTALL_MARKER}-end
"#);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&rc_file)
        .unwrap_or_else(|e| exit_err(&format!("failed to open {}: {}", rc_file.display(), e), 1));

    use std::io::Write as _;
    file.write_all(snippet.as_bytes())
        .unwrap_or_else(|e| exit_err(&format!("failed to write to {}: {}", rc_file.display(), e), 1));

    eprintln!("terse shell hook installed in {}", rc_file.display());
    eprintln!();
    eprintln!("Activate it now:");
    eprintln!("  source {}", rc_file.display());
    eprintln!();
    eprintln!("After that, 'claude' will auto-start the terse proxy (port read dynamically from config).");
}

fn default_rc_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc = if shell.contains("zsh") { ".zshrc" } else { ".bashrc" };
    PathBuf::from(home).join(rc)
}

fn run_uninstall(args: &InstallArgs, _cfg: &TerseConfig) {
    let rc_file = args.rc_file.clone().unwrap_or_else(default_rc_file);

    let content = std::fs::read_to_string(&rc_file)
        .unwrap_or_else(|e| exit_err(&format!("failed to read {}: {}", rc_file.display(), e), 1));

    if !content.contains(INSTALL_MARKER) {
        eprintln!("terse shell hook not found in {}", rc_file.display());
        return;
    }

    // Remove everything between INSTALL_MARKER and INSTALL_MARKER-end (inclusive), plus the surrounding newlines
    let marker_start = format!("\n{INSTALL_MARKER}\n");
    let marker_end = format!("\n{INSTALL_MARKER}-end\n");

    let cleaned = if let (Some(start), Some(end)) = (content.find(&marker_start), content.find(&marker_end)) {
        let after_end = end + marker_end.len();
        format!("{}{}", &content[..start], &content[after_end..])
    } else {
        exit_err("could not locate hook block boundaries — remove it manually", 1);
    };

    std::fs::write(&rc_file, cleaned)
        .unwrap_or_else(|e| exit_err(&format!("failed to write {}: {}", rc_file.display(), e), 1));

    eprintln!("terse shell hook removed from {}", rc_file.display());
    eprintln!();
    eprintln!("Activate the change:");
    eprintln!("  source {}", rc_file.display());
}

// ── proxy ─────────────────────────────────────────────────────────────────────

struct ProxyState {
    config: CompressConfig,
    session_id: String,
    client: reqwest::Client,
}

struct ProxyCallStats {
    messages: usize,
    original_tokens: usize,
    compressed_tokens: usize,
    saved_tokens: usize,
    saved_percent: i64,
}

fn run_proxy(args: &ProxyArgs, cfg: &TerseConfig) {
    // Handle `terse proxy stop`
    if let Some(ProxyCommand::Stop { pid_file }) = &args.action {
        let path = pid_file.clone().unwrap_or_else(default_pid_file);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
            eprintln!("No proxy running (pid file not found: {})", path.display());
            std::process::exit(1);
        });
        let pid: u32 = content.trim().parse().unwrap_or_else(|_| {
            eprintln!("Invalid pid file: {}", path.display());
            std::process::exit(1);
        });
        match std::process::Command::new("kill").arg(pid.to_string()).status() {
            Ok(s) if s.success() => {
                let _ = std::fs::remove_file(&path);
                eprintln!("terse proxy stopped (pid {})", pid);
            }
            _ => {
                eprintln!("Failed to stop proxy (pid {}). Already stopped?", pid);
                let _ = std::fs::remove_file(&path);
                std::process::exit(1);
            }
        }
        return;
    }

    let port = args.port.or(cfg.proxy.port).unwrap_or(3847);
    let config = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
    let pid_file = args.pid_file.clone().unwrap_or_else(default_pid_file);

    let session_id = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        format!("{}-{}", ts, std::process::id())
    };

    // Write PID file
    if let Some(parent) = pid_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&pid_file, std::process::id().to_string()).unwrap_or_else(|e| {
        eprintln!("warning: could not write pid file {}: {}", pid_file.display(), e);
    });

    eprintln!("terse proxy  http://localhost:{}", port);
    eprintln!("session      ~/.terse/claude/{}.json", session_id);
    eprintln!("Stop with:   terse proxy stop");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(proxy_serve(port, config, session_id, pid_file));
}

async fn proxy_serve(port: u16, config: CompressConfig, session_id: String, pid_file: PathBuf) {
    use axum::{Router, routing::any};
    use std::sync::Arc;

    let state = Arc::new(ProxyState {
        config,
        session_id,
        client: reqwest::Client::new(),
    });

    let app = Router::new()
        .fallback(any(proxy_handler))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("error: failed to bind {}: {}", addr, e);
        std::process::exit(1);
    });

    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result { eprintln!("terse proxy error: {}", e); }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("terse proxy shutting down");
        }
    }

    let _ = std::fs::remove_file(&pid_file);
}

async fn proxy_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<ProxyState>>,
    request: axum::http::Request<axum::body::Body>,
) -> axum::http::Response<axum::body::Body> {
    use axum::http::StatusCode;

    let (parts, body) = request.into_parts();

    let is_messages = parts.method == axum::http::Method::POST
        && parts.uri.path() == "/v1/messages";

    let body_bytes = match axum::body::to_bytes(body, 32 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return axum::http::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    let (final_bytes, call_stats) = if is_messages {
        match proxy_compress(&body_bytes, &state.config) {
            Ok((b, s)) => (b, Some(s)),
            Err(e) => {
                eprintln!("terse proxy: compression skipped: {}", e);
                (body_bytes, None)
            }
        }
    } else {
        (body_bytes, None)
    };

    // Build upstream URL
    let upstream_url = format!(
        "https://api.anthropic.com{}",
        parts.uri.path_and_query().map(|p| p.as_str()).unwrap_or("")
    );

    // Forward request
    let mut req_builder = state.client
        .request(parts.method.clone(), &upstream_url)
        .body(final_bytes.to_vec());

    for (name, value) in &parts.headers {
        let n = name.as_str();
        if n == "host" || n == "content-length" || n == "transfer-encoding" {
            continue;
        }
        req_builder = req_builder.header(name, value);
    }

    let upstream = match req_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("terse proxy: upstream error: {}", e);
            return axum::http::Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    // Persist stats
    if let Some(stats) = call_stats {
        let session_id = state.session_id.clone();
        tokio::spawn(async move {
            write_proxy_stats(&session_id, &stats).await;
        });
    }

    // Stream response back
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let stream = upstream.bytes_stream();

    let mut response = axum::http::Response::new(axum::body::Body::from_stream(stream));
    *response.status_mut() = status;
    for (k, v) in headers {
        if let Some(name) = k {
            if name.as_str() != "transfer-encoding" {
                response.headers_mut().insert(name, v);
            }
        }
    }
    response
}

/// Flatten a block-array content value into a plain string.
/// Joins all `{"type":"text","text":"..."}` blocks; returns None if none found.
fn flatten_content_blocks(val: &serde_json::Value) -> Option<String> {
    let blocks = val.as_array()?;
    let text: String = blocks
        .iter()
        .filter_map(|b| {
            if b["type"].as_str() == Some("text") {
                b["text"].as_str()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() { None } else { Some(text) }
}

fn proxy_compress(
    body: &bytes::Bytes,
    config: &CompressConfig,
) -> Result<(bytes::Bytes, ProxyCallStats), String> {
    let mut json: serde_json::Value =
        serde_json::from_slice(body).map_err(|e| e.to_string())?;

    let messages_val = json
        .get("messages")
        .ok_or("no messages field")?
        .clone();

    // Normalize block-array content to plain strings so the compression
    // pipeline can process them (it only handles MessageContent::Text).
    let messages_val = {
        let mut arr = messages_val
            .as_array()
            .ok_or("messages is not an array")?
            .clone();
        for msg in &mut arr {
            if let Some(content) = msg.get("content") {
                if content.is_array() {
                    if let Some(flat) = flatten_content_blocks(content) {
                        msg["content"] = serde_json::Value::String(flat);
                    }
                }
            }
        }
        serde_json::Value::Array(arr)
    };

    let messages: Vec<Message> =
        serde_json::from_value(messages_val).map_err(|e| e.to_string())?;

    let original_count = messages.len();
    let result = compress_history(messages, config).map_err(|e| format!("{}", e))?;

    let stats = ProxyCallStats {
        messages: original_count,
        original_tokens: result.stats.total_original_tokens,
        compressed_tokens: result.stats.total_compressed_tokens,
        saved_tokens: result.stats.total_saved_tokens,
        saved_percent: result.stats.total_saved_percent,
    };

    // Rebuild messages without _stats
    let compressed: Vec<serde_json::Value> = result
        .messages
        .iter()
        .map(|m| {
            let content = match &m.content {
                Some(MessageContent::Text(s)) => serde_json::Value::String(s.clone()),
                Some(MessageContent::Blocks(b)) => serde_json::Value::Array(b.clone()),
                Some(MessageContent::Null) | None => serde_json::Value::Null,
            };
            let mut obj = serde_json::Map::new();
            obj.insert("role".into(), serde_json::Value::String(m.role.clone()));
            obj.insert("content".into(), content);
            for (k, v) in &m.extra {
                obj.insert(k.clone(), v.clone());
            }
            serde_json::Value::Object(obj)
        })
        .collect();

    json["messages"] = serde_json::Value::Array(compressed);

    let out = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    Ok((bytes::Bytes::from(out), stats))
}

async fn write_proxy_stats(session_id: &str, stats: &ProxyCallStats) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = std::path::PathBuf::from(home).join(".terse").join("claude");

    if tokio::fs::create_dir_all(&dir).await.is_err() {
        return;
    }

    let path = dir.join(format!("{}.json", session_id));

    let mut doc: serde_json::Value = tokio::fs::read_to_string(&path)
        .await
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "session_id": session_id,
                "calls": [],
                "total": { "calls": 0, "original_tokens": 0, "compressed_tokens": 0, "saved_tokens": 0, "saved_percent": 0 }
            })
        });

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Some(calls) = doc["calls"].as_array_mut() {
        calls.push(serde_json::json!({
            "timestamp": ts,
            "messages": stats.messages,
            "original_tokens": stats.original_tokens,
            "compressed_tokens": stats.compressed_tokens,
            "saved_tokens": stats.saved_tokens,
            "saved_percent": stats.saved_percent,
        }));
    }

    // Recompute totals from all calls
    let empty = vec![];
    let calls = doc["calls"].as_array().unwrap_or(&empty);
    let n = calls.len();
    let total_orig: u64 = calls.iter().filter_map(|c| c["original_tokens"].as_u64()).sum();
    let total_comp: u64 = calls.iter().filter_map(|c| c["compressed_tokens"].as_u64()).sum();
    let total_saved = total_orig.saturating_sub(total_comp);
    let total_pct = if total_orig > 0 { (total_saved * 100 / total_orig) as i64 } else { 0 };

    doc["total"] = serde_json::json!({
        "calls": n,
        "original_tokens": total_orig,
        "compressed_tokens": total_comp,
        "saved_tokens": total_saved,
        "saved_percent": total_pct,
    });

    if let Ok(s) = serde_json::to_string_pretty(&doc) {
        let _ = tokio::fs::write(&path, s).await;
    }
}

// ── config ────────────────────────────────────────────────────────────────────

fn run_config(cfg: &TerseConfig) {
    let mode = cfg.mode.as_deref().unwrap_or("trim");
    let tokenizer = cfg.tokenizer.as_deref().unwrap_or("tiktoken");
    let proxy_port = cfg.proxy.port.unwrap_or(3847);

    println!("mode:        {}", mode);
    println!("tokenizer:   {}", tokenizer);
    println!("proxy.port:  {}", proxy_port);

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(&home).join(".terse").join("config.json");
    println!();
    println!("config file:  {}", path.display());
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let cfg = load_config();

    match cli.command {
        Some(Command::Install(args)) => run_install(&args, &cfg),
        Some(Command::Uninstall(args)) => run_uninstall(&args, &cfg),
        Some(Command::Proxy(args)) => run_proxy(&args, &cfg),
        Some(Command::Compress(args)) => run_compress(&args, &cfg),
        Some(Command::Stats(args)) => run_stats(&args, &cfg),
        Some(Command::Diff(args)) => run_diff(&args, &cfg),
        Some(Command::Config) => run_config(&cfg),
        None => {
            print!("{}", banner());
            println!();
            let _ = Cli::command().print_help();
            println!();
        }
    }
}
