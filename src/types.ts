export interface AccountToken {
  id: string
  label: string
  token: string
  user_id: string
  email: string
  name: string
  status: string // "available" | "current" | "expired" | "error"
  user_type: string
  expire_date: string
}

export interface ProxyStatus {
  running: boolean
  port: number
  auto_rotate: boolean
  token_count: number
  current_token_id: string | null
  cert_installed: boolean
}

export interface MetricsSnapshot {
  total_requests: number
  intercepted: number
  passthrough: number
  auth_failures: number
  rotations: number
  bytes_proxied: number
  uptime_seconds: number
  intercept_rate: number
}

export interface RequestLogEntry {
  id: number
  timestamp: string
  method: string
  host: string
  path: string
  status: number | null
  injected: boolean
  token_label: string | null
}

export interface UserPlan {
  user_type: string
  plan_name: string
  start_date: number
  end_date: number
  is_expired: boolean
  days_remaining: number
}

export interface QuotaResult {
  account_id: string
  label: string
  plan: UserPlan | null
  error: string | null
}

export interface ProxyConfig {
  port: number
  target_domains: string[]
  auto_rotate: boolean
  rotate_strategy: 'Sequential' | 'Random' | 'Priority' | 'LeastUsed' | 'ByExpiry'
  auto_start: boolean
  max_retry: number
  language: string
}

export interface FetchedAccount {
  user_id: string
  token: string
  name: string
  email: string
  user_type: string
  expire_date: string
  machine_token: string
  machine_id: string
}
