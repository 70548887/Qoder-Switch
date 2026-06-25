<template>
  <div class="flex flex-col h-full">
    <!-- 顶部标题区 -->
    <div class="px-4 pt-3 pb-2 border-b border-gray-700">
      <div class="flex items-center justify-between mb-2">
        <h2 class="text-white font-semibold text-sm">账号列表</h2>
        <div class="flex items-center gap-3">
          <span class="text-blue-400 text-xs">{{ accounts.length }}</span>
          <span v-if="selectedIds.length > 0" class="text-blue-400 text-xs">已选 {{ selectedIds.length }}</span>
          <button 
            v-if="selectedIds.length > 0" 
            @click="handleBatchDelete" 
            class="text-red-400 text-xs hover:text-red-300"
          >批量删除</button>
        </div>
      </div>
      <!-- 自动切换 toggle -->
      <div class="flex items-center gap-3">
        <button 
          @click="toggleAutoRotate"
          class="w-11 h-6 rounded-full transition-colors relative"
          :class="autoRotate ? 'bg-blue-600' : 'bg-gray-600'"
        >
          <span 
            class="absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform"
            :class="autoRotate ? 'left-[22px]' : 'left-0.5'"
          ></span>
        </button>
        <span class="text-white text-sm font-medium">自动切换</span>
        <span class="text-gray-500 text-xs">余量不足时自动接入下一个账号</span>
      </div>
    </div>

    <!-- 表格区域 -->
    <div class="flex-1 overflow-auto">
      <table class="w-full text-sm">
        <thead class="sticky top-0 bg-dark-800 border-b border-gray-700">
          <tr class="text-gray-400 text-xs">
            <th class="py-2 px-2 w-8">
              <input type="checkbox" :checked="isAllSelected" @change="toggleAll" class="accent-blue-500" />
            </th>
            <th class="py-2 px-2 text-left w-10">#</th>
            <th class="py-2 px-2 text-left">名称</th>
            <th class="py-2 px-2 text-left">邮箱</th>
            <th class="py-2 px-2 text-left w-24">类型</th>
            <th class="py-2 px-2 text-left w-28">到期</th>
            <th class="py-2 px-2 text-left w-16">余额</th>
            <th class="py-2 px-2 text-left w-16">状态</th>
            <th class="py-2 px-2 text-right w-40">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr 
            v-for="(account, index) in accounts" 
            :key="account.id"
            class="border-b border-gray-800 hover:bg-dark-700/50 transition"
          >
            <td class="py-2.5 px-2">
              <input type="checkbox" :checked="selectedIds.includes(account.id)" @change="toggleSelect(account.id)" class="accent-blue-500" />
            </td>
            <td class="py-2.5 px-2 text-gray-500">{{ index + 1 }}</td>
            <td class="py-2.5 px-2 text-white font-medium">{{ account.label || account.name || '-' }}</td>
            <td class="py-2.5 px-2 text-gray-400">{{ account.email || '-' }}</td>
            <td class="py-2.5 px-2 text-gray-400">{{ account.user_type || '-' }}</td>
            <td class="py-2.5 px-2 text-gray-400">{{ formatDate(account.expire_date) }}</td>
            <td class="py-2.5 px-2" :class="getBalanceClass(account)">{{ getBalanceText(account) }}</td>
            <td class="py-2.5 px-2">
              <span 
                class="text-xs px-2 py-0.5 rounded"
                :class="getStatusClass(account)"
              >{{ getStatusText(account) }}</span>
            </td>
            <td class="py-2.5 px-2 text-right">
              <div class="flex items-center justify-end gap-2">
                <button 
                  @click="store.switchAccount(account.id)" 
                  class="text-blue-400 text-xs hover:text-blue-300"
                >接入</button>
                <button 
                  @click="handleCheckQuota(account.id)" 
                  :disabled="checkingIds.has(account.id)"
                  class="text-blue-400 text-xs hover:text-blue-300 disabled:text-gray-500 disabled:cursor-wait"
                >{{ checkingIds.has(account.id) ? '查询中...' : '查余额' }}</button>
                <button 
                  @click="store.removeAccount(account.id)" 
                  class="text-red-400 text-xs hover:text-red-300"
                >删除</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      
      <!-- 空状态 -->
      <div v-if="accounts.length === 0" class="flex items-center justify-center h-48 text-gray-500 text-sm">
        暂无账号，请通过密钥提取或手动添加
      </div>
    </div>


  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '../stores/app'
import { invoke } from '@tauri-apps/api/core'
import type { UserPlan } from '../types'

const store = useAppStore()
const selectedIds = ref<string[]>([])
const autoRotate = ref(true)
const checkingIds = ref<Set<string>>(new Set())

const accounts = computed(() => store.accounts)
const currentAccount = computed(() => {
  if (!store.status?.current_token_id) return null
  return accounts.value.find(a => a.id === store.status?.current_token_id)
})

const isAllSelected = computed(() => {
  return accounts.value.length > 0 && selectedIds.value.length === accounts.value.length
})

function toggleAll() {
  if (isAllSelected.value) {
    selectedIds.value = []
  } else {
    selectedIds.value = accounts.value.map(a => a.id)
  }
}

function toggleSelect(id: string) {
  const idx = selectedIds.value.indexOf(id)
  if (idx >= 0) {
    selectedIds.value.splice(idx, 1)
  } else {
    selectedIds.value.push(id)
  }
}

async function toggleAutoRotate() {
  autoRotate.value = !autoRotate.value
  await invoke('set_auto_rotate', { enabled: autoRotate.value })
}

async function handleBatchDelete() {
  if (!confirm(`确定删除选中的 ${selectedIds.value.length} 个账号？`)) return
  for (const id of selectedIds.value) {
    await store.removeAccount(id)
  }
  selectedIds.value = []
}

async function handleCheckQuota(id: string) {
  try {
    checkingIds.value.add(id)
    await invoke<UserPlan>('check_quota', { id })
    await store.fetchAccounts()
  } catch (e) {
    console.error('查余额失败:', e)
  } finally {
    checkingIds.value.delete(id)
  }
}

function getStatusText(account: any) {
  if (store.status?.current_token_id === account.id) return '当前'
  if (account.status === 'expired') return '过期'
  return '可用'
}

function getStatusClass(account: any) {
  if (store.status?.current_token_id === account.id) return 'bg-green-600/20 text-green-400'
  if (account.status === 'expired') return 'bg-red-600/20 text-red-400'
  return 'bg-blue-600/20 text-blue-400'
}

function formatDate(dateStr: string | undefined) {
  if (!dateStr) return '-'
  try {
    const d = new Date(dateStr)
    if (isNaN(d.getTime())) return dateStr
    return d.toISOString().split('T')[0]
  } catch {
    return dateStr
  }
}

function getDaysRemaining(account: any): number | null {
  if (!account.expire_date) return null
  try {
    const d = new Date(account.expire_date)
    if (isNaN(d.getTime())) return null
    const now = new Date()
    const diff = Math.ceil((d.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
    return diff
  } catch {
    return null
  }
}

function getBalanceText(account: any): string {
  const days = getDaysRemaining(account)
  if (days === null) return '—'
  if (days <= 0) return '已过期'
  return `${days}天`
}

function getBalanceClass(account: any): string {
  const days = getDaysRemaining(account)
  if (days === null) return 'text-gray-500'
  if (days <= 0) return 'text-red-400'
  if (days <= 7) return 'text-yellow-400'
  return 'text-green-400'
}

// 初始化时获取 auto_rotate 状态
store.fetchStatus().then(() => {
  autoRotate.value = store.status?.auto_rotate ?? true
})
</script>
