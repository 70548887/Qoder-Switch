use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PoolResponse {
    success: bool,
    data: Option<PoolAccountData>,
}

#[derive(Deserialize, Clone)]
pub struct PoolAccountData {
    pub email: String,
    pub user_type: String,
    pub expire_date: String,
    pub user_info_json: String,
    pub user_plan_json: String,
    pub machine_token: String,
    pub machine_id: String,
    pub machine_code: String,
    pub machine_type: String,
}

#[derive(Deserialize)]
struct UserInfo {
    id: Option<String>,
    token: Option<String>,
    name: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct FetchedAccount {
    pub user_id: String,
    pub token: String,
    pub name: String,
    pub email: String,
    pub user_type: String,
    pub expire_date: String,
    pub machine_token: String,
    pub machine_id: String,
}

/// 从账号池获取账号
pub async fn fetch_account(secret_key: &str, api_url: &str) -> Result<FetchedAccount, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("创建客户端失败: {}", e))?;

    let body = serde_json::json!({ "secret_key": secret_key });

    let resp = client
        .post(api_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求账号池失败: {}", e))?;

    let pool_resp: PoolResponse = resp.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if !pool_resp.success {
        return Err("账号池返回失败，密钥无效或已过期".to_string());
    }

    let data = pool_resp.data.ok_or("响应中无账号数据".to_string())?;

    // 解析 user_info_json
    let user_info: UserInfo = serde_json::from_str(&data.user_info_json)
        .map_err(|e| format!("解析用户信息失败: {}", e))?;

    let token = user_info.token.ok_or_else(|| "账号池返回数据缺少 token".to_string())?;
    let user_id = user_info.id.unwrap_or_default();

    Ok(FetchedAccount {
        user_id,
        token,
        name: user_info.name.unwrap_or_default(),
        email: user_info.email.unwrap_or(data.email.clone()),
        user_type: data.user_type,
        expire_date: data.expire_date,
        machine_token: data.machine_token,
        machine_id: data.machine_id,
    })
}
