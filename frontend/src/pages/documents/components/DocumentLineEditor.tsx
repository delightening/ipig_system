import React, { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { Product, DocType, PoReceiptStatus, WarehouseTreeNode } from '@/lib/api'
import { WarehouseShelfTreeSelect, type WarehouseShelfValue } from '@/components/inventory/WarehouseShelfTreeSelect'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { DateTextInput } from '@/components/ui/DateTextInput'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Plus, Trash2, Search } from 'lucide-react'
import { formatNumber, formatUom } from '@/lib/utils'
import type { DocumentFormData, DocumentLine } from '../types'
import type { InputRefs } from '../hooks/useDocumentForm'
import type { AdjMode } from '../DocumentEditPage'
import { BatchNumberSelect } from './BatchNumberSelect'
import { ProductSearchDialog, type ProductSelectExtraData } from './ProductSearchDialog'
import { ShelfPickerDialog } from './ShelfPickerDialog'
import { warehouseChipStyle, warehouseColor } from '../warehouseColors'

/** SO 跨倉儲位輔助資料（儲位→所屬倉映射 + 倉別 chip 顯示旗標） */
export interface SoShelfContext {
  openPicker: (lineId: string) => void
  shelfInfo: Map<string, { whId: string; whName: string; whIdx: number; label: string }>
  /** 單內出現 ≥2 個倉才顯示倉別 chip（單倉單零噪音，2026-07-21 定案） */
  showChips: boolean
}

interface DocumentLineEditorProps {
  formData: DocumentFormData
  lineAmounts: Record<string, number>
  inputRefs: React.MutableRefObject<InputRefs>
  productSearchOpen: boolean
  setProductSearchOpen: (open: boolean) => void
  productSearch: string
  setProductSearch: (v: string) => void
  products: Product[] | undefined
  addLine: () => void
  removeLine: (lineId: string) => void
  selectProduct: (product: Product) => void
  openProductSearch: (lineId: string) => void
  handleBatchChange: (lineId: string, batchNo: string, expiryDate?: string, sourceIacuc?: string) => void
  handleLineBlur: (lineId: string) => void
  updateLineAmount: (lineId: string) => void
  updateLineField: <K extends keyof DocumentLine>(lineId: string, field: K, value: DocumentLine[K]) => void
  setFormData: React.Dispatch<React.SetStateAction<DocumentFormData>>
  needsShelf: boolean
  poReceiptStatus?: PoReceiptStatus
  categoryCode?: string
  setCategoryCode: (code: string | undefined) => void
  adjMode?: AdjMode
  /** 批次套用儲位（TR: 來源儲位；其他庫存模式：當前儲位）— 用於品項清單過濾 */
  batchStorageLocationFromId?: string
  batchStorageLocationId?: string
}

