use std::path::PathBuf;

use crate::settings::TerseConfig;

pub fn run_config(cfg: &TerseConfig) {
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
