import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import WindowsView from '../views/WindowsView.vue'
import ChatView from '../views/ChatView.vue'

const routes = [
  {
    path: '/',
    name: 'windows',
    component: WindowsView,
  },
  {
    path: '/chat/:windowName',
    name: 'chat',
    component: ChatView,
    props: true,
  },
]

// 单文件模式下用 hash 路由，file:// 协议不支持 history 模式
const history = import.meta.env.VITE_ROUTER_MODE === 'hash'
  ? createWebHashHistory()
  : createWebHistory()

const router = createRouter({
  history,
  routes,
})

export default router