export function DocumentLineEditor({
  formData,
  lineAmounts,
  inputRefs,
  productSearchOpen,
  setProductSearchOpen,
  productSearch,
  setProductSearch,
  products,
  addLine,
  removeLine,
  selectProduct: _originalSelectProduct,
  openProductSearch,
  handleBatchChange,
  handleLineBlur,
  updateLineAmount,
  updateLineField,
  setFormData,
  needsShelf,
  poReceiptStatus,
  categoryCode,
  setCategoryCode,
  adjMode,
  batchStorageLocationFromId,
  batchStorageLocationId,
}: DocumentLineEditorProps) {
  // SO 為內部耗材領用不記金額（2026-07-21 裁定；byproduct 另有模組），不顯示單價欄；
  // DO 已棄用僅舊單顯示相容。
  const showPriceColumns = ['PO', 'GRN'].includes(formData.doc_type)
  const isSo = formData.doc_type === 'SO'
  const [activeLineId, setActiveLineId] = useState<string | null>(null)
  const [shelfPickerLineId, setShelfPickerLineId] = useState<string | null>(null)

  // SO 跨倉：儲位 → 所屬倉映射（標籤 / 倉別 chip / 批號過濾用；與樹選單共用快取）
  const { data: whTree } = useQuery({
    queryKey: ['warehouses-with-shelves'],
    queryFn: async () => {
      const res = await api.get<WarehouseTreeNode[]>('/warehouses/with-shelves')
      return res.data
    },
    staleTime: 5 * 60 * 1000,
    enabled: isSo,
  })
  const shelfInfo = useMemo(() => {
    const m = new Map<string, { whId: string; whName: string; whIdx: number; label: string }>()
    whTree?.forEach((wh, i) =>
      wh.shelves.forEach((s) =>
        m.set(s.id, { whId: wh.id, whName: wh.name, whIdx: i, label: `${wh.name} - ${s.name || s.code}` }),
      ),
    )
    return m
  }, [whTree])
  const distinctWhCount = useMemo(() => {
    if (!isSo) return 0
    const set = new Set<string>()
    formData.lines.forEach((l) => {
      const whId =
        l.warehouse_id || (l.storage_location_id ? shelfInfo.get(l.storage_location_id)?.whId : undefined)
      if (whId) set.add(whId)
    })
    return set.size
  }, [isSo, formData.lines, shelfInfo])
  const openShelfPicker = useCallback((lineId: string) => setShelfPickerLineId(lineId), [])
  const soShelf: SoShelfContext | undefined = isSo
    ? { openPicker: openShelfPicker, shelfInfo, showChips: distinctWhCount >= 2 }
    : undefined
  const pickerLine = formData.lines.find((l) => l.id === shelfPickerLineId)
  const isPoLinkedGrn = formData.doc_type === 'GRN' && !!formData.source_doc_id
  const targetWarehouseId = formData.doc_type === 'TR' ? formData.warehouse_from_id : formData.warehouse_id
  const filterStorageLocationId =
    formData.doc_type === 'TR' ? batchStorageLocationFromId : batchStorageLocationId

  const handleOpenSearch = (lineId: string) => {
    setActiveLineId(lineId)
    openProductSearch(lineId)
  }

  const handleSelectProduct = (product: Product, extraData?: ProductSelectExtraData) => {
    _originalSelectProduct(product)

    if (activeLineId) {
      setFormData((prev) => ({
        ...prev,
        lines: prev.lines.map((l) => {
          if (l.id !== activeLineId) return l

          const newLine: DocumentLine = {
            ...l,
            product_id: product.id,
            product_sku: product.sku,
            product_name: product.name,
            uom: product.base_uom,
          }

          if (isPoLinkedGrn && extraData) {
            newLine.unit_price = String(extraData.unit_price || '')
            newLine.qty = String(extraData.remaining_qty || '')
          } else if (extraData) {
            newLine.batch_no = extraData.batch_no || ''
            newLine.expiry_date = extraData.expiry_date || ''
            if (formData.doc_type === 'TR') {
              newLine.storage_location_from_id = extraData.storage_location_id || l.storage_location_from_id
            } else if (needsShelf) {
              newLine.storage_location_id = extraData.storage_location_id || l.storage_location_id
              // SO 跨倉：行倉庫＝存貨列來源倉（未分配列也正確）→ 儲位反推；
              // shelfInfo 未載入時清空，讓 soLineWarehouseId 之後重推，不沿用可能錯配的舊值。
              if (isSo) {
                newLine.warehouse_id =
                  extraData.warehouse_id ||
                  (newLine.storage_location_id
                    ? shelfInfo.get(newLine.storage_location_id)?.whId
                    : undefined)
              }
            }
          }
          return newLine
        })
      }))

      // qty / unit_price 已為 controlled input，不需直接寫 DOM ref
      if (extraData && !isPoLinkedGrn) {
        const refs = inputRefs.current[activeLineId]
        if (refs?.batch_no) refs.batch_no.value = extraData.batch_no || ''
        if (refs?.expiry_date) {
          const dateEl = refs.expiry_date as HTMLInputElement & { setIsoValue?: (v: string) => void }
          if (dateEl.setIsoValue) {
            dateEl.setIsoValue(extraData.expiry_date || '')
          }
        }
      }
    }
    setProductSearchOpen(false)
  }

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>單據明細</CardTitle>
          <Button onClick={addLine} size="sm">
            <Plus className="mr-2 h-4 w-4" />
            新增明細
          </Button>
        </CardHeader>
        <CardContent className="@container">
          {formData.lines.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">尚無明細，請點擊「新增明細」</p>
            </div>
          ) : (
            <>
              <div className="hidden @[900px]:block">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[60px]">項次</TableHead>
                      <TableHead className="w-[300px]">品項</TableHead>
                      <TableHead className="w-[100px] text-right">數量</TableHead>
                      <TableHead className="w-[80px]">單位</TableHead>
                      {showPriceColumns && (
                        <>
                          <TableHead className="w-[100px] text-right">單價</TableHead>
                          <TableHead className="w-[100px] text-right">金額</TableHead>
                        </>
                      )}
                      {formData.doc_type === 'TR' ? (
                        <>
                          <TableHead className="w-[180px]">來源儲位</TableHead>
                          <TableHead className="w-[180px]">目標儲位</TableHead>
                        </>
                      ) : needsShelf ? (
                        <TableHead className="w-[180px]">儲位</TableHead>
                      ) : null}
                      <TableHead className="w-[120px]">效期</TableHead>
                      <TableHead className="w-[120px]">批號</TableHead>
                      <TableHead className="w-[50px]" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {formData.lines.map((line, index) => (
                      <LineRow
                        key={line.id}
                        line={line}
                        index={index}
                        docType={formData.doc_type}
                        warehouseId={formData.warehouse_id}
                        showPriceColumns={showPriceColumns}
                        needsShelf={needsShelf}
                        lineAmounts={lineAmounts}
                        inputRefs={inputRefs}
                        onOpenSearch={handleOpenSearch}
                        onBatchChange={handleBatchChange}
                        onLineBlur={handleLineBlur}
                        onUpdateLineAmount={updateLineAmount}
                        updateLineField={updateLineField}
                        onRemoveLine={removeLine}
                        products={products}
                        soShelf={soShelf}
                      />
                    ))}
                  </TableBody>
                </Table>
              </div>

              <div className="@[900px]:hidden space-y-3">
                {formData.lines.map((line, index) => (
                  <LineCard
                    key={line.id}
                    line={line}
                    index={index}
                    docType={formData.doc_type}
                    warehouseId={formData.warehouse_id}
                    showPriceColumns={showPriceColumns}
                    needsShelf={needsShelf}
                    lineAmounts={lineAmounts}
                    inputRefs={inputRefs}
                    onOpenSearch={handleOpenSearch}
                    onBatchChange={handleBatchChange}
                    onLineBlur={handleLineBlur}
                    onUpdateLineAmount={updateLineAmount}
                    updateLineField={updateLineField}
                    onRemoveLine={removeLine}
                    products={products}
                    soShelf={soShelf}
                  />
                ))}
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {isSo && (
        <ShelfPickerDialog
          open={!!shelfPickerLineId}
          onOpenChange={(o) => {
            if (!o) setShelfPickerLineId(null)
          }}
          productId={pickerLine?.product_id || ''}
          productName={pickerLine?.product_name}
          defaultWarehouseId={formData.warehouse_id || undefined}
          value={pickerLine?.storage_location_id || undefined}
          onSelect={(sel) => {
            if (shelfPickerLineId) {
              updateLineField(shelfPickerLineId, 'storage_location_id', sel.locationId)
              updateLineField(shelfPickerLineId, 'warehouse_id', sel.warehouseId)
            }
          }}
        />
      )}

      <ProductSearchDialog
        open={productSearchOpen}
        onOpenChange={setProductSearchOpen}
        docType={formData.doc_type}
        targetWarehouseId={targetWarehouseId}
        filterStorageLocationId={filterStorageLocationId}
        isPoLinkedGrn={isPoLinkedGrn}
        poReceiptStatus={poReceiptStatus}
        productSearch={productSearch}
        setProductSearch={setProductSearch}
        categoryCode={categoryCode}
        setCategoryCode={setCategoryCode}
        products={products}
        onSelect={handleSelectProduct}
        adjMode={adjMode}
      />
    </>
  )
}

