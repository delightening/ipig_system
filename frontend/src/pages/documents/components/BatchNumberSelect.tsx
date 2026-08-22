/**
 * 批號選擇元件 — 依據單據類型自動切換輸入/選擇模式
 */
import { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { StockLedgerDetail, DocType } from '@/lib/api'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { STALE_TIME } from '@/lib/query'
import { compareByFefo, expiryStatus, isExpiringSoon } from '../expiry'

interface BatchNumberSelectProps {
  productId: string
  warehouseId: string
  batchNo: string
  docType: DocType
  onBatchChange: (batchNo: string, expiryDate?: string, sourceIacuc?: string) => void
  onBlur?: () => void
  inputRef?: (el: HTMLInputElement | null) => void
}

export function BatchNumberSelect({
  productId,
  warehouseId,
  batchNo,
  docType,
  onBatchChange,
  onBlur,
  inputRef,
}: BatchNumberSelectProps) {
  const isSalesDoc = docType === 'SO'
  const isPurchaseDoc = ['PO', 'GRN', 'PR'].includes(docType)

  const { data: stockLedger } = useQuery({
    queryKey: ['stock-ledger', productId, warehouseId],
    queryFn: async () => {
      if (!productId || !warehouseId) return []
      const response = await api.get<StockLedgerDetail[]>(
        `/inventory/ledger?product_id=${productId}&warehouse_id=${warehouseId}`
      )
      return response.data
    },
    enabled: !!productId && !!warehouseId && isSalesDoc,
    staleTime: STALE_TIME.REALTIME,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false,
  })

  const batchOptions = useMemo(() => {
    if (!stockLedger?.length) return []
    const batchMap = new Map<string, { qty: number; expiry: string; sourceIacuc?: string }>()
    stockLedger.forEach((entry) => {
      if (!entry.batch_no?.trim()) return
      const batch = entry.batch_no
      const qty = parseFloat(entry.qty_base) || 0
      const isIn = ['in', 'transfer_in', 'adjust_in'].includes(entry.direction)
      const qtyChange = isIn ? qty : -qty
      if (batchMap.has(batch)) {
        const existing = batchMap.get(batch)!
        existing.qty += qtyChange
        if (entry.expiry_date && !existing.expiry) existing.expiry = entry.expiry_date
        if (entry.iacuc_no && !existing.sourceIacuc) existing.sourceIacuc = entry.iacuc_no
      } else {
        batchMap.set(batch, { qty: qtyChange, expiry: entry.expiry_date || '', sourceIacuc: entry.iacuc_no })
      }
    })
    return (
      Array.from(batchMap.entries())
        .filter(([, data]) => data.qty > 0)
        .map(([batch, data]) => ({
          batch,
          expiry: data.expiry,
          sourceIacuc: data.sourceIacuc,
          ...expiryStatus(data.expiry),
        }))
        // FEFO：先到期先出。規則與理由見 `compareByFefo`（抽成純函式以便測試）。
        .sort(compareByFefo)
    )
  }, [stockLedger])

  const setInputRef = useCallback(
    (el: HTMLInputElement | null) => { if (inputRef) inputRef(el) },
    [inputRef]
  )

  const handleBatchChangeInternal = useCallback(
    (value: string) => {
      const selected = batchOptions.find((opt) => opt.batch === value)
      onBatchChange(value, selected?.expiry, selected?.sourceIacuc)
    },
    [batchOptions, onBatchChange]
  )

  if (isPurchaseDoc) {
    return (
      <Input
        ref={setInputRef}
        type="text"
        value={batchNo}
        onChange={(e) => onBatchChange(e.target.value)}
        placeholder="輸入批號"
        onBlur={onBlur}
        aria-label="批號"
      />
    )
  }

  if (isSalesDoc) {
    if (!productId || !warehouseId) {
      return <Input type="text" value={batchNo} readOnly placeholder="批號" disabled aria-label="批號" />
    }
    if (batchOptions.length > 0) {
      return (
        <Select value={batchNo} onValueChange={handleBatchChangeInternal}>
          <SelectTrigger>
            <SelectValue placeholder="選擇批號" />
          </SelectTrigger>
          <SelectContent>
            {batchOptions.map((opt) => (
              <SelectItem key={opt.batch} value={opt.batch}>
                <span className="flex items-center gap-2">
                  <span>{opt.batch}</span>
                  {/* 判斷依據是 `daysLeft !== null`（能不能判讀）而非 `opt.expiry`
                      （有沒有字串）：效期若是 2026-02-31 這種不存在的日期，
                      expiryStatus 會回 null，此時不該把原字串當成有效效期顯示，
                      更不該推算天數（CodeRabbit 於 PR #143 指出）。 */}
                  {opt.daysLeft !== null && (
                    <span
                      className={
                        opt.expired
                          ? 'text-destructive font-medium'
                          : isExpiringSoon(opt)
                            ? 'text-amber-600 dark:text-amber-500'
                            : 'text-muted-foreground'
                      }
                    >
                      {opt.expiry}
                      {opt.expired
                        ? `（已過期 ${Math.abs(opt.daysLeft)} 天）`
                        : isExpiringSoon(opt)
                          ? `（剩 ${opt.daysLeft} 天）`
                          : ''}
                    </span>
                  )}
                  {/* 有字串但判讀不出來 → 明白告訴使用者資料有問題，不靜默隱藏。 */}
                  {opt.daysLeft === null && opt.expiry && (
                    <span className="text-muted-foreground italic">
                      {opt.expiry}（效期格式異常）
                    </span>
                  )}
                </span>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )
    }
    return <Input type="text" value={batchNo} readOnly placeholder="無可用批號" disabled aria-label="批號" />
  }

  return (
    <Input
      ref={setInputRef}
      type="text"
      value={batchNo}
      onChange={(e) => onBatchChange(e.target.value)}
      placeholder="批號"
      onBlur={onBlur}
      aria-label="批號"
    />
  )
}
