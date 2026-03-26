import { useRef, useState } from 'react'
import { Download, Upload, Loader2, AlertCircle } from 'lucide-react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'

interface DataExportImportCardProps {
  canExport: boolean
  canImport: boolean
}

interface ImportResult {
  errors: string[]
  skipped_details: { table: string; reason: string; count?: number }[]
}

export function DataExportImportCard({ canExport, canImport }: DataExportImportCardProps) {
  const queryClient = useQueryClient()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const [includeAudit, setIncludeAudit] = useState(false)
  const [exportAsZip, setExportAsZip] = useState(false)
  const [importResultOpen, setImportResultOpen] = useState(false)
  const [lastImportResult, setLastImportResult] = useState<ImportResult | null>(null)

  const exportMutation = useMutation({
    mutationFn: async () => {
      const res = await api.get<Blob>('/admin/data-export', {
        params: { include_audit: includeAudit, format: exportAsZip ? 'zip' : 'json' },
        responseType: 'blob',
      })
      const blob = new Blob([res.data], {
        type: exportAsZip ? 'application/zip' : 'application/json;charset=utf-8',
      })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      const cd = res.headers['content-disposition']
      const match = cd && typeof cd === 'string' && cd.match(/filename="?([^"]+)"?/)
      a.download = match
        ? match[1]
        : `ipig_export_${new Date().toISOString().slice(0, 19).replace(/[-:]/g, '').replace('T', '_')}.${exportAsZip ? 'zip' : 'json'}`
      a.click()
      URL.revokeObjectURL(url)
    },
    onSuccess: () => {
      toast({ title: '匯出成功', description: '全庫資料已下載' })
    },
    onError: (err) => {
      toast({ title: '匯出失敗', description: getErrorMessage(err) || '請稍後再試', variant: 'destructive' })
    },
  })

  const handleFullExport = () => {
    if (!canExport) return
    exportMutation.mutate()
  }

  const importMutation = useMutation({
    mutationFn: async (file: File) => {
      const fd = new FormData()
      fd.append('file', file)
      const res = await api.post<{
        tables_processed: number
        rows_inserted: number
        rows_skipped: number
        errors: string[]
        skipped_details: { table: string; reason: string; count?: number }[]
      }>('/admin/data-import', fd)
      return res.data
    },
    onSuccess: async (d) => {
      const msg = d.errors.length > 0
        ? `${d.tables_processed} 表處理，${d.rows_inserted} 筆新增，${d.rows_skipped} 筆略過；${d.errors.length} 個錯誤`
        : `${d.tables_processed} 表處理，${d.rows_inserted} 筆新增，${d.rows_skipped} 筆略過`
      toast({ title: '匯入完成', description: msg })

      const hasDetails = d.errors.length > 0 || (d.skipped_details?.length ?? 0) > 0
      if (hasDetails) {
        setLastImportResult({ errors: d.errors, skipped_details: d.skipped_details ?? [] })
        setImportResultOpen(true)
      }

      queryClient.invalidateQueries()
      await queryClient.refetchQueries({ queryKey: ['system-settings'] })
      await queryClient.refetchQueries({ queryKey: ['notification-settings'] })
      await queryClient.refetchQueries({ queryKey: ['warehouses-list'] })
      if (fileInputRef.current) fileInputRef.current.value = ''
    },
    onError: (err) => {
      toast({ title: '匯入失敗', description: getErrorMessage(err) || '請稍後再試', variant: 'destructive' })
    },
    onSettled: () => {
      if (fileInputRef.current) fileInputRef.current.value = ''
    },
  })

  const handleFullImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!canImport || !e.target.files?.[0]) return
    const file = e.target.files[0]
    if (!file.name.endsWith('.json') && !file.name.endsWith('.zip')) {
      toast({ title: '請選擇 .json 或 .zip 檔', variant: 'destructive' })
      e.target.value = ''
      return
    }
    importMutation.mutate(file)
  }

  return (
    <>
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            全庫資料匯出 / 匯入
          </CardTitle>
          <CardDescription>
            IDXF JSON 格式，含 AUP、動物、倉庫進銷存、使用者、訓練紀錄等，可在不同 migration 版本間讀取
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {canExport && (
            <>
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Checkbox id="includeAudit" checked={includeAudit} onCheckedChange={(v) => setIncludeAudit(!!v)} />
                  <Label htmlFor="includeAudit" className="cursor-pointer text-sm">
                    包含稽核大表（user_activity_logs、login_events）
                  </Label>
                </div>
                <div className="flex items-center space-x-2">
                  <Checkbox id="exportAsZip" checked={exportAsZip} onCheckedChange={(v) => setExportAsZip(!!v)} />
                  <Label htmlFor="exportAsZip" className="cursor-pointer text-sm">
                    輸出為 Zip 分包（大表以 NDJSON 儲存，建議資料量大時使用）
                  </Label>
                </div>
              </div>
              <Button variant="outline" onClick={handleFullExport} disabled={exportMutation.isPending}>
                {exportMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
                一鍵匯出全庫
              </Button>
            </>
          )}
          {canImport && (
            <div className="flex items-center gap-2 pt-2 border-t">
              <input
                ref={fileInputRef}
                type="file"
                accept=".json,.zip"
                className="hidden"
                aria-label="選擇 IDXF JSON 檔案"
                onChange={handleFullImport}
                disabled={importMutation.isPending}
              />
              <Button variant="outline" onClick={() => fileInputRef.current?.click()} disabled={importMutation.isPending}>
                {importMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Upload className="mr-2 h-4 w-4" />}
                上傳 IDXF 匯入
              </Button>
              <span className="text-xs text-muted-foreground">
                遇重複則取代；支援 JSON 或 Zip，最大 100 MB
              </span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Import result dialog */}
      <Dialog open={importResultOpen} onOpenChange={setImportResultOpen}>
        <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-amber-600">
              <AlertCircle className="h-5 w-5" />
              匯入結果詳情
            </DialogTitle>
            <DialogDescription>
              {lastImportResult?.errors.length ? `以下 ${lastImportResult.errors.length} 個資料表匯入時發生錯誤。` : ''}
              {lastImportResult?.skipped_details?.length ? `略過項目：${lastImportResult.skipped_details.length} 項。` : ''}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-y-auto flex-1 min-h-0 space-y-4">
            {lastImportResult?.errors.length ? (
              <div>
                <h4 className="font-medium text-destructive mb-2">錯誤</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.errors.map((err, i) => (
                    <li key={`err-${i}`} className="text-destructive/90 break-words">{i + 1}. {err}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {lastImportResult?.skipped_details?.length ? (
              <div>
                <h4 className="font-medium text-amber-600 mb-2">略過項目</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.skipped_details.map((s, i) => (
                    <li key={`skip-${s.table}-${i}`} className="break-words">
                      <span className="font-medium">{s.table}</span>：{s.reason}
                      {s.count != null ? `（${s.count} 筆）` : ''}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}
