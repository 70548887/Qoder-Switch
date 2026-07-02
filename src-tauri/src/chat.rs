use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use rusqlite::OpenFlags;

// === 数据结构 ===

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatHistory {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub context: Vec<ChatContext>,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default, rename = "sessionId")]
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatContext {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub context_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

// === 核心函数 ===

/// 获取 Qoder IDE 的数据基路径
/// Windows: %APPDATA%\Qoder
/// macOS: ~/Library/Application Support/Qoder
/// Linux: ~/.config/Qoder
pub fn get_qoder_base_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("Qoder");
        }
        dirs::data_dir().unwrap_or_default().join("Qoder")
    }
    #[cfg(target_os = "macos")]
    {
        dirs::data_dir().unwrap_or_default().join("Qoder")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().unwrap_or_default().join("Qoder")
    }
}

/// 列出所有工作区
/// 扫描 {base}/User/workspaceStorage/ 目录，读取每个子目录的 workspace.json
pub fn list_workspaces() -> Result<Vec<WorkspaceInfo>, String> {
    let base = get_qoder_base_path();
    let ws_dir = base.join("User").join("workspaceStorage");

    if !ws_dir.exists() {
        log::info!("[chat] workspaceStorage 目录不存在: {:?}", ws_dir);
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(&ws_dir)
        .map_err(|e| format!("读取工作区目录失败: {}", e))?;

    let mut workspaces = Vec::new();

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }

        let workspace_id = entry.file_name().to_string_lossy().to_string();
        let ws_json_path = entry.path().join("workspace.json");

        if let Ok(data) = std::fs::read_to_string(&ws_json_path) {
            if let Ok(ws) = serde_json::from_str::<serde_json::Value>(&data) {
                let folder = ws.get("folder")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let full_path = decode_workspace_path(&folder);

                // 提取最后一个目录名作为显示名称
                let name = if full_path.is_empty() {
                    // 尝试从 workspace.json 的其他字段获取名称
                    let alt_name = ws.get("workspace")
                        .and_then(|w| w.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();
                    if alt_name.is_empty() {
                        format!("未命名-{}", &workspace_id[..std::cmp::min(8, workspace_id.len())])
                    } else {
                        alt_name
                    }
                } else {
                    // 取路径最后一个非空段
                    full_path.trim_end_matches('/')
                        .rsplit('/')
                        .next()
                        .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) })
                        .or_else(|| {
                            full_path.trim_end_matches('\\')
                                .rsplit('\\')
                                .next()
                                .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) })
                        })
                        .unwrap_or_else(|| full_path.clone())
                };

                workspaces.push(WorkspaceInfo {
                    id: workspace_id,
                    name,
                    path: full_path,
                });
            }
        }
    }

    // 按有无聊天记录排序：有记录的排前面
    let global_db = base.join("User").join("globalStorage").join("state.vscdb");
    if global_db.exists() {
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &global_db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            workspaces.sort_by(|a, b| {
                let has_a = conn.query_row(
                    "SELECT 1 FROM ItemTable WHERE key = ?1 LIMIT 1",
                    [&format!("lingma.chat.localHistory.{}", a.id)],
                    |_| Ok(()),
                ).is_ok();
                let has_b = conn.query_row(
                    "SELECT 1 FROM ItemTable WHERE key = ?1 LIMIT 1",
                    [&format!("lingma.chat.localHistory.{}", b.id)],
                    |_| Ok(()),
                ).is_ok();
                has_b.cmp(&has_a)
            });
        }
    }

    log::info!("[chat] 发现 {} 个工作区", workspaces.len());
    Ok(workspaces)
}

