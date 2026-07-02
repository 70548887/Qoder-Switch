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
  quota_used?: number | null
  quota_total?: number | null
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
  request_headers: string | null
  request_body: string | null
  response_headers: string | null
  response_body: string | null
  is_websocket: boolean
}

export interface QuotaResult {
  account_id: string
  label: string
  plan_name: string
  user_type: string
  quota_used: number
  quota_total: number
  quota_remaining: number
  quota_unit: string
  is_exceeded: boolean
  expire_date: string
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
  account_pool_url: string
  balance_threshold: number
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

export interface ChatHistory {
  id: string
  title: string
  context: Array<{ id: string; name: string; type: string }>
  timestamp: number
  session_id: string
  sessionId?: string
}

export interface WorkspaceInfo {
  id: string
  name: string
  path: string
}

export interface SessionBackup {
  backupTime: string
  workspaceId: string
  workspacePath: string
  userId: string
  chatHistory: ChatHistory[]
  chatViews: any
  chatTabs: any
}

export interface BackupFileInfo {
  file_path: string
  file_name: string
  backup_time: string
  workspace_id: string
  workspace_path: string
  file_size: number
}
