import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'

const STORAGE_KEY = 'cookie-consent'

export function CookieConsent() {
  const [visible, setVisible] = useState(
    () => localStorage.getItem(STORAGE_KEY) !== 'accepted',
  )

  if (!visible) return null

  const handleAccept = () => {
    localStorage.setItem(STORAGE_KEY, 'accepted')
    setVisible(false)
  }

  return (
    <div className="fixed bottom-0 inset-x-0 z-50 bg-gray-900/95 text-white px-4 py-3 text-sm backdrop-blur-sm">
      <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-3">
        <p className="flex-1 min-w-0">
          本系統使用必要性 Cookie
          以維持您的登入狀態與安全性。繼續使用即表示您同意我們的 Cookie 使用。
          <Link to="/privacy" className="ml-1 underline underline-offset-2 hover:text-gray-300">
            了解更多
          </Link>
        </p>
        <Button size="sm" onClick={handleAccept} className="shrink-0">
          接受
        </Button>
      </div>
    </div>
  )
}
