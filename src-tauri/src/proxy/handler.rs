use crate::auth::cosy::build_cosy_headers;
use crate::auth::token::TokenManager;
use crate::config::ProxyConfig;
use crate::logger::RequestLogger;
use crate::metrics::ProxyMetrics;
use bytes::Bytes;
use http_body_util::BodyExt;
use hudsucker::{HttpContext, HttpHandler, RequestOrResponse, Body};
use hyper::{Request, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct QoderProxyHandler {
    pub token_manager: Arc<TokenManager>,
    pub auto_rotate: Arc<RwLock<bool>>,
    pub metrics: Arc<ProxyMetrics>,
    pub logger: Arc<RequestLogger>,
    pub config: Arc<RwLock<ProxyConfig>>,
}

impl HttpHandler for QoderProxyHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        let host = req.uri().host().unwrap_or("").to_string();
        let path = req.uri().path().to_string();
        let method = req.method().to_string();
        let is_target = self.logger.is_target_domain(&host).await;

        // 记录域名
        self.logger.record_domain(host.clone()).await;

        let mut current_label: Option<String> = None;

        let result = if is_target {
            if let Some(account) = self.token_manager.get_current().await {
                current_label = Some(account.name.clone());

                // 读取 body
                let (parts, body) = req.into_parts();
                let body_bytes = body.collect().await
                    .map(|b| b.to_bytes())
                    .unwrap_or_else(|_| Bytes::new());
                let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                self.metrics.record_request(true, body_bytes.len() as u64);

                // 重新组装请求 body
                let new_body = Body::from(body_bytes);
                let mut req = Request::from_parts(parts, new_body);

                if let Ok(headers) = build_cosy_headers(
                    &account.token,
                    &account.user_id,
                    &account.name,
                    &account.email,
                    &path,
                    &body_str,
                ) {
                    let h = req.headers_mut();
                    if let Ok(v) = headers.authorization.parse() { h.insert("authorization", v); }
                    if let Ok(v) = headers.cosy_user.parse() { h.insert("cosy-user", v); }
                    if let Ok(v) = headers.cosy_key.parse() { h.insert("cosy-key", v); }
                    if let Ok(v) = headers.cosy_date.parse() { h.insert("cosy-date", v); }
                    if let Ok(v) = headers.request_id.parse() { h.insert("x-request-id", v); }
                }

                RequestOrResponse::Request(req)
            } else {
                RequestOrResponse::Request(req)
            }
        } else {
            self.metrics.record_request(false, 0);
            RequestOrResponse::Request(req)
        };

        // 记录日志
        let entry = crate::logger::RequestLogEntry {
            id: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            method,
            host,
            path,
            status: None,
            injected: is_target,
            token_label: current_label,
        };
        self.logger.log_request(entry).await;

        result
    }

    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        resp: Response<Body>,
    ) -> Response<Body> {
        let status = resp.status().as_u16();
        if (status == 401 || status == 403) && *self.auto_rotate.read().await {
            log::warn!("收到 {} 响应，自动轮换 Token", status);
            self.metrics.record_auth_failure();
            self.metrics.record_rotation();
            let strategy = self.config.read().await.rotate_strategy.clone();
            self.token_manager.rotate_with_strategy(strategy).await;
        }
        resp
    }
}
