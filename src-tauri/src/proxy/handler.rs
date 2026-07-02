use crate::auth::cosy::build_cosy_headers;
use crate::auth::token::TokenManager;
use crate::config::ProxyConfig;
use crate::logger::{RequestLogger, TrafficLogger, format_headers, is_websocket_upgrade, truncate_str};
use crate::metrics::ProxyMetrics;
use bytes::Bytes;
use http_body_util::BodyExt;
use hudsucker::{HttpContext, HttpHandler, RequestOrResponse, Body};
use hyper::{Request, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_BODY_CAPTURE: usize = 2000;

#[derive(Clone)]
pub struct QoderProxyHandler {
    pub token_manager: Arc<TokenManager>,
    pub auto_rotate: Arc<RwLock<bool>>,
    pub metrics: Arc<ProxyMetrics>,
    pub logger: Arc<RequestLogger>,
    pub traffic_logger: Arc<TrafficLogger>,
    pub config: Arc<RwLock<ProxyConfig>>,
    pub last_log_id: Arc<RwLock<u64>>,
    pub last_request_is_target: Arc<RwLock<bool>>,
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

        // 记录 is_target 状态供 handle_response 使用
        *self.last_request_is_target.write().await = is_target;

        // 记录域名
        self.logger.record_domain(host.clone()).await;

        // 捕获请求头
        let req_headers_str = format_headers(req.headers());
        let is_ws = is_websocket_upgrade(req.headers());

        let mut current_label: Option<String> = None;
        #[allow(unused_assignments)]
        let mut req_body_str = String::new();

        let result = if is_target {
            // 主动余额检查：请求前检测余额是否低于阈值
            if *self.auto_rotate.read().await {
                let threshold = self.config.read().await.balance_threshold;
                if threshold > 0 {
                    if let Some(current) = self.token_manager.get_current().await {
                        let remaining = current.quota_total.unwrap_or(u64::MAX)
                            .saturating_sub(current.quota_used.unwrap_or(0));
                        if remaining < threshold {
                            log::warn!("[auto-rotate] 当前账号 {} 余额 {} < 阈值 {}，尝试自动切换",
                                current.name, remaining, threshold);
                            if let Some(new_id) = self.token_manager.rotate_to_sufficient_balance(threshold).await {
                                self.metrics.record_rotation();
                                log::info!("[auto-rotate] 已切换到账号 {}", new_id);
                            } else {
                                log::warn!("[auto-rotate] 未找到满足阈值的账号，保持当前");
                            }
                        }
                    }
                }
            }

            if let Some(account) = self.token_manager.get_current().await {
                current_label = Some(account.name.clone());

                // 读取 body
                let (parts, body) = req.into_parts();
                let body_bytes = body.collect().await
                    .map(|b| b.to_bytes())
                    .unwrap_or_else(|_| Bytes::new());
                let body_str = String::from_utf8_lossy(&body_bytes).to_string();

                // 捕获请求体（截断）
                req_body_str = truncate_str(&body_str, MAX_BODY_CAPTURE);

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
                // 没有当前账号，但仍然记录 body
                let (parts, body) = req.into_parts();
                let body_bytes = body.collect().await
                    .map(|b| b.to_bytes())
                    .unwrap_or_else(|_| Bytes::new());
                req_body_str = truncate_str(&String::from_utf8_lossy(&body_bytes), MAX_BODY_CAPTURE);
                let new_body = Body::from(body_bytes);
                let req = Request::from_parts(parts, new_body);
                RequestOrResponse::Request(req)
            }
        } else {
            // 非目标域名：也读取 body 用于日志记录
            let (parts, body) = req.into_parts();
            let body_bytes = body.collect().await
                .map(|b| b.to_bytes())
                .unwrap_or_else(|_| Bytes::new());
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            req_body_str = truncate_str(&body_str, MAX_BODY_CAPTURE);

            self.metrics.record_request(false, body_bytes.len() as u64);

            let new_body = Body::from(body_bytes);
            let req = Request::from_parts(parts, new_body);
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
            request_headers: Some(req_headers_str),
            request_body: if req_body_str.is_empty() { None } else { Some(req_body_str) },
            response_headers: None,
            response_body: None,
            is_websocket: is_ws,
        };

        // 写入文件日志
        self.traffic_logger.log_traffic(&entry);

        let entry_id = self.logger.log_request(entry).await;

        // 记录当前请求的 log entry id 供 handle_response 使用
        *self.last_log_id.write().await = entry_id;

        result
    }

    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        resp: Response<Body>,
    ) -> Response<Body> {
        let status = resp.status().as_u16();
        let is_target = *self.last_request_is_target.read().await;

        // 401/403 自动轮换逻辑
        if (status == 401 || status == 403) && *self.auto_rotate.read().await {
            log::warn!("收到 {} 响应，自动轮换 Token", status);
            self.metrics.record_auth_failure();
            self.metrics.record_rotation();
            let strategy = self.config.read().await.rotate_strategy.clone();
            self.token_manager.rotate_with_strategy(strategy).await;
        }

        // 捕获响应头
        let resp_headers_str = format_headers(resp.headers());

        // 获取关联的请求 log entry id
        let entry_id = *self.last_log_id.read().await;

        if is_target {
            // 目标域名：收集 body 用于日志
            let (parts, body) = resp.into_parts();
            let body_bytes = body.collect().await
                .map(|b| b.to_bytes())
                .unwrap_or_else(|_| Bytes::new());
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            let resp_body_truncated = truncate_str(&body_str, MAX_BODY_CAPTURE);

            // 更新日志
            self.logger.update_response(entry_id, status, resp_headers_str.clone(), resp_body_truncated.clone()).await;
            self.traffic_logger.log_response(entry_id, status, &resp_headers_str, &resp_body_truncated);

            let new_body = Body::from(body_bytes);
            Response::from_parts(parts, new_body)
        } else {
            // 非目标域名：只记录响应头，body 直接透传
            self.logger.update_response(entry_id, status, resp_headers_str.clone(), "(non-target, body not captured)".to_string()).await;
            self.traffic_logger.log_response(entry_id, status, &resp_headers_str, "(non-target)");

            resp
        }
    }
}
