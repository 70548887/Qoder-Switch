use aes_gcm::{Aes256Gcm, AeadCore, aead::{Aead, KeyInit, OsRng}};
use aes_gcm::aead::generic_array::GenericArray;
use rand::Rng;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::RotateStrategy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountToken {
    pub id: String,
    pub label: String,
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub user_type: String,
    #[serde(default)]
    pub expire_date: String,
    #[serde(default)]
    pub quota_used: Option<u64>,
    #[serde(default)]
    pub quota_total: Option<u64>,
    #[serde(default)]
    pub machine_token: String,
    #[serde(default)]
    pub machine_id: String,
    #[serde(default)]
    pub machine_code: String,
    #[serde(default)]
    pub machine_type: String,
}

pub struct TokenManager {
    accounts: Arc<RwLock<Vec<AccountToken>>>,
    current: Arc<RwLock<usize>>,
    storage_path: PathBuf,
}

impl TokenManager {
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(Vec::new())),
            current: Arc::new(RwLock::new(0)),
            storage_path,
        }
    }

    pub async fn get_current(&self) -> Option<AccountToken> {
        let accounts = self.accounts.read().await;
        let idx = *self.current.read().await;
        accounts.get(idx).cloned()
    }

    pub async fn rotate(&self) -> Option<String> {
        self.rotate_with_strategy(RotateStrategy::Sequential).await
    }

    pub async fn rotate_with_strategy(&self, strategy: RotateStrategy) -> Option<String> {
        let mut accounts = self.accounts.write().await;
        let mut current = self.current.write().await;

        if accounts.is_empty() {
            return None;
        }

        let next_idx = match strategy {
            RotateStrategy::Sequential => {
                (*current + 1) % accounts.len()
            }
            RotateStrategy::Random => {
                if accounts.len() <= 1 { 0 } else {
                    let mut rng = rand::thread_rng();
                    let mut idx = rng.gen_range(0..accounts.len());
                    // 避免选到当前的
                    while idx == *current && accounts.len() > 1 {
                        idx = rng.gen_range(0..accounts.len());
                    }
                    idx
                }
            }
            RotateStrategy::Priority => {
                // 选择列表中下一个可用的（按顺序优先）
                let mut idx = (*current + 1) % accounts.len();
                for _ in 0..accounts.len() {
                    if accounts[idx].status != "expired" {
                        break;
                    }
                    idx = (idx + 1) % accounts.len();
                }
                idx
            }
            RotateStrategy::LeastUsed | RotateStrategy::ByExpiry => {
                // 这两种策略需要额外信息，暂时 fallback 到 Sequential
                (*current + 1) % accounts.len()
            }
        };

        *current = next_idx;
        accounts[next_idx].status = "current".to_string();
        // 将旧的标记回 available
        for (i, acc) in accounts.iter_mut().enumerate() {
            if i != next_idx && acc.status == "current" {
                acc.status = "available".to_string();
            }
        }

        let id = accounts[next_idx].id.clone();
        drop(accounts);
        drop(current);
        let _ = self.save().await;
        Some(id)
    }

    /// 查找并切换到第一个余额 >= threshold 的账号
    /// 如果找不到满足条件的账号，不切换（保持当前）
    pub async fn rotate_to_sufficient_balance(&self, threshold: u64) -> Option<String> {
        let mut accounts = self.accounts.write().await;
        let mut current = self.current.write().await;
        
        if accounts.is_empty() { return None; }
        
        let len = accounts.len();
        let start = (*current + 1) % len;
        
        for i in 0..len {
            let idx = (start + i) % len;
            if idx == *current { continue; }
            
            let acc = &accounts[idx];
            if acc.status == "expired" { continue; }
            
            let remaining = acc.quota_total.unwrap_or(u64::MAX)
                .saturating_sub(acc.quota_used.unwrap_or(0));
            if remaining >= threshold {
                // 找到满足条件的账号，切换
                accounts[*current].status = "available".to_string();
                *current = idx;
                accounts[idx].status = "current".to_string();
                let id = accounts[idx].id.clone();
                drop(accounts);
                drop(current);
                let _ = self.save().await;
                return Some(id);
            }
        }
        None // 没有找到余额充足的账号
    }

    pub async fn add(&self, token: String, label: String, user_id: String, email: String, name: String, user_type: String, expire_date: String) {
        let id = format!("tok_{:x}", chrono::Utc::now().timestamp_millis());
        let entry = AccountToken {
            id,
            label,
            token,
            user_id,
            email,
            name,
            status: "available".to_string(),
            user_type,
            expire_date,
            quota_used: None,
            quota_total: None,
            machine_token: String::new(),
            machine_id: String::new(),
            machine_code: String::new(),
            machine_type: String::new(),
        };
        let mut accounts = self.accounts.write().await;
        accounts.push(entry);
        drop(accounts);
        if let Err(e) = self.save().await {
            log::warn!("保存账号失败: {}", e);
        }
    }

    pub async fn update_machine_info(&self, id: &str, machine_token: String, machine_id: String, machine_code: String, machine_type: String) {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
            account.machine_token = machine_token;
            account.machine_id = machine_id;
            account.machine_code = machine_code;
            account.machine_type = machine_type;
        }
        drop(accounts);
        let _ = self.save().await;
    }

    pub async fn bulk_add(&self, tokens_str: &str) {
        let tokens: Vec<&str> = tokens_str.split_whitespace().collect();
        for (i, t) in tokens.iter().enumerate() {
            if !t.is_empty() {
                self.add(
                    t.to_string(),
                    format!("Token #{}", i + 1),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ).await;
            }
        }
    }

    pub async fn update_account_info(&self, id: &str, user_type: String, expire_date: String) {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
            account.user_type = user_type;
            account.expire_date = expire_date;
        }
        drop(accounts);
        let _ = self.save().await;
    }

    pub async fn update_account_quota(&self, id: &str, quota_used: Option<u64>, quota_total: Option<u64>) {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
            account.quota_used = quota_used;
            account.quota_total = quota_total;
        }
        drop(accounts);
        let _ = self.save().await;
    }

    pub async fn remove(&self, id: &str) {
        let mut accounts = self.accounts.write().await;
        accounts.retain(|a| a.id != id);
        drop(accounts);
        if let Err(e) = self.save().await {
            log::warn!("保存账号失败: {}", e);
        }
    }

    pub async fn set_current(&self, id: &str) -> bool {
        let accounts = self.accounts.read().await;
        if let Some(idx) = accounts.iter().position(|a| a.id == id) {
            drop(accounts);
            let mut current = self.current.write().await;
            *current = idx;
            if let Err(e) = self.save().await {
                log::warn!("保存账号失败: {}", e);
            }
            true
        } else {
            false
        }
    }

    pub async fn list(&self) -> Vec<AccountToken> {
        let accounts = self.accounts.read().await;
        let idx = *self.current.read().await;
        accounts.iter().enumerate().map(|(i, a)| {
            let mut entry = a.clone();
            entry.status = if i == idx { "current".to_string() } else { "available".to_string() };
            entry
        }).collect()
    }

    pub async fn save(&self) -> Result<(), String> {
        let accounts = self.accounts.read().await;
        let data = serde_json::to_string_pretty(&*accounts)
            .map_err(|e| format!("序列化账号失败: {}", e))?;
        let encrypted = Self::encrypt_data(data.as_bytes())?;
        std::fs::write(&self.storage_path, encrypted)
            .map_err(|e| format!("写入账号文件失败: {}", e))?;
        Ok(())
    }

    pub async fn load(&self) -> Result<(), String> {
        match std::fs::read(&self.storage_path) {
            Ok(raw) => {
                // 尝试解密；如果失败则尝试明文 JSON（向后兼容旧格式）
                let json_str = match Self::decrypt_data(&raw) {
                    Ok(decrypted) => String::from_utf8(decrypted)
                        .map_err(|e| format!("解密数据非有效 UTF-8: {}", e))?,
                    Err(_) => {
                        // 可能是旧版明文 JSON 格式，尝试直接解析
                        match String::from_utf8(raw.clone()) {
                            Ok(s) => {
                                log::info!("检测到旧版明文存储，将自动迁移为加密格式");
                                s
                            }
                            Err(e) => return Err(format!("读取账号数据失败: {}", e)),
                        }
                    }
                };
                let accounts: Vec<AccountToken> = serde_json::from_str(&json_str)
                    .map_err(|e| format!("解析账号文件失败: {}", e))?;
                *self.accounts.write().await = accounts;
                // 如果是旧格式，重新保存为加密格式
                drop(json_str);
                if Self::decrypt_data(&std::fs::read(&self.storage_path).unwrap_or_default()).is_err() {
                    if let Err(e) = self.save().await {
                        log::warn!("迁移加密格式失败: {}", e);
                    }
                }
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // 首次运行
            Err(e) => Err(format!("读取账号文件失败: {}", e)),
        }
    }

    /// 获取基于机器信息的加密密钥
    fn get_machine_key() -> [u8; 32] {
        let machine_name = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "default_host".to_string());
        let user_name = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "default_user".to_string());
        let machine_id = format!("{}_{}", machine_name, user_name);
        let mut hasher = Sha256::new();
        hasher.update(machine_id.as_bytes());
        hasher.finalize().into()
    }

    /// AES-256-GCM 加密：输出格式 [12 bytes nonce][ciphertext+tag]
    fn encrypt_data(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = Self::get_machine_key();
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("加密失败: {}", e))?;
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// AES-256-GCM 解密：输入格式 [12 bytes nonce][ciphertext+tag]
    fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
        if data.len() < 12 {
            return Err("数据太短，不是有效的加密格式".to_string());
        }
        let key = Self::get_machine_key();
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let nonce = GenericArray::from_slice(&data[..12]);
        let plaintext = cipher.decrypt(nonce, &data[12..])
            .map_err(|e| format!("解密失败: {}", e))?;
        Ok(plaintext)
    }
}
