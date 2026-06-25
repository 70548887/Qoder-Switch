use crate::auth::token::TokenManager;
use crate::config::ProxyConfig;
use crate::logger::RequestLogger;
use crate::metrics::ProxyMetrics;
use crate::proxy::handler::QoderProxyHandler;
use hudsucker::certificate_authority::RcgenAuthority;
use hudsucker::Proxy;
use rcgen::{Issuer, KeyPair};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProxyServer {
    pub port: u16,
    pub token_manager: Arc<TokenManager>,
    pub auto_rotate: Arc<RwLock<bool>>,
    pub running: Arc<RwLock<bool>>,
    pub metrics: Arc<ProxyMetrics>,
    pub logger: Arc<RequestLogger>,
    pub config: Arc<RwLock<ProxyConfig>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ProxyServer {
    pub fn new(port: u16, token_manager: Arc<TokenManager>, metrics: Arc<ProxyMetrics>, logger: Arc<RequestLogger>, config: Arc<RwLock<ProxyConfig>>) -> Self {
        Self {
            port,
            token_manager,
            auto_rotate: Arc::new(RwLock::new(true)),
            running: Arc::new(RwLock::new(false)),
            metrics,
            logger,
            config,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self, ca_key_pem: &str, ca_cert_pem: &str) -> Result<(), String> {
        let key_pair = KeyPair::from_pem(ca_key_pem)
            .map_err(|e| format!("解析 CA 私钥失败: {}", e))?;
        let issuer = Issuer::from_ca_cert_pem(ca_cert_pem, key_pair)
            .map_err(|e| format!("解析 CA 证书失败: {}", e))?;
        let ca = RcgenAuthority::new(issuer, 1000, hudsucker::rustls::crypto::aws_lc_rs::default_provider());

        let handler = QoderProxyHandler {
            token_manager: self.token_manager.clone(),
            auto_rotate: self.auto_rotate.clone(),
            metrics: self.metrics.clone(),
            logger: self.logger.clone(),
            config: self.config.clone(),
        };

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(tx);

        let proxy = Proxy::builder()
            .with_addr(([127, 0, 0, 1], self.port).into())
            .with_ca(ca)
            .with_rustls_connector(hudsucker::rustls::crypto::aws_lc_rs::default_provider())
            .with_http_handler(handler)
            .with_graceful_shutdown(async move {
                rx.await.ok();
            })
            .build()
            .map_err(|e| format!("构建代理失败: {}", e))?;

        *self.running.write().await = true;
        log::info!("代理服务启动: 127.0.0.1:{}", self.port);

        let running = self.running.clone();
        tokio::spawn(async move {
            if let Err(e) = proxy.start().await {
                log::error!("代理服务错误: {}", e);
            }
            *running.write().await = false;
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        *self.running.write().await = false;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}
