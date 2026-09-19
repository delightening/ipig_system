import { useTranslation } from 'react-i18next'

import i18n from '@/lib/i18n'
import { Partner } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import {
  formatPartnerType,
  formatSupplierCategory,
  formatCustomerCategory,
} from '../constants'

export function usePartnerExport(partners: Partner[] | undefined) {
  const { t } = useTranslation()

  const handleExportCSV = () => {
    if (!partners || partners.length === 0) {
      toast({
        title: t('erpMaster.products.toast.nothingToExport'),
        description: t('erpMaster.partners.toast.nothingToExportHint'),
        variant: 'destructive',
      })
      return
    }

    // 內部匯出檔固定中文（使用者裁定 2026-09-19）
    const tZh = i18n.getFixedT('zh-TW')
    const headers = [
      tZh('erpMaster.partners.table.type'),
      tZh('erpMaster.common.code'),
      tZh('erpMaster.common.name'),
      tZh('erpMaster.partners.supplierCategoryHeader'),
      tZh('erpMaster.partners.customerCategoryLabel'),
      tZh('erpMaster.partners.taxId'),
      tZh('erpMaster.partners.phone'),
      tZh('common.email'),
      tZh('erpMaster.partners.address'),
      tZh('erpMaster.common.status'),
    ]
    const rows = partners.map(p => {
      const ext = p as Partner & { supplier_category?: string }
      return [
        formatPartnerType(p.partner_type, 'zh-TW'),
        p.code,
        p.name,
        formatSupplierCategory(ext.supplier_category, 'zh-TW'),
        formatCustomerCategory(p.customer_category, 'zh-TW'),
        p.tax_id || '',
        p.phone || '',
        p.email || '',
        p.address || '',
        p.is_active ? tZh('erpMaster.common.active') : tZh('erpMaster.common.inactive'),
      ]
    })

    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
      .join('\n')

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `partners_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
    URL.revokeObjectURL(link.href)
    toast({
      title: t('common.exportSuccess'),
      description: t('erpMaster.partners.toast.exported', { count: partners.length }),
    })
  }

  return { handleExportCSV }
}
