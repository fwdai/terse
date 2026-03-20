use std::path::PathBuf;

use crate::cli::GainsArgs;

pub fn run_gains(args: &GainsArgs) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(home).join(".terse").join("sessions").join("claude");

    if args.watch {
        watch_loop(&dir, args.interval);
    } else {
        print_gains(&dir);
    }
}

fn watch_loop(dir: &PathBuf, interval_secs: u64) {
    loop {
        // Clear screen and move cursor to top-left
        print!("\x1b[2J\x1b[H");
        print_gains(dir);
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));
    }
}

fn print_gains(dir: &PathBuf) {
    let sessions = load_sessions(dir);

    if sessions.is_empty() {
        eprintln!("No session data found in {}", dir.display());
        eprintln!("Run 'claude' with the terse proxy to start tracking gains.");
        return;
    }

    let global = aggregate(&sessions);
    let latest = sessions.iter().max_by_key(|s| s.mtime).map(|s| &s.total);

    print_section("TOTAL SAVINGS", Some(sessions.len()), &global);
    if let Some(last) = latest {
        println!();
        print_section("LAST SESSION", None, last);
    }
}

// ── data ──────────────────────────────────────────────────────────────────────

struct SessionFile {
    total: Totals,
    mtime: u64,
}

#[derive(Default)]
struct Totals {
    calls: u64,
    original_tokens: u64,
    compressed_tokens: u64,
    saved_tokens: u64,
}

fn load_sessions(dir: &PathBuf) -> Vec<SessionFile> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    entries
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "json"))
        .filter_map(|entry| {
            let mtime = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let text = std::fs::read_to_string(entry.path()).ok()?;
            let doc: serde_json::Value = serde_json::from_str(&text).ok()?;
            let t = &doc["total"];

            Some(SessionFile {
                mtime,
                total: Totals {
                    calls:             t["calls"].as_u64().unwrap_or(0),
                    original_tokens:   t["original_tokens"].as_u64().unwrap_or(0),
                    compressed_tokens: t["compressed_tokens"].as_u64().unwrap_or(0),
                    saved_tokens:      t["saved_tokens"].as_u64().unwrap_or(0),
                },
            })
        })
        .collect()
}

fn aggregate(sessions: &[SessionFile]) -> Totals {
    let mut out = Totals::default();
    for s in sessions {
        out.calls             += s.total.calls;
        out.original_tokens   += s.total.original_tokens;
        out.compressed_tokens += s.total.compressed_tokens;
        out.saved_tokens      += s.total.saved_tokens;
    }
    out
}

// ── display ───────────────────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const DIM:   &str = "\x1b[2;38;5;240m"; // dimmed grey separator
const CYAN:  &str = "\x1b[38;5;45m";    // title (matches banner)
const GREEN: &str = "\x1b[38;5;82m";    // progress bar

const WIDTH: usize = 50;
const BAR_WIDTH: usize = 28;

fn print_section(label: &str, session_count: Option<usize>, t: &Totals) {
    let saved_pct = if t.original_tokens > 0 {
        t.saved_tokens as f64 / t.original_tokens as f64 * 100.0
    } else {
        0.0
    };

    let sep = format!("{}{}{}", DIM, "═".repeat(WIDTH), RESET);
    let title = match session_count {
        Some(n) => format!("{} ({} {})", label, n, if n == 1 { "session" } else { "sessions" }),
        None => label.to_string(),
    };

    println!("{}", sep);
    println!("  {}{}{}", CYAN, title, RESET);
    println!("{}", sep);
    println!();
    println!("  Total calls:      {}", fmt_num(t.calls));
    println!("  Tokens processed: {}", fmt_tokens(t.original_tokens));
    println!("  Tokens forwarded: {}", fmt_tokens(t.compressed_tokens));
    println!("  Tokens saved:     {} ({:.1}%)", fmt_tokens(t.saved_tokens), saved_pct);
    println!();
    println!("  Efficiency  {}", progress_line(saved_pct));
    println!();
}

fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn progress_line(percent: f64) -> String {
    let clamped = percent.clamp(0.0, 100.0);
    let filled = (clamped / 100.0 * BAR_WIDTH as f64).round() as usize;
    let empty = BAR_WIDTH - filled.min(BAR_WIDTH);
    format!("[{}{}{}{}{} ] {:.1}%", GREEN, "█".repeat(filled), RESET, "░".repeat(empty), RESET, percent)
}
