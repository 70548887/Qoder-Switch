use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProxyConfig {
    pub port: u16,
    pub target_domains: Vec<String>,
    pub auto_rotate: bool,
    pub rotate_strategy: RotateStrategy,
    pub auto_start: bool,
    pub max_retry: u32,
    pub language: String,
    pub account_pool_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RotateStrategy {
    Sequential,
    Random,
    Priority,
    LeastUsed,
    ByExpiry,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port: 5888,
            target_domains: vec![
                "center.qoder.sh".to_string(),
                "openapi.qoder.sh".to_string(),
            ],
            auto_rotate: true,
            rotate_strategy: RotateStrategy::Sequential,
            auto_start: false,
            max_retry: 3,
            language: "zh".to_string(),
            account_pool_url: "http://124.223.41.64/api/fetch-account".to_string(),
        }
    }
}

impl ProxyConfig {
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        std::fs::write(path, data)
            .map_err(|e| format!("写入配置失败: {}", e))?;
        Ok(())
    }
}
