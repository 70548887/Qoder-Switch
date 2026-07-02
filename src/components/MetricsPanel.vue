<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold text-gray-200">仪表盘</h2>
      <button @click="refresh" class="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors">刷新</button>
    </div>

    <!-- 统计卡片网格 -->
    <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
      <!-- 账号总数 -->
      <div class="bg-dark-900 rounded-lg p-4 border border-gray-700">
        <div class="text-3xl font-bold text-white">{{ stats.total_accounts }}</div>
        <div class="text-sm text-gray-400 mt-1">账号总数</div>
      </div>

      <!-- 已使用 -->
      <div class="bg-dark-900 rounded-lg p-4 border border-gray-700">
        <div class="text-3xl font-bold text-orange-400">{{ stats.used_accounts }}</div>
        <div class="text-sm text-gray-400 mt-1">已使用</div>
      </div>

      <!-- 未使用 -->
      <div class="bg-dark-900 rounded-lg p-4 border border-gray-700">
        <div class="text-3xl font-bold text-green-400">{{ stats.unused_accounts }}</div>
        <div class="text-sm text-gray-400 mt-1">未使用</div>
      </div>

      <!-- 会话总数 -->
      <div class="bg-dark-900 rounded-lg p-4 border border-gray-700">
        <div class="text-3xl font-bold text-blue-400">{{ stats.total_chats }}</div>
        <div class="text-sm text-gray-400 mt-1">会话对话</div>
      </div>

      <!-- 备份数量 -->
      <div class="bg-dark-900 rounded-lg p-4 border border-gray-700">
        <div class="text-3xl font-bold text-purple-400">{{ stats.total_backups }}</div>
        <div class="text-sm text-gray-400 mt-1">备份数量</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface DashboardStats {
  total_accounts: number
  used_accounts: number
  unused_accounts: number
  total_chats: number
  total_backups: number
}

const stats = ref<DashboardStats>({
  total_accounts: 0,
  used_accounts: 0,
  unused_accounts: 0,
  total_chats: 0,
  total_backups: 0,
})

async function refresh() {
  try {
    stats.value = await invoke<DashboardStats>('get_dashboard_stats')
  } catch (e) {
    console.error('获取仪表盘数据失败:', e)
  }
}

onMounted(() => {
  refresh()
})
</script>
