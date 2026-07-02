use crate::chat;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub async fn list_chat_workspaces() -> AppResult<Vec<chat::WorkspaceInfo>> {
    chat::list_workspaces().map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn get_workspace_chats(workspace_id: String) -> AppResult<Vec<chat::ChatHistory>> {
    chat::get_chat_history(&workspace_id).map_err(|e| AppError::Chat(e))
}

/// 搜索工作区对话（后端过滤）
/// 注意：当前前端已改为本地过滤（ChatHistoryPanel.vue 中的 filteredChats），
/// 此命令保留以备后续扩展使用（如支持更复杂的搜索、外部调用等）
#[tauri::command]
pub async fn search_workspace_chats(
    workspace_id: String,
    query: String,
) -> AppResult<Vec<chat::ChatHistory>> {
    let all = chat::get_chat_history(&workspace_id).map_err(|e| AppError::Chat(e))?;
    let query_lower = query.to_lowercase();
    let filtered: Vec<_> = all
        .into_iter()
        .filter(|h| h.title.to_lowercase().contains(&query_lower))
        .collect();
    Ok(filtered)
}

#[tauri::command]
pub async fn delete_workspace_chats(
    workspace_id: String,
    chat_ids: Vec<String>,
) -> AppResult<()> {
    chat::delete_chats(&workspace_id, &chat_ids).map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn backup_workspace_session(workspace_id: String) -> AppResult<String> {
    let backup = chat::backup_workspace(&workspace_id).map_err(|e| AppError::Chat(e))?;
    let path = chat::save_backup(&backup).map_err(|e| AppError::Chat(e))?;
    Ok(path)
}

#[tauri::command]
pub async fn restore_workspace_session(workspace_id: Option<String>, backup_path: String) -> AppResult<()> {
    chat::restore_backup(workspace_id.as_deref(), &backup_path).map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn list_session_backups() -> AppResult<Vec<chat::BackupFileInfo>> {
    chat::list_backups().map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn backup_all_workspaces() -> AppResult<String> {
    chat::backup_all_workspaces().map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn export_chats_markdown() -> AppResult<String> {
    chat::export_markdown().map_err(|e| AppError::Chat(e))
}

#[tauri::command]
pub async fn delete_backup_file(file_path: String) -> AppResult<()> {
    chat::delete_backup_file(&file_path).map_err(|e| AppError::Chat(e))
}
