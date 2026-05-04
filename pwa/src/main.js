import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import { registerSW } from 'virtual:pwa-register'

// 自动更新 service worker；新版本就绪后静默刷新
registerSW({ immediate: true })

createApp(App).use(router).mount('#app')
