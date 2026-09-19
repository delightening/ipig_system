import { useState, useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Copy, Check, Info, ChevronDown, ChevronUp, RefreshCw, Loader2, AlertCircle, Settings2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn, uiLocale } from '@/lib/utils'

// SKU 狀態定義
export type SkuStatus = 'S0' | 'S1' | 'S2' | 'S3' | 'S4' | 'S5' | 'S6'

export interface SkuSegment {
  code: string
  label: string
  value: string
  source: string
  isUpdated?: boolean
}

export interface SkuPreviewResult {
  preview_sku: string
  segments: SkuSegment[]
  rule_version: string
  rule_updated_at?: string
  rule_change_summary?: string
}

export interface SkuPreviewError {
  code: 'E1' | 'E2' | 'E3' | 'E4' | 'E5'
  message: string
  suggestion?: string
  field?: string
  failed_segment?: string
}

export interface MissingField {
  field: string
  label: string
}

interface SkuPreviewBlockProps {
  status: SkuStatus
  previewResult?: SkuPreviewResult | null
  error?: SkuPreviewError | null
  missingFields?: MissingField[]
  finalSku?: string
  isLoading?: boolean
  canUseAdvancedMode?: boolean
  onRefresh?: () => void
  onFieldClick?: (field: string) => void
  className?: string
  compact?: boolean
}

// textKey / titleKey 為 i18n 鍵（模組頂層不存翻譯後字串），渲染時才 t()
const statusLabels: Record<SkuStatus, { textKey: string; variant: 'default' | 'secondary' | 'outline' | 'destructive' | 'success' }> = {
  S0: { textKey: 'erpMaster.skuPreview.status.S0', variant: 'outline' },
  S1: { textKey: 'erpMaster.skuPreview.status.S1', variant: 'secondary' },
  S2: { textKey: 'erpMaster.skuPreview.status.S2', variant: 'secondary' },
  S3: { textKey: 'erpMaster.skuPreview.status.S3', variant: 'default' },
  S4: { textKey: 'erpMaster.skuPreview.status.S4', variant: 'destructive' },
  S5: { textKey: 'erpMaster.skuPreview.status.S5', variant: 'secondary' },
  S6: { textKey: 'erpMaster.skuPreview.status.S6', variant: 'success' },
}

const errorMessages: Record<string, { titleKey: string; icon: React.ReactNode }> = {
  E1: { titleKey: 'erpMaster.skuPreview.error.E1', icon: <AlertCircle className="h-4 w-4" /> },
  E2: { titleKey: 'erpMaster.skuPreview.error.E2', icon: <AlertCircle className="h-4 w-4" /> },
  E3: { titleKey: 'erpMaster.skuPreview.error.E3', icon: <AlertCircle className="h-4 w-4" /> },
  E4: { titleKey: 'erpMaster.skuPreview.error.E4', icon: <AlertCircle className="h-4 w-4" /> },
  E5: { titleKey: 'erpMaster.skuPreview.error.E5', icon: <AlertCircle className="h-4 w-4" /> },
}

// 已知片段代碼 → 顯示名稱的 i18n 鍵（呼叫端傳入的 label 可能是舊語系下產生的，這裡以代碼為準）
const SEGMENT_LABEL_KEYS: Record<string, string> = {
  CATEGORY: 'erpMaster.skuPreview.segment.CATEGORY',
  ITEM: 'erpMaster.skuPreview.segment.ITEM',
  SERIAL: 'erpMaster.skuPreview.segment.SERIAL',
}

