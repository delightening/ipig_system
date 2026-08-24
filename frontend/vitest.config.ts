import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';
const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

// vite.config.ts 改用 callback form (mode-aware) 後，mergeConfig 不接受 callback；
// 需先 resolve 為 plain config 物件再合併。
const resolvedViteConfig = typeof viteConfig === 'function'
  ? viteConfig({ command: 'serve', mode: 'test' })
  : viteConfig;

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default mergeConfig(resolvedViteConfig, defineConfig({
  test: {
    // 覆蓋率設定
    coverage: {
      // R82-4 code review (Gemini)：v8 provider 在 browser mode（storybook project 走
      // Playwright/CDP）已知不穩定（vitest-dev/vitest#6380 branch coverage 錯誤/CDP 相關錯誤）；
      // istanbul 用原始碼插樁，跨 jsdom / browser mode 一致可靠。
      provider: 'istanbul',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/__tests__/**', 'src/**/*.d.ts'],
      // R82-4 coverage ratchet：autoUpdate=true 時，現值高於門檻會就地改寫下方數字（只升不降）；
      // main push 由 CI 的 "Coverage ratchet 門檻變更偵測" step commit 回 repo。
      // 下列數字為本機實測（unit project，jsdom）現值；storybook project（browser mode）
      // 本機因缺 chrome-headless-shell 二進位無法跑，CI 有裝 Playwright 會一併納入量測，
      // 首次 main push 後 autoUpdate 會再收斂到含 storybook 覆蓋率貢獻的真實下限。
      thresholds: {
        autoUpdate: true,
        lines: 9.52,
        functions: 6.17,
        branches: 5.82,
        statements: 9.24
      }
    },
    projects: [{
      extends: true,
      name: 'unit',
      test: {
        // 使用 jsdom 模擬瀏覽器環境
        environment: 'jsdom',
        // 全域匯入測試工具
        globals: true,
        // 設定檔案
        setupFiles: ['./src/__tests__/setup.ts'],
        // 明確包含單元測試，排除 e2e / story
        include: ['src/**/*.test.{ts,tsx}'],
        exclude: ['e2e/**', 'node_modules/**', '**/*.stories.{ts,tsx}']
      }
    }, {
      extends: true,
      plugins: [
      // The plugin will run tests for the stories defined in your Storybook config
      // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
      storybookTest({
        configDir: path.join(dirname, '.storybook')
      })],
      test: {
        name: 'storybook',
        browser: {
          enabled: true,
          headless: true,
          provider: playwright({}),
          instances: [{
            browser: 'chromium'
          }]
        },
        setupFiles: ['.storybook/vitest.setup.ts']
      }
    }]
  }
}));