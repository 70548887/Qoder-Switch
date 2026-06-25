<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-bold text-gray-200">请求日志</h2>
      <button
        @click="store.fetchLogs()"
        class="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors"
      >刷新</button>
    </div>

    <div class="overflow-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-gray-500 text-left border-b border-gray-700">
            <th class="py-2 pr-3">时间</th>
            <th class="py-2 pr-3">方法</th>
            <th class="py-2 pr-3">域名</th>
            <th class="py-2 pr-3">路径</th>
            <th class="py-2 pr-3">状态</th>
            <th class="py-2">注入</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="log in sortedLogs"
            :key="log.id"
            class="border-b border-gray-700/50 hover:bg-dark-700/50"
          >
            <td class="py-2 pr-3 text-xs text-gray-400 whitespace-nowrap">{{ formatTime(log.timestamp) }}</td>
            <td class="py-2 pr-3 font-mono text-xs text-gray-300">{{ log.method }}</td>
            <td class="py-2 pr-3 text-gray-300 text-xs">{{ log.host }}</td>
            <td class="py-2 pr-3 text-gray-400 text-xs font-mono truncate max-w-[200px]">{{ log.path }}</td>
            <td class="py-2 pr-3">
              <span
                class="text-xs font-medium"
                :class="statusColor(log.status)"
              >{{ log.status ?? '-' }}</span>
            </td>
            <td class="py-2 text-center">
              <span v-if="log.injected" class="text-green-400">✓</span>
              <span v-else class="text-gray-600">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="!store.logs.length" class="text-center text-gray-500 py-8">
      暂无请求日志
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()

const sortedLogs = computed(() => {
  return [...store.logs].sort((a, b) => {
    return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  })
})

function formatTime(ts: string) {
  const d = new Date(ts)
  return d.toLocaleTimeString('zh-CN', { hour12: false })
}

function statusColor(status: number | null) {
  if (status === null) return 'text-gray-500'
  if (status >= 200 && status < 300) return 'text-green-400'
  if (status >= 400 && status < 500) return 'text-orange-400'
  if (status >= 500) return 'text-red-400'
  return 'text-gray-400'
}

onMounted(() => {
  store.fetchLogs()
})
</script>