/// 获取指定工作区的聊天历史
pub fn get_chat_history(workspace_id: &str) -> Result<Vec<ChatHistory>, String> {
    let db_path = get_qoder_base_path()
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if !db_path.exists() {
        log::info!("[chat] state.vscdb 不存在: {:?}", db_path);
        return Ok(vec![]);
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("打开数据库失败: {}", e))?;

    let key = format!("lingma.chat.localHistory.{}", workspace_id);

    let value: Result<String, _> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        [&key],
        |row| row.get(0),
    );

    match value {
        Ok(json_str) => {
            let history: Vec<ChatHistory> = serde_json::from_str(&json_str)
                .map_err(|e| format!("解析聊天历史 JSON 失败: {}", e))?;
            log::info!("[chat] 工作区 {} 有 {} 条对话", workspace_id, history.len());
            Ok(history)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            log::info!("[chat] 工作区 {} 无聊天记录", workspace_id);
            Ok(vec![])
        }
        Err(e) => Err(format!("查询数据库失败: {}", e)),
    }
}

/// 删除指定的聊天记录（读-过滤-回写模式）
pub fn delete_chats(workspace_id: &str, chat_ids: &[String]) -> Result<(), String> {
    let db_path = get_qoder_base_path()
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if !db_path.exists() {
        return Err("数据库文件不存在".to_string());
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("打开数据库失败（写模式）: {}", e))?;

    let key = format!("lingma.chat.localHistory.{}", workspace_id);

    // 1. 读取原始数据
    let value: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            [&key],
            |row| row.get(0),
        )
        .map_err(|e| format!("读取聊天历史失败: {}", e))?;

    let mut history: Vec<ChatHistory> =
        serde_json::from_str(&value).map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let original_count = history.len();

    // 2. 过滤掉要删除的 ID
    history.retain(|h| !chat_ids.contains(&h.id));

    let deleted_count = original_count - history.len();
    log::info!(
        "[chat] 删除 {} 条对话（工作区 {}）",
        deleted_count,
        workspace_id
    );

    // 3. 序列化并回写
    let new_value =
        serde_json::to_string(&history).map_err(|e| format!("序列化 JSON 失败: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable(key, value) VALUES(?1, ?2)",
        rusqlite::params![key, new_value],
    )
    .map_err(|e| format!("回写数据库失败: {}", e))?;

    Ok(())
}

/// 解码工作区路径（处理 file:/// URI 和 URL 编码）
fn decode_workspace_path(raw: &str) -> String {
    let path = raw.strip_prefix("file:///").unwrap_or(raw);
    percent_decode(path)
}

/// UTF-8 安全的百分号解码（对畸形编码保留原始字符）
fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next();
            let h2 = chars.next();
            match (h1, h2) {
                (Some(a), Some(b)) if a.is_ascii_hexdigit() && b.is_ascii_hexdigit() => {
                    let hex = format!("{}{}", a, b);
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        bytes.push(byte);
                    }
                }
                // 非法编码：保留原始字符
                (a, b) => {
                    bytes.push(b'%');
                    if let Some(a) = a {
                        bytes.extend_from_slice(a.to_string().as_bytes());
                    }
                    if let Some(b) = b {
                        bytes.extend_from_slice(b.to_string().as_bytes());
                    }
                }
            }
        } else {
            bytes.extend_from_slice(c.to_string().as_bytes());
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

// === 备份/恢复 ===

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionBackup {
    #[serde(rename = "backupTime")]
    pub backup_time: String,
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    #[serde(rename = "workspacePath")]
    pub workspace_path: String,
    #[serde(rename = "userId", default)]
    pub user_id: String,
    #[serde(rename = "chatHistory")]
    pub chat_history: Vec<ChatHistory>,
    #[serde(rename = "chatViews", default)]
    pub chat_views: serde_json::Value,
    #[serde(rename = "chatTabs", default)]
    pub chat_tabs: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct BackupFileInfo {
    pub file_path: String,
    pub file_name: String,
    pub backup_time: String,
    pub workspace_id: String,
    pub workspace_path: String,
    pub file_size: u64,
}

/// 通用函数：从 SQLite 数据库读取 ItemTable 中的 value
fn get_raw_db_value(db_path: &std::path::Path, key: &str) -> Result<Option<String>, String> {
    if !db_path.exists() {
        return Ok(None);
    }
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| format!("打开数据库失败: {}", e))?;

    let value: Result<String, _> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        [key],
        |row| row.get(0),
    );

    match value {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("查询失败: {}", e)),
    }
}

