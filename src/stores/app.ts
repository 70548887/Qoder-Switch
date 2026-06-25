import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AccountToken, ProxyStatus, MetricsSnapshot, RequestLogEntry, QuotaResult, ProxyConfig } from '../types'

interface StructuredAccount {
  token: string
  label?: string
  user_id?: string
  email?: string
  name?: string
}

export const useAppStore = defineStore('app', {
  state: () => ({
    accounts: [] as AccountToken[],
    status: null as ProxyStatus | null,
    activeTab: 'accounts' as string,
    loading: false,
    metrics: null as MetricsSnapshot | null,
    logs: [] as RequestLogEntry[],
    quotas: [] as QuotaResult[],
    discoveredDomains: [] as string[],
    config: null as ProxyConfig | null,
  }),

  actions: {
    async fetchStatus() {
      this.status = await invoke<ProxyStatus>('get_proxy_status')
    },
    async fetchAccounts() {
      this.accounts = await invoke<AccountToken[]>('list_accounts')
    },
    async fetchPoolAccount(secretKey: string) {
      await invoke('fetch_pool_account', { secret_key: secretKey })
      await this.fetchAccounts()
      await this.fetchStatus()
    },
    async addAccounts(tokens: string) {
      await invoke('add_accounts', { tokens })
      await this.fetchAccounts()
      await this.fetchStatus()
    },
    async addStructuredAccounts(accounts: StructuredAccount[]) {
      const tokens = JSON.stringify(accounts)
      await invoke('add_accounts', { tokens })
      await this.fetchAccounts()
      await this.fetchStatus()
    },
    async removeAccount(id: string) {
      await invoke('remove_account', { id })
      await this.fetchAccounts()
    },
    async switchAccount(id: string) {
      await invoke('switch_account', { id })
      await this.fetchAccounts()
      await this.fetchStatus()
    },
    async startProxy() {
      await invoke('start_proxy')
      await this.fetchStatus()
    },
    async stopProxy() {
      await invoke('stop_proxy')
      await this.fetchStatus()
    },
    async installCert() {
      const confirmed = window.confirm(
        '即将为当前用户安装 Qoder Proxy CA 根证书。\n\n' +
        '影响范围：当前 Windows 用户下所有使用系统根证书的应用将信任此 CA。\n' +
        '用途：仅用于本地代理 (127.0.0.1) 解密 HTTPS 流量。\n\n' +
        '确定继续？'
      )
      if (!confirmed) return
      await invoke('install_cert')
      await this.fetchStatus()
    },
    async uninstallCert() {
      await invoke('uninstall_cert')
      await this.fetchStatus()
    },
    async toggleAutoRotate(enabled: boolean) {
      await invoke('set_auto_rotate', { enabled })
      await this.fetchStatus()
    },
    async fetchMetrics() {
      this.metrics = await invoke<MetricsSnapshot>('get_metrics')
    },
    async resetMetrics() {
      await invoke('reset_metrics')
      this.metrics = await invoke<MetricsSnapshot>('get_metrics')
    },
    async fetchLogs(limit = 100) {
      this.logs = await invoke<RequestLogEntry[]>('get_request_logs', { limit })
    },
    async checkAllQuotas() {
      this.quotas = await invoke<QuotaResult[]>('check_all_quotas')
    },
    async fetchDomains() {
      this.discoveredDomains = await invoke<string[]>('get_discovered_domains')
    },
    async setTargetDomains(domains: string[]) {
      await invoke('set_target_domains', { domains })
    },
    async fetchConfig() {
      this.config = await invoke<ProxyConfig>('get_config')
    },
    async updateConfig(config: ProxyConfig) {
      await invoke('update_config', { config })
      this.config = config
    },
  }
})
