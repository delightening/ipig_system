import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery } from '@tanstack/react-query'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2, Download, FileText } from 'lucide-react'

// 匯出類型
type ExportType = 'single_animal' | 'batch_project'
// 暫時只開 PDF：後端 /export（非 -pdf）目前只回 JSON、不產生 xlsx，CSV 後端則根本沒有這個格式。
// Excel 匯出實作完成前不顯示另外兩個選項，避免下載到副檔名是 .xlsx、內容卻是 JSON 的壞檔。
type ExportFormat = 'pdf'
// 原本的 7 個「匯出內容」勾選項已移除：勾選狀態從未送到後端，後端一律匯出固定內容
// （基本資料／觀察／手術／體重／疫苗／犧牲，見 AnimalMedicalService::get_animal_medical_data），
// 「病理」更是後端根本沒有的區塊。保留只會讓使用者以為能挑內容。

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  type: ExportType
  animalId?: string
  earTag?: string
}

export function ExportDialog({ open, onOpenChange, type, animalId, earTag }: Props) {
  const { t } = useTranslation()
  const [format, setFormat] = useState<ExportFormat>('pdf')
  const [selectedProject, setSelectedProject] = useState<string>('')

  // 取得計畫列表（用於批次匯出）
  const { data: projects } = useQuery({
    queryKey: ['export-projects'],
    queryFn: async () => {
      const res = await api.get<{ iacuc_no: string; title: string; animal_count: number }[]>(
        '/animals/projects-summary'
      )
      return res.data
    },
    enabled: type === 'batch_project' && open,
    staleTime: 600_000,
  })

  const exportMutation = useMutation({
    mutationFn: async () => {
      const pdfSuffix = format === 'pdf' ? '-pdf' : ''
      const endpoint =
        type === 'single_animal'
          ? `/animals/${animalId}/export${pdfSuffix}`
          : `/projects/${selectedProject}/export${pdfSuffix}`

      const body = {
        format,
        export_type: 'medical_summary', // 主要對應病歷紀錄進程
        animal_id: animalId,
        iacuc_no: selectedProject,
      }

      const response = await api.post(endpoint, body, {
        responseType: 'blob',
        _silentError: true,
      })

      // 下載檔案
      const blob = new Blob([response.data], {
        type: format === 'pdf' ? 'application/pdf' : 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.style.display = 'none'

      // 檔名日期用台灣日期：toISOString() 是 UTC，台北 00:00–08:00 匯出會得到前一天
      const dateStr = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })
      const fileExt = format === 'pdf' ? 'pdf' : 'xlsx'
      const filename =
        type === 'single_animal'
          ? t('animalActions.importExport.export.fileNameSingle', { earTag, date: dateStr, ext: fileExt })
          : t('animalActions.importExport.export.fileNameBatch', { project: selectedProject, date: dateStr, ext: fileExt })

      a.download = filename
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)

      return response.data
    },
    onSuccess: () => {
      toast({ title: t('common.exportSuccess') })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.exportFailed'),
        description: getApiErrorMessage(error, t('common.exportFailed')),
        variant: 'destructive',
      })
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            {type === 'single_animal' ? t('animalActions.importExport.export.titleSingle') : t('animalActions.importExport.export.titleBatch')}
          </DialogTitle>
          <DialogDescription>
            {type === 'single_animal'
              ? t('animalActions.importExport.export.descriptionSingle', { earTag })
              : t('animalActions.importExport.export.descriptionBatch')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* 計畫選擇（批次匯出時） */}
          {type === 'batch_project' && (
            <div className="space-y-2">
              <Label>{t('animalActions.importExport.export.selectProject')}</Label>
              <Select value={selectedProject} onValueChange={setSelectedProject}>
                <SelectTrigger>
                  <SelectValue placeholder={t('animalActions.importExport.export.selectProjectHint')} />
                </SelectTrigger>
                <SelectContent>
                  {projects?.map((project) => (
                    <SelectItem key={project.iacuc_no} value={project.iacuc_no}>
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{project.iacuc_no}</span>
                        <span className="text-muted-foreground">({t('animalActions.common.animalCount', { count: project.animal_count })})</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {/* 匯出格式 */}
          <div className="space-y-2">
            <Label>{t('animalActions.importExport.export.format')}</Label>
            <div className="flex gap-3">
              <button
                type="button"
                onClick={() => setFormat('pdf')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${format === 'pdf'
                  ? 'border-purple-500 bg-status-purple-bg text-status-purple-text'
                  : 'border-border hover:border-border'
                  }`}
              >
                <FileText className="h-5 w-5" />
                <span className="font-medium">PDF</span>
              </button>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={() => exportMutation.mutate()}
            disabled={
              exportMutation.isPending ||
              (type === 'batch_project' && !selectedProject)
            }
            className="bg-status-purple-solid hover:bg-status-purple-solid/90"
          >
            {exportMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            <Download className="h-4 w-4 mr-2" />
            {t('animalActions.importExport.export.action')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
