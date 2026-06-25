<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold text-gray-200">域名管理</h2>
      <button
        @click="handleSave"
        class="px-4 py-1.5 bg-accent hover:bg-accent-hover text-white text-xs rounded transition-colors"
      >保存</button>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- 左栏：已发现域名 -->
      <div>
        <h3 class="text-sm font-medium text-gray-300 mb-3">已发现域名</h3>
        <div class="bg-dark-700 rounded-lg p-3 max-h-80 overflow-auto space-y-1">
          <div
            v-for="domain in store.discoveredDomains"
            :key="domain"
            class="flex items-center justify-between py-1.5 px-2 rounded hover:bg-dark-600 group"
          >
            <span class="text-sm text-gray-300 font-mono">{{ domain }}</span>
            <button
              v-if="!targetDomains.includes(domain)"
              @click="addDomain(domain)"
              class="text-xs text-accent opacity-0 group-hover:opacity-100 transition-opacity"
            >+ 添加</button>
            <span v-else class="text-xs text-green-400">已添加</span>
          </div>
          <div v-if="!store.discoveredDomains.length" class="text-gray-500 text-sm text-center py-4">
            暂无发现的域名
          </div>
        </div>
        <button
          @click="store.fetchDomains()"
          class="mt-2 text-xs text-gray-400 hover:text-gray-300"
        >刷新列表</button>
      </div>

      <!-- 右栏：拦截目标 -->
      <div>
        <h3 class="text-sm font-medium text-gray-300 mb-3">拦截目标</h3>
        <div class="bg-dark-700 rounded-lg p-3 max-h-80 overflow-auto space-y-1">
          <div
            v-for="domain in targetDomains"
            :key="domain"
            class="flex items-center justify-between py-1.5 px-2 rounded hover:bg-dark-600 group"
          >
            <span class="text-sm text-gray-300 font-mono">{{ domain }}</span>
            <button
              @click="removeDomain(domain)"
              class="text-xs text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
            >删除</button>
          </div>
          <div v-if="!targetDomains.length" class="text-gray-500 text-sm text-center py-4">
            暂无拦截目标，从左侧添加
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()
const targetDomains = ref<string[]>([])

function addDomain(domain: string) {
  if (!targetDomains.value.includes(domain)) {
    targetDomains.value.push(domain)
  }
}

function removeDomain(domain: string) {
  targetDomains.value = targetDomains.value.filter(d => d !== domain)
}

async function handleSave() {
  await store.setTargetDomains(targetDomains.value)
}

onMounted(async () => {
  await store.fetchDomains()
  await store.fetchConfig()
  if (store.config) {
    targetDomains.value = [...store.config.target_domains]
  }
})
</script>
