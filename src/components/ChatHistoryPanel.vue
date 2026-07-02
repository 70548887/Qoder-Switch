<template>
  <div class="bg-dark-800 rounded-xl p-6 space-y-6">
    <!-- 顶部标题行：标题 + 全局操作 -->
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-bold text-gray-200">对话记录</h2>
      <div class="flex gap-2">
        <button @click="handleBackupAll" class="px-4 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors">备份全部</button>
        <button @click="handleExportMD" class="px-4 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-xs rounded transition-colors">导出MD</button>
        <button @click="refresh" class="px-4 py-1.5 bg-dark-700 hover:bg-dark-600 text-gray-300 text-xs rounded transition-colors">刷新</button>
      </div>
    </div>

    <!-- Tab 切换条 -->
    <div class="flex border-b border-gray-700 mb-4">
      <button
        @click="activeSubTab = 'chats'"
        :class="activeSubTab === 'chats' ? 'border-b-2 border-accent text-white' : 'text-gray-400 hover:text-gray-200'"
        class="px-4 py-2 text-sm font-medium transition-colors"
      >对话列表</button>
      <button
        @click="switchToBackups"
        :class="activeSubTab === 'backups' ? 'border-b-2 border-accent text-white' : 'text-gray-400 hover:text-gray-200'"
        class="px-4 py-2 text-sm font-medium transition-colors"
      >备份管理</button>
    </div>

    <!-- Tab 内容：对话列表 -->
    <div v-if="activeSubTab === 'chats'">
      <div class="space-y-3">
        <!-- 工作区选择 + 搜索 + 操作（一行） -->
        <div class="flex gap-3 items-center">
          <select v-model="selectedWorkspace" @change="onWorkspaceChange" class="bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white text-sm w-56">
            <option value="" disabled>选择工作区</option>
            <option v-for="ws in store.chatWorkspaces" :key="ws.id" :value="ws.id">{{ ws.name || ws.id }}</option>
          </select>
          <input v-model="searchQuery" @input="handleSearch" type="text" placeholder="搜索对话标题..." class="flex-1 bg-dark-900 border border-gray-600 rounded px-3 py-2 text-white text-sm" />
          <button @click="handleBackup" :disabled="!selectedWorkspace" class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white text-xs rounded transition-colors">备份</button>
          <button @click="handleRestore" class="px-3 py-1.5 bg-green-600 hover:bg-green-700 text-white text-xs rounded transition-colors">恢复</button>
        </div>
      </div>

      <!-- 批量操作栏 -->
      <div v-if="selectedIds.length > 0" class="flex items-center gap-3 px-2 mt-3">
        <span class="text-sm text-gray-400">已选 {{ selectedIds.length }} 条</span>
        <button @click="handleBatchDelete" class="px-3 py-1 bg-red-600 hover:bg-red-700 text-white text-xs rounded transition-colors">批量删除</button>
      </div>

      <!-- 对话列表 -->
      <div class="border border-gray-700 rounded-lg overflow-hidden mt-4">
        <div class="grid grid-cols-[40px_1fr_140px_70px_60px] bg-dark-900 px-4 py-2 text-xs text-gray-500 font-medium border-b border-gray-700">
          <div><input type="checkbox" :checked="isAllSelected" @change="toggleAll" class="accent-accent" /></div>
          <div>标题</div>
          <div>时间</div>
          <div>上下文</div>
          <div>操作</div>
        </div>

        <div v-if="!selectedWorkspace" class="text-center text-gray-500 py-10 text-sm">请选择一个工作区</div>
        <div v-else-if="filteredChats.length === 0" class="text-center text-gray-500 py-10 text-sm">暂无对话记录</div>

        <template v-for="chat in filteredChats" :key="chat.id">
          <div class="grid grid-cols-[40px_1fr_140px_70px_60px] px-4 py-3 border-b border-gray-800 hover:bg-dark-700/50 items-center cursor-pointer" @dblclick="toggleExpand(chat.id)">
            <div @click.stop>
              <input type="checkbox" :checked="selectedIds.includes(chat.id)" @change="toggleSelect(chat.id)" class="accent-accent" />
            </div>
            <div class="text-white text-sm truncate pr-4">{{ chat.title || '(无标题)' }}</div>
            <div class="text-gray-400 text-xs">{{ formatTime(chat.timestamp) }}</div>
            <div class="text-gray-400 text-xs">{{ chat.context?.length || 0 }}</div>
            <div @click.stop>
              <button @click="handleDeleteChat(chat.id)" class="text-red-400 hover:text-red-300 text-xs">删除</button>
            </div>
          </div>
          <!-- 展开详情 -->
          <div v-if="expandedChatId === chat.id" class="bg-dark-900 px-6 py-3 border-b border-gray-800">
            <div class="text-xs text-gray-400 space-y-2">
              <div>会话ID: <span class="text-gray-300 font-mono text-xs">{{ (chat as any).sessionId || chat.session_id || '-' }}</span></div>
              <div v-if="chat.context && chat.context.length > 0">
                <span class="text-gray-400">引用上下文:</span>
                <ul class="ml-4 mt-1 space-y-0.5">
                  <li v-for="(ctx, idx) in chat.context" :key="idx" class="text-gray-300">
                    {{ ctx.name }} <span class="text-gray-500">({{ ctx.type }})</span>
                  </li>
                </ul>
              </div>
              <div v-else class="text-gray-500">无引用上下文</div>
            </div>
          </div>
        </template>
      </div>

      <div class="text-xs text-gray-500 text-right mt-2">共 {{ filteredChats.length }} 条对话</div>
    </div>

    <!-- Tab 内容：备份管理 -->
    <div v-else-if="activeSubTab === 'backups'">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-gray-300">备份文件列表</h3>
        <span class="text-xs text-gray-500">{{ store.backupList.length }} 个备份</span>
      </div>

      <div v-if="store.backupList.length === 0" class="text-center text-gray-500 py-10 text-sm">暂无备份文件</div>
      <div v-else class="space-y-2 max-h-96 overflow-y-auto">
        <div v-for="backup in store.backupList" :key="backup.file_path" class="flex items-center justify-between p-3 bg-dark-900 rounded hover:bg-dark-700 transition">
          <div class="flex-1 min-w-0">
            <div class="text-sm text-white truncate">{{ backup.file_name }}</div>
            <div class="text-xs text-gray-400 flex gap-3 mt-0.5">
              <span>{{ formatBackupTime(backup.backup_time) }}</span>
              <span>{{ backup.workspace_path || backup.workspace_id }}</span>
              <span>{{ formatFileSize(backup.file_size) }}</span>
            </div>
          </div>
          <div class="flex gap-2 ml-3">
            <button @click="handleRestoreFromList(backup)" class="px-2 py-1 bg-green-600 hover:bg-green-700 text-white text-xs rounded transition-colors">恢复</button>
            <button @click="handleDeleteBackup(backup.file_path)" class="px-2 py-1 bg-red-600 hover:bg-red-700 text-white text-xs rounded transition-colors">删除</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 隐藏的文件选择器 -->
    <input ref="restoreFileInput" type="file" accept=".json" style="display: none" @change="handleRestoreFileSelected" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAppStore } from '../stores/app'