/// 备份指定工作区的会话数据
pub fn backup_workspace(workspace_id: &str) -> Result<SessionBackup, String> {
    let base = get_qoder_base_path();

    // 获取工作区路径信息
    let ws_json_path = base.join("User").join("workspaceStorage")
        .join(workspace_id).join("workspace.json");
    let workspace_path = std::fs::read_to_string(&ws_json_path)
        .ok()
        .and_then(|data| serde_json::from_str::<serde_json::Value>(&data).ok())
        .and_then(|v| v.get("folder").and_then(|f| f.as_str()).map(|s| s.to_string()))
        .unwrap_or_default();

    // 读取用户 ID
    let user_id = std::fs::read_to_string(base.join(".auth").join("id"))
        .unwrap_or_default()
        .trim()
        .to_string();

    // 获取聊天历史（从全局数据库）
    let chat_history = get_chat_history(workspace_id)?;

    // 获取视图配置（从工作区数据库）
    let ws_db_path = base.join("User").join("workspaceStorage")
        .join(workspace_id).join("state.vscdb");
    let chat_views = get_raw_db_value(&ws_db_path, "aicoding.chat.views")?
        .and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok())
        .unwrap_or(serde_json::Value::Null);

    // 获取标签配置（从工作区数据库）
    let chat_tabs = get_raw_db_value(&ws_db_path, "aicoding.chat.tabs")?
        .and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok())
        .unwrap_or(serde_json::Value::Null);

    let backup = SessionBackup {
        backup_time: chrono::Local::now().to_rfc3339(),
        workspace_id: workspace_id.to_string(),
        workspace_path: decode_workspace_path(&workspace_path),
        user_id,
        chat_history,
        chat_views,
        chat_tabs,
    };

    log::info!("[chat] 备份工作区 {}：{}条对话", workspace_id, backup.chat_history.len());
    Ok(backup)
}

/// 保存备份到文件
pub fn save_backup(backup: &SessionBackup) -> Result<String, String> {
    let backup_dir = get_backup_directory();
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("qoder-backup-{}.json", timestamp);
    let backup_path = backup_dir.join(&filename);

    let json = serde_json::to_string_pretty(backup)
        .map_err(|e| format!("序列化备份失败: {}", e))?;

    std::fs::write(&backup_path, json)
        .map_err(|e| format!("写入备份文件失败: {}", e))?;

    let path_str = backup_path.to_string_lossy().to_string();
    log::info!("[chat] 备份已保存: {}", path_str);
    Ok(path_str)
}

/// 批量备份所有工作区
pub fn backup_all_workspaces() -> Result<String, String> {
    let workspaces = list_workspaces()?;
    let mut all_backups = Vec::new();

    for ws in &workspaces {
        match backup_workspace(&ws.id) {
            Ok(backup) => all_backups.push(backup),
            Err(e) => log::warn!("[chat] 备份工作区 {} 失败: {}", ws.id, e),
        }
    }

    let backup_dir = get_backup_directory();
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("qoder-backup-all-{}.json", timestamp);
    let path = backup_dir.join(&filename);

    let json = serde_json::to_string_pretty(&all_backups)
        .map_err(|e| format!("序列化备份失败: {}", e))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("写入备份文件失败: {}", e))?;

    let path_str = path.to_string_lossy().to_string();
    log::info!("[chat] 批量备份完成：{}个工作区，保存到 {}", all_backups.len(), path_str);
    Ok(path_str)
}

