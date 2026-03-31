import { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'

const STORAGE_KEY = 'cookie-consent'

type ConsentLevel = 'all' | 'essential'

/** 讀取目前的 consent 狀態（供外部使用） */
// eslint-disable-next-line react-refresh/only-export-components
export function getCookieConsent(): ConsentLevel | null {
  const value = localStorage.getItem(STORAGE_KEY)
  if (value === 'all' || value === 'essential') return value
  return null
}

/** 使用者是否已接受非必要 Cookie（含第三方資源） */
// eslint-disable-next-line react-refresh/only-export-components
export function hasFullConsent(): boolean {
  return getCookieConsent() === 'all'
}

/**
 * 根據 consent 狀態動態載入 Google Fonts。
 * 僅在使用者選擇「接受全部」時注入外部字型樣式表。
 */
function injectGoogleFonts() {
  if (document.querySelector('link[data-consent-font]')) return
  const preconnect1 = document.createElement('link')
  preconnect1.rel = 'preconnect'
  preconnect1.href = 'https://fonts.googleapis.com'
  preconnect1.setAttribute('data-consent-font', 'true')

  const preconnect2 = document.createElement('link')
  preconnect2.rel = 'preconnect'
  preconnect2.href = 'https://fonts.gstatic.com'
  preconnect2.crossOrigin = 'anonymous'
  preconnect2.setAttribute('data-consent-font', 'true')

  const stylesheet = document.createElement('link')
  stylesheet.rel = 'stylesheet'
  stylesheet.href =
    'https://fonts.googleapis.com/css2?family=Noto+Sans+TC:wght@300;400;500;600;700&display=swap'
  stylesheet.setAttribute('data-consent-font', 'true')

  document.head.append(preconnect1, preconnect2, stylesheet)
}

export function CookieConsent() {
  const [visible, setVisible] = useState(
    () => getCookieConsent() === null,
  )

  // 如果使用者先前已同意全部，啟動時即注入第三方字型
  useEffect(() => {
    if (hasFullConsent()) {
      injectGoogleFonts()
    }
  }, [])

  if (!visible) return null

  const handleAccept = (level: ConsentLevel) => {
    localStorage.setItem(STORAGE_KEY, level)
    setVisible(false)
    if (level === 'all') {
      injectGoogleFonts()
    }
  }

  return (
    <div className="fixed bottom-0 inset-x-0 z-50 bg-gray-900/95 text-white px-4 py-3 text-sm backdrop-blur-sm">
      <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-3">
        <p className="flex-1 min-w-0">
          本系統使用必要性 Cookie 以維持您的登入狀態與安全性。
          選擇「接受全部」將允許載入第三方字型等外部資源以提升顯示效果。
          <Link to="/privacy" className="ml-1 underline underline-offset-2 hover:text-muted-foreground">
            了解更多
          </Link>
        </p>
        <div className="flex gap-2 shrink-0">
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleAccept('essential')}
            className="border-gray-500 text-muted-foreground hover:bg-gray-800 hover:text-white"
          >
            僅必要 Cookie
          </Button>
          <Button size="sm" onClick={() => handleAccept('all')}>
            接受全部
          </Button>
        </div>
      </div>
    </div>
  )
}