import type { BackupFileInfo } from '../types'

const store = useAppStore()
const activeSubTab = ref<'chats' | 'backups'>('chats')
const selectedWorkspace = ref('')
const searchQuery = ref('')
const selectedIds = ref<string[]>([])
const expandedChatId = ref<string | null>(null)
const restoreFileInput = ref<HTMLInputElement>()

const filteredChats = computed(() => {
  if (!searchQuery.value) return store.chatList
  const q = searchQuery.value.toLowerCase()
  return store.chatList.filter(c => c.title?.toLowerCase().includes(q))
})

const isAllSelected = computed(() => {
  return filteredChats.value.length > 0 && selectedIds.value.length === filteredChats.value.length
})

function switchToBackups() {
  activeSubTab.value = 'backups'
  store.fetchBackupList()
}

function toggleAll() {
  if (isAllSelected.value) {
    selectedIds.value = []
  } else {
    selectedIds.value = filteredChats.value.map(c => c.id)
  }
}

function toggleSelect(id: string) {
  const idx = selectedIds.value.indexOf(id)
  if (idx >= 0) selectedIds.value.splice(idx, 1)
  else selectedIds.value.push(id)
}

function toggleExpand(id: string) {
  expandedChatId.value = expandedChatId.value === id ? null : id
}

