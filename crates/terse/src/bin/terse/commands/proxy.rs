use std::path::PathBuf;

use terse::{compress_history, types::{CompressConfig, Message, MessageContent}};

use crate::cli::{ProxyArgs, ProxyCommand};
use crate::settings::TerseConfig;
use crate::utils::{default_pid_file, make_config};

pub fn run_proxy(args: &ProxyArgs, cfg: &TerseConfig) {
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
        let session_id = state.session_id.clone();
        let claude_session_id = parts.headers
            .get("anthropic-client-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        tokio::spawn(async move {
            write_proxy_stats(&session_id, &stats, claude_session_id.as_deref()).await;
        });
    }

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
        messages: original_count,
        original_tokens: result.stats.total_original_tokens,
        compressed_tokens: result.stats.total_compressed_tokens,
        saved_tokens: result.stats.total_saved_tokens,
        saved_percent: result.stats.total_saved_percent,
    };

    let mut compressed: Vec<serde_json::Value> = result
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

    // Append current user message unchanged at the end
    if let Some(msg) = current_message {
        compressed.push(msg);
    }

    json["messages"] = serde_json::Value::Array(compressed);

    let out = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    Ok((bytes::Bytes::from(out), stats))
}

async fn write_proxy_stats(session_id: &str, stats: &ProxyCallStats, claude_session_id: Option<&str>) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let base = std::path::PathBuf::from(home);
    write_stats_to(base, session_id, stats, claude_session_id).await;
}

async fn write_stats_to(base: std::path::PathBuf, session_id: &str, stats: &ProxyCallStats, claude_session_id: Option<&str>) {
    let dir = base.join(".terse").join("claude");

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
                "terse_version": env!("CARGO_PKG_VERSION"),
                "claude_session_id": claude_session_id,
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
        // Last message is verbatim
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

        // The assistant message should have been compressed (shorter)
        let compressed_assistant = messages[1]["content"].as_str().unwrap();
        assert!(
            compressed_assistant.len() < boilerplate.len(),
            "expected assistant message to be compressed: {:?}", compressed_assistant
        );
        assert!(stats.saved_tokens > 0);
    }

    #[test]
    fn last_non_user_message_is_compressed() {
        // When last message is assistant, it should be compressed normally (no preservation)
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

    fn dummy_stats() -> ProxyCallStats {
        ProxyCallStats { messages: 3, original_tokens: 100, compressed_tokens: 70, saved_tokens: 30, saved_percent: 30 }
    }

    #[tokio::test]
    async fn session_file_includes_terse_version() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "sess-1", &dummy_stats(), None).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/claude/sess-1.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["terse_version"].as_str().unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn session_file_includes_claude_session_id() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "sess-2", &dummy_stats(), Some("claude-abc-123")).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/claude/sess-2.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["claude_session_id"].as_str().unwrap(), "claude-abc-123");
    }

    #[tokio::test]
    async fn session_file_claude_session_id_null_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        write_stats_to(tmp.path().to_path_buf(), "sess-3", &dummy_stats(), None).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/claude/sess-3.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json["claude_session_id"].is_null());
    }

    #[tokio::test]
    async fn session_file_metadata_not_overwritten_on_subsequent_calls() {
        let tmp = tempfile::tempdir().unwrap();
        // First call sets the metadata
        write_stats_to(tmp.path().to_path_buf(), "sess-4", &dummy_stats(), Some("original-id")).await;
        // Second call (e.g. with different session id header — shouldn't happen, but safeguard)
        write_stats_to(tmp.path().to_path_buf(), "sess-4", &dummy_stats(), Some("other-id")).await;
        let content = std::fs::read_to_string(tmp.path().join(".terse/claude/sess-4.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        // File existed on second call, so metadata from first call is preserved
        assert_eq!(json["claude_session_id"].as_str().unwrap(), "original-id");
        // But calls accumulate
        assert_eq!(json["total"]["calls"].as_u64().unwrap(), 2);
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
        // Last user message preserved verbatim (block-format last message would be preserved as-is)
        assert_eq!(messages[2]["content"].as_str().unwrap(), "Current question");
    }
}
