import React, { useState } from 'react'
import { Product, DocType, PoReceiptStatus } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Plus } from 'lucide-react'
import type { DocumentFormData, DocumentLine } from '../types'
import type { InputRefs } from '../hooks/useDocumentForm'
import { LineRow } from './LineRow'
import { ProductSearchDialog, type ProductSelectExtraData } from './ProductSearchDialog'

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
  setFormData: React.Dispatch<React.SetStateAction<DocumentFormData>>
  needsShelf: boolean
  poReceiptStatus?: PoReceiptStatus
  categoryCode?: string
  setCategoryCode: (code: string | undefined) => void
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
  setFormData,
  needsShelf,
  poReceiptStatus,
  categoryCode,
  setCategoryCode,
}: DocumentLineEditorProps) {
  const showPriceColumns = ['PO', 'GRN', 'DO'].includes(formData.doc_type)
  const [activeLineId, setActiveLineId] = useState<string | null>(null)
  const isPoLinkedGrn = formData.doc_type === 'GRN' && !!formData.source_doc_id
  const targetWarehouseId = formData.doc_type === 'TR' ? formData.warehouse_from_id : formData.warehouse_id

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
            }
          }
          return newLine
        })
      }))
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
                ) : needsShelf ? (
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
                    colSpan={showPriceColumns ? (needsShelf ? 10 : 9) : (needsShelf ? 8 : 7)}
                    className="text-center py-8"
                  >
                    <p className="text-muted-foreground">尚無明細，請點擊「新增明細」</p>
                  </TableCell>
                </TableRow>
              ) : (
                formData.lines.map((line, index) => (
                  <LineRow
                    key={line.id || `temp-${index}`}
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
                    onRemoveLine={removeLine}
                    setFormData={setFormData}
                  />
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <ProductSearchDialog
        open={productSearchOpen}
        onOpenChange={setProductSearchOpen}
        docType={formData.doc_type}
        targetWarehouseId={targetWarehouseId}
        isPoLinkedGrn={isPoLinkedGrn}
        poReceiptStatus={poReceiptStatus}
        productSearch={productSearch}
        setProductSearch={setProductSearch}
        categoryCode={categoryCode}
        setCategoryCode={setCategoryCode}
        products={products}
        onSelect={handleSelectProduct}
      />
    </>
  )
}
