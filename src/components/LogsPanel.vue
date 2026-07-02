<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-bold text-gray-200">请求日志</h2>
      <button
        @click="store.fetchLogs()"
        class="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors"
      >刷新</button>
    </div>

    <div class="overflow-auto max-h-[600px]">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-gray-500 text-left border-b border-gray-700">
            <th class="py-2 pr-2 w-6"></th>
            <th class="py-2 pr-3">时间</th>
            <th class="py-2 pr-3">方法</th>
            <th class="py-2 pr-3">域名</th>
            <th class="py-2 pr-3">路径</th>
            <th class="py-2 pr-3">状态</th>
            <th class="py-2 pr-2">注入</th>
            <th class="py-2">WS</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="log in sortedLogs" :key="log.id">
            <tr
              class="border-b border-gray-700/50 hover:bg-dark-700/50 cursor-pointer"
              @click="toggleDetail(log.id)"
            >
              <td class="py-2 pr-2 text-gray-500 text-xs">
                <span :class="expandedId === log.id ? 'rotate-90' : ''" class="inline-block transition-transform">▶</span>
              </td>
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
              <td class="py-2 pr-2 text-center">
                <span v-if="log.injected" class="text-green-400">✓</span>
                <span v-else class="text-gray-600">—</span>
              </td>
              <td class="py-2 text-center">
                <span v-if="log.is_websocket" class="text-purple-400 text-xs font-medium">WS</span>
                <span v-else class="text-gray-600">—</span>
              </td>
            </tr>
            <!-- 展开的详情区域 -->
            <tr v-if="expandedId === log.id">
              <td colspan="8" class="p-0">
                <div class="bg-dark-900 border border-gray-700 rounded-lg m-2 p-4 text-xs">
                  <div class="grid grid-cols-2 gap-4">
                    <!-- 请求部分 -->
                    <div>
                      <h4 class="text-gray-300 font-semibold mb-2">请求头</h4>
                      <pre class="bg-dark-700 rounded p-2 text-gray-400 overflow-auto max-h-[200px] whitespace-pre-wrap break-all">{{ log.request_headers || '(无)' }}</pre>
                      <h4 class="text-gray-300 font-semibold mt-3 mb-2">请求体</h4>
                      <pre class="bg-dark-700 rounded p-2 text-gray-400 overflow-auto max-h-[150px] whitespace-pre-wrap break-all">{{ log.request_body || '(无)' }}</pre>
                    </div>
                    <!-- 响应部分 -->
                    <div>
                      <h4 class="text-gray-300 font-semibold mb-2">响应头</h4>
                      <pre class="bg-dark-700 rounded p-2 text-gray-400 overflow-auto max-h-[200px] whitespace-pre-wrap break-all">{{ log.response_headers || '(等待响应...)' }}</pre>
                      <h4 class="text-gray-300 font-semibold mt-3 mb-2">响应体</h4>
                      <pre class="bg-dark-700 rounded p-2 text-gray-400 overflow-auto max-h-[150px] whitespace-pre-wrap break-all">{{ log.response_body || '(等待响应...)' }}</pre>
                    </div>
                  </div>
                  <div v-if="log.token_label" class="mt-3 text-gray-500">
                    Token: <span class="text-gray-300">{{ log.token_label }}</span>
                  </div>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <div v-if="!store.logs.length" class="text-center text-gray-500 py-8">
      暂无请求日志
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()
const expandedId = ref<number | null>(null)

function toggleDetail(id: number) {
  expandedId.value = expandedId.value === id ? null : id
}

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
