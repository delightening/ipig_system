// For more info, see https://github.com/storybookjs/eslint-plugin-storybook#configuration-flat-config-format
import storybook from "eslint-plugin-storybook";

import js from '@eslint/js'
import tsPlugin from '@typescript-eslint/eslint-plugin'
import tsParser from '@typescript-eslint/parser'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import globals from 'globals'

export default [{ ignores: ['dist', 'e2e', '*.config.ts', 'storybook-static'] }, {
  files: ['**/*.{ts,tsx}'],
  languageOptions: {
    parser: tsParser,
    ecmaVersion: 2020,
    globals: globals.browser,
  },
  plugins: {
    '@typescript-eslint': tsPlugin,
    'react-hooks': reactHooks,
    'react-refresh': reactRefresh,
  },
  rules: {
    ...js.configs.recommended.rules,
    ...tsPlugin.configs.recommended.rules,
    // Only use the classic two react-hooks rules (v4 behavior)
    'react-hooks/rules-of-hooks': 'error',
    // warn 模式：偵測缺少的 deps。注意：無法自動偵測「不穩定 deps」（如 custom hook 回傳的 plain object）
    // 規則：絕不將 custom hook 回傳的整個物件放入 deps，應解構取出 ref/setter/useCallback 值
    'react-hooks/exhaustive-deps': 'warn',
    // TypeScript handles undefined variable checks
    'no-undef': 'off',
    'react-refresh/only-export-components': [
      'warn',
      { allowConstantExport: true },
    ],
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-empty-object-type': 'warn',
    'no-redeclare': 'off',
    'no-useless-assignment': 'warn',
  },
}, {
  // shadcn/ui 元件同時 export 元件與 variants 函式，為常見模式
  files: ['src/components/ui/badge.tsx', 'src/components/ui/button.tsx'],
  rules: { 'react-refresh/only-export-components': 'off' },
}, ...storybook.configs["flat/recommended"]];
