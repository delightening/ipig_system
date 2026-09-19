import { useTranslation } from 'react-i18next'

import i18n from '@/lib/i18n'
import { formatUom } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { Check, PowerOff, Download, Tags } from 'lucide-react'

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

  const headers = [
    'SKU',
    i18n.t('erpMaster.common.name'),
    i18n.t('erpMaster.common.spec'),
    i18n.t('erpMaster.products.category'),
    i18n.t('erpMaster.products.subcategory'),
    i18n.t('erpMaster.common.unit'),
    i18n.t('erpMaster.products.safetyStock'),
    i18n.t('erpMaster.products.trackBatch'),
    i18n.t('erpMaster.products.trackExpiry'),
    i18n.t('erpMaster.common.status'),
  ]
  const rows = products.map(p => [
    p.sku,
    p.name,
    p.spec || '',
    p.category_code || '',
    p.subcategory_code || '',
    formatUom(p.base_uom),
    p.safety_stock?.toString() ?? '',
    p.track_batch ? i18n.t('common.yes') : i18n.t('common.no'),
    p.track_expiry ? i18n.t('common.yes') : i18n.t('common.no'),
    p.is_active ? i18n.t('erpMaster.common.active') : i18n.t('erpMaster.common.inactive'),
  ])

  const csvContent = [headers, ...rows]
    .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
    .join('\n')

  const blob = new Blob(['﻿' + csvContent], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = `${filenamePrefix}_${new Date().toISOString().split('T')[0]}.csv`
  link.click()
  URL.revokeObjectURL(link.href)

  toast({
    title: i18n.t('common.exportSuccess'),
    description: i18n.t('erpMaster.products.exportedCount', { count: products.length }),
  })
}

// eslint-disable-next-line react-refresh/only-export-components
export { exportProductsCsv }

export function ProductBatchActions({
  selectionSize,
  selectedIds,
  products,
  onBatchDeactivate,
  onClearSelection,
}: ProductBatchActionsProps) {
  const { t } = useTranslation()

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
          {t('erpMaster.products.batch.selected', { count: selectionSize })}
        </span>
      </div>
      <div className="flex-1" />
      <Button variant="outline" size="sm" onClick={onBatchDeactivate}>
        <PowerOff className="mr-2 h-4 w-4" />
        {t('erpMaster.products.batch.deactivate')}
      </Button>
      <Button variant="outline" size="sm" onClick={handleBatchExport}>
        <Download className="mr-2 h-4 w-4" />
        {t('erpMaster.products.batch.export')}
      </Button>
      <Button variant="outline" size="sm" disabled>
        <Tags className="mr-2 h-4 w-4" />
        {t('erpMaster.products.batch.setTags')}
      </Button>
      <Button variant="ghost" size="sm" onClick={onClearSelection}>
        {t('erpMaster.products.batch.clearSelection')}
      </Button>
    </div>
  )
}
