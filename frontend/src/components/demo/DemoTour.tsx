/**
 * DemoTour — 訪客模式 9 站系統導覽
 *
 * 啟動方式：訪客 banner 或 dashboard 上的「🎓 系統導覽」按鈕，
 * 也可以從 URL `?tour=1` 自動啟動。
 *
 * 結構：
 * - DemoTour（容器）：狀態 / 自動 navigate / mode 切換
 * - TourIntroDialog：第 1 站歡迎，使用 Radix Dialog（focus trap + Escape + aria）
 * - TourWalkingCard：第 2~9 站，右下 floating card（非 modal）
 *
 * 訪客每次進站都會自動開啟（不使用 localStorage 持久化），可從按鈕或 ?tour=1 重啟。
 */

import { useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { ChevronLeft, ChevronRight, X, GraduationCap } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogTitle, DialogDescription } from '@/components/ui/dialog'

interface TourStopMeta {
  path?: string    // 點下一步時自動 navigate
  ctaPath?: string // 點 CTA 時跳到的路徑（未指定 = 用 stop.path）
}

interface TourStopContent {
  title: string
  body: string[]
  cta?: string
}

// 路徑類技術資訊保留在 code，文案 (title/body/cta) 全走 i18n (`guest.tour.stops`)
const TOUR_STOP_META: TourStopMeta[] = [
  { path: '/dashboard' },
  { path: '/protocols', ctaPath: '/protocols/demo-p1' },
  { path: '/protocols/new' },
  { path: '/animals', ctaPath: '/animals/demo-a1' },
  { path: '/animals/demo-a1' },
  { path: '/animals/demo-a1' },
  { path: '/admin/audit-logs' },
  { path: '/admin/qau' },
  {},
]

interface DemoTourProps {
  open: boolean
  onClose: () => void
}

export function DemoTour({ open, onClose }: DemoTourProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [stepIdx, setStepIdx] = useState(0)
  // 第一站（歡迎）以 modal 居中呈現吸引注意；點「開始」後切換為右下角 floating
  const [mode, setMode] = useState<'intro' | 'walking'>('intro')
  const stops = useMemo(
    () => (t('guest.tour.stops', { returnObjects: true }) as TourStopContent[]),
    [t],
  )
  const stop = stops[stepIdx] ?? { title: '', body: [] }
  const meta = useMemo(() => TOUR_STOP_META[stepIdx] ?? {}, [stepIdx])
  const totalStops = TOUR_STOP_META.length - 1
  const isLast = stepIdx === TOUR_STOP_META.length - 1

  // 只在「站點切換」時自動 navigate；使用者自行導航（如點計畫書 row 進
  // detail 頁）不會被拉回。用 ref 記住上次站點，僅在切換時觸發 navigate
  const prevStepRef = useRef<number | null>(null)
  useEffect(() => {
    if (!open || mode === 'intro') {
      prevStepRef.current = null
      return
    }
    if (prevStepRef.current === stepIdx) return  // 沒切站，不導航
    prevStepRef.current = stepIdx
    if (meta.path) navigate(meta.path)
  }, [open, mode, stepIdx, meta, navigate])

  // 重新開啟時重設到 intro 第 1 站
  useEffect(() => {
    if (open) {
      setStepIdx(0)
      setMode('intro')
    }
  }, [open])

  if (!open) return null

  const handleStart = () => {
    setMode('walking')
    setStepIdx(1) // 跳到第 1 站（申請書管理）
  }

  const handleNext = () => {
    if (isLast) {
      onClose()
      return
    }
    setStepIdx(i => i + 1)
  }

  const handlePrev = () => setStepIdx(i => i - 1)

  if (mode === 'intro') {
    return (
      <TourIntroDialog
        open={open}
        intro={stops[0] ?? { title: '', body: [] }}
        totalStops={totalStops}
        onSkip={onClose}
        onStart={handleStart}
      />
    )
  }

  return (
    <TourWalkingCard
      view={{ stop, meta, stepIdx, totalStops, isLast }}
      actions={{
        onPrev: handlePrev,
        onNext: handleNext,
        onSkip: onClose,
        onCta: (target) => navigate(target),
      }}
    />
  )
}

// ──────────────────────────────────────────────────────────────────────
// TourIntroDialog — 第 1 站歡迎畫面，Radix Dialog（focus trap + Escape）
// ──────────────────────────────────────────────────────────────────────

interface TourIntroDialogProps {
  open: boolean
  intro: TourStopContent
  totalStops: number
  onSkip: () => void
  onStart: () => void
}

