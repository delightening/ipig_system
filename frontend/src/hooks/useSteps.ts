import { useState, useCallback } from 'react'

/**
 * 多步驟 wizard 流程狀態管理。
 * 適用於 CreateProductPage、表單 wizard 等場景。
 *
 * @param totalSteps 總步驟數（0-based index）
 * @param initialStep 初始步驟，預設 0
 * @returns step、setStep、next、prev、goTo、isFirst、isLast
 */
export function useSteps(totalSteps: number, initialStep = 0) {
  const [step, setStep] = useState(initialStep)

  const next = useCallback(() => {
    setStep((s) => Math.min(s + 1, totalSteps - 1))
  }, [totalSteps])

  const prev = useCallback(() => {
    setStep((s) => Math.max(s - 1, 0))
  }, [])

  const goTo = useCallback((index: number) => {
    setStep(Math.max(0, Math.min(index, totalSteps - 1)))
  }, [totalSteps])

  const isFirst = step === 0
  const isLast = step >= totalSteps - 1

  return {
    step,
    setStep,
    next,
    prev,
    goTo,
    isFirst,
    isLast,
  }
}
