import { useTranslation } from 'react-i18next'

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

    const headers = [
      t('erpMaster.partners.table.type'),
      t('erpMaster.common.code'),
      t('erpMaster.common.name'),
      t('erpMaster.partners.supplierCategoryHeader'),
      t('erpMaster.partners.customerCategoryLabel'),
      t('erpMaster.partners.taxId'),
      t('erpMaster.partners.phone'),
      t('common.email'),
      t('erpMaster.partners.address'),
      t('erpMaster.common.status'),
    ]
    const rows = partners.map(p => {
      const ext = p as Partner & { supplier_category?: string }
      return [
        formatPartnerType(p.partner_type),
        p.code,
        p.name,
        formatSupplierCategory(ext.supplier_category),
        formatCustomerCategory(p.customer_category),
        p.tax_id || '',
        p.phone || '',
        p.email || '',
        p.address || '',
        p.is_active ? t('erpMaster.common.active') : t('erpMaster.common.inactive'),
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
