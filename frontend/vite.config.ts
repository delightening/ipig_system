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
          // 辦公文件 (PDF)
          'vendor-office': ['jspdf', 'html2canvas'],
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
      '/api': {
        // In Docker, 'api' is the service name. Locally, it should be 'localhost'.
        // We use an environment variable VITE_API_URL or fallback.
        target: process.env.VITE_API_URL || 'http://localhost:8000',
        changeOrigin: true,
      },
    },
  },
})
