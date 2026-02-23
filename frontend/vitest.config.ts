import { defineConfig, mergeConfig } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      // 使用 jsdom 模擬瀏覽器環境
      environment: 'jsdom',
      // 全域匯入測試工具
      globals: true,
      // 設定檔案
      setupFiles: ['./src/__tests__/setup.ts'],
      // 排除 e2e 測試
      exclude: ['e2e/**', 'node_modules/**'],
      // 覆蓋率設定
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html'],
        include: ['src/**/*.{ts,tsx}'],
        exclude: ['src/__tests__/**', 'src/**/*.d.ts'],
      },
    },
  }),
)
