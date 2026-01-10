import { useState } from 'react'
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
import { Loader2, Download, FileSpreadsheet, FileText } from 'lucide-react'

// 匯出類型
type ExportType = 'single_pig' | 'batch_project'
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
  pigId?: number
  earTag?: string
}

export function ExportDialog({ open, onOpenChange, type, pigId, earTag }: Props) {
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
      const res = await api.get<{ iacuc_no: string; title: string; pig_count: number }[]>(
        '/pigs/projects-summary'
      )
      return res.data
    },
    enabled: type === 'batch_project' && open,
  })

  const exportMutation = useMutation({
    mutationFn: async () => {
      const endpoint =
        type === 'single_pig' ? `/pigs/${pigId}/export` : `/pigs/batch-export`

      const params: any = {
        format,
        include: Object.entries(options)
          .filter(([_, v]) => v)
          .map(([k]) => k)
          .join(','),
      }

      if (type === 'batch_project' && selectedProject) {
        params.iacuc_no = selectedProject
      }

      const response = await api.get(endpoint, {
        params,
        responseType: 'blob',
      })

      // 下載檔案
      const blob = new Blob([response.data], {
        type: format === 'pdf' ? 'application/pdf' : 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url

      const filename =
        type === 'single_pig'
          ? `豬隻病歷_${earTag}_${new Date().toISOString().split('T')[0]}.${format === 'pdf' ? 'pdf' : 'xlsx'}`
          : `計畫病歷匯出_${selectedProject}_${new Date().toISOString().split('T')[0]}.${format === 'pdf' ? 'pdf' : 'xlsx'}`

      a.download = filename
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)

      return response.data
    },
    onSuccess: () => {
      toast({ title: '成功', description: '檔案匯出成功' })
      onOpenChange(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '匯出失敗',
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
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            {type === 'single_pig' ? '匯出豬隻病歷' : '批次匯出計畫病歷'}
          </DialogTitle>
          <DialogDescription>
            {type === 'single_pig'
              ? `匯出耳號 ${earTag} 的完整病歷資料`
              : '選擇計畫並匯出該計畫下所有豬隻的病歷資料'}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* 計畫選擇（批次匯出時） */}
          {type === 'batch_project' && (
            <div className="space-y-2">
              <Label>選擇計畫 (IACUC NO.) *</Label>
              <Select value={selectedProject} onValueChange={setSelectedProject}>
                <SelectTrigger>
                  <SelectValue placeholder="選擇要匯出的計畫" />
                </SelectTrigger>
                <SelectContent>
                  {projects?.map((project) => (
                    <SelectItem key={project.iacuc_no} value={project.iacuc_no}>
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{project.iacuc_no}</span>
                        <span className="text-slate-500">({project.pig_count} 隻)</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {/* 匯出格式 */}
          <div className="space-y-2">
            <Label>匯出格式</Label>
            <div className="flex gap-3">
              <button
                type="button"
                onClick={() => setFormat('pdf')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${
                  format === 'pdf'
                    ? 'border-purple-500 bg-purple-50 text-purple-700'
                    : 'border-slate-200 hover:border-slate-300'
                }`}
              >
                <FileText className="h-5 w-5" />
                <span className="font-medium">PDF</span>
              </button>
              <button
                type="button"
                onClick={() => setFormat('excel')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${
                  format === 'excel'
                    ? 'border-purple-500 bg-purple-50 text-purple-700'
                    : 'border-slate-200 hover:border-slate-300'
                }`}
              >
                <FileSpreadsheet className="h-5 w-5" />
                <span className="font-medium">Excel</span>
              </button>
              <button
                type="button"
                onClick={() => setFormat('csv')}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg border-2 transition-colors ${
                  format === 'csv'
                    ? 'border-purple-500 bg-purple-50 text-purple-700'
                    : 'border-slate-200 hover:border-slate-300'
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
              <Label>匯出內容</Label>
              <button
                type="button"
                onClick={() => handleSelectAll(!allSelected)}
                className="text-sm text-purple-600 hover:text-purple-700"
              >
                {allSelected ? '取消全選' : '全選'}
              </button>
            </div>

            <div className="grid grid-cols-2 gap-3 p-4 bg-slate-50 rounded-lg">
              <Checkbox
                label="豬隻基本資料"
                checked={options.basic_info}
                onChange={(e) => handleOptionChange('basic_info', e.target.checked)}
              />
              <Checkbox
                label="觀察試驗紀錄"
                checked={options.observations}
                onChange={(e) => handleOptionChange('observations', e.target.checked)}
              />
              <Checkbox
                label="手術紀錄"
                checked={options.surgeries}
                onChange={(e) => handleOptionChange('surgeries', e.target.checked)}
              />
              <Checkbox
                label="體重紀錄"
                checked={options.weights}
                onChange={(e) => handleOptionChange('weights', e.target.checked)}
              />
              <Checkbox
                label="疫苗/驅蟲紀錄"
                checked={options.vaccinations}
                onChange={(e) => handleOptionChange('vaccinations', e.target.checked)}
              />
              <Checkbox
                label="犧牲/採樣紀錄"
                checked={options.sacrifice}
                onChange={(e) => handleOptionChange('sacrifice', e.target.checked)}
              />
              <Checkbox
                label="病理組織報告"
                checked={options.pathology}
                onChange={(e) => handleOptionChange('pathology', e.target.checked)}
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            onClick={() => exportMutation.mutate()}
            disabled={
              exportMutation.isPending ||
              !someSelected ||
              (type === 'batch_project' && !selectedProject)
            }
            className="bg-purple-600 hover:bg-purple-700"
          >
            {exportMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            <Download className="h-4 w-4 mr-2" />
            匯出
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
