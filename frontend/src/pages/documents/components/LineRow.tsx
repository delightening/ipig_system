import React from 'react'
import { DocType } from '@/lib/api'
import { WarehouseShelfTreeSelect, type WarehouseShelfValue } from '@/components/inventory/WarehouseShelfTreeSelect'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { DateTextInput } from '@/components/ui/DateTextInput'
import { TableCell, TableRow } from '@/components/ui/table'
import { Trash2, Search } from 'lucide-react'
import { formatNumber, formatUom } from '@/lib/utils'
import type { DocumentFormData, DocumentLine } from '../types'
import type { InputRefs } from '../hooks/useDocumentForm'
import { BatchNumberSelect } from './BatchNumberSelect'

export interface LineRowProps {
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
  onRemoveLine: (lineId: string) => void
  setFormData: React.Dispatch<React.SetStateAction<DocumentFormData>>
}

export function LineRow({
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
  onRemoveLine,
  setFormData,
}: LineRowProps) {
  const lineId = line.id || `temp-${index}`
  if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}

  const qtyDefault = String(line.qty || '')
  const unitPriceDefault = String(line.unit_price || '')
  const expiryDateDefault = String(line.expiry_date || '')
  const batchNoDefault = String(line.batch_no || '')

  const lineBatchChange = (batchNo: string, expiryDate?: string, sourceIacuc?: string) =>
    onBatchChange(lineId, batchNo, expiryDate, sourceIacuc)

  const updateStorageLocation = (lineId: string, field: keyof DocumentLine, id: string) => {
    setFormData((prev) => ({
      ...prev,
      lines: prev.lines.map((l) => l.id === lineId ? { ...l, [field]: id } : l),
    }))
  }

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
          defaultValue={qtyDefault}
          ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].qty = el } }}
          className="text-right"
          min="0"
          onChange={() => { if (showPriceColumns) onUpdateLineAmount(lineId) }}
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
              defaultValue={unitPriceDefault}
              ref={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].unit_price = el } }}
              className="text-right"
              min="0"
              step="0.01"
              onChange={() => onUpdateLineAmount(lineId)}
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
                onBatchChange(lineId, batchNoDefault, expiryDateDefault)
                updateStorageLocation(lineId, 'storage_location_from_id', id)
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
                updateStorageLocation(lineId, 'storage_location_to_id', id)
              }}
              selectLevel="shelf"
              allowAll={false}
              className="w-full text-xs"
              placeholder="選擇目標儲位"
            />
          </TableCell>
        </>
      ) : needsShelf ? (
        <TableCell>
          <WarehouseShelfTreeSelect
            value={line.storage_location_id ? `loc:${line.storage_location_id}` : ''}
            onValueChange={(v: WarehouseShelfValue) => {
              const id = v.startsWith('loc:') ? v.slice(4) : ''
              updateStorageLocation(lineId, 'storage_location_id', id)
            }}
            selectLevel="shelf"
            allowAll={false}
            className="w-full text-xs"
            placeholder="選擇儲位"
          />
        </TableCell>
      ) : null}
      <TableCell>
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
            ref={(el) => { if (el && inputRefs.current[lineId]) inputRefs.current[lineId].expiry_date = el }}
            onBlur={() => onLineBlur(lineId)}
          />
        )}
      </TableCell>
      <TableCell>
        <BatchNumberSelect
          productId={line.product_id}
          warehouseId={warehouseId}
          batchNo={batchNoDefault}
          docType={docType}
          onBatchChange={lineBatchChange}
          onBlur={() => onLineBlur(lineId)}
          inputRef={(el) => { if (el) { if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}; inputRefs.current[lineId].batch_no = el } }}
        />
      </TableCell>
      <TableCell>
        <Button variant="ghost" size="icon" onClick={() => onRemoveLine(lineId)} className="text-red-500 hover:text-red-700">
          <Trash2 className="h-4 w-4" />
        </Button>
      </TableCell>
    </TableRow>
  )
}
