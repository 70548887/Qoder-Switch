use crate::auth::cosy::build_cosy_headers;
use crate::auth::token::TokenManager;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserPlan {
    pub user_type: String,
    pub plan_name: String,
    pub start_date: i64,
    pub end_date: i64,
    pub is_expired: bool,
    pub days_remaining: i64,
}

#[derive(Serialize, Clone)]
pub struct QuotaResult {
    pub account_id: String,
    pub label: String,
    pub plan: Option<UserPlan>,
    pub error: Option<String>,
}

pub async fn check_user_plan(
    token: &str,
    user_id: &str,
    name: &str,
    email: &str,
) -> Result<UserPlan, String> {
    let headers = build_cosy_headers(token, user_id, name, email, "/algo/api/v2/user", "")
        .map_err(|e| format!("构建 Cosy 头失败: {}", e))?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let resp = client
        .post("https://center.qoder.sh/algo/api/v2/user")
        .header("authorization", &headers.authorization)
        .header("cosy-user", &headers.cosy_user)
        .header("cosy-key", &headers.cosy_key)
        .header("cosy-date", &headers.cosy_date)
        .header("x-request-id", &headers.request_id)
        .header("content-length", "0")
        .header("content-type", "application/json")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API 返回 {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let end_date = body["end_date"].as_i64().unwrap_or(0);
    let start_date = body["start_date"].as_i64().unwrap_or(0);
    let days_remaining = (end_date - now_ms) / (1000 * 60 * 60 * 24);

    Ok(UserPlan {
        user_type: body["user_type"].as_str().unwrap_or("unknown").to_string(),
        plan_name: body["plan_tier_name"].as_str().unwrap_or("Unknown").to_string(),
        start_date,
        end_date,
        is_expired: end_date < now_ms,
        days_remaining: days_remaining.max(0),
    })
}

pub async fn check_all_plans(token_manager: &TokenManager) -> Vec<QuotaResult> {
    let accounts = token_manager.list().await;
    let mut handles = Vec::new();

    for account in accounts {
        let token = account.token.clone();
        let user_id = account.user_id.clone();
        let name_val = account.name.clone();
        let email = account.email.clone();
        let id = account.id.clone();
        let label = account.label.clone();

        handles.push(tokio::spawn(async move {
            let plan = check_user_plan(&token, &user_id, &name_val, &email).await;
            QuotaResult {
                account_id: id,
                label,
                plan: plan.as_ref().ok().cloned(),
                error: plan.err(),
            }
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                log::warn!("额度查询任务异常: {}", e);
            }
        }
    }
    results
}
