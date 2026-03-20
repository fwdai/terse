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

const VALID_MODES: &[&str] = &["trim", "compress", "rewrite"];
const VALID_TOKENIZERS: &[&str] = &["tiktoken", "approximation"];

pub fn load_config() -> TerseConfig {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(home).join(".terse").join("config.json");

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, DEFAULT_CONFIG_JSON);
    }

    let cfg: TerseConfig = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| {
            serde_json::from_str(&s).map_err(|e| {
                eprintln!(
                    "terse: invalid config {}: {}\nUsing defaults.",
                    path.display(), e
                );
            }).ok()
        })
        .unwrap_or_default();

    validate_config(&cfg, &path);
    cfg
}

fn validate_config(cfg: &TerseConfig, path: &PathBuf) {
    for warning in config_warnings(cfg, path) {
        eprintln!("{}", warning);
    }
}

fn config_warnings(cfg: &TerseConfig, path: &PathBuf) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(mode) = &cfg.mode {
        if !VALID_MODES.contains(&mode.as_str()) {
            warnings.push(format!(
                "terse: config {}: invalid mode {:?} — expected one of: {}\nUsing default (trim).",
                path.display(), mode, VALID_MODES.join(", ")
            ));
        }
    }
    if let Some(tokenizer) = &cfg.tokenizer {
        if !VALID_TOKENIZERS.contains(&tokenizer.as_str()) {
            warnings.push(format!(
                "terse: config {}: invalid tokenizer {:?} — expected one of: {}\nUsing default (tiktoken).",
                path.display(), tokenizer, VALID_TOKENIZERS.join(", ")
            ));
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> PathBuf { PathBuf::from("/fake/config.json") }

    #[test]
    fn valid_config_no_warnings() {
        let cfg = TerseConfig { mode: Some("trim".into()), tokenizer: Some("tiktoken".into()), proxy: Default::default() };
        assert!(config_warnings(&cfg, &path()).is_empty());
    }

    #[test]
    fn invalid_mode_produces_warning() {
        let cfg = TerseConfig { mode: Some("turbo".into()), tokenizer: None, proxy: Default::default() };
        let w = config_warnings(&cfg, &path());
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("invalid mode"));
        assert!(w[0].contains("turbo"));
        assert!(w[0].contains("trim, compress, rewrite"));
    }

    #[test]
    fn invalid_tokenizer_produces_warning() {
        let cfg = TerseConfig { mode: None, tokenizer: Some("gpt4".into()), proxy: Default::default() };
        let w = config_warnings(&cfg, &path());
        assert_eq!(w.len(), 1);
        assert!(w[0].contains("invalid tokenizer"));
        assert!(w[0].contains("gpt4"));
        assert!(w[0].contains("tiktoken, approximation"));
    }

    #[test]
    fn both_invalid_produces_two_warnings() {
        let cfg = TerseConfig { mode: Some("fast".into()), tokenizer: Some("chars".into()), proxy: Default::default() };
        assert_eq!(config_warnings(&cfg, &path()).len(), 2);
    }

    #[test]
    fn none_values_produce_no_warnings() {
        let cfg = TerseConfig { mode: None, tokenizer: None, proxy: Default::default() };
        assert!(config_warnings(&cfg, &path()).is_empty());
    }

    #[test]
    fn malformed_json_falls_back_to_default() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        std::fs::write(&config_path, b"{ not valid json }").unwrap();
        let cfg: TerseConfig = std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        assert!(cfg.mode.is_none());
        assert!(cfg.tokenizer.is_none());
    }
}
