use crate::error::AppResult;
use crate::quota::{self, QuotaResult};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn check_quota(state: State<'_, AppState>, id: String) -> AppResult<QuotaResult> {
    // 1. 获取账号
    let accounts = state.token_manager.list().await;
    let account = accounts.iter().find(|a| a.id == id)
        .ok_or_else(|| crate::error::AppError::Auth(format!("账号不存在: {}", id)))?;

    let token = account.token.clone();
    let account_id = account.id.clone();
    let label = account.label.clone();

    // 2. 调用 Plan API 获取订阅信息
    let plan = quota::check_user_plan(&token)
        .await
        .map_err(|e| crate::error::AppError::Auth(e))?;

    // 3. 调用 Quota/Usage API 获取真实余额
    let usage = quota::check_quota_usage(&token)
        .await
        .map_err(|e| crate::error::AppError::Auth(e))?;

    // 4. 更新账号类型和到期日
    let expire_date = {
        let ts = plan.end_date / 1000; // ms -> s
        chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    };
    state.token_manager.update_account_info(&account_id, plan.user_type.clone(), expire_date.clone()).await;

    // 5. 更新余额信息
    state.token_manager.update_account_quota(
        &account_id,
        Some(usage.user_quota.used as u64),
        Some(usage.user_quota.total as u64),
    ).await;

    // 6. 返回结果给前端
    Ok(QuotaResult {
        account_id,
        label,
        plan_name: plan.plan_tier_name,
        user_type: plan.user_type,
        quota_used: usage.user_quota.used,
        quota_total: usage.user_quota.total,
        quota_remaining: usage.user_quota.remaining,
        quota_unit: usage.user_quota.unit,
        is_exceeded: usage.is_quota_exceeded,
        expire_date,
        error: None,
    })
}

#[tauri::command]
pub async fn check_all_quotas(state: State<'_, AppState>) -> AppResult<Vec<QuotaResult>> {
    let accounts = state.token_manager.list().await;
    let mut results = Vec::new();

    for account in &accounts {
        let token = account.token.clone();
        let id = account.id.clone();
        let label = account.label.clone();

        // 逐个查询，避免并发过多请求被限流
        match quota::check_user_plan(&token).await {
            Ok(plan) => {
                match quota::check_quota_usage(&token).await {
                    Ok(usage) => {
                        let expire_date = {
                            let ts = plan.end_date / 1000;
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        };
                        state.token_manager.update_account_info(&id, plan.plan_tier_name.clone(), expire_date.clone()).await;
                        state.token_manager.update_account_quota(&id, Some(usage.user_quota.used as u64), Some(usage.user_quota.total as u64)).await;

                        results.push(QuotaResult {
                            account_id: id,
                            label,
                            plan_name: plan.plan_tier_name,
                            user_type: plan.user_type,
                            quota_used: usage.user_quota.used,
                            quota_total: usage.user_quota.total,
                            quota_remaining: usage.user_quota.remaining,
                            quota_unit: usage.user_quota.unit,
                            is_exceeded: usage.is_quota_exceeded,
                            expire_date,
                            error: None,
                        });
                    }
                    Err(e) => {
                        log::warn!("[quota] 账号 {} Quota 查询失败: {}", id, e);
                        results.push(QuotaResult {
                            account_id: id,
                            label,
                            plan_name: String::new(),
                            user_type: String::new(),
                            quota_used: 0.0,
                            quota_total: 0.0,
                            quota_remaining: 0.0,
                            quota_unit: String::new(),
                            is_exceeded: false,
                            expire_date: String::new(),
                            error: Some(format!("Quota 查询失败: {}", e)),
                        });
                    }
                }
            }
            Err(e) => {
                log::warn!("[quota] 账号 {} Plan 查询失败: {}", id, e);
                results.push(QuotaResult {
                    account_id: id,
                    label,
                    plan_name: String::new(),
                    user_type: String::new(),
                    quota_used: 0.0,
                    quota_total: 0.0,
                    quota_remaining: 0.0,
                    quota_unit: String::new(),
                    is_exceeded: false,
                    expire_date: String::new(),
                    error: Some(format!("Plan 查询失败: {}", e)),
                });
            }
        }
    }

    Ok(results)
}
