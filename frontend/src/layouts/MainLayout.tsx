import { useState, useEffect, Suspense } from 'react'
import { Outlet } from 'react-router-dom'
import { PageErrorBoundary } from '@/components/ui/page-error-boundary'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'
import api from '@/lib/api'
import { useSecurityAlerts } from '@/hooks/useSecurityAlerts'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Globe,
  Menu,
  UserCircle,
  ArrowLeft,
  AlertTriangle,
  ShieldCheck,
  Info,
} from 'lucide-react'
import { Sidebar } from '@/components/layout/Sidebar'
import { NotificationDropdown } from '@/components/layout/NotificationDropdown'
import { PasswordChangeDialog } from '@/components/layout/PasswordChangeDialog'
import { useSidebarStore } from '@/stores/sidebar'

function DelayedFallback({ delay = 300 }: { delay?: number }) {
  const [show, setShow] = useState(false)

  useEffect(() => {
    const timer = setTimeout(() => setShow(true), delay)
    return () => clearTimeout(timer)
  }, [delay])

  if (!show) return null

  return (
    <div className="flex items-center justify-center min-h-[60vh] animate-in fade-in duration-200">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>
  )
}

export function MainLayout() {
  const { user, hasRole, isGuest, isImpersonating, stopImpersonating } = useAuthStore()
  const { t, i18n } = useTranslation()
  useSecurityAlerts()

  const { sidebarOpen, setSidebarOpen } = useSidebarStore()
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false)
  const [showPasswordDialog, setShowPasswordDialog] = useState(false)

  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang)
  }

  const [showConfigWarnings, setShowConfigWarnings] = useState(false)
  const configWarningsSessionKey = 'ipig-config-warnings-dismissed'

  const { data: configWarningsData } = useQuery({
    queryKey: ['admin-config-warnings'],
    queryFn: async () => {
      const res = await api.get<{ warnings: { level: string; title: string; detail: string | null }[]; warn_count: number }>('/admin/config-warnings')
      return res.data
    },
    enabled: hasRole('admin') && !sessionStorage.getItem(configWarningsSessionKey),
    staleTime: Infinity,
    retry: false,
  })

  useEffect(() => {
    if (configWarningsData && configWarningsData.warn_count > 0 && !sessionStorage.getItem(configWarningsSessionKey)) {
      setShowConfigWarnings(true)
    }
  }, [configWarningsData])

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        sidebarOpen={sidebarOpen}
        setSidebarOpen={setSidebarOpen}
        mobileSidebarOpen={mobileSidebarOpen}
        setMobileSidebarOpen={setMobileSidebarOpen}
        onChangePassword={() => setShowPasswordDialog(true)}
      />

      <main
        style={{ contain: 'layout style' }}
        className={cn(
          "flex-1 overflow-y-auto transition-all duration-300 relative",
          'ml-0',
          sidebarOpen ? 'md:ml-0' : 'md:ml-0'
        )}
      >
        {isGuest() && (
          <div className="bg-[var(--color-status-warning-bg)] border-b border-[var(--color-status-warning-border)] px-4 py-2 text-sm text-[var(--color-status-warning-text)] text-center sticky top-0 z-[60]">
            訪客模式 — 僅供瀏覽系統架構，無法查看實際資料
          </div>
        )}

        {isImpersonating && (
          <div className="bg-primary text-primary-foreground px-4 py-2 flex items-center justify-between sticky top-0 z-[60] shadow-md">
            <div className="flex items-center gap-2">
              <UserCircle className="h-5 w-5" />
              <span className="text-sm font-medium">
                {t('common.impersonating')}：<span className="font-bold underline">{user?.display_name || user?.email}</span> ({user?.roles?.join(', ')})
              </span>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={stopImpersonating}
              className="bg-white/20 border-white text-white hover:bg-white hover:text-primary h-8 font-semibold transition-all"
            >
              <ArrowLeft className="h-4 w-4 mr-1" />
              {t('common.backToAdmin')}
            </Button>
          </div>
        )}

        <header className="sticky top-0 z-40 flex h-14 md:h-16 items-center justify-between border-b bg-card px-3 md:px-4 shadow-sm">
          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={() => setMobileSidebarOpen(true)}
            aria-label="開啟選單"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <div className="flex items-center space-x-2 md:space-x-4 ml-auto">
            <span className="text-sm text-muted-foreground hidden md:inline">
              {new Date().toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei' })}
            </span>

            <NotificationDropdown />

            <Select value={i18n.language} onValueChange={handleLanguageChange}>
              <SelectTrigger className="w-[60px] md:w-[120px] h-9" data-testid="language-selector">
                <Globe className="h-4 w-4 mr-2" />
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="zh-TW">{t('language.zhTW')}</SelectItem>
                <SelectItem value="en">{t('language.en')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </header>

        <div className="p-3 md:p-4" style={{ contain: 'layout style' }}>
          <PageErrorBoundary>
            <Suspense fallback={<DelayedFallback />}>
              <Outlet />
            </Suspense>
          </PageErrorBoundary>
        </div>
      </main>

      <PasswordChangeDialog
        open={showPasswordDialog}
        onOpenChange={setShowPasswordDialog}
      />

      <Dialog open={showConfigWarnings} onOpenChange={(open) => { if (!open) { sessionStorage.setItem(configWarningsSessionKey, '1'); setShowConfigWarnings(false) } }}>
        <DialogContent className="sm:max-w-lg" onPointerDownOutside={(e) => e.preventDefault()} onEscapeKeyDown={(e) => e.preventDefault()}>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-status-warning-text">
              <AlertTriangle className="h-5 w-5" />
              啟動配置警告
            </DialogTitle>
            <DialogDescription>
              系統偵測到以下配置需要注意，請管理員確認。
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3 py-2">
            {configWarningsData?.warnings.map((item, idx) => (
              <div
                key={idx}
                className={`flex items-start gap-3 rounded-lg p-3 ${item.level === 'warn'
                  ? 'bg-status-warning-bg border border-status-warning-border'
                  : item.level === 'ok'
                    ? 'bg-status-success-bg border border-status-success-border'
                    : 'bg-status-info-bg border border-status-info-border'
                  }`}
              >
                <div className="mt-0.5 shrink-0">
                  {item.level === 'warn' && <AlertTriangle className="h-5 w-5 text-status-warning-solid" />}
                  {item.level === 'ok' && <ShieldCheck className="h-5 w-5 text-status-success-solid" />}
                  {item.level === 'info' && <Info className="h-5 w-5 text-status-info-solid" />}
                </div>
                <div className="min-w-0">
                  <p className={`font-medium text-sm ${item.level === 'warn' ? 'text-status-warning-text' : item.level === 'ok' ? 'text-status-success-text' : 'text-status-info-text'
                    }`}>
                    {item.title}
                  </p>
                  {item.detail && (
                    <p className={`text-xs mt-1 ${item.level === 'warn' ? 'text-status-warning-text' : item.level === 'ok' ? 'text-status-success-text' : 'text-status-info-text'
                      }`}>
                      {item.detail}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
          <DialogFooter>
            <Button
              onClick={() => {
                sessionStorage.setItem(configWarningsSessionKey, '1')
                setShowConfigWarnings(false)
              }}
              className="w-full"
            >
              確認
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
