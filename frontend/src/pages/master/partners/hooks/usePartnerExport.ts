import { Partner } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import {
  formatPartnerType,
  formatSupplierCategory,
  formatCustomerCategory,
} from '../constants'

export function usePartnerExport(partners: Partner[] | undefined) {
  const handleExportCSV = () => {
    if (!partners || partners.length === 0) {
      toast({ title: '無資料可匯出', description: '請先新增夥伴', variant: 'destructive' })
      return
    }

    const headers = ['類型', '代碼', '名稱', '供應商類別', '客戶分類', '統編', '電話', 'Email', '地址', '狀態']
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
        p.is_active ? '啟用' : '停用',
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
    toast({ title: '匯出成功', description: `已匯出 ${partners.length} 筆夥伴` })
  }

  return { handleExportCSV }
}
