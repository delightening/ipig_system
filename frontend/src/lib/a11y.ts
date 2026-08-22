import type { KeyboardEvent } from 'react'

/**
 * 鍵盤可達輔助：Enter / Space 觸發等同 onClick 的動作（用於以 Badge/div 等
 * 非原生 button 元素充當按鈕時）。回傳可直接掛在 onKeyDown 的 handler。
 *
 * 用法：`<Badge role="button" tabIndex={0} onClick={fn} onKeyDown={onActivateKey(fn)} />`
 */
export function onActivateKey(action: () => void) {
  return (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      action()
    }
  }
}
