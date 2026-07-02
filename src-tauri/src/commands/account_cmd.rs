use crate::state::AppState;
use crate::auth::token::AccountToken;
use crate::error::AppResult;
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
struct AccountInput {
    token: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    user_type: String,
    #[serde(default)]
    expire_date: String,
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> AppResult<Vec<AccountToken>> {
    Ok(state.token_manager.list().await)
}

#[tauri::command]
pub async fn add_accounts(state: State<'_, AppState>, tokens: String) -> AppResult<()> {
    // 尝试解析为 JSON 数组格式 [{token, label, user_id, email, name, user_type, expire_date}]
    if let Ok(accounts) = serde_json::from_str::<Vec<AccountInput>>(&tokens) {
        for (i, input) in accounts.into_iter().enumerate() {
            let label = if input.label.is_empty() {
                format!("Token #{}", i + 1)
            } else {
                input.label
            };
            state.token_manager.add(
                input.token,
                label,
                input.user_id,
                input.email,
                input.name,
                input.user_type,
                input.expire_date,
            ).await;
        }
    } else {
        // 按换行/空格分隔的纯 token 字符串处理（向后兼容）
        state.token_manager.bulk_add(&tokens).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.token_manager.remove(&id).await;
    Ok(())
}

#[tauri::command]
pub async fn switch_account(state: State<'_, AppState>, id: String) -> AppResult<()> {
    if !state.token_manager.set_current(&id).await {
        return Err(crate::error::AppError::AccountNotFound(id));
    }
    Ok(())
}

#[tauri::command]
pub async fn fetch_pool_account(state: State<'_, AppState>, secret_key: String) -> AppResult<()> {
    let url = state.config.read().await.account_pool_url.clone();
    let fetched = crate::account_pool::fetch_account(&secret_key, &url).await
        .map_err(|e| crate::error::AppError::Auth(e))?;

    // 自动添加到 TokenManager
    state.token_manager.add(
        fetched.token,
        format!("{} ({})", fetched.name, fetched.user_type),
        fetched.user_id,
        fetched.email,
        fetched.name.clone(),
        fetched.user_type,
        fetched.expire_date,
    ).await;

    // 更新 machine 信息到最新添加的账号
    let accounts = state.token_manager.list().await;
    if let Some(last) = accounts.last() {
        state.token_manager.update_machine_info(
            &last.id,
            fetched.machine_token,
            fetched.machine_id,
            fetched.machine_code,
            fetched.machine_type,
        ).await;
    }

    Ok(())
}
