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
      </div>

      <div v-if="result" class="result-banner" :class="result.success ? 'success' : 'fail'">
        {{ result.message }}
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
})

const mac = getMacById(props.macId)
const API_BASE = mac ? mac.address : (import.meta.env.VITE_API_BASE || 'http://127.0.0.1:3030')

const chatContent = ref('')
const openChat = ref(true)
const sending = ref(false)
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
    setTimeout(() => { result.value = null }, 3000)
  }
}
</script>
