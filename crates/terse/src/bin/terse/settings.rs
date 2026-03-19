use std::path::PathBuf;

#[derive(serde::Deserialize, Default)]
pub struct TerseConfig {
    /// Compression mode: "trim", "compress", or "rewrite"
    pub mode: Option<String>,
    /// Tokenizer to use: "tiktoken" or "approximation"
    pub tokenizer: Option<String>,
    #[serde(default)]
    pub proxy: ProxyConfig,
}

#[derive(serde::Deserialize, Default)]
pub struct ProxyConfig {
    pub port: Option<u16>,
}

const DEFAULT_CONFIG_JSON: &str = r#"{
  "mode": "trim",
  "tokenizer": "tiktoken",
  "proxy": {
    "port": 3847
  }
}
"#;

pub fn load_config() -> TerseConfig {
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
