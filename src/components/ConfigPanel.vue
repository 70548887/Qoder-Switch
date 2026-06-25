<template>
  <div class="bg-dark-800 rounded-xl p-6 max-w-2xl">
    <h2 class="text-xl font-bold text-gray-200 mb-6">设置</h2>

    <div class="space-y-6">
      <!-- 代理端口 -->
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">代理端口</label>
        <input
          :value="form.port"
          readonly
          class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-32 cursor-not-allowed opacity-70"
        />
        <p class="text-xs text-gray-500 mt-1">修改端口需重启代理</p>
      </div>

      <!-- 轮换策略 -->
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">轮换策略</label>
        <select
          v-model="form.rotate_strategy"
          class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-48"
        >
          <option value="Sequential">顺序</option>
          <option value="Random">随机</option>
          <option value="Priority">优先级</option>
          <option value="LeastUsed">最少使用</option>
          <option value="ByExpiry">按到期时间</option>
        </select>
      </div>

      <!-- 自动轮换 -->
      <div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            v-model="form.auto_rotate"
            class="accent-accent w-4 h-4"
          />
          <span class="text-sm font-medium text-gray-300">自动轮换</span>
        </label>
        <p class="text-xs text-gray-500 mt-1">余量不足时自动切换到下一个账号</p>
      </div>

      <!-- 自动启动 -->
      <div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            v-model="form.auto_start"
            class="accent-accent w-4 h-4"
          />
          <span class="text-sm font-medium text-gray-300">自动启动代理</span>
        </label>
      </div>

      <!-- 最大重试次数 -->
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">最大重试次数</label>
        <input
          type="number"
          v-model.number="form.max_retry"
          min="0"
          max="10"
          class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-24"
        />
      </div>

      <!-- 语言 -->
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">语言</label>
        <select
          v-model="form.language"
          @change="handleLanguageChange(form.language)"
          class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-48"
        >
          <option value="zh">中文</option>
          <option value="en">English</option>
        </select>
      </div>

      <!-- 数据目录 -->
      <div>
        <label class="block text-sm font-medium text-gray-300 mb-1">数据目录</label>
        <div class="text-sm text-gray-400 font-mono">%APPDATA%\Qoder\</div>
      </div>

      <!-- 保存按钮 -->
      <div class="pt-2">
        <button
          @click="handleSave"
          class="px-4 py-2 bg-accent hover:bg-accent-hover text-white text-sm rounded transition-colors"
        >保存配置</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import { useI18n } from '../i18n'
import type { ProxyConfig } from '../types'

const store = useAppStore()
const { t, setLocale } = useI18n()

const form = reactive<ProxyConfig>({
  port: 5888,
  target_domains: [],
  auto_rotate: true,
  rotate_strategy: 'Sequential',
  auto_start: false,
  max_retry: 3,
  language: 'zh',
})

function handleLanguageChange(lang: string) {
  setLocale(lang)
}

async function handleSave() {
  await store.updateConfig({ ...form })
}

onMounted(async () => {
  await store.fetchConfig()
  if (store.config) {
    Object.assign(form, store.config)
  }
})
</script>
