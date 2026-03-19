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
        tokio::spawn(async move {
            write_proxy_stats(&session_id, &stats).await;
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

    let messages_val = json.get("messages").ok_or("no messages field")?.clone();

    let messages_val = {
        let mut arr = messages_val.as_array().ok_or("messages is not an array")?.clone();
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
