<template>
  <div class="bg-dark-800 rounded-xl p-6 max-w-2xl">
    <h2 class="text-xl font-bold text-gray-200 mb-4">设置</h2>

    <!-- Tab 切换 -->
    <div class="flex border-b border-gray-700 mb-4">
      <button @click="settingsTab = 'general'" :class="settingsTab === 'general' ? 'border-b-2 border-accent text-white' : 'text-gray-400 hover:text-gray-200'" class="px-4 py-2 text-sm font-medium transition-colors">基本设置</button>
      <button @click="settingsTab = 'domains'" :class="settingsTab === 'domains' ? 'border-b-2 border-accent text-white' : 'text-gray-400 hover:text-gray-200'" class="px-4 py-2 text-sm font-medium transition-colors">域名管理</button>
    </div>

    <!-- Tab 1: 基本设置 -->
    <div v-if="settingsTab === 'general'" class="space-y-6">
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

      <!-- 自动切换阈值 -->
      <div v-if="form.auto_rotate">
        <label class="block text-sm font-medium text-gray-300 mb-1">自动切换阈值</label>
        <div class="flex items-center gap-3">
          <input
            type="number"
            v-model.number="form.balance_threshold"
            min="0"
            max="10000"
            step="1"
            class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white w-32"
          />
          <span class="text-xs text-gray-400">credits</span>
        </div>
        <p class="text-xs text-gray-500 mt-1">余额低于此值时自动切换到下一个账号（0=不限制）</p>
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

    <!-- Tab 2: 域名管理 -->
    <div v-if="settingsTab === 'domains'">
      <div class="flex items-center justify-between mb-4">
        <span class="text-sm text-gray-400">管理代理拦截的目标域名</span>
        <button
          @click="handleDomainsSave"
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
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import { useI18n } from '../i18n'
import type { ProxyConfig } from '../types'

const store = useAppStore()
const { setLocale } = useI18n()

const settingsTab = ref<'general' | 'domains'>('general')

const form = reactive<ProxyConfig>({
  port: 5888,
  target_domains: [],
  auto_rotate: true,
  rotate_strategy: 'Sequential',
  auto_start: false,
  max_retry: 3,
  language: 'zh',
  account_pool_url: '',
  balance_threshold: 10,
})

// 域名管理相关
const targetDomains = ref<string[]>([])

function addDomain(domain: string) {
  if (!targetDomains.value.includes(domain)) {
    targetDomains.value.push(domain)
  }
}

function removeDomain(domain: string) {
  targetDomains.value = targetDomains.value.filter(d => d !== domain)
}

async function handleDomainsSave() {
  await store.setTargetDomains(targetDomains.value)
}

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
    targetDomains.value = [...store.config.target_domains]
  }
  await store.fetchDomains()
})
</script>
