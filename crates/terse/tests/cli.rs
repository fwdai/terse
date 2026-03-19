use assert_cmd::Command;
use predicates::prelude::*;

fn terse() -> Command {
    Command::cargo_bin("terse").unwrap()
}

// ── root ──────────────────────────────────────────────────────────────────────

#[test]
fn no_args_shows_banner_and_help() {
    terse()
        .assert()
        .success()
        .stdout(predicate::str::contains("T  ·  E  ·  R  ·  S  ·  E"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn help_flag() {
    terse()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("proxy"))
        .stdout(predicate::str::contains("compress"));
}

#[test]
fn version_flag() {
    terse()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// ── config ────────────────────────────────────────────────────────────────────

#[test]
fn config_shows_effective_values() {
    terse()
        .arg("config")
        .assert()
        .success()
        .stdout(predicate::str::contains("mode:"))
        .stdout(predicate::str::contains("tokenizer:"))
        .stdout(predicate::str::contains("proxy.port:"))
        .stdout(predicate::str::contains("config file:"));
}

// ── compress ──────────────────────────────────────────────────────────────────

// General input (used for most tests — just needs to be non-empty)
const VERBOSE_TEXT: &str =
    "I was wondering if you could help me with this problem. \
     I'd be happy to assist. Thank you so much for your help!";

// Clearly compressible assistant text (boilerplate openers + closers)
const ASSISTANT_BOILERPLATE: &str =
    "Certainly! I'd be happy to help you with that. \
     Here is the information you requested. \
     I hope this helps! Let me know if you have any other questions.";

#[test]
fn compress_text_via_stdin() {
    terse()
        .args(["compress", "--tokenizer", "approximation"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn compress_text_is_shorter_than_input() {
    let output = terse()
        .args(["compress", "--tokenizer", "approximation"])
        .write_stdin(ASSISTANT_BOILERPLATE)
        .output()
        .unwrap();

    let compressed = String::from_utf8_lossy(&output.stdout);
    assert!(
        compressed.len() < ASSISTANT_BOILERPLATE.len(),
        "compressed ({} chars) should be shorter than input ({} chars)",
        compressed.len(),
        ASSISTANT_BOILERPLATE.len()
    );
}

#[test]
fn compress_history_json_via_stdin() {
    let history = r#"[
        {"role": "user", "content": "I was wondering if you could please help me with this."},
        {"role": "assistant", "content": "Certainly! I'd be happy to help you with that."}
    ]"#;

    terse()
        .args(["compress", "--tokenizer", "approximation"])
        .write_stdin(history)
        .assert()
        .success()
        .stdout(predicate::str::contains("role"))
        .stdout(predicate::str::contains("content"));
}

#[test]
fn compress_history_preserves_structure() {
    let history = r#"[
        {"role": "user", "content": "Can you help me?"},
        {"role": "assistant", "content": "Certainly! I'd be happy to help."}
    ]"#;

    let output = terse()
        .args(["compress", "--tokenizer", "approximation"])
        .write_stdin(history)
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");

    assert!(json.is_array());
    assert_eq!(json.as_array().unwrap().len(), 2);
    assert_eq!(json[0]["role"], "user");
    assert_eq!(json[1]["role"], "assistant");
}

#[test]
fn compress_with_stats_flag_prints_to_stderr() {
    terse()
        .args(["compress", "--stats", "--tokenizer", "approximation"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success()
        .stderr(predicate::str::contains("→"))
        .stderr(predicate::str::contains("saved"));
}

#[test]
fn compress_with_mode_flag() {
    terse()
        .args(["compress", "--mode", "trim", "--tokenizer", "approximation"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success();
}

#[test]
fn compress_invalid_mode_fails() {
    terse()
        .args(["compress", "--mode", "turbo"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown mode"));
}

#[test]
fn compress_invalid_tokenizer_fails() {
    terse()
        .args(["compress", "--tokenizer", "magic"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown tokenizer"));
}

#[test]
fn compress_output_to_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    terse()
        .args(["compress", "--tokenizer", "approximation", "-o", tmp.path().to_str().unwrap()])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(!content.is_empty());
}

// ── stats ─────────────────────────────────────────────────────────────────────

#[test]
fn stats_text_via_stdin() {
    terse()
        .args(["stats", "--tokenizer", "approximation"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success()
        .stdout(predicate::str::contains("→"))
        .stdout(predicate::str::contains("saved"));
}

#[test]
fn stats_history_via_stdin() {
    let history = r#"[
        {"role": "user", "content": "I was wondering if you could help me with this."},
        {"role": "assistant", "content": "Certainly! I'd be happy to help you with that."}
    ]"#;

    terse()
        .args(["stats", "--tokenizer", "approximation"])
        .write_stdin(history)
        .assert()
        .success()
        .stdout(predicate::str::contains("messages"))
        .stdout(predicate::str::contains("→"));
}

// ── diff ──────────────────────────────────────────────────────────────────────

#[test]
fn diff_text_via_stdin() {
    terse()
        .args(["diff", "--tokenizer", "approximation"])
        .write_stdin(VERBOSE_TEXT)
        .assert()
        .success()
        .stdout(predicate::str::contains("--- original"))
        .stdout(predicate::str::contains("+++ compressed"));
}

#[test]
fn diff_history_via_stdin() {
    let history = r#"[
        {"role": "user", "content": "I was wondering if you could help me with this problem."},
        {"role": "assistant", "content": "Certainly! I'd be happy to help you with that."}
    ]"#;

    terse()
        .args(["diff", "--tokenizer", "approximation"])
        .write_stdin(history)
        .assert()
        .success()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("+++"));
}

// ── install / uninstall ───────────────────────────────────────────────────────

#[test]
fn install_writes_hook_to_rc_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    terse()
        .args(["install", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(content.contains("# terse-install"));
    assert!(content.contains("ANTHROPIC_BASE_URL"));
    assert!(content.contains("terse proxy"));
}

#[test]
fn install_is_idempotent() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    terse()
        .args(["install", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    // Second install should not add a second block
    terse()
        .args(["install", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("already installed"));

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    assert_eq!(
        content.matches("# terse-install\n").count(),
        1,
        "hook block should appear exactly once"
    );
}

#[test]
fn uninstall_removes_hook() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    terse()
        .args(["install", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    terse()
        .args(["uninstall", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    assert!(!content.contains("# terse-install"), "hook should be removed");
    assert!(!content.contains("ANTHROPIC_BASE_URL"), "hook body should be removed");
}

#[test]
fn uninstall_with_no_hook_reports_gracefully() {
    let tmp = tempfile::NamedTempFile::new().unwrap();

    terse()
        .args(["uninstall", "--rc-file", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("not found"));
}

// ── proxy ─────────────────────────────────────────────────────────────────────

#[test]
fn proxy_stop_with_no_pid_file_fails_gracefully() {
    terse()
        .args(["proxy", "stop", "--pid-file", "/tmp/terse-test-nonexistent.pid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No proxy running"));
}