/// 导出所有工作区对话为 Markdown
pub fn export_markdown() -> Result<String, String> {
    let workspaces = list_workspaces()?;
    let mut md = String::new();
    md.push_str("# Qoder 会话导出\n\n");
    md.push_str(&format!("导出时间: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    for ws in &workspaces {
        if let Ok(history) = get_chat_history(&ws.id) {
            if history.is_empty() { continue; }
            md.push_str(&format!("## 工作区: {}\n", ws.name));
            md.push_str(&format!("会话数: {}\n\n", history.len()));

            for (i, h) in history.iter().enumerate() {
                let time_str = chrono::DateTime::from_timestamp_millis(h.timestamp)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default();
                md.push_str(&format!("### {}. {}\n", i + 1, h.title));
                md.push_str(&format!("- 时间: {}\n", time_str));
                md.push_str(&format!("- 会话ID: {}\n", h.session_id));
                if !h.context.is_empty() {
                    md.push_str("- 引用上下文:\n");
                    for ctx in &h.context {
                        md.push_str(&format!("  - {} ({})\n", ctx.name, ctx.context_type));
                    }
                }
                md.push_str("\n");
            }
            md.push_str("---\n\n");
        }
    }

    let export_dir = get_backup_directory();
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| format!("创建导出目录失败: {}", e))?;

    let filename = format!("qoder-export-{}.md", chrono::Local::now().format("%Y%m%d-%H%M%S"));
    let path = export_dir.join(&filename);

    std::fs::write(&path, &md)
        .map_err(|e| format!("写入导出文件失败: {}", e))?;

    let path_str = path.to_string_lossy().to_string();
    log::info!("[chat] Markdown 导出完成: {}", path_str);
    Ok(path_str)
}

/// 删除备份文件（仅允许删除备份目录内的文件）
pub fn delete_backup_file(file_path: &str) -> Result<(), String> {
    let backup_dir = get_backup_directory();
    
    // 确保备份目录存在才能 canonicalize
    std::fs::create_dir_all(&backup_dir).ok();
    
    let target = std::fs::canonicalize(file_path)
        .map_err(|e| format!("解析路径失败: {}", e))?;
    let safe_dir = std::fs::canonicalize(&backup_dir)
        .map_err(|e| format!("解析备份目录失败: {}", e))?;
    
    if !target.starts_with(&safe_dir) {
        return Err("不允许删除备份目录之外的文件".to_string());
    }
    
    std::fs::remove_file(&target)
        .map_err(|e| format!("删除备份文件失败: {}", e))
}

/// 从备份文件恢复会话数据
/// target_workspace_id: 如果提供 Some，则恢复到该工作区；如果 None，恢复到备份文件原始工作区
pub fn restore_backup(target_workspace_id: Option<&str>, backup_path: &str) -> Result<(), String> {
    let data = std::fs::read_to_string(backup_path)
        .map_err(|e| format!("读取备份文件失败: {}", e))?;

    // 先尝试解析为单个备份
    if let Ok(backup) = serde_json::from_str::<SessionBackup>(&data) {
        let workspace_id = target_workspace_id.unwrap_or(&backup.workspace_id);
        if workspace_id.is_empty() {
            return Err("工作区 ID 为空，无法恢复。请选择目标工作区后重试。".to_string());
        }
        return restore_single_backup(workspace_id, &backup);
    }

    // 再尝试解析为多工作区备份数组
    if let Ok(backups) = serde_json::from_str::<Vec<SessionBackup>>(&data) {
        if backups.is_empty() {
            return Err("备份文件为空".to_string());
        }
        if let Some(target_id) = target_workspace_id {
            if target_id.is_empty() {
                return Err("目标工作区 ID 为空".to_string());
            }
            // 指定目标工作区：在数组中查找匹配的，或使用第一个
            let backup = backups.iter().find(|b| b.workspace_id == target_id)
                .unwrap_or(&backups[0]);
            return restore_single_backup(target_id, backup);
        }
        // 未指定目标：逐个恢复到各自原始工作区
        for backup in &backups {
            if backup.workspace_id.is_empty() {
                log::warn!("[chat] 跳过工作区 ID 为空的备份");
                continue;
            }
            if let Err(e) = restore_single_backup(&backup.workspace_id, backup) {
                log::warn!("[chat] 恢复工作区 {} 失败: {}", backup.workspace_id, e);
            }
        }
        return Ok(());
    }

    Err("无法识别的备份文件格式".to_string())
}

