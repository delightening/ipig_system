import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Loader2, Download, FileText } from 'lucide-react'

interface AuditLogExportBarProps {
  isExporting: boolean
  totalCount: number | undefined
  onExportCSV: () => void
  onExportPDF: () => void
}

export function AuditLogExportBar({
  isExporting,
  totalCount,
  onExportCSV,
  onExportPDF,
}: AuditLogExportBarProps) {
  const { t } = useTranslation()

  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size="sm"
        disabled={isExporting}
        onClick={onExportCSV}
      >
        {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
        {t('admin.auditLogExportBar.exportCsv')}
      </Button>
      <Button
        variant="outline"
        size="sm"
        disabled={isExporting}
        onClick={onExportPDF}
      >
        {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileText className="h-4 w-4 mr-1" />}
        {t('admin.auditLogExportBar.exportPdf')}
      </Button>
      {totalCount !== undefined && (
        <span className="text-sm text-muted-foreground ml-auto">
          {t('admin.auditLogExportBar.totalCount', { count: totalCount })}
        </span>
      )}
    </div>
  )
}
