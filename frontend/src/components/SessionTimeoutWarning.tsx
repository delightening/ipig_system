import { useState, useEffect, useCallback, useRef } from 'react'
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
  const warningTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    if (!isAuthenticated || !sessionExpiresAt) {
      setShowWarning(false)
      return
    }

    const cleanup = () => {
      if (warningTimerRef.current) clearTimeout(warningTimerRef.current)
      if (countdownRef.current) clearInterval(countdownRef.current)
      warningTimerRef.current = null
      countdownRef.current = null
    }

    cleanup()

    const msUntilExpiry = sessionExpiresAt - Date.now()

    if (msUntilExpiry <= 0) {
      logout()
      return
    }

    const msUntilWarning = msUntilExpiry - WARNING_THRESHOLD_MS

    const startCountdown = () => {
      setShowWarning(true)
      setRemaining(Math.max(0, Math.ceil((sessionExpiresAt - Date.now()) / 1000)))

      countdownRef.current = setInterval(() => {
        const ms = sessionExpiresAt - Date.now()
        if (ms <= 0) {
          cleanup()
          setShowWarning(false)
          logout()
        } else {
          setRemaining(Math.ceil(ms / 1000))
        }
      }, 1000)
    }

    if (msUntilWarning <= 0) {
      startCountdown()
    } else {
      warningTimerRef.current = setTimeout(startCountdown, msUntilWarning)
    }

    return cleanup
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
          <DialogTitle className="flex items-center gap-2 text-status-warning-text">
            <Timer className="h-5 w-5" />
            {t('session.timeoutTitle', '登入即將過期')}
          </DialogTitle>
          <DialogDescription>
            {t('session.timeoutDescription', '您的登入階段即將結束，請選擇續期或登出。')}
          </DialogDescription>
        </DialogHeader>
        <div className="flex items-center justify-center py-4">
          <div className="text-4xl font-mono font-bold text-status-warning-text tabular-nums">
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
