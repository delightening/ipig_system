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
  const isSalesDoc = ['SO', 'DO'].includes(docType)
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
    return Array.from(batchMap.entries())
      .filter(([, data]) => data.qty > 0)
      .map(([batch, data]) => ({ batch, expiry: data.expiry, sourceIacuc: data.sourceIacuc }))
      .sort((a, b) => (a.expiry && b.expiry ? a.expiry.localeCompare(b.expiry) : a.batch.localeCompare(b.batch)))
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
      <Input ref={setInputRef} type="text" defaultValue={batchNo} placeholder="輸入批號" onBlur={onBlur} />
    )
  }

  if (isSalesDoc) {
    if (!productId || !warehouseId) {
      return <Input type="text" defaultValue={batchNo} placeholder="批號" disabled />
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
                {opt.batch}{opt.expiry && ` (${opt.expiry})`}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )
    }
    return <Input type="text" defaultValue={batchNo} placeholder="無可用批號" disabled />
  }

  return <Input ref={setInputRef} type="text" defaultValue={batchNo} placeholder="批號" onBlur={onBlur} />
}