// --- 單行渲染元件 ---

interface LineRowProps {
  line: DocumentLine
  index: number
  docType: DocType
  warehouseId: string
  showPriceColumns: boolean
  needsShelf: boolean
  lineAmounts: Record<string, number>
  inputRefs: React.MutableRefObject<InputRefs>
  onOpenSearch: (lineId: string) => void
  onBatchChange: (lineId: string, batchNo: string, expiryDate?: string, sourceIacuc?: string) => void
  onLineBlur: (lineId: string) => void
  onUpdateLineAmount: (lineId: string) => void
  updateLineField: <K extends keyof DocumentLine>(lineId: string, field: K, value: DocumentLine[K]) => void
  onRemoveLine: (lineId: string) => void
  products: Product[] | undefined
  /** SO 跨倉儲位模式（僅 SO 有值）：儲位欄改開挑選器、倉別 chip、批號按行倉過濾 */
  soShelf?: SoShelfContext
}

/** SO 儲位挑選按鈕 + 倉別 chip（表格/卡片共用；行倉庫＝儲位所屬倉） */
function SoShelfCell({
  line,
  soShelf,
}: {
  line: DocumentLine
  soShelf: SoShelfContext
}) {
  const info = line.storage_location_id ? soShelf.shelfInfo.get(line.storage_location_id) : undefined
  return (
    <div className="space-y-1">
      <button
        type="button"
        onClick={() => soShelf.openPicker(line.id)}
        className="flex w-full items-center gap-1.5 rounded-md border px-2 py-1.5 text-left text-xs transition-colors hover:border-primary hover:text-primary"
      >
        {info && (
          <span
            className="h-2 w-2 shrink-0 rounded-sm"
            style={{ backgroundColor: warehouseColor(info.whIdx) }}
          />
        )}
        <span className="truncate">
          {info?.label || (line.storage_location_id ? '已選儲位' : '選擇儲位')}
        </span>
      </button>
      {soShelf.showChips && info && (
        <span
          className="inline-block rounded-full px-2 py-0.5 text-[10px] font-semibold"
          style={warehouseChipStyle(info.whIdx)}
        >
          {info.whName}
        </span>
      )}
    </div>
  )
}

