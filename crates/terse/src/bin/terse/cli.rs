use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "terse", version, about = "Compress LLM conversation history")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
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
    /// Show token savings across all sessions
    Gains(GainsArgs),
    /// Upgrade terse to the latest release
    Upgrade,
}

#[derive(Args, Clone)]
pub struct CommonArgs {
    /// File to read (stdin if omitted)
    pub file: Option<PathBuf>,

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    pub mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    pub tokenizer: Option<String>,

    /// Role for text mode (user or assistant)
    #[arg(long, default_value = "assistant")]
    pub role: String,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct CompressArgs {
    /// File to read (stdin if omitted)
    pub file: Option<PathBuf>,

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    pub mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    pub tokenizer: Option<String>,

    /// Role for text mode (user or assistant)
    #[arg(long, default_value = "assistant")]
    pub role: String,

    /// History mode: emit plain text instead of JSON (role: content per line)
    #[arg(long)]
    pub text: bool,

    /// Print compression stats to stderr
    #[arg(long)]
    pub stats: bool,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: Option<ProxyCommand>,

    /// Port to listen on [config: proxy.port]
    #[arg(long)]
    pub port: Option<u16>,

    /// Compression mode: trim, compress, or rewrite [config: mode]
    #[arg(long)]
    pub mode: Option<String>,

    /// Tokenizer to use: tiktoken or approximation [config: tokenizer]
    #[arg(long)]
    pub tokenizer: Option<String>,

    /// PID file path [default: ~/.terse/terse.pid]
    #[arg(long)]
    pub pid_file: Option<PathBuf>,
}

#[derive(Subcommand, Clone)]
pub enum ProxyCommand {
    /// Stop a running terse proxy
    Stop {
        /// PID file path [default: ~/.terse/terse.pid]
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
    /// Show status and stats for the running proxy
    Status {
        /// PID file path [default: ~/.terse/terse.pid]
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
}

#[derive(Args, Clone)]
pub struct GainsArgs {
    /// Refresh every N seconds (default: 2)
    #[arg(short = 'n', long, default_value = "2")]
    pub interval: u64,

    /// Watch mode: live redraw as sessions update
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Args, Clone)]
pub struct InstallArgs {
    /// Shell RC file to write to [default: ~/.zshrc or ~/.bashrc based on $SHELL]
    #[arg(long)]
    pub rc_file: Option<PathBuf>,

    /// Proxy port to use in the shell hook [config: proxy.port]
    #[arg(long)]
    pub port: Option<u16>,
}
