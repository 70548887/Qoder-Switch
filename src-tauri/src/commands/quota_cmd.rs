use crate::error::AppResult;
use crate::quota::{self, QuotaResult, UserPlan};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn check_quota(state: State<'_, AppState>, id: String) -> AppResult<UserPlan> {
    let accounts = state.token_manager.list().await;
    let account = accounts.iter().find(|a| a.id == id)
        .ok_or_else(|| crate::error::AppError::Auth(format!("账号不存在: {}", id)))?;

    let plan = quota::check_user_plan(&account.token, &account.user_id, &account.name, &account.email)
        .await
        .map_err(|e| crate::error::AppError::Auth(e))?;

    // 将查询结果持久化回账号信息
    let expire_date = {
        let ts = plan.end_date / 1000; // ms -> s
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    };
    state.token_manager.update_account_info(&id, plan.user_type.clone(), expire_date).await;

    Ok(plan)
}

#[tauri::command]
pub async fn check_all_quotas(state: State<'_, AppState>) -> AppResult<Vec<QuotaResult>> {
    Ok(quota::check_all_plans(&state.token_manager).await)
}
