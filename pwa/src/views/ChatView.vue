<template>
  <div class="screen">
    <div class="nav-bar">
      <button class="nav-back" @click="goBack">‹ 返回</button>
      <h1 class="nav-title">{{ shortName }}</h1>
    </div>

    <div class="content">
      <div class="chat-form">
        <label class="field-label">发送到 Copilot Chat</label>
        <textarea
          v-model="chatContent"
          class="chat-textarea"
          placeholder="输入聊天内容..."
          rows="6"
          :disabled="sending"
        ></textarea>

        <div class="options-group">
          <label class="toggle-row">
            <span class="toggle-label">打开 Chat 面板</span>
            <input type="checkbox" v-model="openChat" class="toggle-input" />
            <span class="toggle-track" :class="{ on: openChat }"></span>
          </label>
        </div>

        <button
          class="send-btn"
          :disabled="!chatContent.trim() || sending"
          @click="sendChat"
        >
          <span v-if="sending" class="btn-spinner"></span>
          <span v-else>发送</span>
        </button>

        <button class="close-btn" :disabled="sending || closing" @click="closeWindow">
          <span v-if="closing" class="btn-spinner"></span>
          <span v-else>关闭窗口</span>
        </button>

        <button class="close-btn secondary" :disabled="sending || closing || loadingSessions" @click="listSessions">
          <span v-if="loadingSessions" class="btn-spinner"></span>
          <span v-else>列出会话</span>
        </button>
      </div>

      <div v-if="result" class="result-banner" :class="result.success ? 'success' : 'fail'">
        {{ result.message }}
      </div>

      <div v-if="sessions.length" class="session-panel">
        <div class="section-title">会话列表</div>
        <div class="session-list">
          <button
            v-for="session in sessions"
            :key="session.id"
            class="session-item"
            @click="openSession(session)"
          >
            <span class="session-item-title">{{ session.title }}</span>
            <span class="session-item-chevron">›</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { getMacById } from '../macStore'

const router = useRouter()
const props = defineProps({
  macId: { type: String, required: true },
  windowName: { type: String, required: true },
  appName: { type: String, default: '' },
})

const mac = getMacById(props.macId)
const API_BASE = mac ? mac.address : (import.meta.env.VITE_API_BASE || 'http://127.0.0.1:3030')

const chatContent = ref('')
const openChat = ref(true)
const sending = ref(false)
const closing = ref(false)
const loadingSessions = ref(false)
const sessions = ref([])

function openSession(session) {
  router.push({
    name: 'sessionDetail',
    params: { macId: props.macId, sessionId: session.id },
    query: { appName: props.appName || undefined, title: session.title },
  })
}
const result = ref(null)

const shortName = computed(() => {
  const parts = props.windowName.split(' — ')
  return parts[0] || props.windowName
})

function goBack() {
  router.push({ name: 'windows', params: { macId: props.macId } })
}

async function sendChat() {
  if (!chatContent.value.trim()) return
  sending.value = true
  result.value = null
  try {
    const res = await fetch(`${API_BASE}/api/focus`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_name: props.appName || undefined,
        window_name: props.windowName,
        open_chat: openChat.value,
        chat_content: chatContent.value,
      }),
    })
    const data = await res.json()
    result.value = { success: data.success, message: data.success ? '发送成功 ✓' : (data.message || '发送失败') }
    if (data.success) chatContent.value = ''
  } catch {
    result.value = { success: false, message: '网络错误，请重试' }
  } finally {
    sending.value = false
    if (result.value?.success) {
      setTimeout(() => { result.value = null }, 3000)
    }
  }
}

async function closeWindow() {
  const confirmed = window.confirm(`确定要关闭这个窗口吗？\n\n${props.windowName}`)
  if (!confirmed) return

  closing.value = true
  result.value = null
  try {
    const res = await fetch(`${API_BASE}/api/close-window`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_name: props.appName || undefined,
        window_name: props.windowName,
      }),
    })
    const data = await res.json()
    result.value = { success: data.success, message: data.success ? '窗口已关闭' : (data.message || '关闭失败') }
  } catch {
    result.value = { success: false, message: '网络错误，请重试' }
  } finally {
    closing.value = false
    if (result.value?.success) {
      setTimeout(() => {
        result.value = null
        goBack()
      }, 2000)
    }
  }
}

async function listSessions() {
  loadingSessions.value = true
  result.value = null

  try {
    const res = await fetch(`${API_BASE}/api/list-sessions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_name: props.appName || undefined,
        window_name: props.windowName,
      }),
    })

    const data = await res.json()
    if (!res.ok || !data.success) {
      throw new Error(data.message || '列出会话失败')
    }

    sessions.value = Array.isArray(data.sessions) ? data.sessions : []  // [{id, title}]
    result.value = {
      success: true,
      message: sessions.value.length > 0 ? `共找到 ${sessions.value.length} 个会话` : '未找到会话',
    }
  } catch (error) {
    result.value = {
      success: false,
      message: `列出会话失败: ${error.message || error}`,
    }
    sessions.value = []
  } finally {
    loadingSessions.value = false
  }
}
</script>
