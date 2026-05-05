<template>
  <div class="screen">
    <div class="nav-bar">
      <h1 class="nav-title">选择 Mac</h1>
    </div>

    <div class="content">
      <div class="state-container" v-if="macs.length === 0">
        <div class="empty-icon">🖥️</div>
        <p class="state-text">当前未配置任何 Mac</p>
        <button class="retry-btn" @click="goSettings">前往设置</button>
      </div>

      <div v-else class="list-group">
        <button
          v-for="mac in macs"
          :key="mac.id"
          class="list-item"
          @click="selectMac(mac.id)"
        >
          <div class="list-item-icon">🖥️</div>
          <div class="list-item-content">
            <span class="list-item-title">{{ mac.name }}</span>
            <span class="list-item-subtitle">{{ mac.address }}</span>
          </div>
          <div class="list-item-chevron">›</div>
        </button>
      </div>
    </div>

    <div class="toolbar">
      <button class="toolbar-btn" @click="goSettings">
        <span class="toolbar-icon">⚙️</span>
        <span class="toolbar-label">设置</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { loadMacs } from '../macStore'
import { useRouter } from 'vue-router'
import { ref } from 'vue'

const router = useRouter()
const macs = ref(loadMacs())

function selectMac(macId) {
  router.push({ name: 'windows', params: { macId } })
}

function goSettings() {
  router.push({ name: 'settings' })
}
</script>
