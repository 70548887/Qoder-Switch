<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold text-gray-200">仪表盘</h2>
      <button
        @click="handleReset"
        class="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors"
      >重置统计</button>
    </div>

    <!-- 数据卡片 -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
      <div class="bg-dark-700 rounded-lg p-4">
        <div class="text-xs text-gray-400 mb-1">总请求数</div>
        <div class="text-2xl font-bold text-gray-200">{{ metrics?.total_requests ?? 0 }}</div>
      </div>
      <div class="bg-dark-700 rounded-lg p-4">
        <div class="text-xs text-gray-400 mb-1">已拦截</div>
        <div class="text-2xl font-bold text-green-400">{{ metrics?.intercepted ?? 0 }}</div>
      </div>
      <div class="bg-dark-700 rounded-lg p-4">
        <div class="text-xs text-gray-400 mb-1">认证失败</div>
        <div class="text-2xl font-bold text-red-400">{{ metrics?.auth_failures ?? 0 }}</div>
      </div>
      <div class="bg-dark-700 rounded-lg p-4">
        <div class="text-xs text-gray-400 mb-1">自动轮换</div>
        <div class="text-2xl font-bold text-accent">{{ metrics?.rotations ?? 0 }}</div>
      </div>
    </div>

    <!-- 拦截率与运行时间 -->
    <div class="flex items-center gap-6 text-sm text-gray-400">
      <span>拦截率: <strong class="text-gray-200">{{ interceptRate }}</strong></span>
      <span>运行时间: <strong class="text-gray-200">{{ uptime }}</strong></span>
      <span>流量: <strong class="text-gray-200">{{ bytesDisplay }}</strong></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()
const metrics = computed(() => store.metrics)

const interceptRate = computed(() => {
  if (!metrics.value) return '0%'
  return (metrics.value.intercept_rate * 100).toFixed(1) + '%'
})

const uptime = computed(() => {
  if (!metrics.value) return '0s'
  const s = metrics.value.uptime_seconds
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s % 60}s`
  return `${s}s`
})

const bytesDisplay = computed(() => {
  if (!metrics.value) return '0 B'
  const b = metrics.value.bytes_proxied
  if (b < 1024) return b + ' B'
  if (b < 1024 * 1024) return (b / 1024).toFixed(1) + ' KB'
  return (b / 1024 / 1024).toFixed(1) + ' MB'
})

async function handleReset() {
  await store.resetMetrics()
}

let timer: number | undefined

onMounted(async () => {
  await store.fetchMetrics()
  timer = window.setInterval(() => store.fetchMetrics(), 3000)
})

onUnmounted(() => {
  if (timer !== undefined) clearInterval(timer)
})
</script>