/** SO 行的批號過濾倉：行倉庫（儲位所屬倉）優先，未選儲位時退回表頭預設倉 */
function soLineWarehouseId(line: DocumentLine, soShelf: SoShelfContext, headerWarehouseId: string): string {
  return (
    line.warehouse_id ||
    (line.storage_location_id ? soShelf.shelfInfo.get(line.storage_location_id)?.whId : undefined) ||
    headerWarehouseId
  )
}

function LineRow({
  line,
  index,
  docType,
  warehouseId,
  showPriceColumns,
  needsShelf,
  lineAmounts,
  inputRefs,
  onOpenSearch,
  onBatchChange,
  onLineBlur,
  onUpdateLineAmount,
  updateLineField,
  onRemoveLine,
  products,
  soShelf,
}: LineRowProps) {
  const lineId = line.id
  if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}

  const product = products?.find((p) => p.id === line.product_id)
  const showExpiry = !product || product.track_expiry
  const showBatch = !product || product.track_batch

  const expiryDateDefault = String(line.expiry_date || '')
  const batchNoDefault = String(line.batch_no || '')
  const batchWarehouseId = soShelf ? soLineWarehouseId(line, soShelf, warehouseId) : warehouseId

  const lineBatchChange = (batchNo: string, expiryDate?: string, sourceIacuc?: string) =>
    onBatchChange(lineId, batchNo, expiryDate, sourceIacuc)

  return (
    <TableRow>
      <TableCell className="text-center">{index + 1}</TableCell>
      <TableCell>
        {line.product_id ? (
          <div>
            <div className="font-medium">{line.product_name}</div>
            <div className="text-xs text-muted-foreground">{line.product_sku}</div>
          </div>
        ) : (
          <Button variant="outline" size="sm" onClick={() => onOpenSearch(lineId)} className="w-full justify-start">
            <Search className="mr-2 h-4 w-4" />
            選擇品項
          </Button>
        )}
      </TableCell>
      <TableCell>
        <Input
          type="number"
          value={line.qty ?? ''}
          ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].qty = el } }}
          className="text-right"
          min="0"
          onChange={(e) => {
            updateLineField(lineId, 'qty', e.target.value)
            if (showPriceColumns) onUpdateLineAmount(lineId)
          }}
          onBlur={() => onLineBlur(lineId)}
        />
      </TableCell>
      <TableCell>
        <span className="text-sm">{formatUom(line.uom) || '-'}</span>
      </TableCell>
      {showPriceColumns && (
        <>
          <TableCell>
            <Input
              type="number"
              value={line.unit_price ?? ''}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].unit_price = el } }}
              className="text-right"
              min="0"
              step="0.01"
              onChange={(e) => {
                updateLineField(lineId, 'unit_price', e.target.value)
              }}
              onBlur={() => onLineBlur(lineId)}
            />
          </TableCell>
          <TableCell className="text-right font-medium">
            ${formatNumber(lineAmounts[lineId] || 0, 2)}
          </TableCell>
        </>
      )}
      {docType === 'TR' ? (
        <>
          <TableCell>
            <WarehouseShelfTreeSelect
              value={line.storage_location_from_id ? `loc:${line.storage_location_from_id}` : ''}
              onValueChange={(v: WarehouseShelfValue) => {
                const id = v.startsWith('loc:') ? v.slice(4) : ''
                updateLineField(lineId, 'storage_location_from_id', id)
              }}
              selectLevel="shelf"
              allowAll={false}
              className="w-full text-xs"
              placeholder="選擇來源儲位"
            />
          </TableCell>
          <TableCell>
            <WarehouseShelfTreeSelect
              value={line.storage_location_to_id ? `loc:${line.storage_location_to_id}` : ''}
              onValueChange={(v: WarehouseShelfValue) => {
                const id = v.startsWith('loc:') ? v.slice(4) : ''
                updateLineField(lineId, 'storage_location_to_id', id)
              }}
              selectLevel="shelf"
              allowAll={false}
              className="w-full text-xs"
              placeholder="選擇目標儲位"
            />
          </TableCell>
        </>
      ) : needsShelf && soShelf ? (
        <TableCell>
          <SoShelfCell line={line} soShelf={soShelf} />
        </TableCell>
      ) : needsShelf ? (
        <TableCell>
          <WarehouseShelfTreeSelect
            value={line.storage_location_id ? `loc:${line.storage_location_id}` : ''}
            onValueChange={(v: WarehouseShelfValue) => {
              const id = v.startsWith('loc:') ? v.slice(4) : ''
              updateLineField(lineId, 'storage_location_id', id)
            }}
            selectLevel="shelf"
            allowAll={false}
            className="w-full text-xs"
            placeholder="選擇儲位"
          />
        </TableCell>
      ) : null}
      <TableCell>
        {showExpiry ? (
          ['SO', 'DO'].includes(docType) ? (
            <DateTextInput
              defaultValue={expiryDateDefault}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].expiry_date = el } }}
              readOnly
              disabled
              className="bg-muted cursor-not-allowed"
            />
          ) : (
            <DateTextInput
              defaultValue={expiryDateDefault}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].expiry_date = el } }}
              onChange={(iso) => updateLineField(lineId, 'expiry_date', iso)}
              onBlur={() => onLineBlur(lineId)}
            />
          )
        ) : (
          <span className="text-muted-foreground text-center block">-</span>
        )}
      </TableCell>
      <TableCell>
        {showBatch ? (
          <BatchNumberSelect
            productId={line.product_id}
            warehouseId={batchWarehouseId}
            batchNo={batchNoDefault}
            docType={docType}
            onBatchChange={lineBatchChange}
            onBlur={() => onLineBlur(lineId)}
            inputRef={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].batch_no = el } }}
          />
        ) : (
          <span className="text-muted-foreground text-center block">-</span>
        )}
      </TableCell>
      <TableCell>
        <Button variant="ghost" size="icon" onClick={() => onRemoveLine(lineId)} className="text-destructive hover:text-destructive/80" aria-label="刪除">
          <Trash2 className="h-4 w-4" />
        </Button>
      </TableCell>
    </TableRow>
  )
}

