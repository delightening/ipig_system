import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
// E2E coverage：CI 跑 Playwright 收集 V8 coverage 時，需 source map 把 minified
// bundle 對回 .ts 原始檔。由 docker-compose.test.yml 傳入 build arg 觸發。
const e2eCoverage = process.env.VITE_E2E_COVERAGE === '1'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    chunkSizeWarningLimit: 600, // vendor chunks 可能略超 500KB
    // E2E coverage 用 inline-URL sourcemap（true）而非 'hidden'：
    // monocart-reporter 透過 V8 coverage entry 的 //# sourceMappingURL=... 註解
    // 載入 .map 還原回 src/*.ts；hidden 模式不寫註解 → monocart 找不到 map →
    // lcov 只看得到 minified bundle 外殼，per-file LOC 都是 2-10 行（假覆蓋率）。
    // E2E test build 才會走這條（prod build VITE_E2E_COVERAGE=0），不影響 prod。
    sourcemap: e2eCoverage ? true : false,
    rollupOptions: {
      output: {
        // Rollup 5 (Vite 8) 移除 manualChunks 的 object 形式，僅支援 function 形式。
        // recharts / @fullcalendar 不在此列：僅 lazy-loaded 頁面使用，Vite 自動 code-split。
        manualChunks(id) {
          if (!id.includes('node_modules')) return;
          if (/[\\/]node_modules[\\/](react|react-dom|react-router-dom)[\\/]/.test(id)) return 'vendor-react';
          if (/[\\/]node_modules[\\/](@tanstack[\\/](react-query|react-table)|axios|zustand)[\\/]/.test(id)) return 'vendor-data';
          if (/[\\/]node_modules[\\/]@radix-ui[\\/]/.test(id)) return 'vendor-radix';
          if (/[\\/]node_modules[\\/](i18next|react-i18next|i18next-browser-languagedetector)[\\/]/.test(id)) return 'vendor-i18n';
        },
      },
    },
  },
  server: {
    port: 8080,
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
