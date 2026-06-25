<template>
  <div class="bg-dark-800 rounded-xl p-6">
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-xl font-bold text-gray-200">额度查询</h2>
      <button
        @click="store.checkAllQuotas()"
        class="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors"
      >刷新额度</button>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div
        v-for="item in store.quotas"
        :key="item.account_id"
        class="bg-dark-700 rounded-lg p-4 border"
        :class="cardBorder(item)"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium text-gray-200">{{ item.label }}</span>
          <span
            class="px-2 py-0.5 rounded text-xs font-medium"
            :class="statusBadge(item)"
          >{{ statusText(item) }}</span>
        </div>

        <template v-if="item.plan">
          <div class="text-sm text-gray-400 space-y-1">
            <div>Plan: <span class="text-gray-300">{{ item.plan.plan_name }}</span></div>
            <div>类型: <span class="text-gray-300">{{ item.plan.user_type }}</span></div>
            <div>
              剩余:
              <span :class="daysColor(item.plan)">
                {{ item.plan.days_remaining }} 天
              </span>
            </div>
          </div>
        </template>
        <template v-else-if="item.error">
          <div class="text-sm text-red-400">{{ item.error }}</div>
        </template>
      </div>
    </div>

    <div v-if="!store.quotas.length" class="text-center text-gray-500 py-8">
      点击"刷新额度"查询所有账号状态
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import type { QuotaResult, UserPlan } from '../types'

const store = useAppStore()

function cardBorder(item: QuotaResult) {
  if (item.plan?.is_expired) return 'border-red-500/50'
  if (item.plan && item.plan.days_remaining <= 3) return 'border-orange-500/50'
  return 'border-gray-700'
}

function statusBadge(item: QuotaResult) {
  if (item.error) return 'bg-red-500/20 text-red-400'
  if (item.plan?.is_expired) return 'bg-red-500/20 text-red-400'
  if (item.plan && item.plan.days_remaining <= 3) return 'bg-orange-500/20 text-orange-400'
  return 'bg-green-500/20 text-green-400'
}

function statusText(item: QuotaResult) {
  if (item.error) return '错误'
  if (item.plan?.is_expired) return '已过期'
  if (item.plan && item.plan.days_remaining <= 3) return '即将过期'
  return '正常'
}

function daysColor(plan: UserPlan) {
  if (plan.is_expired) return 'text-red-400 font-medium'
  if (plan.days_remaining <= 3) return 'text-orange-400 font-medium'
  return 'text-green-400'
}

onMounted(() => {
  store.checkAllQuotas()
})
</script>
