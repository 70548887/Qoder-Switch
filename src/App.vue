<template>
  <div class="min-h-screen bg-dark-900 flex flex-col">
    <!-- 顶部状态栏 -->
    <StatusBar @showGuide="store.activeTab = 'guide'" />

    <!-- 操作栏 -->
    <AccountForm />

    <!-- 主内容区 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧标签导航 -->
      <nav class="w-28 bg-dark-800 border-r border-gray-700 flex flex-col items-center py-4 gap-2">
        <NavItem icon="👤" :label="t('nav.accounts')" :active="store.activeTab === 'accounts'" @click="store.activeTab = 'accounts'" />
        <NavItem icon="📊" :label="t('nav.metrics')" :active="store.activeTab === 'metrics'" @click="store.activeTab = 'metrics'" />
        <NavItem icon="📝" :label="t('nav.logs')" :active="store.activeTab === 'logs'" @click="store.activeTab = 'logs'" />
        <NavItem icon="💳" :label="t('nav.quota')" :active="store.activeTab === 'quota'" @click="store.activeTab = 'quota'" />
        <NavItem icon="🌐" :label="t('nav.domains')" :active="store.activeTab === 'domains'" @click="store.activeTab = 'domains'" />
        <NavItem icon="📋" :label="t('nav.guide')" :active="store.activeTab === 'guide'" @click="store.activeTab = 'guide'" />
        <NavItem icon="⚙️" :label="t('nav.settings')" :active="store.activeTab === 'settings'" @click="store.activeTab = 'settings'" />
      </nav>

      <!-- 右侧内容 -->
      <main class="flex-1 overflow-auto p-4">
        <component :is="currentPanel" />
      </main>
    </div>

    <!-- 底部状态栏 -->
    <footer class="h-8 bg-dark-800 border-t border-gray-700 flex items-center px-4 text-xs text-gray-500">
      <span v-if="currentAccount" class="flex items-center gap-1">
        <span class="w-2 h-2 rounded-full bg-green-500"></span>
        自动接入: {{ currentAccount.label }}
      </span>
      <span class="ml-auto">头数量: {{ store.accounts.length }}</span>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from './stores/app'
import { useI18n } from './i18n'
import StatusBar from './components/StatusBar.vue'
import AccountForm from './components/AccountForm.vue'
import AccountList from './components/AccountList.vue'
import GuidePanel from './components/GuidePanel.vue'
import NavItem from './components/NavItem.vue'
import MetricsPanel from './components/MetricsPanel.vue'
import LogsPanel from './components/LogsPanel.vue'
import QuotaPanel from './components/QuotaPanel.vue'
import DomainsPanel from './components/DomainsPanel.vue'
import ConfigPanel from './components/ConfigPanel.vue'

const store = useAppStore()
const { t, setLocale } = useI18n()
const currentAccount = computed(() => store.accounts.find(a => a.status === 'current'))

const panelMap: Record<string, any> = {
  accounts: AccountList,
  metrics: MetricsPanel,
  logs: LogsPanel,
  quota: QuotaPanel,
  domains: DomainsPanel,
  guide: GuidePanel,
  settings: ConfigPanel,
}

const currentPanel = computed(() => panelMap[store.activeTab] || AccountList)

let statusTimer: number | undefined

onMounted(async () => {
  await store.fetchConfig()
  if (store.config) {
    setLocale(store.config.language)
  }
  if (store.config?.auto_start) {
    await store.startProxy()
  }
  await store.fetchStatus()
  await store.fetchAccounts()

  // 托盘事件监听
  await listen('tray-start-proxy', () => { store.startProxy() })
  await listen('tray-stop-proxy', () => { store.stopProxy() })

  // 每 5 秒刷新状态
  statusTimer = window.setInterval(() => store.fetchStatus(), 5000)
})

onUnmounted(() => {
  if (statusTimer !== undefined) {
    clearInterval(statusTimer)
  }
})
</script>