// SKU 片段色彩映射
const segmentColors: Record<string, { bg: string; text: string; border: string }> = {
  CATEGORY: { bg: 'bg-blue-50 dark:bg-blue-900/30', text: 'text-blue-700 dark:text-blue-300', border: 'border-blue-200 dark:border-blue-800' },
  ITEM: { bg: 'bg-indigo-50 dark:bg-indigo-900/30', text: 'text-indigo-700 dark:text-indigo-300', border: 'border-indigo-200 dark:border-indigo-800' },
  SERIAL: { bg: 'bg-emerald-50 dark:bg-emerald-900/30', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
  // Legacy or other mappings
  NAME: { bg: 'bg-violet-50 dark:bg-violet-900/30', text: 'text-violet-700 dark:text-violet-300', border: 'border-violet-200 dark:border-violet-800' },
  SPEC: { bg: 'bg-cyan-50 dark:bg-cyan-900/30', text: 'text-cyan-700 dark:text-cyan-300', border: 'border-cyan-200 dark:border-cyan-800' },
  SEQ: { bg: 'bg-emerald-50 dark:bg-emerald-900/30', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
}

const getSegmentColor = (code: string) => segmentColors[code] || segmentColors.ORG

export function SkuPreviewBlock({
  status,
  previewResult,
  error,
  missingFields = [],
  finalSku,
  isLoading = false,
  canUseAdvancedMode = false,
  onRefresh,
  onFieldClick,
  className,
  compact = false,
}: SkuPreviewBlockProps) {
  const { t } = useTranslation()
  const [copied, setCopied] = useState(false)
  const [expanded, setExpanded] = useState(true)
  const [infoOpen, setInfoOpen] = useState(false)
  const [advancedMode, setAdvancedMode] = useState(false)
  const [ruleVersionExpanded, setRuleVersionExpanded] = useState(false)
  const [hoveredSegment, setHoveredSegment] = useState<string | null>(null)

  const displaySku = useMemo(() => {
    if (status === 'S6' && finalSku) return finalSku
    if (status === 'S3' && previewResult?.preview_sku) return previewResult.preview_sku
    if (status === 'S2') return t('erpMaster.skuPreview.calculating')
    if (status === 'S4') return t('erpMaster.skuPreview.status.S4')
    if (status === 'S5') return t('erpMaster.skuPreview.creating')
    if (status === 'S1' && previewResult?.preview_sku) return previewResult.preview_sku
    return '— — — — — —'
  }, [status, previewResult, finalSku, t])

  const segmentLabel = (seg: SkuSegment) =>
    SEGMENT_LABEL_KEYS[seg.code] ? t(SEGMENT_LABEL_KEYS[seg.code]) : seg.label

  const canCopy = useMemo(() => {
    return ['S1', 'S3', 'S6'].includes(status) && (previewResult?.preview_sku || finalSku)
  }, [status, previewResult, finalSku])

  const canRefresh = useMemo(() => {
    return ['S1', 'S3', 'S4'].includes(status) && !isLoading
  }, [status, isLoading])

  const handleCopy = useCallback(async () => {
    const textToCopy = finalSku || previewResult?.preview_sku
    if (textToCopy) {
      await navigator.clipboard.writeText(textToCopy)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }, [finalSku, previewResult])

  const segments = useMemo(() => {
    if (previewResult?.segments) return previewResult.segments

    // 預設結構：種類、品項、流水號
    return [
      { code: 'CATEGORY', label: t('erpMaster.skuPreview.segment.CATEGORY'), value: '—', source: t('erpMaster.skuPreview.sourceMainCategory') },
      { code: 'ITEM', label: t('erpMaster.skuPreview.segment.ITEM'), value: '—', source: t('erpMaster.skuPreview.sourceSubcategory') },
      { code: 'SERIAL', label: t('erpMaster.skuPreview.segment.SERIAL'), value: status === 'S6' ? '—' : '001', source: t('erpMaster.skuPreview.serialSource') },
    ]
  }, [previewResult, status, t])

  return (
    <div className={cn(
      "rounded-xl border overflow-hidden",
      "bg-gradient-to-br from-slate-50 via-white to-slate-50/50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900/50",
      "shadow-xs",
      status === 'S6' && "ring-2 ring-success/50",
      className
    )}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200 dark:border-slate-700 bg-slate-50/50 dark:bg-slate-800/50">
        <div className="flex items-center gap-2">
          <h3 className="font-semibold text-slate-900 dark:text-slate-100">
            {t('erpMaster.skuPreview.title')}
          </h3>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0 text-slate-500 hover:text-slate-700"
            onClick={() => setInfoOpen(true)}
          >
            <Info className="h-3.5 w-3.5" />
          </Button>
        </div>
        <Badge
          variant={statusLabels[status].variant}
          className={cn(
            status === 'S6' && "bg-success text-success-foreground"
          )}
        >
          {t(statusLabels[status].textKey)}
        </Badge>
      </div>

      {/* SKU Display */}
      <div className="px-4 py-5">
        <div className="flex items-center gap-3">
          <div className={cn(
            "flex-1 px-5 py-4 rounded-lg font-mono text-lg tracking-wider sku-display",
            "bg-white dark:bg-slate-800/80",
            "border-2 transition-all duration-300",
            status === 'S2' || status === 'S5' ? "animate-pulse border-slate-200 dark:border-slate-700" : "",
            status === 'S4' ? "text-red-500 border-red-200 dark:border-red-800 animate-shake" : "",
            status === 'S6' ? "border-success/50 shadow-[0_0_0_3px_hsl(var(--success)/0.1)]" : "border-primary/30 shadow-[0_0_0_3px_hsl(var(--primary)/0.05)]",
            status === 'S0' && "text-slate-400",
            (status === 'S3' || status === 'S6') && "text-slate-900 dark:text-slate-100",
          )}>
            {status === 'S2' || status === 'S5' ? (
              <div className="flex items-center gap-2">
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
                <span className="text-slate-500">{status === 'S2' ? t('erpMaster.skuPreview.calculating') : t('erpMaster.skuPreview.creating')}</span>
              </div>
            ) : (status === 'S3' || status === 'S6' || status === 'S1') && previewResult?.segments ? (
              <div className="flex items-center">
                {previewResult.segments.map((seg, idx) => (
                  <div key={seg.code} className="flex items-center">
                    <div
                      className={cn(
                        "px-3 py-1.5 rounded-md font-bold tracking-tight transition-all duration-300 cursor-default border-2",
                        getSegmentColor(seg.code).bg,
                        getSegmentColor(seg.code).text,
                        getSegmentColor(seg.code).border,
                        hoveredSegment === seg.code
                          ? "scale-110 shadow-lg ring-4 ring-primary/10 -translate-y-0.5"
                          : "opacity-90 hover:opacity-100"
                      )}
                      onMouseEnter={() => setHoveredSegment(seg.code)}
                      onMouseLeave={() => setHoveredSegment(null)}
                    >
                      {seg.value || '—'}
                    </div>
                    {idx < previewResult.segments.length - 1 && (
                      <div className="mx-3 flex flex-col items-center justify-center opacity-30">
                        <div className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            ) : status === 'S6' ? (
              <div className="flex items-center gap-2">
                <Check className="h-5 w-5 text-success animate-success-bounce" />
                <span>{displaySku}</span>
              </div>
            ) : (
              displaySku
            )}
          </div>

          {/* Copy Button */}
          <Button
            variant="outline"
            size="icon"
            disabled={!canCopy}
            onClick={handleCopy}
            className="shrink-0 h-10 w-10"
          >
            {copied ? (
              <Check className="h-4 w-4 text-success" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>

          {/* Refresh Button */}
          {canRefresh && onRefresh && (
            <Button
              variant="outline"
              size="icon"
              onClick={onRefresh}
              disabled={isLoading}
              className="shrink-0 h-10 w-10"
            >
              <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
            </Button>
          )}
        </div>

        {/* Rule Version */}
        {previewResult?.rule_version && (
          <div className="mt-3">
            <button
              type="button"
              className="text-xs text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 flex items-center gap-1 transition-colors"
              onClick={() => setRuleVersionExpanded(!ruleVersionExpanded)}
            >
              {t('erpMaster.skuPreview.ruleVersion', { version: previewResult.rule_version })}
              {ruleVersionExpanded ? (
                <ChevronUp className="h-3 w-3" />
              ) : (
                <ChevronDown className="h-3 w-3" />
              )}
            </button>
            {ruleVersionExpanded && (
              <div className="mt-2 p-3 rounded-lg bg-slate-100 dark:bg-slate-800 text-xs space-y-2 animate-fade-in">
                <div className="flex justify-between">
                  <span className="text-slate-500">{t('erpMaster.skuPreview.lastUpdated')}</span>
                  <span className="text-slate-700 dark:text-slate-300 font-medium">
                    {previewResult.rule_updated_at
                      ? new Date(previewResult.rule_updated_at).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })
                      : '—'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">{t('erpMaster.skuPreview.changeSummary')}</span>
                  <span className="text-slate-700 dark:text-slate-300">
                    {previewResult.rule_change_summary || t('erpMaster.skuPreview.initialVersion')}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">{t('erpMaster.skuPreview.ruleStatus')}</span>
                  <span className="text-success font-medium">{t('erpMaster.skuPreview.ruleActive')}</span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Error Display */}
      {status === 'S4' && error && (
        <div className="mx-4 mb-4 p-4 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 animate-fade-in">
          <div className="flex items-start gap-3">
            <AlertCircle className="h-5 w-5 text-red-500 mt-0.5 shrink-0" />
            <div className="flex-1">
              <p className="text-sm font-semibold text-red-700 dark:text-red-400">
                {errorMessages[error.code] ? t(errorMessages[error.code].titleKey) : t('erpMaster.skuPreview.error.generic')}
              </p>
              {error.failed_segment && (
                <p className="text-xs text-red-500 dark:text-red-400 mt-1">
                  {t('erpMaster.skuPreview.failedSegment')}<span className="font-mono font-bold">{error.failed_segment}</span>
                </p>
              )}
              <p className="text-sm text-red-600 dark:text-red-300 mt-1">
                {error.message}
              </p>
              {error.suggestion && (
                <p className="text-sm text-red-500/80 dark:text-red-400/80 mt-1">
                  💡 {error.suggestion}
                </p>
              )}
              {error.field && onFieldClick && (
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0 mt-2 text-red-600 dark:text-red-400 font-medium"
                  onClick={() => onFieldClick(error.field!)}
                >
                  {t('erpMaster.skuPreview.goFix')}
                </Button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Missing Fields */}
      {status === 'S0' && missingFields.length > 0 && (
        <div className="mx-4 mb-4 p-4 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
          <p className="text-sm font-medium text-amber-700 dark:text-amber-400 mb-3">
            {t('erpMaster.skuPreview.fillMissing')}
          </p>
          <div className="flex flex-wrap gap-2">
            {missingFields.map((field) => (
              <Button
                key={field.field}
                variant="outline"
                size="sm"
                className="h-8 text-xs border-amber-300 text-amber-700 hover:bg-amber-100 dark:border-amber-700 dark:text-amber-400 dark:hover:bg-amber-900/30"
                onClick={() => onFieldClick?.(field.field)}
              >
                {field.label} →
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* Segments Detail */}
      {!compact && (
        <div className="border-t border-slate-200 dark:border-slate-700">
          <button
            type="button"
            className="w-full flex items-center justify-between px-4 py-3 text-sm text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800/50 transition-colors"
            onClick={() => setExpanded(!expanded)}
          >
            <span className="font-medium">{t('erpMaster.skuPreview.segmentBreakdown')}</span>
            {expanded ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <ChevronDown className="h-4 w-4" />
            )}
          </button>

          {expanded && (
            <div className="px-4 pb-4 animate-fade-in">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                {segments.map((seg, index) => {
                  const colors = getSegmentColor(seg.code)
                  const isPlaceholder = seg.value === '—' || seg.value === 'XXX' || seg.value === 'X'

                  return (
                    <div
                      key={seg.code}
                      className={cn(
                        "sku-segment relative p-3 rounded-lg border cursor-default",
                        colors.bg, colors.border,
                        seg.isUpdated && "animate-segment-highlight",
                        hoveredSegment === seg.code && "ring-2 ring-primary/30"
                      )}
                      style={{ animationDelay: `${index * 50}ms` }}
                      onMouseEnter={() => setHoveredSegment(seg.code)}
                      onMouseLeave={() => setHoveredSegment(null)}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className={cn("font-mono text-[10px] font-semibold", colors.text)}>
                          {seg.code}
                        </span>
                        <span className="text-[10px] text-slate-400">
                          {segmentLabel(seg)}
                        </span>
                      </div>
                      <div className={cn(
                        "font-mono text-sm font-medium truncate",
                        isPlaceholder ? "text-slate-400" : colors.text
                      )}>
                        {seg.value || '—'}
                      </div>
                      <div className="text-[10px] text-slate-400 truncate mt-1">
                        {seg.source}
                      </div>

                      {/* Tooltip on hover */}
                      {hoveredSegment === seg.code && (
                        <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 rounded-lg bg-slate-900 text-white text-xs whitespace-nowrap z-10 shadow-lg">
                          <div className="font-semibold mb-1">{t('erpMaster.skuPreview.segmentTitle', { label: segmentLabel(seg) })}</div>
                          <div className="text-slate-300">{t('erpMaster.skuPreview.segmentSource', { source: seg.source })}</div>
                          <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-slate-900" />
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Advanced Mode */}
      {canUseAdvancedMode && status !== 'S6' && !compact && (
        <div className="border-t border-slate-200 dark:border-slate-700 px-4 py-3">
          <Button
            variant="ghost"
            size="sm"
            className="text-slate-500 hover:text-slate-700 w-full justify-start"
            onClick={() => setAdvancedMode(!advancedMode)}
          >
            <Settings2 className="h-4 w-4 mr-2" />
            {t('erpMaster.skuPreview.advanced.title')}
            {advancedMode ? (
              <ChevronUp className="h-4 w-4 ml-auto" />
            ) : (
              <ChevronDown className="h-4 w-4 ml-auto" />
            )}
          </Button>
          {advancedMode && (
            <div className="mt-3 space-y-4 animate-fade-in">
              <p className="text-xs text-slate-500 dark:text-slate-400">
                {t('erpMaster.skuPreview.advanced.description')}
              </p>

              <div className="p-3 rounded-lg bg-slate-100 dark:bg-slate-800">
                <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('erpMaster.skuPreview.advanced.nameStrategy')}
                </label>
                <div className="mt-2 space-y-2">
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="name-strategy" defaultChecked className="h-3 w-3" />
                    <span>{t('erpMaster.skuPreview.advanced.nameAuto')}</span>
                  </label>
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="name-strategy" className="h-3 w-3" />
                    <span>{t('erpMaster.skuPreview.advanced.nameKeep6')}</span>
                  </label>
                </div>
              </div>

              <div className="p-3 rounded-lg bg-slate-100 dark:bg-slate-800">
                <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
                  {t('erpMaster.skuPreview.advanced.specStrategy')}
                </label>
                <div className="mt-2 space-y-2">
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="spec-strategy" defaultChecked className="h-3 w-3" />
                    <span>{t('erpMaster.skuPreview.advanced.specFull')}</span>
                  </label>
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="spec-strategy" className="h-3 w-3" />
                    <span>{t('erpMaster.skuPreview.advanced.specNumeric')}</span>
                  </label>
                </div>
              </div>

              <p className="text-[10px] text-amber-600 dark:text-amber-400">
                ⚠️ {t('erpMaster.skuPreview.advanced.reasonRequired')}
              </p>
            </div>
          )}
        </div>
      )}

      {/* Info Dialog */}
      <Dialog open={infoOpen} onOpenChange={setInfoOpen}>
        <DialogContent size="lg">
          <DialogHeader>
            <DialogTitle>{t('erpMaster.skuPreview.info.title')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-6">
            {/* New Structure */}
            <div>
              <h4 className="font-medium mb-3">{t('erpMaster.skuPreview.info.structure')}</h4>
              <div className="p-4 bg-slate-100 dark:bg-slate-800 rounded-lg font-mono text-sm">
                <div className="flex flex-wrap items-center gap-3">
                  {['CATEGORY', 'ITEM', 'SERIAL'].map((seg) => {
                    const colors = getSegmentColor(seg)
                    return (
                      <span
                        key={seg}
                        className={cn("px-4 py-2 rounded-lg border font-bold text-base", colors.bg, colors.text, colors.border)}
                      >
                        {t(SEGMENT_LABEL_KEYS[seg])}
                      </span>
                    )
                  })}
                </div>
                <p className="text-slate-500 text-xs mt-4">
                  {t('erpMaster.skuPreview.info.structureFormat')}
                </p>
              </div>
            </div>

            {/* Current Mapping */}
            <div>
              <h4 className="font-medium mb-3">{t('erpMaster.skuPreview.info.mapping')}</h4>
              <div className="grid grid-cols-2 gap-2 text-sm">
                {segments.map((seg) => {
                  const colors = getSegmentColor(seg.code)
                  return (
                    <div key={seg.code} className={cn("flex justify-between p-2 rounded", colors.bg, colors.border, "border")}>
                      <span className={cn("font-mono", colors.text)}>{seg.code}</span>
                      <span className="font-medium">{seg.value || '—'}</span>
                    </div>
                  )
                })}
              </div>
            </div>

            {/* FAQ */}
            <div>
              <h4 className="font-medium mb-3">{t('erpMaster.skuPreview.info.faq')}</h4>
              <div className="space-y-3 text-sm">
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    {t('erpMaster.skuPreview.info.q1')}
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    {t('erpMaster.skuPreview.info.a1')}
                  </p>
                </details>
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    {t('erpMaster.skuPreview.info.q2')}
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    {t('erpMaster.skuPreview.info.a2')}
                  </p>
                </details>
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    {t('erpMaster.skuPreview.info.q3')}
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    {t('erpMaster.skuPreview.info.a3', { zhExample: '手套 → SLT' })}
                  </p>
                </details>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
