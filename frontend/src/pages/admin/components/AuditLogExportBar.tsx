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
  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size="sm"
        disabled={isExporting}
        onClick={onExportCSV}
      >
        {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
        匯出 CSV
      </Button>
      <Button
        variant="outline"
        size="sm"
        disabled={isExporting}
        onClick={onExportPDF}
      >
        {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileText className="h-4 w-4 mr-1" />}
        匯出 PDF
      </Button>
      {totalCount !== undefined && (
        <span className="text-sm text-muted-foreground ml-auto">
          共 {totalCount} 筆紀錄
        </span>
      )}
    </div>
  )
}
