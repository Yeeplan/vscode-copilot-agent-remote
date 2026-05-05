import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import { registerSW } from 'virtual:pwa-register'

const updateSW = registerSW({
	immediate: true,
	onNeedRefresh() {
		updateSW(true)
	},
	onOfflineReady() {},
	onRegisteredSW(swUrl, registration) {
		if (!registration) {
			return
		}

		window.setInterval(() => {
			registration.update()
		}, 60 * 1000)
	},
})

createApp(App).use(router).mount('#app')
