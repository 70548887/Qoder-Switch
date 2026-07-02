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
            <td class="py-2.5 px-2 text-white font-medium">{{ getDisplayName(account) }}</td>
            <td class="py-2.5 px-2 text-gray-400">{{ account.email || '-' }}</td>
            <td class="py-2.5 px-2 text-gray-400">{{ getUserType(account) }}</td>
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
import type { QuotaResult } from '../types'

const store = useAppStore()
const selectedIds = ref<string[]>([])
const autoRotate = ref(true)
const checkingIds = ref<Set<string>>(new Set())

const accounts = computed(() => store.accounts)

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
    const result = await invoke<QuotaResult>('check_quota', { id })
    console.log('[quota] 查询成功:', result)
    await store.fetchAccounts()
  } catch (e: any) {
    const msg = typeof e === 'string' ? e : (e?.message || JSON.stringify(e))
    alert('查询余额失败: ' + msg)
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

function getBalanceText(account: any): string {
  if (account.quota_used == null && account.quota_total == null) return '—'
  if (account.quota_total === 0) return '—'
  const remaining = (account.quota_total ?? 0) - (account.quota_used ?? 0)
  return `${remaining} / ${account.quota_total}`
}

function getBalanceClass(account: any): string {
  if (account.quota_used == null || account.quota_total == null) return 'text-gray-500'
  const remaining = account.quota_total - account.quota_used
  if (remaining <= 0) return 'text-red-400'  // 已耗尽
  if (remaining < account.quota_total * 0.2) return 'text-yellow-400'  // 不足20%
  return 'text-green-400'  // 充足
}

// 初始化时获取 auto_rotate 状态
store.fetchStatus().then(() => {
  autoRotate.value = store.status?.auto_rotate ?? true
})

// 从 label 中提取名称和类型（格式: "Name (Type)"）
function getDisplayName(account: any): string {
  if (account.name) return account.name
  if (account.label) {
    // 从 "Kevin Nelson (Pro Trial)" 提取 "Kevin Nelson"
    const match = account.label.match(/^(.+?)\s*\(.*\)$/)
    return match ? match[1] : account.label
  }
  return '-'
}

function getUserType(account: any): string {
  if (account.user_type) return account.user_type
  // 从 label 中提取类型作为 fallback
  if (account.label) {
    const match = account.label.match(/\((.+)\)$/)
    if (match) return match[1]
  }
  return '-'
}
</script>
