use std::io::Write;
use std::path::PathBuf;

use crate::cli::InstallArgs;
use crate::settings::TerseConfig;
use crate::utils::exit_err;

const INSTALL_MARKER: &str = "# terse-install";

pub fn run_install(args: &InstallArgs, cfg: &TerseConfig) {
    let rc_file = args.rc_file.clone().unwrap_or_else(default_rc_file);

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

    file.write_all(snippet.as_bytes())
        .unwrap_or_else(|e| exit_err(&format!("failed to write to {}: {}", rc_file.display(), e), 1));

    eprintln!("terse shell hook installed in {}", rc_file.display());
    eprintln!();
    eprintln!("Activate it now:");
    eprintln!("  source {}", rc_file.display());
    eprintln!();
    eprintln!("After that, 'claude' will auto-start the terse proxy (port read dynamically from config).");
}

pub fn run_uninstall(args: &InstallArgs, _cfg: &TerseConfig) {
    let rc_file = args.rc_file.clone().unwrap_or_else(default_rc_file);

    let content = std::fs::read_to_string(&rc_file)
        .unwrap_or_else(|e| exit_err(&format!("failed to read {}: {}", rc_file.display(), e), 1));

    if !content.contains(INSTALL_MARKER) {
        eprintln!("terse shell hook not found in {}", rc_file.display());
        return;
    }

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

fn default_rc_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc = if shell.contains("zsh") { ".zshrc" } else { ".bashrc" };
    PathBuf::from(home).join(rc)
}
