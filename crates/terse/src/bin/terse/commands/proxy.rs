use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use terse::{compress_history, types::{CompressConfig, Message, MessageContent}};

use crate::cli::{ProxyArgs, ProxyCommand};
use crate::settings::TerseConfig;
use crate::utils::{default_pid_file, make_config};

// ── helpers ───────────────────────────────────────────────────────────────────

fn unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn relative_time(secs_ago: u64) -> String {
    if secs_ago < 60       { "just now".to_string() }
    else if secs_ago < 3600  { format!("{} min ago", secs_ago / 60) }
    else if secs_ago < 86400 { format!("{} hr ago", secs_ago / 3600) }
    else                     { format!("{} days ago", secs_ago / 86400) }
}

fn pid_is_running(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn terse_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".terse")
}

// ── session tracking (in-memory + proxy.json) ─────────────────────────────────

struct SessionEntry {
    first_seen: u64,
    last_seen: u64,
}

// ── proxy state ───────────────────────────────────────────────────────────────

struct ProxyState {
    config: CompressConfig,
    client: reqwest::Client,
    port: u16,
    started_at: u64,
    proxy_json_path: PathBuf,
    home: PathBuf,
    sessions: Mutex<HashMap<String, SessionEntry>>,
}

impl ProxyState {
    /// Update first_seen/last_seen for a session and flush to proxy.json.
    fn update_session(&self, session_id: &str) {
        let now = unix_ts();
        let mut sessions = self.sessions.lock().unwrap();
        let entry = sessions.entry(session_id.to_string()).or_insert(SessionEntry {
            first_seen: now,
            last_seen: now,
        });
        entry.last_seen = now;
        self.flush_proxy_json(&sessions);
    }

    fn flush_proxy_json(&self, sessions: &HashMap<String, SessionEntry>) {
        let sessions_val: serde_json::Map<String, serde_json::Value> = sessions
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::json!({
                "first_seen": v.first_seen,
                "last_seen":  v.last_seen,
            })))
            .collect();
        let doc = serde_json::json!({
            "port":       self.port,
            "started_at": self.started_at,
            "sessions":   sessions_val,
        });
        let _ = std::fs::write(
            &self.proxy_json_path,
            serde_json::to_string_pretty(&doc).unwrap(),
        );
    }
}

// ── subcommand handlers ───────────────────────────────────────────────────────

pub fn run_proxy(args: &ProxyArgs, cfg: &TerseConfig) {
    let dir = terse_dir();

    match &args.action {
        Some(ProxyCommand::Stop { pid_file }) => {
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
                    let _ = std::fs::remove_file(dir.join("proxy.json"));
                    eprintln!("terse proxy stopped (pid {})", pid);
                }
                _ => {
                    eprintln!("Failed to stop proxy (pid {}). Already stopped?", pid);
                    let _ = std::fs::remove_file(&path);
                    let _ = std::fs::remove_file(dir.join("proxy.json"));
                    std::process::exit(1);
                }
            }
            return;
        }

        Some(ProxyCommand::Status { pid_file }) => {
            let path = pid_file.clone().unwrap_or_else(default_pid_file);
            let pid_opt: Option<u32> = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| s.trim().parse().ok());

            let proxy_json: Option<serde_json::Value> =
                std::fs::read_to_string(dir.join("proxy.json"))
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok());

            match pid_opt {
                None => {
                    eprintln!("terse proxy   not running");
                    std::process::exit(0);
                }
                Some(pid) => {
                    if pid_is_running(pid) {
                        eprintln!("terse proxy   running (pid {})", pid);
                    } else {
                        eprintln!("terse proxy   stopped (stale pid file)");
                    }
                }
            }

            if let Some(ref doc) = proxy_json {
                let port       = doc["port"].as_u64().unwrap_or(3847);
                let started_at = doc["started_at"].as_u64().unwrap_or(0);
                let now        = unix_ts();
                eprintln!("port          {}", port);
                eprintln!("started       {}", relative_time(now.saturating_sub(started_at)));

                if let Some(sessions) = doc["sessions"].as_object() {
                    if sessions.is_empty() {
                        eprintln!("\nNo sessions yet.");
                    } else {
                        eprintln!("\nSessions ({}):", sessions.len());
                        let mut entries: Vec<_> = sessions.iter().collect();
                        entries.sort_by(|a, b| {
                            b.1["last_seen"].as_u64().unwrap_or(0)
                                .cmp(&a.1["last_seen"].as_u64().unwrap_or(0))
                        });
                        for (sid, entry) in entries {
                            let last_seen = entry["last_seen"].as_u64().unwrap_or(0);
                            let stats_line = session_stats_line(&dir, sid);
                            eprintln!(
                                "  {}   last seen {}   {}",
                                sid,
                                relative_time(now.saturating_sub(last_seen)),
                                stats_line,
                            );
                        }
                    }
                }
            }
            return;
        }

        None => {}
    }

    // ── start proxy ───────────────────────────────────────────────────────────
    let port       = args.port.or(cfg.proxy.port).unwrap_or(3847);
    let config     = make_config(args.mode.as_deref(), args.tokenizer.as_deref(), cfg);
    let pid_file   = args.pid_file.clone().unwrap_or_else(default_pid_file);
    let started_at = unix_ts();
    let home       = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));

    if let Some(parent) = pid_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Plain PID file — shell hook reads this with `cat`
    std::fs::write(&pid_file, std::process::id().to_string()).unwrap_or_else(|e| {
        eprintln!("warning: could not write pid file {}: {}", pid_file.display(), e);
    });

    // proxy.json — port, start time, live session map
    let proxy_json_path = dir.join("proxy.json");
    let initial = serde_json::json!({ "port": port, "started_at": started_at, "sessions": {} });
    let _ = std::fs::write(&proxy_json_path, serde_json::to_string_pretty(&initial).unwrap());

    eprintln!("terse proxy  http://localhost:{}", port);
    eprintln!("Stop with:   terse proxy stop");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(proxy_serve(port, config, pid_file, proxy_json_path, home, started_at));
}

