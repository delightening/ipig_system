import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
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
import { Loader2, Download, FileSpreadsheet, FileText } from 'lucide-react'

// 匯出類型
type ExportType = 'single_animal' | 'batch_project'
type ExportFormat = 'pdf' | 'excel' | 'csv'

interface ExportOptions {
  observations: boolean
  surgeries: boolean
  weights: boolean
  vaccinations: boolean
  sacrifice: boolean
  pathology: boolean
  basic_info: boolean
}

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
  const [options, setOptions] = useState<ExportOptions>({
    observations: true,
    surgeries: true,
    weights: true,
    vaccinations: true,
    sacrifice: true,
    pathology: true,
    basic_info: true,
  })

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

      const dateStr = new Date().toISOString().split('T')[0]
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

  const handleOptionChange = (key: keyof ExportOptions, value: boolean) => {
    setOptions((prev) => ({ ...prev, [key]: value }))
  }

  const handleSelectAll = (selected: boolean) => {
    setOptions({
      observations: selected,
      surgeries: selected,
      weights: selected,
      vaccinations: selected,
      sacrifice: selected,
      pathology: selected,
      basic_info: selected,
    })
  }

  const allSelected = Object.values(options).every((v) => v)
  const someSelected = Object.values(options).some((v) => v)

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
              <button
                type="button"
                onClick={() => setFormat('excel')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${format === 'excel'
                  ? 'border-purple-500 bg-status-purple-bg text-status-purple-text'
                  : 'border-border hover:border-border'
                  }`}
              >
                <FileSpreadsheet className="h-5 w-5" />
                <span className="font-medium">Excel</span>
              </button>
              <button
                type="button"
                onClick={() => setFormat('csv')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${format === 'csv'
                  ? 'border-purple-500 bg-status-purple-bg text-status-purple-text'
                  : 'border-border hover:border-border'
                  }`}
              >
                <FileSpreadsheet className="h-5 w-5" />
                <span className="font-medium">CSV</span>
              </button>
            </div>
          </div>

          {/* 匯出內容選項 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label>{t('animalActions.importExport.export.content')}</Label>
              <button
                type="button"
                onClick={() => handleSelectAll(!allSelected)}
                className="text-sm text-status-purple-text hover:text-status-purple-text"
              >
                {allSelected ? t('animalActions.importExport.export.deselectAll') : t('animalActions.importExport.export.selectAll')}
              </button>
            </div>

            <div className="grid grid-cols-2 gap-3 p-4 bg-muted rounded-lg">
              <Checkbox
                label={t('animalActions.common.animalBasicInfo')}
                checked={options.basic_info}
                onCheckedChange={(checked) => handleOptionChange('basic_info', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.observations')}
                checked={options.observations}
                onCheckedChange={(checked) => handleOptionChange('observations', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.surgeries')}
                checked={options.surgeries}
                onCheckedChange={(checked) => handleOptionChange('surgeries', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.weights')}
                checked={options.weights}
                onCheckedChange={(checked) => handleOptionChange('weights', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.vaccinations')}
                checked={options.vaccinations}
                onCheckedChange={(checked) => handleOptionChange('vaccinations', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.sacrifice')}
                checked={options.sacrifice}
                onCheckedChange={(checked) => handleOptionChange('sacrifice', checked)}
              />
              <Checkbox
                label={t('animalDetail.tabs.pathology')}
                checked={options.pathology}
                onCheckedChange={(checked) => handleOptionChange('pathology', checked)}
              />
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
              !someSelected ||
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
