import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import {
  Loader2,
  Upload,
  Download,
  FileSpreadsheet,
  AlertCircle,
  CheckCircle2,
} from 'lucide-react'

interface WarehouseImportErrorDetail {
  row: number
  code?: string
  error: string
}

interface WarehouseImportResult {
  success_count: number
  error_count: number
  errors?: WarehouseImportErrorDetail[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function WarehouseImportDialog({ open, onOpenChange }: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [file, setFile] = useState<File | null>(null)
  const [result, setResult] = useState<WarehouseImportResult | null>(null)

  const importMutation = useMutation({
    mutationFn: async (f: File) => {
      const formData = new FormData()
      formData.append('file', f)

      const res = await api.post<WarehouseImportResult>('/warehouses/import', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      })
      return res.data
    },
    onSuccess: (data) => {
      setResult(data)
      queryClient.invalidateQueries({ queryKey: ['warehouses'] })
      // 選單/佈局頁的倉庫清單用 ['all-warehouses']（見 WarehouseActionHeader）；
      // 缺這個 → 匯入的新倉庫不會出現在選單，需手動重整。
      queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
      if (data.error_count === 0) {
        toast({
          title: t('erpDocs.warehouse.import.successTitle'),
          description: t('erpDocs.warehouse.import.successDesc', { count: data.success_count }),
        })
      } else {
        toast({
          title: t('erpDocs.warehouse.import.partialTitle'),
          description: t('erpDocs.warehouse.import.partialDesc', { success: data.success_count, failed: data.error_count }),
          variant: 'destructive',
        })
      }
    },
    onError: (error: unknown) => {
      toast({
        title: t('erpDocs.warehouse.import.failedTitle'),
        description: getApiErrorMessage(error, t('common.unknown_error')),
        variant: 'destructive',
      })
    },
  })

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (selectedFiles && selectedFiles.length > 0) {
      setFile(selectedFiles[0])
    }
  }

  const handleImport = () => {
    if (!file) {
      toast({ title: t('common.error'), description: t('erpDocs.warehouse.import.selectFileFirst'), variant: 'destructive' })
      return
    }
    importMutation.mutate(file)
  }

  const handleClose = () => {
    setFile(null)
    setResult(null)
    onOpenChange(false)
  }

