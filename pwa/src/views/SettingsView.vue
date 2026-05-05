<template>
  <div class="screen">
    <div class="nav-bar">
      <button class="nav-back" @click="$router.back()">‹ 返回</button>
      <h1 class="nav-title">Mac 设置</h1>
    </div>

    <div class="content">
      <div v-if="macs.length === 0" class="state-container">
        <div class="empty-icon">🖥️</div>
        <p class="state-text">还没有配置任何 Mac，可新增一个设备。</p>
      </div>

      <div class="settings-list">
        <div v-for="(mac, index) in macs" :key="mac.id" class="settings-card">
          <div class="settings-row">
            <label class="settings-label">名称</label>
            <input
              type="text"
              v-model="mac.name"
              class="settings-input"
              placeholder="例如：办公 Mac"
            />
          </div>
          <div class="settings-row">
            <label class="settings-label">地址</label>
            <input
              type="text"
              v-model="mac.address"
              class="settings-input"
              placeholder="例如：http://127.0.0.1:3030"
            />
          </div>
          <button class="delete-btn" @click="removeMac(index)">删除</button>
        </div>
      </div>
    </div>

    <div class="toolbar settings-toolbar">
      <button class="toolbar-btn" @click="addMac">
        <span class="toolbar-icon">＋</span>
        <span class="toolbar-label">新增 Mac</span>
      </button>
      <button class="toolbar-btn primary" @click="saveMacsConfig">
        <span class="toolbar-icon">💾</span>
        <span class="toolbar-label">保存</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { loadMacs, saveMacs, normalizeAddress } from '../macStore'

const router = useRouter()
const macs = ref(loadMacs())

function addMac() {
  macs.value.push({ id: String(Date.now()) + Math.random().toString(16).slice(2), name: '', address: '' })
}

function removeMac(index) {
  macs.value.splice(index, 1)
}

function saveMacsConfig() {
  const cleaned = macs.value.map((item) => ({
    ...item,
    name: item.name.trim() || '未命名 Mac',
    address: normalizeAddress(item.address),
  }))
  saveMacs(cleaned)
  router.push({ name: 'macs' })
}
</script>
