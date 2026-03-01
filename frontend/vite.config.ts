import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    chunkSizeWarningLimit: 600, // vendor chunks 可能略超 500KB
    rollupOptions: {
      output: {
        manualChunks: {
          // React 核心
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          // 資料管理
          'vendor-data': ['@tanstack/react-query', '@tanstack/react-table', 'axios', 'zustand'],
          // Radix UI 元件庫
          'vendor-radix': [
            '@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-select', '@radix-ui/react-tabs',
            '@radix-ui/react-popover', '@radix-ui/react-tooltip',
            '@radix-ui/react-alert-dialog', '@radix-ui/react-checkbox',
            '@radix-ui/react-switch', '@radix-ui/react-toast',
            '@radix-ui/react-avatar', '@radix-ui/react-label',
            '@radix-ui/react-separator', '@radix-ui/react-slot',
            '@radix-ui/react-icons',
          ],
          // 圖表
          'vendor-charts': ['recharts'],
          // 行事曆
          'vendor-calendar': [
            '@fullcalendar/core', '@fullcalendar/daygrid',
            '@fullcalendar/interaction', '@fullcalendar/react',
            '@fullcalendar/timegrid',
          ],
          // 國際化
          'vendor-i18n': ['i18next', 'react-i18next', 'i18next-browser-languagedetector'],
        },
      },
    },
  },
  server: {
    port: 5173,
    host: '0.0.0.0', // Ensure it binds to all interfaces for Docker
    proxy: {
      // SSE 長連線需優先匹配，避免被 /api 的預設 timeout 影響
      '/api/admin/audit/alerts/sse': {
        target: process.env.VITE_API_URL || 'http://localhost:8000',
        changeOrigin: true,
        timeout: 0, // 無限制，SSE 為長連線
        proxyTimeout: 0,
      },
      '/api': {
        target: process.env.VITE_API_URL || 'http://localhost:8000',
        changeOrigin: true,
      },
    },
  },
})
