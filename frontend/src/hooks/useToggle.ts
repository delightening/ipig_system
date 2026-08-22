import { useState, useCallback } from 'react'

/**
 * 通用布林切換 hook，適用於密碼可見、進階篩選展開等場景。
 * @param initial 初始值，預設 false
 * @returns [value, toggle, setValue] - 當前值、切換函式、直接設定函式
 */
export function useToggle(initial = false): [boolean, () => void, (value: boolean) => void] {
  const [value, setValue] = useState(initial)
  const toggle = useCallback(() => setValue((v) => !v), [])
  return [value, toggle, setValue]
}