// --- 單行 Card 版本（mobile） ---

function LineCard({
  line,
  index,
  docType,
  warehouseId,
  showPriceColumns,
  needsShelf,
  lineAmounts,
  inputRefs,
  onOpenSearch,
  onBatchChange,
  onLineBlur,
  onUpdateLineAmount,
  updateLineField,
  onRemoveLine,
  products,
  soShelf,
}: LineRowProps) {
  const lineId = line.id
  if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}

  const product = products?.find((p) => p.id === line.product_id)
  const showExpiry = !product || product.track_expiry
  const showBatch = !product || product.track_batch

  const expiryDateDefault = String(line.expiry_date || '')
  const batchNoDefault = String(line.batch_no || '')
  const batchWarehouseId = soShelf ? soLineWarehouseId(line, soShelf, warehouseId) : warehouseId

  const lineBatchChange = (batchNo: string, expiryDate?: string, sourceIacuc?: string) =>
    onBatchChange(lineId, batchNo, expiryDate, sourceIacuc)

  return (
    <div className="rounded-lg border bg-card p-3 space-y-3">
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="text-xs text-muted-foreground">項次 #{index + 1}</div>
          {line.product_id ? (
            <div className="mt-1">
              <div className="font-medium break-words">{line.product_name}</div>
              <div className="text-xs text-muted-foreground">{line.product_sku}</div>
            </div>
          ) : (
            <Button variant="outline" size="sm" onClick={() => onOpenSearch(lineId)} className="w-full justify-start mt-1">
              <Search className="mr-2 h-4 w-4" />
              選擇品項
            </Button>
          )}
        </div>
        <Button variant="ghost" size="icon" onClick={() => onRemoveLine(lineId)} className="shrink-0 text-destructive hover:text-destructive/80" aria-label="刪除">
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div>
          <Label className="text-xs text-muted-foreground">數量</Label>
          <Input
            type="number"
            value={line.qty ?? ''}
            ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].qty = el } }}
            className="text-right"
            min="0"
            onChange={(e) => {
              updateLineField(lineId, 'qty', e.target.value)
              if (showPriceColumns) onUpdateLineAmount(lineId)
            }}
            onBlur={() => onLineBlur(lineId)}
          />
        </div>
        <div>
          <Label className="text-xs text-muted-foreground">單位</Label>
          <div className="h-9 flex items-center text-sm">{formatUom(line.uom) || '-'}</div>
        </div>
      </div>

      {showPriceColumns && (
        <div className="grid grid-cols-2 gap-2">
          <div>
            <Label className="text-xs text-muted-foreground">單價</Label>
            <Input
              type="number"
              value={line.unit_price ?? ''}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].unit_price = el } }}
              className="text-right"
              min="0"
              step="0.01"
              onChange={(e) => {
                updateLineField(lineId, 'unit_price', e.target.value)
              }}
              onBlur={() => onLineBlur(lineId)}
            />
          </div>
          <div>
            <Label className="text-xs text-muted-foreground">金額</Label>
            <div className="h-9 flex items-center text-sm font-medium">${formatNumber(lineAmounts[lineId] || 0, 2)}</div>
          </div>
        </div>
      )}

      {docType === 'TR' ? (
        <div className="space-y-2">
          <div>
            <Label className="text-xs text-muted-foreground">來源儲位</Label>
            <WarehouseShelfTreeSelect
              value={line.storage_location_from_id ? `loc:${line.storage_location_from_id}` : ''}
              onValueChange={(v: WarehouseShelfValue) => {
                const id = v.startsWith('loc:') ? v.slice(4) : ''
                updateLineField(lineId, 'storage_location_from_id', id)
              }}
              selectLevel="shelf"
              allowAll={false}
              className="w-full text-xs"
              placeholder="選擇來源儲位"
            />
          </div>
          <div>
            <Label className="text-xs text-muted-foreground">目標儲位</Label>
            <WarehouseShelfTreeSelect
              value={line.storage_location_to_id ? `loc:${line.storage_location_to_id}` : ''}
              onValueChange={(v: WarehouseShelfValue) => {
                const id = v.startsWith('loc:') ? v.slice(4) : ''
                updateLineField(lineId, 'storage_location_to_id', id)
              }}
              selectLevel="shelf"
              allowAll={false}
              className="w-full text-xs"
              placeholder="選擇目標儲位"
            />
          </div>
        </div>
      ) : needsShelf && soShelf ? (
        <div>
          <Label className="text-xs text-muted-foreground">儲位</Label>
          <SoShelfCell line={line} soShelf={soShelf} />
        </div>
      ) : needsShelf ? (
        <div>
          <Label className="text-xs text-muted-foreground">儲位</Label>
          <WarehouseShelfTreeSelect
            value={line.storage_location_id ? `loc:${line.storage_location_id}` : ''}
            onValueChange={(v: WarehouseShelfValue) => {
              const id = v.startsWith('loc:') ? v.slice(4) : ''
              updateLineField(lineId, 'storage_location_id', id)
            }}
            selectLevel="shelf"
            allowAll={false}
            className="w-full text-xs"
            placeholder="選擇儲位"
          />
        </div>
      ) : null}

      {showExpiry && (
        <div>
          <Label className="text-xs text-muted-foreground">效期</Label>
          {['SO', 'DO'].includes(docType) ? (
            <DateTextInput
              defaultValue={expiryDateDefault}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].expiry_date = el } }}
              readOnly
              disabled
              className="bg-muted cursor-not-allowed"
            />
          ) : (
            <DateTextInput
              defaultValue={expiryDateDefault}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].expiry_date = el } }}
              onChange={(iso) => updateLineField(lineId, 'expiry_date', iso)}
              onBlur={() => onLineBlur(lineId)}
            />
          )}
        </div>
      )}

      {showBatch && (
        <div>
          <Label className="text-xs text-muted-foreground">批號</Label>
          <BatchNumberSelect
            productId={line.product_id}
            warehouseId={batchWarehouseId}
            batchNo={batchNoDefault}
            docType={docType}
            onBatchChange={lineBatchChange}
            onBlur={() => onLineBlur(lineId)}
            inputRef={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].batch_no = el } }}
          />
        </div>
      )}
    </div>
  )
}
