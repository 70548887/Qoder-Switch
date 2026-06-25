<template>
  <div class="bg-dark-800 border-b border-gray-700 px-4 py-2.5 flex items-center justify-between">
    <!-- 左侧：Logo + 产品名 + 版本 + 按钮 -->
    <div class="flex items-center gap-3">
      <!-- Logo -->
      <div class="w-7 h-7 bg-blue-600 rounded-lg flex items-center justify-center">
        <span class="text-white font-bold text-sm">Q</span>
      </div>
      <!-- 产品名 -->
      <span class="text-white font-semibold text-sm">Qoder Proxy</span>
      <!-- 版本号 -->
      <span class="bg-gray-700 text-gray-300 text-xs px-2 py-0.5 rounded">v1.0.0</span>
      <!-- 检测更新 -->
      <button @click="checkUpdate" class="text-blue-400 text-xs hover:text-blue-300 transition">
        检测更新
      </button>
      <!-- 使用说明 -->
      <button @click="$emit('showGuide')" class="border border-blue-500 text-blue-400 text-xs px-3 py-1 rounded hover:bg-blue-500/10 transition">
        使用说明
      </button>
    </div>
    
    <!-- 右侧：状态指示灯 -->
    <div class="flex items-center gap-6">
      <!-- 代理状态 -->
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full" :class="store.status?.running ? 'bg-green-500' : 'bg-gray-500'"></span>
        <span class="text-gray-400 text-xs">代理</span>
        <span class="text-xs" :class="store.status?.running ? 'text-green-400' : 'text-gray-500'">
          {{ store.status?.running ? '运行中' : '已停止' }}
        </span>
      </div>
      <!-- 证书状态 -->
      <div class="flex items-center gap-2">
        <span class="w-2.5 h-2.5 rounded-full" :class="store.status?.cert_installed ? 'bg-green-500' : 'bg-gray-500'"></span>
        <span class="text-gray-400 text-xs">证书</span>
        <span class="text-xs" :class="store.status?.cert_installed ? 'text-green-400' : 'text-gray-500'">
          {{ store.status?.cert_installed ? '已安装' : '未安装' }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useAppStore } from '../stores/app'

const store = useAppStore()

defineEmits(['showGuide'])

async function checkUpdate() {
  alert('当前版本 v1.0.0，自动更新将在后续版本启用')
}
</script>