/// 恢复单个备份到指定工作区
fn restore_single_backup(workspace_id: &str, backup: &SessionBackup) -> Result<(), String> {
    let base = get_qoder_base_path();

    // 1. 恢复聊天历史（写入全局数据库）
    let global_db = base.join("User").join("globalStorage").join("state.vscdb");
    let conn = rusqlite::Connection::open_with_flags(
        &global_db,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(|e| format!("打开全局数据库失败: {}", e))?;

    let key = format!("lingma.chat.localHistory.{}", workspace_id);
    let value = serde_json::to_string(&backup.chat_history)
        .map_err(|e| format!("序列化聊天历史失败: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable(key, value) VALUES(?1, ?2)",
        rusqlite::params![key, value],
    ).map_err(|e| format!("恢复聊天历史失败: {}", e))?;

    // 2. 恢复视图和标签（仅当工作区数据库存在时）
    let ws_db = base.join("User").join("workspaceStorage")
        .join(workspace_id).join("state.vscdb");
    if ws_db.exists() {
        if !backup.chat_views.is_null() {
            if let Ok(conn) = rusqlite::Connection::open_with_flags(
                &ws_db,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            ) {
                let views_json = backup.chat_views.to_string();
                if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO ItemTable(key, value) VALUES(?1, ?2)",
                    rusqlite::params!["aicoding.chat.views", views_json],
                ) {
                    log::warn!("[chat] 恢复视图配置失败: {}", e);
                }
            }
        }
        if !backup.chat_tabs.is_null() {
            if let Ok(conn) = rusqlite::Connection::open_with_flags(
                &ws_db,
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            ) {
                let tabs_json = backup.chat_tabs.to_string();
                if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO ItemTable(key, value) VALUES(?1, ?2)",
                    rusqlite::params!["aicoding.chat.tabs", tabs_json],
                ) {
                    log::warn!("[chat] 恢复标签配置失败: {}", e);
                }
            }
        }
    }

    log::info!("[chat] 恢复完成：目标工作区 {}，{}条对话", workspace_id, backup.chat_history.len());
    Ok(())
}

/// 列出备份目录下的所有备份文件
pub fn list_backups() -> Result<Vec<BackupFileInfo>, String> {
    let backup_dir = get_backup_directory();
    if !backup_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(&backup_dir)
        .map_err(|e| format!("读取备份目录失败: {}", e))?;

    let mut backups = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(backup) = serde_json::from_str::<SessionBackup>(&data) {
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let file_name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    backups.push(BackupFileInfo {
                        file_path: path.to_string_lossy().to_string(),
                        file_name,
                        backup_time: backup.backup_time,
                        workspace_id: backup.workspace_id,
                        workspace_path: backup.workspace_path,
                        file_size,
                    });
                } else if let Ok(list) = serde_json::from_str::<Vec<SessionBackup>>(&data) {
                    if !list.is_empty() {
                        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let file_name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        backups.push(BackupFileInfo {
                            file_path: path.to_string_lossy().to_string(),
                            file_name,
                            backup_time: list[0].backup_time.clone(),
                            workspace_id: format!("{}个工作区", list.len()),
                            workspace_path: "批量备份".to_string(),
                            file_size,
                        });
                    }
                }
            }
        }
    }

    backups.sort_by(|a, b| b.backup_time.cmp(&a.backup_time));
    log::info!("[chat] 发现 {} 个备份文件", backups.len());
    Ok(backups)
}

/// 获取备份目录路径
fn get_backup_directory() -> std::path::PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("qoder-backups")
}
