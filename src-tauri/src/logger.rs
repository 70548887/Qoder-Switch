use serde::Serialize;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

#[derive(Serialize, Clone)]
pub struct RequestLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub method: String,
    pub host: String,
    pub path: String,
    pub status: Option<u16>,
    pub injected: bool,
    pub token_label: Option<String>,
}

pub struct RequestLogger {
    logs: RwLock<VecDeque<RequestLogEntry>>,
    domains: RwLock<HashSet<String>>,
    target_domains: RwLock<Vec<String>>,
    counter: AtomicU64,
    max_entries: usize,
}

impl RequestLogger {
    pub fn new(max_entries: usize, initial_targets: Vec<String>) -> Self {
        Self {
            logs: RwLock::new(VecDeque::new()),
            domains: RwLock::new(HashSet::new()),
            target_domains: RwLock::new(initial_targets),
            counter: AtomicU64::new(0),
            max_entries,
        }
    }

    pub async fn log_request(&self, mut entry: RequestLogEntry) {
        entry.id = self.counter.fetch_add(1, Ordering::Relaxed);
        let mut logs = self.logs.write().await;
        if logs.len() >= self.max_entries {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    pub async fn record_domain(&self, domain: String) {
        self.domains.write().await.insert(domain);
    }

    pub async fn get_logs(&self, limit: usize) -> Vec<RequestLogEntry> {
        let logs = self.logs.read().await;
        logs.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_discovered_domains(&self) -> Vec<String> {
        let domains = self.domains.read().await;
        let mut list: Vec<String> = domains.iter().cloned().collect();
        list.sort();
        list
    }

    pub async fn get_target_domains(&self) -> Vec<String> {
        self.target_domains.read().await.clone()
    }

    pub async fn set_target_domains(&self, domains: Vec<String>) {
        *self.target_domains.write().await = domains;
    }

    pub async fn is_target_domain(&self, domain: &str) -> bool {
        let targets = self.target_domains.read().await;
        let domain_lower = domain.to_lowercase();
        targets.iter().any(|d| {
            let d_lower = d.to_lowercase();
            domain_lower == d_lower || domain_lower.ends_with(&format!(".{}", d_lower))
        })
    }
}