fn session_stats_line(dir: &PathBuf, session_id: &str) -> String {
    let path = dir.join("sessions").join("claude").join(format!("{}.json", session_id));
    let Ok(content) = std::fs::read_to_string(&path) else { return String::new() };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&content) else { return String::new() };
    let t     = &doc["total"];
    let calls = t["calls"].as_u64().unwrap_or(0);
    let saved = t["saved_tokens"].as_u64().unwrap_or(0);
    let pct   = t["saved_percent"].as_i64().unwrap_or(0);
    format!("{} calls  {} tokens saved ({}%)", calls, saved, pct)
}

// ── axum server ───────────────────────────────────────────────────────────────

async fn proxy_serve(
    port: u16,
    config: CompressConfig,
    pid_file: PathBuf,
    proxy_json_path: PathBuf,
    home: PathBuf,
    started_at: u64,
) {
    use axum::{Router, routing::any};
    use std::sync::Arc;

    let state = Arc::new(ProxyState {
        config,
        client: reqwest::Client::new(),
        port,
        started_at,
        proxy_json_path: proxy_json_path.clone(),
        home,
        sessions: Mutex::new(HashMap::new()),
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
        result = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        ) => {
            if let Err(e) = result { eprintln!("terse proxy error: {}", e); }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("terse proxy shutting down");
        }
    }

    let _ = std::fs::remove_file(&pid_file);
    let _ = std::fs::remove_file(&proxy_json_path);
}

// ── request handler ───────────────────────────────────────────────────────────

struct ProxyCallStats {
    messages: usize,
    original_tokens: usize,
    compressed_tokens: usize,
    saved_tokens: usize,
    saved_percent: i64,
}

