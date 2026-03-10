import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { Check, PowerOff, Download, Tags } from 'lucide-react'
import { UOM_MAP } from '@/lib/utils'

import type { ExtendedProduct } from './productTypes'

interface ProductBatchActionsProps {
  selectionSize: number
  selectedIds: ReadonlySet<string>
  products: ExtendedProduct[]
  onBatchDeactivate: () => void
  onClearSelection: () => void
}

/** 將產品陣列匯出為 CSV */
function exportProductsCsv(products: ExtendedProduct[], filenamePrefix: string) {
  if (products.length === 0) return

  const headers = ['SKU', '名稱', '規格', '品類', '子類', '單位', '安全庫存', '追蹤批號', '追蹤效期', '狀態']
  const rows = products.map(p => [
    p.sku,
    p.name,
    p.spec || '',
    p.category_code || '',
    p.subcategory_code || '',
    UOM_MAP[p.base_uom] || p.base_uom,
    p.safety_stock?.toString() ?? '',
    p.track_batch ? '是' : '否',
    p.track_expiry ? '是' : '否',
    p.is_active ? '啟用' : '停用',
  ])

  const csvContent = [headers, ...rows]
    .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
    .join('\n')

  const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = `${filenamePrefix}_${new Date().toISOString().split('T')[0]}.csv`
  link.click()
  URL.revokeObjectURL(link.href)

  toast({ title: '匯出成功', description: `已匯出 ${products.length} 筆產品` })
}

export { exportProductsCsv }

export function ProductBatchActions({
  selectionSize,
  selectedIds,
  products,
  onBatchDeactivate,
  onClearSelection,
}: ProductBatchActionsProps) {
  if (selectionSize === 0) return null

  const handleBatchExport = () => {
    const toExport = products.filter(p => selectedIds.has(p.id))
    exportProductsCsv(toExport, 'products_selected')
  }

  return (
    <div className="flex items-center gap-3 p-3 bg-primary/5 border border-primary/20 rounded-lg animate-fade-in">
      <div className="flex items-center gap-2">
        <Check className="h-4 w-4 text-primary" />
        <span className="text-sm font-medium">
          已選擇 {selectionSize} 個產品
        </span>
      </div>
      <div className="flex-1" />
      <Button variant="outline" size="sm" onClick={onBatchDeactivate}>
        <PowerOff className="mr-2 h-4 w-4" />
        批次停用
      </Button>
      <Button variant="outline" size="sm" onClick={handleBatchExport}>
        <Download className="mr-2 h-4 w-4" />
        批次匯出
      </Button>
      <Button variant="outline" size="sm" disabled>
        <Tags className="mr-2 h-4 w-4" />
        批次設定標籤
      </Button>
      <Button variant="ghost" size="sm" onClick={onClearSelection}>
        取消選擇
      </Button>
    </div>
  )
}
