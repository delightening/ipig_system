import React, { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { Product, DocType, StockLedgerDetail } from '@/lib/api'
import { WarehouseShelfTreeSelect, type WarehouseShelfValue } from '@/components/inventory/WarehouseShelfTreeSelect'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Plus, Trash2, Search } from 'lucide-react'
import { STALE_TIME } from '@/lib/query'
import { formatNumber, formatUom } from '@/lib/utils'
import type { DocumentFormData } from '../types'
import type { InputRefs } from '../hooks/useDocumentForm'

function BatchNumberSelectWithQuery({
  lineIndex: _lineIndex,
  productId,
  warehouseId,
  batchNo,
  docType,
  onBatchChange,
  onBlur,
  inputRef,
}: {
  lineIndex: number
  productId: string
  warehouseId: string
  batchNo: string
  docType: DocType
  onBatchChange: (batchNo: string, expiryDate?: string, sourceIacuc?: string) => void
  onBlur?: () => void
  inputRef?: (el: HTMLInputElement | null) => void
}) {
  const isSalesDoc = ['SO', 'DO'].includes(docType)
  const { data: stockLedger } = useQuery({
    queryKey: ['stock-ledger', productId, warehouseId],
    queryFn: async () => {
      if (!productId || !warehouseId) return []
      const response = await api.get<StockLedgerDetail[]>(
        `/inventory/ledger?product_id=${productId}&warehouse_id=${warehouseId}`
      )
      return response.data
    },
    enabled:
      !!productId &&
      !!warehouseId &&
      isSalesDoc,
    staleTime: STALE_TIME.REALTIME,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false,
  })

  const batchOptions = useMemo(() => {
    if (!stockLedger?.length) return []
    const batchMap = new Map<string, { qty: number; expiry: string; sourceIacuc?: string }>()
    stockLedger.forEach((entry) => {
      if (entry.batch_no?.trim()) {
        const batch = entry.batch_no
        const qty = parseFloat(entry.qty_base) || 0
        const isIn = ['in', 'transfer_in', 'adjust_in'].includes(
          entry.direction
        )
        const qtyChange = isIn ? qty : -qty
        if (batchMap.has(batch)) {
          const existing = batchMap.get(batch)!
          existing.qty += qtyChange
          if (entry.expiry_date && !existing.expiry) {
            existing.expiry = entry.expiry_date
          }
          if (entry.iacuc_no && !existing.sourceIacuc) {
            existing.sourceIacuc = entry.iacuc_no
          }
        } else {
          batchMap.set(batch, {
            qty: qtyChange,
            expiry: entry.expiry_date || '',
            sourceIacuc: entry.iacuc_no,
          })
        }
      }
    })
    return Array.from(batchMap.entries())
      .filter(([, data]) => data.qty > 0)
      .map(([batch, data]) => ({ batch, expiry: data.expiry, sourceIacuc: data.sourceIacuc }))
      .sort((a, b) => {
        if (a.expiry && b.expiry) return a.expiry.localeCompare(b.expiry)
        return a.batch.localeCompare(b.batch)
      })
  }, [stockLedger])

  const isPurchaseDoc = ['PO', 'GRN', 'PR'].includes(docType)
  const setInputRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (inputRef) inputRef(el)
    },
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
        defaultValue={batchNo}
        placeholder="輸入批號"
        onBlur={onBlur}
      />
    )
  }

  if (isSalesDoc) {
    if (!productId || !warehouseId) {
      return (
        <Input
          type="text"
          defaultValue={batchNo}
          placeholder="批號"
          disabled
        />
      )
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
                {opt.batch}
                {opt.expiry && ` (${opt.expiry})`}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )
    }
    return (
      <Input
        type="text"
        defaultValue={batchNo}
        placeholder="無可用批號"
        disabled
      />
    )
  }

  return (
    <Input
      ref={setInputRef}
      type="text"
      defaultValue={batchNo}
      placeholder="批號"
    />
  )
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
  setFormData: any
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
  selectProduct,
  openProductSearch,
  handleBatchChange,
  handleLineBlur,
  updateLineAmount,
  setFormData,
}: DocumentLineEditorProps) {
  const showPriceColumns = ['PO', 'GRN', 'DO'].includes(formData.doc_type)

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
        <CardContent>
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
              ) : ['GRN', 'SO', 'ADJ'].includes(formData.doc_type) ? (
                <TableHead className="w-[180px]">儲位</TableHead>
              ) : null}
              <TableHead className="w-[120px]">效期</TableHead>
              <TableHead className="w-[120px]">批號</TableHead>
              <TableHead className="w-[50px]" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {formData.lines.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={showPriceColumns ? 9 : 7}
                  className="text-center py-8"
                >
                  <p className="text-muted-foreground">
                    尚無明細，請點擊「新增明細」
                  </p>
                </TableCell>
              </TableRow>
            ) : (
              formData.lines.map((line, index) => {
                const lineId = line.id || `temp-${index}`
                if (!inputRefs.current[lineId]) {
                  inputRefs.current[lineId] = {}
                }
                const qtyDefault = String(line.qty || '')
                const unitPriceDefault = String(line.unit_price || '')
                const expiryDateDefault = String(line.expiry_date || '')
                const batchNoDefault = String(line.batch_no || '')
                const lineBatchChange = (batchNo: string, expiryDate?: string, sourceIacuc?: string) =>
                  handleBatchChange(lineId, batchNo, expiryDate, sourceIacuc)

                return (
                  <TableRow key={lineId}>
                    <TableCell className="text-center">{index + 1}</TableCell>
                    <TableCell>
                      {line.product_id ? (
                        <div>
                          <div className="font-medium">{line.product_name}</div>
                          <div className="text-xs text-muted-foreground">
                            {line.product_sku}
                          </div>
                        </div>
                      ) : (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => openProductSearch(lineId)}
                          className="w-full justify-start"
                        >
                          <Search className="mr-2 h-4 w-4" />
                          選擇品項
                        </Button>
                      )}
                    </TableCell>
                    <TableCell>
                      <Input
                        type="number"
                        defaultValue={qtyDefault}
                        ref={(el) => {
                          if (el) {
                            if (!inputRefs.current[lineId])
                              inputRefs.current[lineId] = {}
                            inputRefs.current[lineId].qty = el
                          }
                        }}
                        className="text-right"
                        min="0"
                        onChange={() => {
                          if (showPriceColumns) updateLineAmount(lineId)
                        }}
                        onBlur={() => handleLineBlur(lineId)}
                      />
                    </TableCell>
                    <TableCell>
                      <span className="text-sm">
                        {formatUom(line.uom) || '-'}
                      </span>
                    </TableCell>
                    {showPriceColumns && (
                      <>
                        <TableCell>
                          <Input
                            type="number"
                            defaultValue={unitPriceDefault}
                            ref={(el) => {
                              if (el) {
                                if (!inputRefs.current[lineId])
                                  inputRefs.current[lineId] = {}
                                inputRefs.current[lineId].unit_price = el
                              }
                            }}
                            className="text-right"
                            min="0"
                            step="0.01"
                            onChange={() => updateLineAmount(lineId)}
                            onBlur={() => handleLineBlur(lineId)}
                          />
                        </TableCell>
                        <TableCell className="text-right font-medium">
                          ${formatNumber(lineAmounts[lineId] || 0, 2)}
                        </TableCell>
                      </>
                    )}
                    {formData.doc_type === 'TR' ? (
                      <>
                        <TableCell>
                          <WarehouseShelfTreeSelect
                            value={line.storage_location_from_id ? `loc:${line.storage_location_from_id}` : ''}
                            onValueChange={(v: WarehouseShelfValue) => {
                               const id = v.startsWith('loc:') ? v.slice(4) : ''
                               handleBatchChange(lineId, batchNoDefault, expiryDateDefault) // 觸發連動 collect
                               setFormData((prev: any) => ({
                                 ...prev,
                                 lines: prev.lines.map((l: any) => l.id === lineId ? { ...l, storage_location_from_id: id } : l)
                               }))
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
                               setFormData((prev: any) => ({
                                 ...prev,
                                 lines: prev.lines.map((l: any) => l.id === lineId ? { ...l, storage_location_to_id: id } : l)
                               }))
                            }}
                            selectLevel="shelf"
                            allowAll={false}
                            className="w-full text-xs"
                            placeholder="選擇目標儲位"
                          />
                        </TableCell>
                      </>
                    ) : ['GRN', 'SO', 'ADJ'].includes(formData.doc_type) ? (
                      <TableCell>
                        <WarehouseShelfTreeSelect
                          value={line.storage_location_id ? `loc:${line.storage_location_id}` : ''}
                          onValueChange={(v: WarehouseShelfValue) => {
                             const id = v.startsWith('loc:') ? v.slice(4) : ''
                             setFormData((prev: { lines: any[] }) => ({
                               ...prev,
                               lines: prev.lines.map((l) => l.id === lineId ? { ...l, storage_location_id: id } : l)
                             }))
                          }}
                          selectLevel="shelf"
                          allowAll={false}
                          className="w-full text-xs"
                          placeholder="選擇儲位"
                        />
                      </TableCell>
                    ) : null}
                    <TableCell>
                      {['SO', 'DO'].includes(formData.doc_type) ? (
                        <Input
                          type="date"
                          defaultValue={expiryDateDefault}
                          ref={(el) => {
                            if (el) {
                              if (!inputRefs.current[lineId])
                                inputRefs.current[lineId] = {}
                              inputRefs.current[lineId].expiry_date = el
                            }
                          }}
                          readOnly
                          disabled
                          className="bg-muted cursor-not-allowed"
                        />
                      ) : (
                        <Input
                          type="date"
                          defaultValue={expiryDateDefault}
                          ref={(el) => {
                            if (el && inputRefs.current[lineId])
                              inputRefs.current[lineId].expiry_date = el
                          }}
                          placeholder="效期"
                          onBlur={() => handleLineBlur(lineId)}
                        />
                      )}
                    </TableCell>
                    <TableCell>
                      <BatchNumberSelectWithQuery
                        lineIndex={index}
                        productId={line.product_id}
                        warehouseId={formData.warehouse_id}
                        batchNo={batchNoDefault}
                        docType={formData.doc_type}
                        onBatchChange={lineBatchChange}
                        onBlur={() => handleLineBlur(lineId)}
                        inputRef={(el) => {
                          if (el) {
                            if (!inputRefs.current[lineId])
                              inputRefs.current[lineId] = {}
                            inputRefs.current[lineId].batch_no = el
                          }
                        }}
                      />
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => removeLine(lineId)}
                        className="text-red-500 hover:text-red-700"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
        </CardContent>
      </Card>

      <Dialog open={productSearchOpen} onOpenChange={setProductSearchOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>選擇品項</DialogTitle>
            <DialogDescription>搜尋並選擇要新增的品項</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜尋品項..."
                value={productSearch}
                onChange={(e) => setProductSearch(e.target.value)}
                className="pl-9"
                autoFocus
              />
            </div>
            <div className="max-h-[400px] overflow-y-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>SKU</TableHead>
                    <TableHead>品項名稱</TableHead>
                    <TableHead>規格</TableHead>
                    <TableHead>單位</TableHead>
                    <TableHead />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {products?.slice(0, 20).map((product) => (
                    <TableRow
                      key={product.id}
                      className="cursor-pointer hover:bg-muted"
                      onClick={() => selectProduct(product)}
                    >
                      <TableCell className="font-mono text-xs">
                        {product.sku}
                      </TableCell>
                      <TableCell className="font-medium">{product.name}</TableCell>
                      <TableCell>{product.spec || '-'}</TableCell>
                      <TableCell>{formatUom(product.base_uom)}</TableCell>
                      <TableCell>
                        <Button size="sm" variant="outline">
                          選擇
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}
