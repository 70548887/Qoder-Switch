use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ProxyMetrics {
    pub total_requests: AtomicU64,
    pub intercepted: AtomicU64,
    pub passthrough: AtomicU64,
    pub auth_failures: AtomicU64,
    pub rotations: AtomicU64,
    pub bytes_proxied: AtomicU64,
    start_time: Instant,
}

#[derive(Serialize, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub intercepted: u64,
    pub passthrough: u64,
    pub auth_failures: u64,
    pub rotations: u64,
    pub bytes_proxied: u64,
    pub uptime_seconds: u64,
    pub intercept_rate: f64,
}

impl ProxyMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            intercepted: AtomicU64::new(0),
            passthrough: AtomicU64::new(0),
            auth_failures: AtomicU64::new(0),
            rotations: AtomicU64::new(0),
            bytes_proxied: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn record_request(&self, is_intercepted: bool, bytes: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.bytes_proxied.fetch_add(bytes, Ordering::Relaxed);
        if is_intercepted {
            self.intercepted.fetch_add(1, Ordering::Relaxed);
        } else {
            self.passthrough.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rotation(&self) {
        self.rotations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.total_requests.load(Ordering::Relaxed);
        let intercepted = self.intercepted.load(Ordering::Relaxed);
        MetricsSnapshot {
            total_requests: total,
            intercepted,
            passthrough: self.passthrough.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            rotations: self.rotations.load(Ordering::Relaxed),
            bytes_proxied: self.bytes_proxied.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            intercept_rate: if total > 0 { intercepted as f64 / total as f64 } else { 0.0 },
        }
    }

    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.intercepted.store(0, Ordering::Relaxed);
        self.passthrough.store(0, Ordering::Relaxed);
        self.auth_failures.store(0, Ordering::Relaxed);
        self.rotations.store(0, Ordering::Relaxed);
        self.bytes_proxied.store(0, Ordering::Relaxed);
    }
}
