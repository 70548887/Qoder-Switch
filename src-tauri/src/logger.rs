use serde::Serialize;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

const MAX_BODY_CAPTURE: usize = 2000;
const MAX_TRAFFIC_LOG_SIZE: u64 = 50 * 1024 * 1024; // 50MB

#[derive(Serialize, Clone)]
pub struct RequestLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub method: String,
    pub host: String,
    pub path: String,
    pub status: Option<u16>,
    pub injected: bool,
    pub token_label: Option<String>,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub is_websocket: bool,
}

pub struct RequestLogger {
    logs: RwLock<VecDeque<RequestLogEntry>>,
    domains: RwLock<HashSet<String>>,
    target_domains: RwLock<Vec<String>>,
    counter: AtomicU64,
    max_entries: usize,
}

impl RequestLogger {
    pub fn new(max_entries: usize, initial_targets: Vec<String>) -> Self {
        Self {
            logs: RwLock::new(VecDeque::new()),
            domains: RwLock::new(HashSet::new()),
            target_domains: RwLock::new(initial_targets),
            counter: AtomicU64::new(0),
            max_entries,
        }
    }

    pub async fn log_request(&self, mut entry: RequestLogEntry) -> u64 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        entry.id = id;
        let mut logs = self.logs.write().await;
        if logs.len() >= self.max_entries {
            logs.pop_front();
        }
        logs.push_back(entry);
        id
    }

    pub async fn record_domain(&self, domain: String) {
        self.domains.write().await.insert(domain);
    }

    pub async fn get_logs(&self, limit: usize) -> Vec<RequestLogEntry> {
        let logs = self.logs.read().await;
        logs.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_discovered_domains(&self) -> Vec<String> {
        let domains = self.domains.read().await;
        let mut list: Vec<String> = domains.iter().cloned().collect();
        list.sort();
        list
    }

    pub async fn get_target_domains(&self) -> Vec<String> {
        self.target_domains.read().await.clone()
    }

    pub async fn set_target_domains(&self, domains: Vec<String>) {
        *self.target_domains.write().await = domains;
    }

    pub async fn is_target_domain(&self, domain: &str) -> bool {
        let targets = self.target_domains.read().await;
        let domain_lower = domain.to_lowercase();
        targets.iter().any(|d| {
            let d_lower = d.to_lowercase();
            domain_lower == d_lower || domain_lower.ends_with(&format!(".{}", d_lower))
        })
    }

    /// 更新最后一条日志的响应信息（根据 id 匹配）
    pub async fn update_response(&self, id: u64, status: u16, response_headers: String, response_body: String) {
        let mut logs = self.logs.write().await;
        // 从后往前找，效率更高
        for entry in logs.iter_mut().rev() {
            if entry.id == id {
                entry.status = Some(status);
                entry.response_headers = Some(response_headers);
                entry.response_body = Some(truncate_str(&response_body, MAX_BODY_CAPTURE));
                break;
            }
        }
    }
}

/// 截断字符串到指定字节数
pub fn truncate_str(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let truncated: String = s.chars()
            .take_while({
                let mut len = 0;
                move |c| {
                    len += c.len_utf8();
                    len <= max_bytes
                }
            })
            .collect();
        format!("{}...[truncated]", truncated)
    }
}

/// 格式化 HTTP 头为字符串
/// 注意：当前会记录包括 Authorization 在内的所有头部，仅供调试使用。
/// 生产环境建议配置脱敏选项。
pub fn format_headers(headers: &hyper::HeaderMap) -> String {
    let mut result = String::new();
    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            result.push_str(&format!("{}: {}\n", name.as_str(), v));
        } else {
            result.push_str(&format!("{}: <binary>\n", name.as_str()));
        }
    }
    result
}

/// 检测是否为 WebSocket 升级请求
pub fn is_websocket_upgrade(headers: &hyper::HeaderMap) -> bool {
    headers.get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

/// 安全截断日志文件（基于字节操作，不经过 &str 切片，避免 UTF-8 边界 panic）
fn truncate_log_file(path: &std::path::Path) {
    use std::io::{Read, Write, Seek, SeekFrom};

    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    if metadata.len() <= MAX_TRAFFIC_LOG_SIZE {
        return;
    }

    // 保留后半部分（约 25MB）
    let keep_bytes = MAX_TRAFFIC_LOG_SIZE / 2;

    if let Ok(mut file) = std::fs::File::open(path) {
        let len = metadata.len();
        let skip = len - keep_bytes;
        if file.seek(SeekFrom::Start(skip)).is_ok() {
            let mut buf = Vec::new();
            if file.read_to_end(&mut buf).is_ok() {
                // 找到第一个换行符，从完整行开始
                let start = buf.iter().position(|&b| b == b'\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                if let Ok(mut out) = std::fs::File::create(path) {
                    let _ = out.write_all(&buf[start..]);
                }
            }
        }
    }
}

/// TrafficLogger - 专门将详细流量写入文件
pub struct TrafficLogger {
    log_path: PathBuf,
}

impl TrafficLogger {
    pub fn new(data_dir: &std::path::Path) -> Self {
        let log_path = data_dir.join("proxy-traffic.log");
        Self { log_path }
    }

    /// 异步写入流量日志到文件（使用 spawn_blocking 避免阻塞 async 运行时）
    pub fn log_traffic(&self, entry: &RequestLogEntry) {
        let path = self.log_path.clone();
        let line = Self::format_entry(entry);
        tokio::task::spawn_blocking(move || {
            use std::io::Write;
            // 检查文件大小，超过限制则截断
            truncate_log_file(&path);

            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let _ = file.write_all(line.as_bytes());
            }
        });
    }

    /// 记录响应信息到文件
    pub fn log_response(&self, id: u64, status: u16, headers: &str, body: &str) {
        let path = self.log_path.clone();
        let body_truncated = truncate_str(body, MAX_BODY_CAPTURE);
        let headers_owned = headers.to_string();
        tokio::task::spawn_blocking(move || {
            use std::io::Write;
            // 先检查文件大小
            truncate_log_file(&path);

            let line = format!(
                "\n--- RESPONSE [id={}] status={} ---\n{}\n--- BODY ---\n{}\n===END===\n\n",
                id, status, headers_owned, body_truncated
            );
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                let _ = file.write_all(line.as_bytes());
            }
        });
    }

    fn format_entry(entry: &RequestLogEntry) -> String {
        let ws_tag = if entry.is_websocket { " [WebSocket]" } else { "" };
        let mut s = format!(
            "\n====== [{timestamp}] {method} {host}{path}{ws} ======\n",
            timestamp = entry.timestamp,
            method = entry.method,
            host = entry.host,
            path = entry.path,
            ws = ws_tag,
        );
        if let Some(ref h) = entry.request_headers {
            s.push_str("--- REQUEST HEADERS ---\n");
            s.push_str(h);
            s.push('\n');
        }
        if let Some(ref b) = entry.request_body {
            if !b.is_empty() {
                s.push_str("--- REQUEST BODY ---\n");
                s.push_str(b);
                s.push('\n');
            }
        }
        if let Some(ref label) = entry.token_label {
            s.push_str(&format!("[Token: {}]\n", label));
        }
        s
    }

    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}
