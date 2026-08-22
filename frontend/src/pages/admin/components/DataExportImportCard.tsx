import { useRef, useState } from 'react'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useTranslation } from 'react-i18next'
import { Download, Upload, Loader2, AlertCircle, AlertTriangle } from 'lucide-react'
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
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { toast } from '@/components/ui/use-toast'
import api, { confirmPassword } from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'

interface DataExportImportCardProps {
  canExport: boolean
  canImport: boolean
}

interface ImportResult {
  errors: string[]
  skipped_details: { table: string; reason: string; count?: number }[]
}

export function DataExportImportCard({ canExport, canImport }: DataExportImportCardProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const fileInputRef = useRef<HTMLInputElement>(null)

  // R30-19：預設勾選含稽核軌跡，符合 GLP / 21 CFR §11.10(c) 對「準確完整紀錄副本」的要求。
  const [includeAudit, setIncludeAudit] = useState(true)
  const [exportAsZip, setExportAsZip] = useState(false)
  const [importResultOpen, setImportResultOpen] = useState(false)
  const [lastImportResult, setLastImportResult] = useState<ImportResult | null>(null)
  const [reauthOpen, setReauthOpen] = useState(false)
  // R30-19：使用者取消勾選 include_audit 時，匯出前先要求二次確認。
  const [auditWarningOpen, setAuditWarningOpen] = useState(false)

  const exportMutation = useMutation({
    mutationFn: async (reauthToken: string) => {
      const res = await api.get<Blob>('/admin/data-export', {
        params: { include_audit: includeAudit, format: exportAsZip ? 'zip' : 'json' },
        responseType: 'blob',
        headers: { 'X-Reauth-Token': reauthToken },
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
      toast({ title: t('common.exportSuccess'), description: t('admin.dataExportImportCard.exportDownloaded') })
    },
    onError: (err) => {
      toast({ title: t('common.exportFailed'), description: getErrorMessage(err) || t('admin.dataExportImportCard.tryAgainLater'), variant: 'destructive' })
    },
  })

  const handleFullExport = () => {
    if (!canExport) return
    // R30-19：未勾選含稽核軌跡時，先彈出警示 dialog 要求二次確認。
    if (!includeAudit) {
      setAuditWarningOpen(true)
      return
    }
    setReauthOpen(true)
  }

  const handleAuditWarningConfirm = () => {
    setAuditWarningOpen(false)
    setReauthOpen(true)
  }

  const handleReauthSubmit = async (password: string) => {
    const { reauth_token } = await confirmPassword(password)
    exportMutation.mutate(reauth_token)
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
        ? t('admin.dataExportImportCard.importSummaryWithErrors', {
            tables: d.tables_processed,
            inserted: d.rows_inserted,
            skipped: d.rows_skipped,
            errors: d.errors.length,
          })
        : t('admin.dataExportImportCard.importSummary', {
            tables: d.tables_processed,
            inserted: d.rows_inserted,
            skipped: d.rows_skipped,
          })
      toast({ title: t('admin.dataExportImportCard.importComplete'), description: msg })

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
      toast({ title: t('admin.dataExportImportCard.importFailed'), description: getErrorMessage(err) || t('admin.dataExportImportCard.tryAgainLater'), variant: 'destructive' })
    },
    onSettled: () => {
      if (fileInputRef.current) fileInputRef.current.value = ''
    },
  })

  const handleFullImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!canImport || !e.target.files?.[0]) return
    const file = e.target.files[0]
    if (!file.name.endsWith('.json') && !file.name.endsWith('.zip')) {
      toast({ title: t('admin.dataExportImportCard.invalidFileType'), variant: 'destructive' })
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
            {t('admin.dataExportImportCard.title')}
          </CardTitle>
          <CardDescription>
            {t('admin.dataExportImportCard.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {canExport && (
            <>
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="includeAudit"
                    checked={includeAudit}
                    onCheckedChange={(v) => setIncludeAudit(!!v)}
                  />
                  <Label
                    htmlFor="includeAudit"
                    className="cursor-pointer text-sm"
                    title={t('admin.dataExportImportCard.includeAuditHint')}
                  >
                    {t('admin.dataExportImportCard.includeAuditLabel')}
                    <span className="ml-1 text-xs text-status-warning-text">{t('admin.dataExportImportCard.recommendedKeep')}</span>
                  </Label>
                </div>
                <p className="pl-6 text-xs text-muted-foreground">
                  {t('admin.dataExportImportCard.includeAuditHint')}
                </p>
                {!includeAudit && (
                  <div
                    role="alert"
                    className="flex items-start gap-2 rounded border border-status-warning-text/40 bg-status-warning-text/10 p-2 text-xs text-status-warning-text"
                  >
                    <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
                    <span>{t('admin.dataExportImportCard.noAuditWarning')}</span>
                  </div>
                )}
                <div className="flex items-center space-x-2">
                  <Checkbox id="exportAsZip" checked={exportAsZip} onCheckedChange={(v) => setExportAsZip(!!v)} />
                  <Label htmlFor="exportAsZip" className="cursor-pointer text-sm">
                    {t('admin.dataExportImportCard.exportAsZipLabel')}
                  </Label>
                </div>
              </div>
              <Can permission={PERMISSIONS.ADMIN_DATA_EXPORT}>
                <Button variant="outline" onClick={handleFullExport} disabled={exportMutation.isPending}>
                  {exportMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
                  {t('admin.dataExportImportCard.exportButton')}
                </Button>
              </Can>
            </>
          )}
          {canImport && (
            <div className="flex items-center gap-2 pt-2 border-t">
              <input
                ref={fileInputRef}
                type="file"
                accept=".json,.zip"
                className="hidden"
                aria-label={t('admin.dataExportImportCard.fileInputAriaLabel')}
                onChange={handleFullImport}
                disabled={importMutation.isPending}
              />
              <Can permission={PERMISSIONS.ADMIN_DATA_IMPORT}>
                <Button variant="outline" onClick={() => fileInputRef.current?.click()} disabled={importMutation.isPending}>
                  {importMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Upload className="mr-2 h-4 w-4" />}
                  {t('admin.dataExportImportCard.importButton')}
                </Button>
              </Can>
              <span className="text-xs text-muted-foreground">
                {t('admin.dataExportImportCard.importHint')}
              </span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* R30-19：未勾選 include_audit 的二次確認 */}
      <AlertDialog open={auditWarningOpen} onOpenChange={setAuditWarningOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2 text-status-warning-text">
              <AlertTriangle className="h-5 w-5" />
              {t('admin.dataExportImportCard.auditWarningTitle')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('admin.dataExportImportCard.auditWarningDescription')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleAuditWarningConfirm}>{t('admin.dataExportImportCard.confirmContinue')}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* SEC-33: 敏感操作二級認證 */}
      <ConfirmPasswordModal
        open={reauthOpen}
        onOpenChange={setReauthOpen}
        title={t('admin.dataExportImportCard.reauthTitle')}
        description={t('admin.dataExportImportCard.reauthDescription')}
        onSubmit={handleReauthSubmit}
      />

      {/* Import result dialog */}
      <Dialog open={importResultOpen} onOpenChange={setImportResultOpen}>
        <DialogContent size="lg" className="max-h-[80vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-status-warning-text">
              <AlertCircle className="h-5 w-5" />
              {t('admin.dataExportImportCard.importResultTitle')}
            </DialogTitle>
            <DialogDescription>
              {[
                lastImportResult?.errors.length ? t('admin.dataExportImportCard.importResultErrorsDesc', { count: lastImportResult.errors.length }) : null,
                lastImportResult?.skipped_details?.length ? t('admin.dataExportImportCard.importResultSkippedDesc', { count: lastImportResult.skipped_details.length }) : null,
              ].filter(Boolean).join('；')}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-y-auto flex-1 min-h-0 space-y-4">
            {lastImportResult?.errors.length ? (
              <div>
                <h4 className="font-medium text-destructive mb-2">{t('common.error')}</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.errors.map((err, i) => (
                    <li key={`err-${i}`} className="text-destructive/90 break-words">{i + 1}. {err}</li>
                  ))}
                </ul>
              </div>
            ) : null}
            {lastImportResult?.skipped_details?.length ? (
              <div>
                <h4 className="font-medium text-status-warning-text mb-2">{t('admin.dataExportImportCard.skippedItems')}</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.skipped_details.map((s, i) => (
                    <li key={`skip-${s.table}-${i}`} className="break-words">
                      <span className="font-medium">{s.table}</span>
                      {t('admin.dataExportImportCard.skippedReasonSeparator')}{s.reason}
                      {s.count != null ? t('admin.dataExportImportCard.skippedCount', { count: s.count }) : ''}
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
