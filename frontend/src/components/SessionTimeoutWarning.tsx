import { useState, useEffect, useCallback } from 'react'
import { useAuthStore } from '@/stores/auth'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Timer, LogOut } from 'lucide-react'
import { useTranslation } from 'react-i18next'

const WARNING_THRESHOLD_MS = 60_000

export function SessionTimeoutWarning() {
  const { t } = useTranslation()
  const { isAuthenticated, sessionExpiresAt, refreshSession, logout } = useAuthStore()
  const [showWarning, setShowWarning] = useState(false)
  const [remaining, setRemaining] = useState(0)
  const [isRefreshing, setIsRefreshing] = useState(false)

  useEffect(() => {
    if (!isAuthenticated || !sessionExpiresAt) {
      setShowWarning(false)
      return
    }

    const check = () => {
      const ms = sessionExpiresAt - Date.now()
      setRemaining(Math.max(0, Math.ceil(ms / 1000)))

      if (ms <= 0) {
        setShowWarning(false)
        logout()
      } else if (ms <= WARNING_THRESHOLD_MS) {
        setShowWarning(true)
      } else {
        setShowWarning(false)
      }
    }

    check()
    const id = setInterval(check, 1000)
    return () => clearInterval(id)
  }, [isAuthenticated, sessionExpiresAt, logout])

  const handleExtend = useCallback(async () => {
    setIsRefreshing(true)
    const ok = await refreshSession()
    setIsRefreshing(false)
    if (ok) {
      setShowWarning(false)
    } else {
      logout()
    }
  }, [refreshSession, logout])

  if (!showWarning) return null

  return (
    <Dialog open={showWarning} onOpenChange={() => {}}>
      <DialogContent className="sm:max-w-[400px]" onPointerDownOutside={(e) => e.preventDefault()}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-amber-600">
            <Timer className="h-5 w-5" />
            {t('session.timeoutTitle', '登入即將過期')}
          </DialogTitle>
          <DialogDescription>
            {t('session.timeoutDescription', '您的登入階段即將結束，請選擇續期或登出。')}
          </DialogDescription>
        </DialogHeader>
        <div className="flex items-center justify-center py-4">
          <div className="text-4xl font-mono font-bold text-amber-600 tabular-nums">
            {remaining}s
          </div>
        </div>
        <DialogFooter className="gap-2 sm:gap-0">
          <Button variant="outline" onClick={() => logout()}>
            <LogOut className="h-4 w-4 mr-2" />
            {t('session.logout', '登出')}
          </Button>
          <Button onClick={handleExtend} disabled={isRefreshing}>
            {t('session.extend', '繼續使用')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
