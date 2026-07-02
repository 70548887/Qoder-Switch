<template>
  <div class="bg-dark-800 border-b border-gray-700 p-4 flex gap-4">
    <div class="flex-1 flex flex-col gap-2">
      <!-- 模式切换标签 -->
      <div class="flex gap-2 mb-1">
        <button 
          @click="mode = 'key'" 
          class="text-xs px-3 py-1 rounded transition"
          :class="mode === 'key' ? 'bg-accent text-white' : 'bg-dark-700 text-gray-400 hover:text-white'"
        >密钥提取</button>
        <button 
          @click="mode = 'token'" 
          class="text-xs px-3 py-1 rounded transition"
          :class="mode === 'token' ? 'bg-accent text-white' : 'bg-dark-700 text-gray-400 hover:text-white'"
        >手动添加</button>
      </div>
      
      <!-- 密钥模式 -->
      <div v-if="mode === 'key'" class="flex flex-col gap-1">
        <div class="flex gap-2">
          <input
            v-model="secretKey"
            class="flex-1 bg-dark-900 border border-gray-600 rounded px-3 py-2 text-sm text-white focus:border-accent outline-none"
            placeholder="输入账号池密钥 (Secret Key)..."
            @keyup.enter="handleFetchAccount"
          />
          <button @click="handleFetchAccount" class="px-4 py-2 bg-accent text-white rounded text-sm font-medium hover:bg-accent-hover whitespace-nowrap">
            提取账号
          </button>
        </div>
        <span class="text-xs text-yellow-500 mt-1">注意：当前账号池使用 HTTP 连接，请在可信网络环境中使用</span>
      </div>
      
      <!-- Token 模式（保留原有功能） -->
      <textarea
        v-if="mode === 'token'"
        v-model="tokenInput"
        class="w-full bg-dark-900 border border-gray-600 rounded px-3 py-2 text-sm text-white resize-none focus:border-accent outline-none"
        rows="3"
        placeholder="输入密钥，支持空格或换行分隔批量添加..."
      ></textarea>
    </div>
    
    <div class="flex flex-col gap-2">
      <button v-if="mode === 'token'" @click="handleAdd" class="px-4 py-2 bg-accent text-white rounded text-sm font-medium hover:bg-accent-hover">+ 添加</button>
      <div class="flex items-center gap-2 text-sm text-gray-400">
        <span>PORT</span>
        <input :value="store.status?.port || 5888" class="w-16 bg-dark-900 border border-gray-600 rounded px-2 py-1 text-white text-center" readonly />
      </div>
      <button
        @click="store.status?.running ? store.stopProxy() : store.startProxy()"
        class="px-4 py-2 rounded text-sm font-medium"
        :class="store.status?.running ? 'bg-red-600 text-white hover:bg-red-700' : 'bg-green-600 text-white hover:bg-green-700'"
      >
        {{ store.status?.running ? '停止代理' : '启动代理' }}
      </button>
      <div class="flex gap-2">
        <button @click="handleInstallCert" class="px-3 py-1 bg-dark-700 text-gray-300 rounded text-xs hover:bg-dark-600">安装证书</button>
        <button @click="store.uninstallCert()" class="px-3 py-1 bg-dark-700 text-gray-300 rounded text-xs hover:bg-dark-600">卸载证书</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()
const mode = ref<'key' | 'token'>('key')
const secretKey = ref('')
const tokenInput = ref('')

async function handleFetchAccount() {
  if (!secretKey.value.trim()) return
  try {
    await store.fetchPoolAccount(secretKey.value.trim())
    alert('账号提取成功！')
    secretKey.value = ''
  } catch (e: any) {
    alert('提取失败: ' + (e?.message || e))
  }
}

async function handleAdd() {
  if (!tokenInput.value.trim()) return
  await store.addAccounts(tokenInput.value.trim())
  tokenInput.value = ''
}

async function handleInstallCert() {
  await store.installCert()
}
</script>
