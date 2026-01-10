import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import {
  Loader2,
  Upload,
  Download,
  FileSpreadsheet,
  AlertCircle,
  CheckCircle2,
} from 'lucide-react'

type ImportType = 'basic' | 'weight'

interface ImportResult {
  success_count: number
  error_count: number
  errors?: { row: number; message: string }[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  type: ImportType
}

const importTypeConfig: Record<ImportType, { title: string; description: string; templateUrl: string }> = {
  basic: {
    title: '匯入豬隻基本資料',
    description: '支援 Excel (.xlsx, .xls) 或 CSV 格式',
    templateUrl: '/templates/pig_import_template.xlsx',
  },
  weight: {
    title: '匯入豬隻體重資料',
    description: '批次匯入多隻豬的體重紀錄',
    templateUrl: '/templates/pig_weight_import_template.xlsx',
  },
}

export function ImportDialog({ open, onOpenChange, type }: Props) {
  const queryClient = useQueryClient()
  const [files, setFiles] = useState<FileInfo[]>([])
  const [result, setResult] = useState<ImportResult | null>(null)
  const config = importTypeConfig[type]

  const importMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      
      const endpoint = type === 'basic' 
        ? '/pigs/import/basic' 
        : '/pigs/import/weights'
      
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
        queryClient.invalidateQueries({ queryKey: ['pigs'] })
        toast({ 
          title: '匯入成功', 
          description: `成功匯入 ${data.success_count} 筆資料` 
        })
      }
    },
    onError: (error: any) => {
      toast({
        title: '匯入失敗',
        description: error?.response?.data?.error?.message || '發生未知錯誤',
        variant: 'destructive',
      })
    },
  })

  const handleImport = () => {
    if (files.length === 0) {
      toast({ title: '錯誤', description: '請先選擇檔案', variant: 'destructive' })
      return
    }

    // 從 FileInfo 取得原始 File 物件
    // 在實際實作中，需要儲存原始 File 物件
    // 這裡示範呼叫 API
    const mockFile = new File([''], files[0].file_name)
    importMutation.mutate(mockFile)
  }

  const handleClose = () => {
    setFiles([])
    setResult(null)
    onOpenChange(false)
  }

  const handleDownloadTemplate = () => {
    window.open(config.templateUrl, '_blank')
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            {config.title}
          </DialogTitle>
          <DialogDescription>{config.description}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Template Download */}
          <div className="flex items-center justify-between p-3 bg-blue-50 rounded-lg">
            <div className="flex items-center gap-2">
              <FileSpreadsheet className="h-5 w-5 text-blue-600" />
              <span className="text-sm text-blue-800">下載範本檔案</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownloadTemplate}
              className="border-blue-300 text-blue-700 hover:bg-blue-100"
            >
              <Download className="h-4 w-4 mr-1" />
              下載
            </Button>
          </div>

          {/* File Upload */}
          {!result && (
            <div className="space-y-2">
              <Label>選擇檔案</Label>
              <FileUpload
                value={files}
                onChange={setFiles}
                accept=".xlsx,.xls,.csv"
                placeholder="拖曳 Excel 或 CSV 檔案到此處"
                maxSize={10}
                maxFiles={1}
                showPreview={false}
              />
            </div>
          )}

          {/* Import Result */}
          {result && (
            <div className="space-y-4">
              <div className="flex items-center gap-4 p-4 bg-slate-50 rounded-lg">
                <div className="flex-1">
                  <div className="flex items-center gap-2 text-green-600">
                    <CheckCircle2 className="h-5 w-5" />
                    <span className="font-medium">成功匯入</span>
                  </div>
                  <p className="text-2xl font-bold text-green-700 mt-1">
                    {result.success_count} 筆
                  </p>
                </div>
                {result.error_count > 0 && (
                  <div className="flex-1 border-l pl-4">
                    <div className="flex items-center gap-2 text-red-600">
                      <AlertCircle className="h-5 w-5" />
                      <span className="font-medium">匯入失敗</span>
                    </div>
                    <p className="text-2xl font-bold text-red-700 mt-1">
                      {result.error_count} 筆
                    </p>
                  </div>
                )}
              </div>

              {/* Error Details */}
              {result.errors && result.errors.length > 0 && (
                <div className="space-y-2">
                  <Label className="text-red-600">錯誤明細</Label>
                  <div className="max-h-40 overflow-y-auto border rounded-lg">
                    <table className="w-full text-sm">
                      <thead className="bg-slate-100 sticky top-0">
                        <tr>
                          <th className="px-3 py-2 text-left font-medium">列</th>
                          <th className="px-3 py-2 text-left font-medium">錯誤訊息</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.errors.map((error, i) => (
                          <tr key={i} className="border-t">
                            <td className="px-3 py-2">{error.row}</td>
                            <td className="px-3 py-2 text-red-600">{error.message}</td>
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
            <div className="text-sm text-slate-500 space-y-1">
              <p className="font-medium">注意事項：</p>
              <ul className="list-disc list-inside space-y-0.5">
                {type === 'basic' ? (
                  <>
                    <li>耳號為必填欄位，不可重複</li>
                    <li>品種：miniature(迷你豬) / white(白豬) / other(其他)</li>
                    <li>性別：male(公) / female(母)</li>
                    <li>日期格式：YYYY-MM-DD</li>
                  </>
                ) : (
                  <>
                    <li>耳號必須已存在於系統中</li>
                    <li>測量日期格式：YYYY-MM-DD</li>
                    <li>體重單位：公斤 (kg)</li>
                  </>
                )}
              </ul>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {result ? '關閉' : '取消'}
          </Button>
          {!result && (
            <Button
              onClick={handleImport}
              disabled={importMutation.isPending || files.length === 0}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {importMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              開始匯入
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button
              onClick={handleClose}
              className="bg-green-600 hover:bg-green-700"
            >
              完成
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
