import { useEffect, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'

/**
 * 訪客模式 tour launcher：guest 登入即自動開啟（每次都開，不用 localStorage）；
 * 也支援 ?tour=1 query 手動觸發。拆出獨立檔案是為了 Vite Fast Refresh —
 * 同檔同時 export hook 與 component 會讓 HMR 失效。
 */
export function useDemoTourLauncher() {
  const { isGuest } = useAuthStore()
  const [searchParams, setSearchParams] = useSearchParams()
  const [open, setOpen] = useState(false)
  const guest = isGuest()

  useEffect(() => {
    if (guest) setOpen(true)
  }, [guest])

  useEffect(() => {
    if (!guest) return
    if (searchParams.get('tour') === '1') {
      setOpen(true)
      const next = new URLSearchParams(searchParams)
      next.delete('tour')
      setSearchParams(next, { replace: true })
    }
  }, [searchParams, setSearchParams, guest])

  return { open, setOpen, canShow: guest }
}
