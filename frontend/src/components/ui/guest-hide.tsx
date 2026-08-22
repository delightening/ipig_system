import { useAuthIsGuest } from '@/stores/auth'

/**
 * 隱藏 Guest 不應看到的操作元素（新增/編輯/刪除按鈕等）
 */
export function GuestHide({ children }: { children: React.ReactNode }) {
  const isGuest = useAuthIsGuest()
  if (isGuest) return null
  return <>{children}</>
}
