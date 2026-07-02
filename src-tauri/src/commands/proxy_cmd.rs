use crate::state::AppState;
use crate::error::AppResult;
use crate::metrics::MetricsSnapshot;
use crate::proxy::server::ProxyServer;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub port: u16,
    pub auto_rotate: bool,
    pub token_count: usize,
    pub current_token_id: Option<String>,
    pub cert_installed: bool,
}

#[tauri::command]
pub async fn start_proxy(state: State<'_, AppState>) -> AppResult<()> {
    // 读取 CA 证书和私钥
    let ca_key_path = state.cert_manager.ca_key_path();
    let ca_cert_path = state.cert_manager.ca_cert_path();

    // 如果证书不存在，生成自签名 CA
    if !ca_key_path.exists() || !ca_cert_path.exists() {
        generate_ca(&ca_key_path, &ca_cert_path)?;
    }

    let ca_key_pem = std::fs::read_to_string(&ca_key_path)
        .map_err(|e| crate::error::AppError::Cert(format!("读取 CA 私钥失败: {}", e)))?;
    let ca_cert_pem = std::fs::read_to_string(&ca_cert_path)
        .map_err(|e| crate::error::AppError::Cert(format!("读取 CA 证书失败: {}", e)))?;

    let mut server = ProxyServer::new(state.proxy_port, state.token_manager.clone(), state.metrics.clone(), state.logger.clone(), state.traffic_logger.clone(), state.config.clone());

    // 从配置初始化 auto_rotate
    let auto_rotate_val = state.config.read().await.auto_rotate;
    *server.auto_rotate.write().await = auto_rotate_val;

    server.start(&ca_key_pem, &ca_cert_pem).await
        .map_err(|e| crate::error::AppError::Proxy(e))?;

    let mut proxy = state.proxy_server.write().await;
    *proxy = Some(server);
    Ok(())
}

#[tauri::command]
pub async fn stop_proxy(state: State<'_, AppState>) -> AppResult<()> {
    let mut proxy = state.proxy_server.write().await;
    if let Some(ref mut server) = *proxy {
        server.stop().await;
    }
    *proxy = None;
    Ok(())
}

#[tauri::command]
pub async fn get_proxy_status(state: State<'_, AppState>) -> AppResult<ProxyStatus> {
    let proxy = state.proxy_server.read().await;
    let running = if let Some(ref server) = *proxy {
        server.is_running().await
    } else {
        false
    };
    let auto_rotate = if let Some(ref server) = *proxy {
        *server.auto_rotate.read().await
    } else {
        true
    };
    let token_count = state.token_manager.list().await.len();
    let current = state.token_manager.get_current().await;

    Ok(ProxyStatus {
        running,
        port: state.proxy_port,
        auto_rotate,
        token_count,
        current_token_id: current.map(|t| t.id),
        cert_installed: state.cert_manager.is_installed(),
    })
}

#[tauri::command]
pub async fn set_auto_rotate(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    // 更新运行时状态
    let proxy = state.proxy_server.read().await;
    if let Some(ref server) = *proxy {
        *server.auto_rotate.write().await = enabled;
    }
    // 同步到配置并持久化
    let mut cfg = state.config.write().await;
    cfg.auto_rotate = enabled;
    cfg.save(&state.config_path).map_err(|e| crate::error::AppError::Config(e))?;
    Ok(())
}

fn generate_ca(
    key_path: &std::path::Path,
    cert_path: &std::path::Path,
) -> AppResult<()> {
    use rcgen::{CertificateParams, KeyPair, IsCa, BasicConstraints, DistinguishedName, DnType};

    let key_pair = KeyPair::generate()
        .map_err(|e| crate::error::AppError::Cert(format!("生成密钥对失败: {}", e)))?;

    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, "Qoder Proxy CA");
    distinguished_name.push(DnType::OrganizationName, "Qoder");

    let mut params = CertificateParams::default();
    params.distinguished_name = distinguished_name;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let cert = params.self_signed(&key_pair)
        .map_err(|e| crate::error::AppError::Cert(format!("自签名失败: {}", e)))?;

    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(key_path, key_pair.serialize_pem())?;
    std::fs::write(cert_path, cert.pem())?;

    Ok(())
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_accounts: u32,
    pub used_accounts: u32,
    pub unused_accounts: u32,
    pub total_chats: u32,
    pub total_backups: u32,
}

#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> AppResult<DashboardStats> {
    let accounts = state.token_manager.list().await;
    let config = state.config.read().await;
    let threshold = config.balance_threshold;

    let total_accounts = accounts.len() as u32;
    let used_accounts = accounts.iter().filter(|a| {
        // 已过期的
        if a.status == "expired" { return true; }
        // 余额不足的（有余额数据且低于阈值）
        if let (Some(used), Some(total)) = (a.quota_used, a.quota_total) {
            let remaining = total.saturating_sub(used);
            if threshold > 0 && remaining < threshold {
                return true;
            }
        }
        false
    }).count() as u32;
    let unused_accounts = total_accounts.saturating_sub(used_accounts);

    // 统计所有工作区的对话总数
    let total_chats = match crate::chat::list_workspaces() {
        Ok(workspaces) => {
            let mut count = 0u32;
            for ws in &workspaces {
                if let Ok(history) = crate::chat::get_chat_history(&ws.id) {
                    count += history.len() as u32;
                }
            }
            count
        }
        Err(_) => 0,
    };

    // 统计备份文件数
    let total_backups = match crate::chat::list_backups() {
        Ok(backups) => backups.len() as u32,
        Err(_) => 0,
    };

    Ok(DashboardStats {
        total_accounts,
        used_accounts,
        unused_accounts,
        total_chats,
        total_backups,
    })
}

#[tauri::command]
pub async fn get_metrics(state: State<'_, AppState>) -> AppResult<MetricsSnapshot> {
    Ok(state.metrics.snapshot())
}

#[tauri::command]
pub async fn reset_metrics(state: State<'_, AppState>) -> AppResult<()> {
    state.metrics.reset();
    Ok(())
}

#[tauri::command]
pub async fn get_request_logs(state: State<'_, AppState>, limit: Option<usize>) -> AppResult<Vec<crate::logger::RequestLogEntry>> {
    Ok(state.logger.get_logs(limit.unwrap_or(100)).await)
}

#[tauri::command]
pub async fn get_discovered_domains(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    Ok(state.logger.get_discovered_domains().await)
}

#[tauri::command]
pub async fn set_target_domains(state: State<'_, AppState>, domains: Vec<String>) -> AppResult<()> {
    state.logger.set_target_domains(domains.clone()).await;

    // 同步到配置文件
    let mut config = state.config.write().await;
    config.target_domains = domains;
    config.save(&state.config_path)
        .map_err(|e| crate::error::AppError::Config(e))?;

    Ok(())
}