function TourIntroDialog({ open, intro, totalStops, onSkip, onStart }: TourIntroDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) onSkip() }}>
      <DialogContent>
        <div className="flex items-center gap-2 pr-6">
          <GraduationCap className="h-6 w-6 text-primary shrink-0" />
          <DialogTitle className="text-lg font-bold">{intro.title}</DialogTitle>
        </div>
        <div className="space-y-3 pt-2">
          {/* 第一段以 DialogDescription 渲染（兼任 aria-describedby），其餘段落用 <p>，
              避免 sr-only + 可見內容重複造成螢幕閱讀器朗讀兩次 */}
          {intro.body.map((para, i) =>
            i === 0 ? (
              <DialogDescription key={i} className="text-sm leading-relaxed text-foreground">
                {para}
              </DialogDescription>
            ) : (
              <p key={i} className="text-sm leading-relaxed text-foreground">{para}</p>
            ),
          )}
          <p className="text-xs text-muted-foreground pt-2">
            {t('guest.tour.brief', { count: totalStops })}
          </p>
        </div>
        <div className="flex items-center justify-end gap-2 pt-2">
          <Button variant="outline" onClick={onSkip}>
            {t('guest.tour.controls.skip')}
          </Button>
          <Button onClick={onStart}>
            {t('guest.tour.controls.start')}
            <ChevronRight className="ml-1 h-4 w-4" />
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}

// ──────────────────────────────────────────────────────────────────────
// TourWalkingCard — 第 2~9 站，右下 floating card
// ──────────────────────────────────────────────────────────────────────

interface TourWalkingView {
  stop: TourStopContent
  meta: TourStopMeta
  stepIdx: number
  totalStops: number
  isLast: boolean
}

interface TourWalkingActions {
  onPrev: () => void
  onNext: () => void
  onSkip: () => void
  onCta: (target: string) => void
}

interface TourWalkingCardProps {
  view: TourWalkingView
  actions: TourWalkingActions
}

function TourWalkingCard({ view, actions }: TourWalkingCardProps) {
  const { stop, meta, stepIdx, totalStops, isLast } = view
  const { onPrev, onNext, onSkip, onCta } = actions
  const { t } = useTranslation()
  return (
    <div className="fixed bottom-4 right-4 z-[9999] w-[28rem] max-w-[calc(100vw-2rem)] rounded-lg shadow-2xl bg-card border border-border">
      <div className="flex items-start justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <GraduationCap className="h-5 w-5 text-primary" />
          <span className="text-sm font-medium text-muted-foreground">
            {stepIdx} / {totalStops}
          </span>
        </div>
        <button
          onClick={onSkip}
          className="text-muted-foreground hover:text-foreground"
          aria-label={t('guest.tour.controls.closeTour')}
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="px-4 py-3 space-y-3">
        <h3 className="font-bold text-base">{stop.title}</h3>
        {stop.body.map((para, i) => (
          <p key={i} className="text-sm leading-relaxed text-foreground">{para}</p>
        ))}
        {stop.cta && (
          <button
            type="button"
            onClick={() => {
              const target = meta.ctaPath || meta.path
              if (target) onCta(target)
            }}
            className="text-sm font-medium text-primary text-left hover:underline w-full"
          >
            👉 {stop.cta}
          </button>
        )}
      </div>

      {/* footer: progress dots 行 + 操作按鈕行（mobile-safe，避免 overflow） */}
      <div className="border-t border-border px-4 py-3 space-y-2">
        <div className="flex flex-wrap gap-1">
          {Array.from({ length: totalStops }, (_, i) => {
            const realIdx = i + 1
            return (
              <span
                key={i}
                className={`h-1.5 w-4 rounded-full transition-colors ${
                  realIdx === stepIdx ? 'bg-primary' : realIdx < stepIdx ? 'bg-primary/50' : 'bg-muted'
                }`}
              />
            )
          })}
        </div>
        <div className="flex items-center justify-end gap-2">
          {stepIdx > 1 && (
            <Button size="sm" variant="ghost" onClick={onPrev}>
              <ChevronLeft className="h-4 w-4 mr-1" />
              {t('guest.tour.controls.prev')}
            </Button>
          )}
          <Button size="sm" onClick={onNext}>
            {isLast ? t('guest.tour.controls.finish') : t('guest.tour.controls.next')}
            {!isLast && <ChevronRight className="ml-1 h-4 w-4" />}
          </Button>
        </div>
      </div>
    </div>
  )
}