function formatTime(ms: number): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

function formatBackupTime(isoStr: string): string {
  try { return new Date(isoStr).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }) }
  catch { return isoStr }
}

function formatFileSize(bytes: number): string {
  if (!bytes) return '-'
  if (bytes < 1024) return bytes + 'B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + 'KB'
  return (bytes / (1024 * 1024)).toFixed(1) + 'MB'
}

async function onWorkspaceChange() {
  selectedIds.value = []
  searchQuery.value = ''
  expandedChatId.value = null
  if (selectedWorkspace.value) {
    await store.fetchWorkspaceChats(selectedWorkspace.value)
  }
}

async function handleSearch() {
  // 前端即时过滤，不调用后端
}

async function handleBackup() {
  if (!selectedWorkspace.value) return
  try {
    const path = await store.backupWorkspace(selectedWorkspace.value)
    alert('备份成功！\n保存位置: ' + path)
    await store.fetchBackupList()
  } catch (e: any) {
    alert('备份失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

function handleRestore() {
  restoreFileInput.value?.click()
}

async function handleRestoreFileSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const filePath = (file as any).path
  if (!filePath) {
    alert('无法获取文件路径，请确认使用桌面客户端')
    input.value = ''
    return
  }
  const target = selectedWorkspace.value
    ? (confirm('恢复到当前选中的工作区？\n\n确定 = 恢复到当前工作区\n取消 = 恢复到备份原始工作区') ? selectedWorkspace.value : undefined)
    : undefined
  try {
    await store.restoreBackup(filePath, target)
    alert('恢复成功！')
  } catch (e: any) {
    alert('恢复失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
  input.value = ''
}

async function handleRestoreFromList(backup: BackupFileInfo) {
  const target = selectedWorkspace.value
    ? (confirm(`恢复备份 "${backup.file_name}" 到当前选中的工作区？\n\n确定 = 当前工作区\n取消 = 备份原始工作区 (${backup.workspace_path || backup.workspace_id})`) ? selectedWorkspace.value : undefined)
    : undefined
  try {
    await store.restoreBackup(backup.file_path, target)
    alert('恢复成功！')
  } catch (e: any) {
    alert('恢复失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function handleDeleteBackup(filePath: string) {
  if (!confirm('确定要删除此备份文件吗？')) return
  try {
    await store.deleteBackupFile(filePath)
  } catch (e: any) {
    alert('删除失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function handleDeleteChat(chatId: string) {
  if (!confirm('确定要删除这条对话吗？')) return
  try {
    await store.deleteChats(selectedWorkspace.value, [chatId])
    selectedIds.value = selectedIds.value.filter(id => id !== chatId)
  } catch (e: any) {
    alert('删除失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function handleBatchDelete() {
  if (!confirm(`确定要删除选中的 ${selectedIds.value.length} 条对话吗？`)) return
  try {
    await store.deleteChats(selectedWorkspace.value, [...selectedIds.value])
    selectedIds.value = []
  } catch (e: any) {
    alert('批量删除失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function handleBackupAll() {
  try {
    const path = await store.backupAllWorkspaces()
    alert('全部备份成功！\n保存位置: ' + path)
    await store.fetchBackupList()
  } catch (e: any) {
    alert('批量备份失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function handleExportMD() {
  try {
    const path = await store.exportMarkdown()
    alert('导出成功！\n保存位置: ' + path)
  } catch (e: any) {
    alert('导出失败: ' + (typeof e === 'string' ? e : e?.message || JSON.stringify(e)))
  }
}

async function refresh() {
  await store.fetchChatWorkspaces()
  await store.fetchBackupList()
  if (selectedWorkspace.value) {
    await store.fetchWorkspaceChats(selectedWorkspace.value)
  }
}

onMounted(async () => {
  await store.fetchChatWorkspaces()
  await store.fetchBackupList()
  if (store.chatWorkspaces.length > 0 && !selectedWorkspace.value) {
    selectedWorkspace.value = store.chatWorkspaces[0].id
    await store.fetchWorkspaceChats(selectedWorkspace.value)
  }
})
</script>
