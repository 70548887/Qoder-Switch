<template>
  <div class="min-h-screen bg-dark-900 flex flex-col">
    <!-- 顶部状态栏 -->
    <StatusBar @showGuide="store.activeTab = 'guide'" />

    <!-- 操作栏 -->
    <AccountForm />

    <!-- 主内容区 -->
    <div class="flex flex-1 overflow-hidden">
      <!-- 左侧标签导航 -->
      <nav class="w-24 bg-dark-800 border-r border-gray-700 flex flex-col items-center py-4 gap-1">
        <NavItem :icon="icons.metrics" :label="t('nav.metrics')" :active="store.activeTab === 'metrics'" @click="store.activeTab = 'metrics'" />
        <NavItem :icon="icons.accounts" :label="t('nav.accounts')" :active="store.activeTab === 'accounts'" @click="store.activeTab = 'accounts'" />
        <NavItem :icon="icons.chats" label="会话管理" :active="store.activeTab === 'chats'" @click="store.activeTab = 'chats'" />
        <NavItem :icon="icons.guide" :label="t('nav.guide')" :active="store.activeTab === 'guide'" @click="store.activeTab = 'guide'" />
        <NavItem :icon="icons.logs" :label="t('nav.logs')" :active="store.activeTab === 'logs'" @click="store.activeTab = 'logs'" />
        <NavItem :icon="icons.settings" :label="t('nav.settings')" :active="store.activeTab === 'settings'" @click="store.activeTab = 'settings'" />
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

import ConfigPanel from './components/ConfigPanel.vue'
import ChatHistoryPanel from './components/ChatHistoryPanel.vue'

const store = useAppStore()
const { t, setLocale } = useI18n()
const currentAccount = computed(() => store.accounts.find(a => a.status === 'current'))

// SVG 线条图标（对标 Qoder Switch 风格）
const icons = {
  accounts: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>',
  metrics: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>',
  logs: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>',
  chats: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>',
  guide: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg>',
  settings: '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>',
}

const panelMap: Record<string, any> = {
  accounts: AccountList,
  metrics: MetricsPanel,
  logs: LogsPanel,
  chats: ChatHistoryPanel,
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
