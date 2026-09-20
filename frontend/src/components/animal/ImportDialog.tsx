import { useCallback, useRef, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { FileInfo } from '@/components/ui/file-upload'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { getApiErrorMessage } from '@/lib/apiError'
import { ManualWeightEntry, type ManualWeightEntryHandle } from './ManualWeightEntry'
import {
  Loader2,
  Upload,
  Download,
  FileSpreadsheet,
  AlertCircle,
  CheckCircle2,
  ChevronDown,
} from 'lucide-react'

/** 匯入體重 dialog 用的手風琴區塊：open 只切換可見性，children 全程掛載，
 *  避免收合時卸載 ManualWeightEntry 導致已填的列被清空。 */
function AccordionSection({
  title, open, onToggle, bodyClassName, children,
}: {
  title: string
  open: boolean
  onToggle: () => void
  bodyClassName?: string
  children: React.ReactNode
}) {
  return (
    <div className={cn('flex flex-col rounded-lg border', open ? 'min-h-0 flex-1' : 'flex-none')}>
      <button
        type="button"
        onClick={onToggle}
        className="flex flex-none items-center justify-between rounded-t-lg bg-muted px-3.5 py-2.5 text-left"
      >
        <span className="text-sm font-semibold">{title}</span>
        <ChevronDown className={cn('h-4 w-4 text-muted-foreground transition-transform', open && 'rotate-180')} />
      </button>
      <div className={open ? cn('min-h-0 flex-1 p-3.5', bodyClassName ?? 'overflow-y-auto') : 'hidden'}>
        {children}
      </div>
    </div>
  )
}

type ImportType = 'basic' | 'weight'

interface ImportErrorDetail {
  row: number
  ear_tag?: string
  error: string
}

interface ImportResult {
  success_count: number
  error_count: number
  errors?: ImportErrorDetail[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  type: ImportType
}

const importTypeConfig: Record<ImportType, { titleKey: string; descriptionKey: string; templateEndpoint: string }> = {
  basic: {
    titleKey: 'animalActions.importExport.import.basicTitle',
    descriptionKey: 'animalActions.importExport.import.basicDescription',
    templateEndpoint: '/animals/import/template/basic',
  },
  weight: {
    titleKey: 'animalActions.importExport.import.weightTitle',
    descriptionKey: 'animalActions.importExport.import.weightDescription',
    templateEndpoint: '/animals/import/template/weight',
  },
}

export function ImportDialog({ open, onOpenChange, type }: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [files, setFiles] = useState<FileInfo[]>([])
  const [fileObjects, setFileObjects] = useState<File[]>([])
  const [result, setResult] = useState<ImportResult | null>(null)
  const config = importTypeConfig[type]

  // 手動逐筆登錄區（type='weight'）狀態，用以驅動底部統一按鈕
  const manualRef = useRef<ManualWeightEntryHandle>(null)
  const [manualStatus, setManualStatus] = useState({ ready: false, hasInput: false, pending: false })
  const handleManualStatus = useCallback(
    (s: { ready: boolean; hasInput: boolean; pending: boolean }) => setManualStatus(s),
    [],
  )
  const { dialogState, confirm } = useConfirmDialog()

  // 體重匯入 dialog 的手風琴預設狀態：手動逐筆展開、批量匯入收合
  const [manualOpen, setManualOpen] = useState(true)
  const [batchOpen, setBatchOpen] = useState(false)

  const importMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)

      const endpoint = type === 'basic'
        ? '/animals/import/basic'
        : '/animals/import/weights'

      const res = await api.post<ImportResult>(endpoint, formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      })
      return res.data
    },
    onSuccess: (data) => {
      setResult(data)
      if (data.error_count === 0) {
        queryClient.invalidateQueries({ queryKey: ['animals'] })
        queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
        queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
        toast({
          title: t('animalActions.importExport.import.successTitle'),
          description: t('animalActions.importExport.import.successDescription', { count: data.success_count })
        })
      } else {
        toast({
          title: t('animalActions.importExport.import.partialTitle'),
          description: t('animalActions.importExport.import.partialDescription', { success: data.success_count, failed: data.error_count }),
          variant: 'destructive',
        })
      }
    },
    onError: (error: unknown) => {
      toast({
        title: t('animalActions.importExport.import.failedTitle'),
        description: getApiErrorMessage(error, t('animalActions.importExport.unknownError')),
        variant: 'destructive',
      })
    },
  })

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (selectedFiles && selectedFiles.length > 0) {
      const file = selectedFiles[0]
      setFileObjects([file])
      // 同時更新 FileInfo 列表
      const fileInfo: FileInfo = {
        id: `local-${Date.now()}`,
        file_name: file.name,
        file_path: '',
        file_size: file.size,
        file_type: file.type,
      }
      setFiles([fileInfo])
    }
  }

  const handleImport = () => {
    if (fileObjects.length === 0) {
      toast({ title: t('common.error'), description: t('animalActions.importExport.import.selectFileFirst'), variant: 'destructive' })
      return
    }

    importMutation.mutate(fileObjects[0])
  }

  // 底部統一按鈕：依「檔案 / 手動」填了什麼自動分流。
  // 兩者同時填寫時跳出警示確認，並以「檔案匯入」為準（手動列不送）。
  const hasFile = fileObjects.length > 0
  const handleSubmit = async () => {
    if (hasFile && manualStatus.hasInput) {
      const ok = await confirm({
        title: t('animalActions.importExport.import.conflictTitle'),
        description: t('animalActions.importExport.import.conflictDescription'),
        confirmLabel: t('animalActions.importExport.import.start'),
      })
      if (ok) handleImport()
      return
    }
    if (hasFile) {
      handleImport()
      return
    }
    if (manualStatus.ready) manualRef.current?.submit()
  }

  const handleClose = () => {
    setFiles([])
    setFileObjects([])
    setResult(null)
    onOpenChange(false)
  }

  const downloadTemplateMutation = useMutation({
    mutationFn: async (format: 'xlsx' | 'csv') => {
      const endpoint = `${config.templateEndpoint}?format=${format}`
      const response = await api.get(endpoint, {
        responseType: 'blob',
      })

      // 創建下載連結
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url

      // 從 Content-Disposition header 提取檔名，或使用預設檔名
      const contentDisposition = response.headers['content-disposition']
      let filename = format === 'csv' ? 'template.csv' : 'template.xlsx'
      if (contentDisposition) {
        const filenameMatch = contentDisposition.match(/filename="(.+)"/)
        if (filenameMatch) {
          filename = filenameMatch[1]
        }
      } else {
        // 使用預設檔名
        filename = type === 'basic'
          ? (format === 'csv' ? 'animal_basic_import_template.csv' : 'animal_basic_import_template.xlsx')
          : (format === 'csv' ? 'animal_weight_import_template.csv' : 'animal_weight_import_template.xlsx')
      }

      link.setAttribute('download', filename)
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)
    },
    onSuccess: () => {
      toast({
        title: t('animalActions.importExport.import.downloadSuccessTitle'),
        description: t('animalActions.importExport.import.downloadSuccessDescription'),
      })
    },
    onError: (error: unknown) => {
      toast({
        title: t('animalActions.importExport.import.downloadFailedTitle'),
        description: getApiErrorMessage(error, t('animalActions.importExport.import.downloadFailedDescription')),
        variant: 'destructive',
      })
    },
  })

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent size={type === 'weight' ? 'xl' : 'md'}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            {t(config.titleKey)}
          </DialogTitle>
          <DialogDescription>{t(config.descriptionKey)}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Import Result（檔案匯入結果，手動/批量共用同一份摘要） */}
          {result && (
            <div className="space-y-4">
              <div className="flex items-center gap-4 p-4 bg-muted rounded-lg">
                <div className="flex-1">
                  <div className="flex items-center gap-2 text-status-success-text">
                    <CheckCircle2 className="h-5 w-5" />
                    <span className="font-medium">{t('animalActions.importExport.import.imported')}</span>
                  </div>
                  <p className="text-2xl font-bold text-status-success-text mt-1">
                    {t('animalActions.common.recordsCount', { count: result.success_count })}
                  </p>
                </div>
                {result.error_count > 0 && (
                  <div className="flex-1 border-l pl-4">
                    <div className="flex items-center gap-2 text-status-error-text">
                      <AlertCircle className="h-5 w-5" />
                      <span className="font-medium">{t('animalActions.importExport.import.failedTitle')}</span>
                    </div>
                    <p className="text-2xl font-bold text-status-error-text mt-1">
                      {t('animalActions.common.recordsCount', { count: result.error_count })}
                    </p>
                  </div>
                )}
              </div>

              {/* Error Details */}
              {result.errors && result.errors.length > 0 && (
                <div className="space-y-2">
                  <Label className="text-status-error-text">{t('animalActions.importExport.import.errorDetails')}</Label>
                  <div className="max-h-40 overflow-y-auto border rounded-lg">
                    <table className="w-full text-sm">
                      <thead className="bg-muted sticky top-0">
                        <tr>
                          <th className="px-3 py-2 text-left font-medium">{t('animalActions.importExport.import.colRow')}</th>
                          <th className="px-3 py-2 text-left font-medium">{t('animals.earTag')}</th>
                          <th className="px-3 py-2 text-left font-medium">{t('animalActions.importExport.import.colErrorMessage')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.errors.map((error, i) => (
                          <tr key={`err-${error.row}-${i}`} className="border-t">
                            <td className="px-3 py-2">{error.row}</td>
                            <td className="px-3 py-2 font-mono">{error.ear_tag || '-'}</td>
                            <td className="px-3 py-2 text-status-error-text">{error.error}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          )}

          {!result && type === 'basic' && (
            <>
              {/* Template Download */}
              <div className="flex items-center justify-between p-3 bg-status-info-bg rounded-lg">
                <div className="flex items-center gap-2">
                  <FileSpreadsheet className="h-5 w-5 text-status-info-text" />
                  <span className="text-sm text-status-info-text">{t('animalActions.importExport.import.downloadTemplateFile')}</span>
                </div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="border-primary text-status-info-text hover:bg-status-info-bg"
                    onClick={() => downloadTemplateMutation.mutate('csv')}
                    disabled={downloadTemplateMutation.isPending}
                  >
                    <Download className="h-4 w-4 mr-1" />
                    {t('animalActions.importExport.import.downloadTemplateCsv')}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="border-primary text-status-info-text hover:bg-status-info-bg"
                    onClick={() => downloadTemplateMutation.mutate('xlsx')}
                    disabled={downloadTemplateMutation.isPending}
                  >
                    <Download className="h-4 w-4 mr-1" />
                    {t('animalActions.importExport.import.downloadTemplateXlsx')}
                  </Button>
                </div>
              </div>

              {/* File Upload */}
              <label className="block space-y-2">
                <span className="block text-sm font-medium leading-none">{t('animalActions.importExport.import.selectFile')}</span>
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
                {files.length > 0 && (
                  <div className="mt-2 p-2 bg-muted rounded-lg">
                    <p className="text-sm font-medium">{files[0].file_name}</p>
                    <p className="text-xs text-muted-foreground">
                      {(files[0].file_size / 1024 / 1024).toFixed(2)} MB
                    </p>
                  </div>
                )}
              </label>

              {/* Instructions */}
              <div className="text-sm text-muted-foreground space-y-1">
                <p className="font-medium">{t('animalActions.importExport.import.notes')}</p>
                <ul className="list-disc list-inside space-y-0.5">
                  <li>{t('animalActions.importExport.import.basicNotes.earTagRequired')}</li>
                  <li>{t('animalActions.importExport.import.basicNotes.earTagRule')}</li>
                  <li>{t('animalActions.importExport.import.basicNotes.entryWeightRequired')}</li>
                  <li>{t('animalActions.importExport.import.basicNotes.breed')}</li>
                  <li>{t('animalActions.importExport.import.basicNotes.gender')}</li>
                  <li>{t('animalActions.importExport.import.dateFormat')}</li>
                </ul>
              </div>
            </>
          )}

          {/* 體重匯入：手動逐筆登錄（上，預設展開）／批量匯入（下，預設收合）手風琴。
              固定高度讓兩區展開/收合時 dialog 外框不跳動，收合區只留標題列，
              釋出的空間由展開中的區塊填滿。 */}
          {!result && type === 'weight' && (
            <div className="flex h-[28rem] flex-col gap-3">
              <AccordionSection
                title={t('animalActions.importExport.import.manualEntry')}
                open={manualOpen}
                onToggle={() => setManualOpen((v) => !v)}
                bodyClassName="overflow-hidden"
              >
                <ManualWeightEntry ref={manualRef} onStatusChange={handleManualStatus} />
              </AccordionSection>

              <AccordionSection
                title={t('animalActions.importExport.import.batchImport')}
                open={batchOpen}
                onToggle={() => setBatchOpen((v) => !v)}
              >
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-3 bg-status-info-bg rounded-lg">
                    <div className="flex items-center gap-2">
                      <FileSpreadsheet className="h-5 w-5 text-status-info-text" />
                      <span className="text-sm text-status-info-text">{t('animalActions.importExport.import.downloadTemplateFile')}</span>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        className="border-primary text-status-info-text hover:bg-status-info-bg"
                        onClick={() => downloadTemplateMutation.mutate('csv')}
                        disabled={downloadTemplateMutation.isPending}
                      >
                        <Download className="h-4 w-4 mr-1" />
                        {t('animalActions.importExport.import.downloadTemplateCsv')}
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        className="border-primary text-status-info-text hover:bg-status-info-bg"
                        onClick={() => downloadTemplateMutation.mutate('xlsx')}
                        disabled={downloadTemplateMutation.isPending}
                      >
                        <Download className="h-4 w-4 mr-1" />
                        {t('animalActions.importExport.import.downloadTemplateXlsx')}
                      </Button>
                    </div>
                  </div>

                  <label className="block space-y-2">
                    <span className="block text-sm font-medium leading-none">{t('animalActions.importExport.import.selectFile')}</span>
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
                    {files.length > 0 && (
                      <div className="mt-2 p-2 bg-muted rounded-lg">
                        <p className="text-sm font-medium">{files[0].file_name}</p>
                        <p className="text-xs text-muted-foreground">
                          {(files[0].file_size / 1024 / 1024).toFixed(2)} MB
                        </p>
                      </div>
                    )}
                  </label>

                  <div className="text-sm text-muted-foreground space-y-1">
                    <p className="font-medium">{t('animalActions.importExport.import.notes')}</p>
                    <ul className="list-disc list-inside space-y-0.5">
                      <li>{t('animalActions.importExport.import.weightNotes.earTagMustExist')}</li>
                      <li>{t('animalActions.importExport.import.weightNotes.measureDateFormat')}</li>
                      <li>{t('animalActions.importExport.import.weightNotes.weightUnit')}</li>
                    </ul>
                  </div>
                </div>
              </AccordionSection>
            </div>
          )}
        </div>

        <ConfirmDialog state={dialogState} />

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {result ? t('common.closeDialog') : t('common.cancel')}
          </Button>
          {!result && (
            <Button
              onClick={handleSubmit}
              disabled={
                importMutation.isPending ||
                manualStatus.pending ||
                (!hasFile && !manualStatus.ready)
              }
              className="bg-status-purple-solid hover:bg-status-purple-solid/90"
            >
              {(importMutation.isPending || manualStatus.pending) && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              {t('animalActions.importExport.import.start')}
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button
              onClick={handleClose}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {t('animalActions.common.done')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
