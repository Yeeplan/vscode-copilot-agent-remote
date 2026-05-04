<template>
  <div class="screen">
    <div class="nav-bar">
      <h1 class="nav-title">VS Code 窗口</h1>
    </div>

    <div v-if="offline" class="offline-banner">
      📶 无网络 · 显示上次缓存
    </div>

    <div class="content">
      <div v-if="loading" class="state-container">
        <div class="spinner"></div>
        <p class="state-text">加载中...</p>
      </div>

      <div v-else-if="error && windows.length === 0" class="state-container">
        <div class="error-icon">⚠️</div>
        <p class="state-text error-text">{{ error }}</p>
        <button class="retry-btn" @click="loadWindows">重试</button>
      </div>

      <div v-else-if="windows.length === 0" class="state-container">
        <div class="empty-icon">🖥️</div>
        <p class="state-text">未找到 VS Code 窗口</p>
        <button class="retry-btn" @click="loadWindows">刷新</button>
      </div>

      <div v-else class="list-group">
        <button
          v-for="win in windows"
          :key="win"
          class="list-item"
          @click="openChat(win)"
        >
          <div class="list-item-icon">
            <span class="vscode-icon">⌨️</span>
          </div>
          <div class="list-item-content">
            <span class="list-item-title">{{ win }}</span>
          </div>
          <div class="list-item-chevron">›</div>
        </button>
      </div>

      <p v-if="error && windows.length > 0" class="cached-notice">
        ⚠️ 刷新失败，显示缓存列表
      </p>
    </div>

    <div class="toolbar">
      <button class="toolbar-btn" @click="loadWindows" :disabled="loading">
        <span class="toolbar-icon">↻</span>
        <span class="toolbar-label">刷新</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const CACHE_KEY = 'vscode-remote:windows'
const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:3030'

const router = useRouter()
const windows = ref([])
const loading = ref(false)
const error = ref('')
const offline = ref(false)

function loadCache() {
  try {
    const cached = localStorage.getItem(CACHE_KEY)
    if (cached) return JSON.parse(cached)
  } catch {}
  return []
}

function saveCache(list) {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify(list))
  } catch {}
}

async function loadWindows() {
  loading.value = true
  error.value = ''
  offline.value = false
  try {
    const res = await fetch(`${API_BASE}/api/windows`)
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data = await res.json()
    const list = data.windows ?? data
    windows.value = list
    saveCache(list)
  } catch (e) {
    const cached = loadCache()
    if (cached.length > 0) {
      windows.value = cached
      offline.value = true
    }
    error.value = '无法连接到服务器，请确认 vscode-remote-control 已启动'
  } finally {
    loading.value = false
  }
}

function openChat(windowName) {
  router.push({ name: 'chat', params: { windowName } })
}

onMounted(() => {
  // 立即显示缓存，同时尝试刷新
  const cached = loadCache()
  if (cached.length > 0) windows.value = cached
  loadWindows()
})
</script>
