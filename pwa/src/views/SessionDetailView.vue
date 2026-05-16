<template>
  <div class="screen">
    <div class="nav-bar">
      <button class="nav-back" @click="goBack">‹ 返回</button>
      <h1 class="nav-title">{{ sessionTitle }}</h1>
    </div>

    <div class="content">
      <!-- Loading -->
      <div v-if="loading" class="state-container">
        <div class="spinner"></div>
        <p class="state-text">加载中…</p>
      </div>

      <!-- Error -->
      <div v-else-if="error" class="state-container">
        <div class="error-icon">⚠️</div>
        <p class="state-text error-text">{{ error }}</p>
        <button class="retry-btn" @click="load">重试</button>
      </div>

      <!-- Empty -->
      <div v-else-if="messages.length === 0" class="state-container">
        <div class="empty-icon">💬</div>
        <p class="state-text">该会话暂无消息</p>
      </div>

      <!-- Message list -->
      <div v-else class="message-list">
        <div
          v-for="(msg, i) in messages"
          :key="i"
          class="message-bubble"
          :class="msg.role === 'user' ? 'bubble-user' : 'bubble-assistant'"
        >
          <div class="bubble-role">{{ msg.role === 'user' ? '你' : 'Copilot' }}</div>
          <!-- Render markdown-like content: code blocks and inline text -->
          <div class="bubble-text" v-html="renderText(msg.text)"></div>
          <div v-if="msg.timestamp" class="bubble-time">
            {{ formatTime(msg.timestamp) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getMacById } from '../macStore'

const router = useRouter()
const route = useRoute()

const props = defineProps({
  macId: { type: String, required: true },
  sessionId: { type: String, required: true },
  appName: { type: String, default: '' },
  title: { type: String, default: '' },
})

const mac = getMacById(props.macId)
const API_BASE = mac ? mac.address : (import.meta.env.VITE_API_BASE || 'http://127.0.0.1:3030')

const loading = ref(true)
const error = ref(null)
const sessionTitle = ref(props.title || '会话详情')
const messages = ref([])

function goBack() {
  router.back()
}

async function load() {
  loading.value = true
  error.value = null
  try {
    const res = await fetch(`${API_BASE}/api/session-detail`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_name: props.appName || undefined,
        session_id: props.sessionId,
      }),
    })
    const data = await res.json()
    if (!data.success) throw new Error(data.message || '加载失败')
    sessionTitle.value = data.title || props.title || '会话详情'
    messages.value = data.messages || []
  } catch (e) {
    error.value = e.message || '加载失败'
  } finally {
    loading.value = false
  }
}

function formatTime(ts) {
  return new Date(ts).toLocaleString('zh-CN', {
    month: 'numeric', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

/**
 * Very lightweight renderer: escape HTML, then turn fenced code blocks and
 * inline code into styled elements.  No external library needed.
 */
function renderText(raw) {
  // Escape HTML first to prevent XSS.
  const escaped = raw
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')

  // Fenced code blocks  ```lang\n...\n```
  const withCodeBlocks = escaped.replace(
    /```(?:[a-z]*)\n([\s\S]*?)```/g,
    (_, code) => `<pre class="code-block"><code>${code}</code></pre>`
  )

  // Inline code `...`
  const withInlineCode = withCodeBlocks.replace(
    /`([^`]+)`/g,
    (_, code) => `<code class="inline-code">${code}</code>`
  )

  // Bold **text**
  const withBold = withInlineCode.replace(
    /\*\*([^*]+)\*\*/g,
    '<strong>$1</strong>'
  )

  // Newlines → <br> (outside pre blocks)
  return withBold.replace(/\n/g, '<br>')
}

onMounted(load)
</script>

<style scoped>
.message-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-bottom: 24px;
}

.message-bubble {
  max-width: 90%;
  padding: 10px 14px;
  border-radius: 16px;
  font-size: 15px;
  line-height: 1.5;
  word-break: break-word;
}

.bubble-user {
  align-self: flex-end;
  background: #007aff;
  color: #fff;
  border-bottom-right-radius: 4px;
}

.bubble-assistant {
  align-self: flex-start;
  background: #fff;
  color: #1c1c1e;
  border-bottom-left-radius: 4px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.08);
}

.bubble-role {
  font-size: 11px;
  font-weight: 600;
  margin-bottom: 4px;
  opacity: 0.65;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.bubble-time {
  font-size: 11px;
  margin-top: 6px;
  opacity: 0.6;
  text-align: right;
}

:deep(.code-block) {
  background: rgba(0,0,0,0.08);
  border-radius: 8px;
  padding: 10px 12px;
  overflow-x: auto;
  font-size: 13px;
  font-family: 'SF Mono', 'Menlo', monospace;
  margin: 8px 0;
  white-space: pre;
}

.bubble-user :deep(.code-block) {
  background: rgba(255,255,255,0.15);
}

:deep(.inline-code) {
  background: rgba(0,0,0,0.08);
  padding: 1px 5px;
  border-radius: 4px;
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 13px;
}

.bubble-user :deep(.inline-code) {
  background: rgba(255,255,255,0.2);
}
</style>