async fn proxy_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<ProxyState>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    request: axum::http::Request<axum::body::Body>,
) -> axum::http::Response<axum::body::Body> {
    use axum::http::StatusCode;

    // Each Claude Code process opens its own TCP connection; the source port
    // uniquely identifies the session for the lifetime of that process.
    let session_id = format!("conn-{}", addr.port());

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

    // Extract Claude's session ID from metadata.user_id (double-encoded JSON).
    // Falls back to the TCP connection port if absent.
    let session_id = if is_messages {
        extract_claude_session_id(&body_bytes).unwrap_or(session_id)
    } else {
        session_id
    };

    // Track the session on every /v1/messages request (compressed or not).
    if is_messages {
        state.update_session(&session_id);
    }

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

    let upstream_url = format!(
        "https://api.anthropic.com{}",
        parts.uri.path_and_query().map(|p| p.as_str()).unwrap_or("")
    );

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

    if let Some(stats) = call_stats {
        let home = state.home.clone();
        let sid  = session_id.clone();
        tokio::spawn(async move {
            write_stats_to(home, &sid, &stats).await;
        });
    }

    let status  = upstream.status();
    let headers = upstream.headers().clone();
    let stream  = upstream.bytes_stream();

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

// ── compression ───────────────────────────────────────────────────────────────

/// Extract Claude Code's session ID from the request body.
/// Claude Code encodes it as: metadata.user_id = JSON string containing { session_id: "..." }
fn extract_claude_session_id(body: &bytes::Bytes) -> Option<String> {
    let json: serde_json::Value = serde_json::from_slice(body).ok()?;
    let user_id_str = json["metadata"]["user_id"].as_str()?;
    let user_id: serde_json::Value = serde_json::from_str(user_id_str).ok()?;
    user_id["session_id"].as_str().map(|s| s.to_string())
}

fn flatten_content_blocks(val: &serde_json::Value) -> Option<String> {
    let blocks = val.as_array()?;
    let text: String = blocks
        .iter()
        .filter_map(|b| {
            if b["type"].as_str() == Some("text") { b["text"].as_str() } else { None }
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

    let messages_raw = json.get("messages").ok_or("no messages field")?
        .as_array().ok_or("messages is not an array")?.clone();

    // Flatten block-array content to strings for compression
    let mut arr: Vec<serde_json::Value> = messages_raw.into_iter().map(|mut msg| {
        if let Some(content) = msg.get("content") {
            if content.is_array() {
                if let Some(flat) = flatten_content_blocks(content) {
                    msg["content"] = serde_json::Value::String(flat);
                }
            }
        }
        msg
    }).collect();

    // Preserve the last user message unchanged — it's the current prompt being sent.
    // Only compress history (prior turns). The current message will be eligible
    // for compression on the *next* turn when it becomes part of history.
    let current_message = if arr.last().and_then(|m| m["role"].as_str()) == Some("user") {
        arr.pop()
    } else {
        None
    };

    let messages: Vec<Message> =
        serde_json::from_value(serde_json::Value::Array(arr)).map_err(|e| e.to_string())?;

    let original_count = messages.len() + current_message.is_some() as usize;
    let result = compress_history(messages, config).map_err(|e| format!("{}", e))?;

    let stats = ProxyCallStats {
        messages:          original_count,
        original_tokens:   result.stats.total_original_tokens,
        compressed_tokens: result.stats.total_compressed_tokens,
        saved_tokens:      result.stats.total_saved_tokens,
        saved_percent:     result.stats.total_saved_percent,
    };

    let mut compressed: Vec<serde_json::Value> = result
        .messages
        .iter()
        .map(|m| {
            let content = match &m.content {
                Some(MessageContent::Text(s))   => serde_json::Value::String(s.clone()),
                Some(MessageContent::Blocks(b)) => serde_json::Value::Array(b.clone()),
                Some(MessageContent::Null) | None => serde_json::Value::Null,
            };
            let mut obj = serde_json::Map::new();
            obj.insert("role".into(),    serde_json::Value::String(m.role.clone()));
            obj.insert("content".into(), content);
            for (k, v) in &m.extra { obj.insert(k.clone(), v.clone()); }
            serde_json::Value::Object(obj)
        })
        .collect();

    if let Some(msg) = current_message { compressed.push(msg); }

    json["messages"] = serde_json::Value::Array(compressed);
    let out = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    Ok((bytes::Bytes::from(out), stats))
}

// ── per-session stats file ────────────────────────────────────────────────────

async fn write_stats_to(base: PathBuf, session_id: &str, stats: &ProxyCallStats) {
    let dir = base.join(".terse").join("sessions").join("claude");
    if tokio::fs::create_dir_all(&dir).await.is_err() { return; }

    let path = dir.join(format!("{}.json", session_id));

    let mut doc: serde_json::Value = tokio::fs::read_to_string(&path)
        .await
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({
            "session_id":    session_id,
            "terse_version": env!("CARGO_PKG_VERSION"),
            "calls":  [],
            "total":  { "calls": 0, "original_tokens": 0, "compressed_tokens": 0,
                        "saved_tokens": 0, "saved_percent": 0 }
        }));

    let ts = unix_ts();
    if let Some(calls) = doc["calls"].as_array_mut() {
        calls.push(serde_json::json!({
            "timestamp":         ts,
            "messages":          stats.messages,
            "original_tokens":   stats.original_tokens,
            "compressed_tokens": stats.compressed_tokens,
            "saved_tokens":      stats.saved_tokens,
            "saved_percent":     stats.saved_percent,
        }));
    }

    let empty = vec![];
    let calls       = doc["calls"].as_array().unwrap_or(&empty);
    let n           = calls.len();
    let total_orig: u64 = calls.iter().filter_map(|c| c["original_tokens"].as_u64()).sum();
    let total_comp: u64 = calls.iter().filter_map(|c| c["compressed_tokens"].as_u64()).sum();
    let total_saved = total_orig.saturating_sub(total_comp);
    let total_pct   = if total_orig > 0 { (total_saved * 100 / total_orig) as i64 } else { 0 };

    doc["total"] = serde_json::json!({
        "calls":             n,
        "original_tokens":   total_orig,
        "compressed_tokens": total_comp,
        "saved_tokens":      total_saved,
        "saved_percent":     total_pct,
    });

    if let Ok(s) = serde_json::to_string_pretty(&doc) {
        let _ = tokio::fs::write(&path, s).await;
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use terse::config::default_config;

    fn make_body(messages: serde_json::Value) -> bytes::Bytes {
        let body = serde_json::json!({ "model": "claude-3-5-sonnet-20241022", "messages": messages });
        bytes::Bytes::from(serde_json::to_vec(&body).unwrap())
    }

    fn get_messages(body: &bytes::Bytes) -> Vec<serde_json::Value> {
        let json: serde_json::Value = serde_json::from_slice(body).unwrap();
        json["messages"].as_array().unwrap().clone()
    }

    // ── extract_claude_session_id ─────────────────────────────────────────────

    #[test]
    fn extracts_session_id_from_metadata() {
        let body = serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 1,
            "messages": [],
            "metadata": {
                "user_id": "{\"device_id\":\"abc\",\"account_uuid\":\"def\",\"session_id\":\"0d08acc7-30d0-4fbb-a689-ff0f6c7f0d94\"}"
            }
        });
        let bytes = bytes::Bytes::from(serde_json::to_vec(&body).unwrap());
        assert_eq!(
            extract_claude_session_id(&bytes).as_deref(),
            Some("0d08acc7-30d0-4fbb-a689-ff0f6c7f0d94")
        );
    }

    #[test]
    fn returns_none_when_metadata_absent() {
        let body = serde_json::json!({ "model": "claude-haiku-4-5-20251001", "messages": [] });
        let bytes = bytes::Bytes::from(serde_json::to_vec(&body).unwrap());
        assert!(extract_claude_session_id(&bytes).is_none());
    }

    #[test]
    fn returns_none_when_user_id_not_valid_json() {
        let body = serde_json::json!({
            "messages": [],
            "metadata": { "user_id": "not-json" }
        });
        let bytes = bytes::Bytes::from(serde_json::to_vec(&body).unwrap());
        assert!(extract_claude_session_id(&bytes).is_none());
    }

    // ── proxy_compress ────────────────────────────────────────────────────────

    #[test]
    fn last_user_message_preserved_verbatim() {
        let boilerplate = "Certainly! I'd be happy to help you with that. Here is a detailed explanation of the topic you asked about.";
        let current = "What is the meaning of life?";
        let body = make_body(serde_json::json!([
            { "role": "user",      "content": "Hello" },
            { "role": "assistant", "content": boilerplate },
            { "role": "user",      "content": current },
        ]));

        let (out, _) = proxy_compress(&body, &default_config()).unwrap();
        let messages = get_messages(&out);

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[2]["role"].as_str().unwrap(), "user");
        assert_eq!(messages[2]["content"].as_str().unwrap(), current);
    }

    #[test]
    fn prior_assistant_messages_compressed() {
        let boilerplate = "Certainly! I'd be happy to help you with that. Here is a detailed explanation of the topic you asked about. I hope this helps!";
        let body = make_body(serde_json::json!([
            { "role": "user",      "content": "Hello" },
            { "role": "assistant", "content": boilerplate },
            { "role": "user",      "content": "Current question" },
        ]));

        let (out, stats) = proxy_compress(&body, &default_config()).unwrap();
        let messages = get_messages(&out);

        let compressed_assistant = messages[1]["content"].as_str().unwrap();
        assert!(
            compressed_assistant.len() < boilerplate.len(),
            "expected assistant message to be compressed: {:?}", compressed_assistant
        );
        assert!(stats.saved_tokens > 0);
    }

    #[test]
    fn last_non_user_message_is_compressed() {
        let boilerplate = "Certainly! I'd be happy to help you with that. Here is a detailed explanation. I hope this helps!";
        let body = make_body(serde_json::json!([
            { "role": "user",      "content": "Hello" },
            { "role": "assistant", "content": boilerplate },
        ]));

        let (out, _) = proxy_compress(&body, &default_config()).unwrap();
        let messages = get_messages(&out);

        let compressed = messages[1]["content"].as_str().unwrap();
        assert!(
            compressed.len() < boilerplate.len(),
            "expected last assistant message to be compressed: {:?}", compressed
        );
    }

    #[test]
    fn block_format_content_flattened_for_compression() {
        let body = make_body(serde_json::json!([
            { "role": "user", "content": [{ "type": "text", "text": "Hello there" }] },
            { "role": "assistant", "content": "Certainly! I'd be happy to help." },
            { "role": "user", "content": "Current question" },
        ]));

        let (out, _) = proxy_compress(&body, &default_config()).unwrap();
        let messages = get_messages(&out);
        assert_eq!(messages[2]["content"].as_str().unwrap(), "Current question");
    }

    // ── write_stats_to ────────────────────────────────────────────────────────

    fn dummy_stats() -> ProxyCallStats {
        ProxyCallStats {
            messages: 3, original_tokens: 100, compressed_tokens: 70,
            saved_tokens: 30, saved_percent: 30,
        }
    }

    #[tokio::test]
    async fn session_file_written_with_session_id_and_version() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "abc-123", &dummy_stats()).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/sessions/claude/abc-123.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["session_id"].as_str().unwrap(), "abc-123");
        assert_eq!(json["terse_version"].as_str().unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn session_file_accumulates_calls() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "sess-acc", &dummy_stats()).await;
        write_stats_to(tmp.path().to_path_buf(), "sess-acc", &dummy_stats()).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/sessions/claude/sess-acc.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["total"]["calls"].as_u64().unwrap(), 2);
        assert_eq!(json["total"]["saved_tokens"].as_u64().unwrap(), 60);
    }

    #[tokio::test]
    async fn session_id_not_overwritten_on_subsequent_calls() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "sid-first", &dummy_stats()).await;
        write_stats_to(tmp.path().to_path_buf(), "sid-first", &dummy_stats()).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/sessions/claude/sid-first.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        // session_id in the JSON always matches the file key
        assert_eq!(json["session_id"].as_str().unwrap(), "sid-first");
        assert_eq!(json["total"]["calls"].as_u64().unwrap(), 2);
    }

    // ── session tracking (ProxyState) ─────────────────────────────────────────

    #[test]
    fn update_session_sets_first_and_last_seen() {
        let tmp = tempfile::tempdir().unwrap();
        let state = ProxyState {
            config: default_config(),
            client: reqwest::Client::new(),
            port: 3847,
            started_at: unix_ts(),
            proxy_json_path: tmp.path().join("proxy.json"),
            home: tmp.path().to_path_buf(),
            sessions: Mutex::new(HashMap::new()),
        };

        state.update_session("my-session");

        let sessions = state.sessions.lock().unwrap();
        let entry = sessions.get("my-session").unwrap();
        assert_eq!(entry.first_seen, entry.last_seen);
    }

    #[test]
    fn update_session_writes_proxy_json() {
        let tmp = tempfile::tempdir().unwrap();
        let state = ProxyState {
            config: default_config(),
            client: reqwest::Client::new(),
            port: 3847,
            started_at: unix_ts(),
            proxy_json_path: tmp.path().join("proxy.json"),
            home: tmp.path().to_path_buf(),
            sessions: Mutex::new(HashMap::new()),
        };

        state.update_session("my-session");

        let content = std::fs::read_to_string(tmp.path().join("proxy.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json["sessions"]["my-session"]["first_seen"].as_u64().is_some());
        assert!(json["sessions"]["my-session"]["last_seen"].as_u64().is_some());
        assert_eq!(json["port"].as_u64().unwrap(), 3847);
    }

    #[test]
    fn update_session_preserves_first_seen_on_second_call() {
        let tmp = tempfile::tempdir().unwrap();
        let state = ProxyState {
            config: default_config(),
            client: reqwest::Client::new(),
            port: 3847,
            started_at: unix_ts(),
            proxy_json_path: tmp.path().join("proxy.json"),
            home: tmp.path().to_path_buf(),
            sessions: Mutex::new(HashMap::new()),
        };

        state.update_session("my-session");
        let first = state.sessions.lock().unwrap().get("my-session").unwrap().first_seen;

        std::thread::sleep(std::time::Duration::from_millis(10));
        state.update_session("my-session");

        let sessions = state.sessions.lock().unwrap();
        let entry = sessions.get("my-session").unwrap();
        assert_eq!(entry.first_seen, first, "first_seen must not change on subsequent updates");
        // last_seen may or may not advance within 10ms depending on clock resolution
    }
}
