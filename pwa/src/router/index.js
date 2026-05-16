import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import MacListView from '../views/MacListView.vue'
import WindowsView from '../views/WindowsView.vue'
import ChatView from '../views/ChatView.vue'
import SettingsView from '../views/SettingsView.vue'
import SessionDetailView from '../views/SessionDetailView.vue'

const routes = [
  {
    path: '/',
    name: 'macs',
    component: MacListView,
  },
  {
    path: '/mac/:macId',
    name: 'windows',
    component: WindowsView,
    props: true,
  },
  {
    path: '/mac/:macId/chat/:windowName',
    name: 'chat',
    component: ChatView,
    props: route => ({
      macId: route.params.macId,
      windowName: route.params.windowName,
      appName: typeof route.query.appName === 'string' ? route.query.appName : '',
    }),
  },
  {
    path: '/mac/:macId/session/:sessionId',
    name: 'sessionDetail',
    component: SessionDetailView,
    props: route => ({
      macId: route.params.macId,
      sessionId: route.params.sessionId,
      appName: typeof route.query.appName === 'string' ? route.query.appName : '',
      title: typeof route.query.title === 'string' ? route.query.title : '',
    }),
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsView,
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
