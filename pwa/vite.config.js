import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['favicon.svg', 'apple-touch-icon.png'],
      workbox: {
        // SPA 离线导航回退：任何导航请求都返回缓存的 index.html
        navigateFallback: '/index.html',
        // 预缓存所有构建产物
        globPatterns: ['**/*.{js,css,html,ico,png,svg,webmanifest}'],
      },
      manifest: {
        name: 'VSCode Remote',
        short_name: 'VSCode Remote',
        description: 'Remotely control VS Code Copilot Chat',
        theme_color: '#1e1e1e',
        background_color: '#1e1e1e',
        display: 'standalone',
        orientation: 'portrait',
        icons: [
          {
            src: 'pwa-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
          },
          {
            src: 'pwa-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'any maskable',
          },
        ],
      },
    }),
  ],
})