  const downloadTemplateMutation = useMutation({
    mutationFn: async () => {
      const response = await api.get('/warehouses/import/template', {
        responseType: 'blob',
      })

      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url

      const contentDisposition = response.headers['content-disposition']
      let filename = 'warehouse_import_template.xlsx'
      if (contentDisposition) {
        const filenameMatch = contentDisposition.match(/filename="(.+)"/)
        if (filenameMatch) {
          filename = filenameMatch[1]
        }
      }
      link.setAttribute('download', filename)
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
    },
    onSuccess: () => {
      toast({
        title: t('erpDocs.warehouse.import.templateSuccessTitle'),
        description: t('erpDocs.warehouse.import.templateSuccessDesc'),
      })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.downloadFailed'),
        description: getApiErrorMessage(error, t('erpDocs.warehouse.import.templateFailedDesc')),
        variant: 'destructive',
      })
    },
  })

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            {t('erpDocs.warehouse.importWarehouses')}
          </DialogTitle>
          <DialogDescription>
            {t('erpDocs.warehouse.import.dialogDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Template Download */}
          <div className="flex items-center justify-between p-3 bg-status-info-bg rounded-lg">
            <div className="flex items-center gap-2">
              <FileSpreadsheet className="h-5 w-5 text-status-info-text" />
              <span className="text-sm text-status-info-text">{t('erpDocs.warehouse.import.downloadTemplateHint')}</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="border-primary text-status-info-text hover:bg-status-info-bg"
              onClick={() => downloadTemplateMutation.mutate()}
              disabled={downloadTemplateMutation.isPending}
            >
              <Download className="h-4 w-4 mr-1" />
              {t('erpDocs.warehouse.import.downloadTemplate')}
            </Button>
          </div>

          {/* File Upload */}
          {!result && (
            <label className="block space-y-2">
              <span className="block text-sm font-medium leading-none">{t('erpDocs.warehouse.import.selectFile')}</span>
              <input
                type="file"
                accept=".xlsx,.xls,.csv"
                onChange={handleFileInputChange}
                className="block w-full text-sm text-muted-foreground
                  file:mr-4 file:py-2 file:px-4
                  file:rounded-lg file:border-0
                  file:text-sm file:font-semibold
                  file:bg-status-purple-bg file:text-status-purple-text
                  hover:file:bg-status-purple-bg
                  file:cursor-pointer"
              />
              {file && (
                <div className="mt-2 p-2 bg-muted rounded-lg">
                  <p className="text-sm font-medium">{file.name}</p>
                  <p className="text-xs text-muted-foreground">
                    {(file.size / 1024).toFixed(1)} KB
                  </p>
                </div>
              )}
            </label>
          )}

          {/* Import Result */}
          {result && (
            <div className="space-y-4">
              <div className="flex items-center gap-4 p-4 bg-muted rounded-lg">
                <div className="flex-1">
                  <div className="flex items-center gap-2 text-status-success-text">
                    <CheckCircle2 className="h-5 w-5" />
                    <span className="font-medium">{t('erpDocs.warehouse.import.resultSuccess')}</span>
                  </div>
                  <p className="text-2xl font-bold text-status-success-text mt-1">
                    {t('erpDocs.warehouse.import.recordCount', { count: result.success_count })}
                  </p>
                </div>
                {result.error_count > 0 && (
                  <div className="flex-1 border-l pl-4">
                    <div className="flex items-center gap-2 text-status-error-text">
                      <AlertCircle className="h-5 w-5" />
                      <span className="font-medium">{t('erpDocs.warehouse.import.failedTitle')}</span>
                    </div>
                    <p className="text-2xl font-bold text-status-error-text mt-1">
                      {t('erpDocs.warehouse.import.recordCount', { count: result.error_count })}
                    </p>
                  </div>
                )}
              </div>

              {/* Error Details */}
              {result.errors && result.errors.length > 0 && (
                <div className="space-y-2">
                  <Label className="text-status-error-text">{t('erpDocs.warehouse.import.errorDetails')}</Label>
                  <div className="max-h-40 overflow-y-auto border rounded-lg">
                    <table className="w-full text-sm">
                      <thead className="bg-muted sticky top-0">
                        <tr>
                          <th className="px-3 py-2 text-left font-medium">{t('erpDocs.warehouse.import.row')}</th>
                          <th className="px-3 py-2 text-left font-medium">{t('erpDocs.shared.code')}</th>
                          <th className="px-3 py-2 text-left font-medium">{t('erpDocs.warehouse.import.errorMessage')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.errors.map((err, i) => (
                          <tr key={`err-${err.row}-${i}`} className="border-t">
                            <td className="px-3 py-2">{err.row}</td>
                            <td className="px-3 py-2 font-mono">{err.code || '-'}</td>
                            <td className="px-3 py-2 text-status-error-text">{err.error}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Instructions */}
          {!result && (
            <div className="text-sm text-muted-foreground space-y-1">
              <p className="font-medium">{t('erpDocs.warehouse.import.notes')}</p>
              <ul className="list-disc list-inside space-y-0.5">
                <li>{t('erpDocs.warehouse.import.noteName')}</li>
                <li>{t('erpDocs.warehouse.import.noteCode')}</li>
                <li>{t('erpDocs.warehouse.import.noteCsv')}</li>
              </ul>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {result ? t('common.closeDialog') : t('common.cancel')}
          </Button>
          {!result && (
            <Button
              onClick={handleImport}
              disabled={importMutation.isPending || !file}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {importMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('erpDocs.warehouse.import.startImport')}
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button onClick={handleClose} className="bg-status-success-solid hover:bg-green-700">
              {t('erpDocs.warehouse.import.done')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
