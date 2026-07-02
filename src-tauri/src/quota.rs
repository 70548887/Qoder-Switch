use serde::{Deserialize, Serialize};
use std::time::Duration;

// ===== Plan API 响应结构 =====

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserPlan {
    pub user_type: String,
    pub plan_tier_name: String,
    pub is_personal_version: bool,
    pub is_highest_tier: bool,
    pub start_date: i64,
    pub end_date: i64,
}

// ===== Quota/Usage API 响应结构 =====

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuotaUsage {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(rename = "usageType")]
    pub usage_type: String,
    #[serde(rename = "totalUsagePercentage")]
    pub total_usage_percentage: f64,
    #[serde(rename = "isQuotaExceeded")]
    pub is_quota_exceeded: bool,
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    #[serde(rename = "userQuota")]
    pub user_quota: UserQuota,
    #[serde(rename = "isPlanQuotaProrated")]
    pub is_plan_quota_prorated: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserQuota {
    pub total: f64,
    pub used: f64,
    pub remaining: f64,
    pub percentage: f64,
    pub unit: String,
}

// ===== 返回给前端的结构 =====

#[derive(Debug, Serialize, Clone)]
pub struct QuotaResult {
    pub account_id: String,
    pub label: String,
    pub plan_name: String,
    pub user_type: String,
    pub quota_used: f64,
    pub quota_total: f64,
    pub quota_remaining: f64,
    pub quota_unit: String,
    pub is_exceeded: bool,
    pub expire_date: String,
    pub error: Option<String>,
}

// ===== API 调用函数 =====

/// 查询用户订阅计划信息
pub async fn check_user_plan(token: &str) -> Result<UserPlan, String> {
    log::info!("[quota] 查询用户 Plan...");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get("https://openapi.qoder.sh/api/v2/user/plan")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("请求 Plan API 失败: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await
        .map_err(|e| format!("读取 Plan 响应体失败: {}", e))?;

    log::info!("[quota] Plan API 状态码: {}, 响应长度: {} bytes", status, body_text.len());
    log::debug!("[quota] Plan 响应内容: {}", &body_text[..body_text.len().min(500)]);

    if !status.is_success() {
        return Err(format!("Plan API 返回 {}: {}", status, &body_text[..body_text.len().min(200)]));
    }

    let plan: UserPlan = serde_json::from_str(&body_text)
        .map_err(|e| format!("解析 Plan JSON 失败: {} | body: {}", e, &body_text[..body_text.len().min(200)]))?;

    log::info!("[quota] Plan 查询成功: tier={}, type={}", plan.plan_tier_name, plan.user_type);
    Ok(plan)
}

/// 查询用户配额使用情况（真实余额）
pub async fn check_quota_usage(token: &str) -> Result<QuotaUsage, String> {
    log::info!("[quota] 查询 Quota Usage...");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let resp = client
        .get("https://openapi.qoder.sh/api/v2/quota/usage")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("请求 Quota API 失败: {}", e))?;

    let status = resp.status();
    let body_text = resp.text().await
        .map_err(|e| format!("读取 Quota 响应体失败: {}", e))?;

    log::info!("[quota] Quota API 状态码: {}, 响应长度: {} bytes", status, body_text.len());
    log::debug!("[quota] Quota 响应内容: {}", &body_text[..body_text.len().min(500)]);

    if !status.is_success() {
        return Err(format!("Quota API 返回 {}: {}", status, &body_text[..body_text.len().min(200)]));
    }

    let usage: QuotaUsage = serde_json::from_str(&body_text)
        .map_err(|e| format!("解析 Quota JSON 失败: {} | body: {}", e, &body_text[..body_text.len().min(200)]))?;

    log::info!("[quota] Quota 查询成功: used={}/{}, exceeded={}", usage.user_quota.used, usage.user_quota.total, usage.is_quota_exceeded);
    Ok(usage)
}
