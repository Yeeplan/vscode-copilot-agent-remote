// vite.config.singlefile.js
// 产生单一 index.html，所有 JS/CSS 内联，无外部文件依赖。
// 用法：npm run build:single
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { viteSingleFile } from 'vite-plugin-singlefile'

// Stub out virtual:pwa-register (no service worker in single-file mode)
const stubPwaRegister = {
  name: 'stub-pwa-register',
  resolveId(id) {
    if (id === 'virtual:pwa-register') return '\0virtual:pwa-register'
  },
  load(id) {
    if (id === '\0virtual:pwa-register') return 'export function registerSW() {}'
  },
}

export default defineConfig({
  plugins: [
    vue(),
    stubPwaRegister,
    viteSingleFile(),
  ],
  // 不复制 public/ 下的静态资源，保证输出目录只有 index.html
  publicDir: false,
  define: {
    // 启用 hash 路由，使 file:// 协议可用
    'import.meta.env.VITE_ROUTER_MODE': JSON.stringify('hash'),
  },
  build: {
    outDir: 'dist-single',
    // 关闭 chunk 分割，确保所有代码打进同一入口
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
  },
})
