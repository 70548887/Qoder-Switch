use crate::auth::token::TokenManager;
use crate::config::ProxyConfig;
use crate::logger::{RequestLogger, TrafficLogger};
use crate::metrics::ProxyMetrics;
use crate::proxy::cert::CertManager;
use crate::proxy::server::ProxyServer;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub token_manager: Arc<TokenManager>,
    pub cert_manager: CertManager,
    pub proxy_server: Arc<RwLock<Option<ProxyServer>>>,
    pub proxy_port: u16,
    pub metrics: Arc<ProxyMetrics>,
    pub logger: Arc<RequestLogger>,
    pub traffic_logger: Arc<TrafficLogger>,
    pub config: Arc<RwLock<ProxyConfig>>,
    pub config_path: std::path::PathBuf,
}

impl AppState {
    pub fn new(data_dir: &std::path::Path, port: u16) -> Self {
        let token_path = data_dir.join("proxy-tokens.json");
        let token_manager = Arc::new(TokenManager::new(token_path));
        let cert_manager = CertManager::new(data_dir);

        let config_path = data_dir.join("proxy-config.json");
        let config = ProxyConfig::load(&config_path);

        let target_domains = if config.target_domains.is_empty() {
            vec!["center.qoder.sh".to_string(), "openapi.qoder.sh".to_string()]
        } else {
            config.target_domains.clone()
        };

        let proxy_port = if config.port > 0 { config.port } else { port };

        Self {
            token_manager,
            cert_manager,
            proxy_server: Arc::new(RwLock::new(None)),
            proxy_port,
            metrics: Arc::new(ProxyMetrics::new()),
            logger: Arc::new(RequestLogger::new(200, target_domains)),
            traffic_logger: Arc::new(TrafficLogger::new(data_dir)),
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }
}
