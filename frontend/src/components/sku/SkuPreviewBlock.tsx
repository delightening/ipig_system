import { useState, useCallback, useMemo } from 'react'
import { Copy, Check, Info, ChevronDown, ChevronUp, RefreshCw, Loader2, AlertCircle, Settings2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/utils'

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

const statusLabels: Record<SkuStatus, { text: string; variant: 'default' | 'secondary' | 'outline' | 'destructive' | 'success' }> = {
  S0: { text: '尚無法預覽', variant: 'outline' },
  S1: { text: '可預覽', variant: 'secondary' },
  S2: { text: '計算中', variant: 'secondary' },
  S3: { text: '預覽', variant: 'default' },
  S4: { text: '預覽失敗', variant: 'destructive' },
  S5: { text: '建立中', variant: 'secondary' },
  S6: { text: '已建立', variant: 'success' },
}

const errorMessages: Record<string, { title: string; icon: React.ReactNode }> = {
  E1: { title: '缺少必填欄位', icon: <AlertCircle className="h-4 w-4" /> },
  E2: { title: '規則無對應', icon: <AlertCircle className="h-4 w-4" /> },
  E3: { title: '規格值不合法', icon: <AlertCircle className="h-4 w-4" /> },
  E4: { title: '片段生成衝突', icon: <AlertCircle className="h-4 w-4" /> },
  E5: { title: '系統錯誤', icon: <AlertCircle className="h-4 w-4" /> },
}

// SKU 片段色彩映射
const segmentColors: Record<string, { bg: string; text: string; border: string }> = {
  NAME: { bg: 'bg-violet-50 dark:bg-violet-900/30', text: 'text-violet-700 dark:text-violet-300', border: 'border-violet-200 dark:border-violet-800' },
  NAME_ABBR: { bg: 'bg-violet-50 dark:bg-violet-900/30', text: 'text-violet-700 dark:text-violet-300', border: 'border-violet-200 dark:border-violet-800' },
  SPEC: { bg: 'bg-cyan-50 dark:bg-cyan-900/30', text: 'text-cyan-700 dark:text-cyan-300', border: 'border-cyan-200 dark:border-cyan-800' },
  SPEC_CODE: { bg: 'bg-cyan-50 dark:bg-cyan-900/30', text: 'text-cyan-700 dark:text-cyan-300', border: 'border-cyan-200 dark:border-cyan-800' },
  UNIT: { bg: 'bg-amber-50 dark:bg-amber-900/30', text: 'text-amber-700 dark:text-amber-300', border: 'border-amber-200 dark:border-amber-800' },
  DATE: { bg: 'bg-emerald-50 dark:bg-emerald-900/30', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
  YYMMDD: { bg: 'bg-emerald-50 dark:bg-emerald-900/30', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
  SEQ: { bg: 'bg-indigo-50 dark:bg-indigo-900/30', text: 'text-indigo-700 dark:text-indigo-300', border: 'border-indigo-200 dark:border-indigo-800' },
  CHK: { bg: 'bg-pink-50 dark:bg-pink-900/30', text: 'text-pink-700 dark:text-pink-300', border: 'border-pink-200 dark:border-pink-800' },
  // Legacy mappings
  ORG: { bg: 'bg-slate-50 dark:bg-slate-800', text: 'text-slate-700 dark:text-slate-300', border: 'border-slate-200 dark:border-slate-700' },
  CAT: { bg: 'bg-violet-50 dark:bg-violet-900/30', text: 'text-violet-700 dark:text-violet-300', border: 'border-violet-200 dark:border-violet-800' },
  SUB: { bg: 'bg-violet-50 dark:bg-violet-900/30', text: 'text-violet-700 dark:text-violet-300', border: 'border-violet-200 dark:border-violet-800' },
  ATTR: { bg: 'bg-cyan-50 dark:bg-cyan-900/30', text: 'text-cyan-700 dark:text-cyan-300', border: 'border-cyan-200 dark:border-cyan-800' },
  PACK: { bg: 'bg-amber-50 dark:bg-amber-900/30', text: 'text-amber-700 dark:text-amber-300', border: 'border-amber-200 dark:border-amber-800' },
  SRC: { bg: 'bg-teal-50 dark:bg-teal-900/30', text: 'text-teal-700 dark:text-teal-300', border: 'border-teal-200 dark:border-teal-800' },
  YYWW: { bg: 'bg-emerald-50 dark:bg-emerald-900/30', text: 'text-emerald-700 dark:text-emerald-300', border: 'border-emerald-200 dark:border-emerald-800' },
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
  const [copied, setCopied] = useState(false)
  const [expanded, setExpanded] = useState(true)
  const [infoOpen, setInfoOpen] = useState(false)
  const [advancedMode, setAdvancedMode] = useState(false)
  const [ruleVersionExpanded, setRuleVersionExpanded] = useState(false)
  const [hoveredSegment, setHoveredSegment] = useState<string | null>(null)

  const displaySku = useMemo(() => {
    if (status === 'S6' && finalSku) return finalSku
    if (status === 'S3' && previewResult?.preview_sku) return previewResult.preview_sku
    if (status === 'S2') return '計算中...'
    if (status === 'S4') return '預覽失敗'
    if (status === 'S5') return '建立中...'
    if (status === 'S1' && previewResult?.preview_sku) return previewResult.preview_sku
    return '— — — — — —'
  }, [status, previewResult, finalSku])

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
    
    // 新版預設結構（根據 skuSpec.md）
    return [
      { code: 'NAME_ABBR', label: '名稱', value: '—', source: '產品名稱縮寫' },
      { code: 'SPEC_CODE', label: '規格', value: '—', source: '規格摘要' },
      { code: 'UNIT', label: '單位', value: '—', source: '庫存單位' },
      { code: 'YYMMDD', label: '日期', value: '—', source: '系統日期' },
      { code: 'SEQ', label: '序號', value: status === 'S6' ? '—' : 'XXX', source: '建立時分配' },
      { code: 'CHK', label: '檢查碼', value: status === 'S6' ? '—' : 'X', source: '建立時計算' },
    ]
  }, [previewResult, status])

  return (
    <div className={cn(
      "rounded-xl border overflow-hidden",
      "bg-gradient-to-br from-slate-50 via-white to-slate-50/50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900/50",
      "shadow-sm",
      status === 'S6' && "ring-2 ring-success/50",
      className
    )}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-slate-200 dark:border-slate-700 bg-slate-50/50 dark:bg-slate-800/50">
        <div className="flex items-center gap-2">
          <h3 className="font-semibold text-slate-900 dark:text-slate-100">
            SKU 預覽
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
          {statusLabels[status].text}
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
                <span className="text-slate-500">{status === 'S2' ? '計算中...' : '建立中...'}</span>
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
              規則版本 {previewResult.rule_version}
              {ruleVersionExpanded ? (
                <ChevronUp className="h-3 w-3" />
              ) : (
                <ChevronDown className="h-3 w-3" />
              )}
            </button>
            {ruleVersionExpanded && (
              <div className="mt-2 p-3 rounded-lg bg-slate-100 dark:bg-slate-800 text-xs space-y-2 animate-fade-in">
                <div className="flex justify-between">
                  <span className="text-slate-500">最後更新時間</span>
                  <span className="text-slate-700 dark:text-slate-300 font-medium">
                    {previewResult.rule_updated_at 
                      ? new Date(previewResult.rule_updated_at).toLocaleDateString('zh-TW')
                      : '—'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">變更摘要</span>
                  <span className="text-slate-700 dark:text-slate-300">
                    {previewResult.rule_change_summary || '初始版本'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-500">規則狀態</span>
                  <span className="text-success font-medium">啟用中</span>
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
                {errorMessages[error.code]?.title || '發生錯誤'}
              </p>
              {error.failed_segment && (
                <p className="text-xs text-red-500 dark:text-red-400 mt-1">
                  推導失敗於片段：<span className="font-mono font-bold">{error.failed_segment}</span>
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
                  前往修正 →
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
            請填寫以下欄位以預覽 SKU：
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
            <span className="font-medium">SKU 片段解析</span>
            {expanded ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <ChevronDown className="h-4 w-4" />
            )}
          </button>
          
          {expanded && (
            <div className="px-4 pb-4 animate-fade-in">
              <div className="grid grid-cols-3 sm:grid-cols-6 gap-2">
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
                          {seg.label}
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
                          <div className="font-semibold mb-1">{seg.label}片段</div>
                          <div className="text-slate-300">來源：{seg.source}</div>
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
            進階設定
            {advancedMode ? (
              <ChevronUp className="h-4 w-4 ml-auto" />
            ) : (
              <ChevronDown className="h-4 w-4 ml-auto" />
            )}
          </Button>
          {advancedMode && (
            <div className="mt-3 space-y-4 animate-fade-in">
              <p className="text-xs text-slate-500 dark:text-slate-400">
                進階模式允許調整 SKU 生成策略，但不能直接輸入 SKU 值。所有調整將記錄於稽核日誌。
              </p>
              
              <div className="p-3 rounded-lg bg-slate-100 dark:bg-slate-800">
                <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
                  名稱縮寫策略
                </label>
                <div className="mt-2 space-y-2">
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="name-strategy" defaultChecked className="h-3 w-3" />
                    <span>自動縮寫（英文取首字母、中文取拼音）</span>
                  </label>
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="name-strategy" className="h-3 w-3" />
                    <span>保留完整名稱前 6 字元</span>
                  </label>
                </div>
              </div>

              <div className="p-3 rounded-lg bg-slate-100 dark:bg-slate-800">
                <label className="text-xs font-medium text-slate-700 dark:text-slate-300">
                  規格碼策略
                </label>
                <div className="mt-2 space-y-2">
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="spec-strategy" defaultChecked className="h-3 w-3" />
                    <span>數字 + 單位 + 特徵（如 500MGTB）</span>
                  </label>
                  <label className="flex items-center gap-2 text-xs">
                    <input type="radio" name="spec-strategy" className="h-3 w-3" />
                    <span>純數字（如 500）</span>
                  </label>
                </div>
              </div>

              <p className="text-[10px] text-amber-600 dark:text-amber-400">
                ⚠️ 調整生成策略需填寫原因，並將記錄於稽核日誌
              </p>
            </div>
          )}
        </div>
      )}

      {/* Info Dialog */}
      <Dialog open={infoOpen} onOpenChange={setInfoOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>SKU 編碼說明</DialogTitle>
          </DialogHeader>
          <div className="space-y-6">
            {/* New Structure */}
            <div>
              <h4 className="font-medium mb-3">SKU 結構（新版）</h4>
              <div className="p-4 bg-slate-100 dark:bg-slate-800 rounded-lg font-mono text-sm">
                <div className="flex flex-wrap items-center gap-1">
                  {['NAME_ABBR', 'SPEC_CODE', 'UNIT', 'YYMMDD', 'SEQ', 'CHK'].map((seg) => {
                    const colors = getSegmentColor(seg)
                    return (
                      <span
                        key={seg}
                        className={cn("px-2 py-1 rounded", colors.bg, colors.text)}
                      >
                        {seg}
                      </span>
                    )
                  })}
                </div>
                <p className="text-slate-500 text-xs mt-3">
                  名稱-規格-單位-日期-序號-檢查碼
                </p>
              </div>
            </div>

            {/* Current Mapping */}
            <div>
              <h4 className="font-medium mb-3">本頁對照</h4>
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
              <h4 className="font-medium mb-3">常見問題</h4>
              <div className="space-y-3 text-sm">
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    為什麼我不能手改 SKU？
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    SKU 完整值永遠由系統決定，以確保編碼的一致性與唯一性。這也便於後續的追蹤與管理。
                  </p>
                </details>
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    為什麼預覽沒有序號？
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    序號 (SEQ) 和檢查碼 (CHK) 只有在產品正式建立時才會分配，預覽階段不會保留序號。
                  </p>
                </details>
                <details className="group">
                  <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-300 hover:text-slate-900">
                    如何產生名稱縮寫？
                  </summary>
                  <p className="mt-2 text-slate-600 dark:text-slate-400 pl-4">
                    英文取每個單詞首字母（如 Amoxicillin → AMX），中文取拼音首字母（如 手套 → SLT）。
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
