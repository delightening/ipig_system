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

const importTypeConfig: Record<ImportType, { title: string; description: string; templateEndpoint: string }> = {
  basic: {
    title: '匯入豬隻基本資料',
    description: '支援 Excel (.xlsx, .xls) 或 CSV 格式',
    templateEndpoint: '/pigs/import/template/basic',
  },
  weight: {
    title: '匯入豬隻體重資料',
    description: '批次匯入多隻豬的體重紀錄',
    templateEndpoint: '/pigs/import/template/weight',
  },
}

export function ImportDialog({ open, onOpenChange, type }: Props) {
  const queryClient = useQueryClient()
  const [files, setFiles] = useState<FileInfo[]>([])
  const [fileObjects, setFileObjects] = useState<File[]>([])
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
      } else {
        toast({
          title: '匯入完成（部分失敗）',
          description: `成功: ${data.success_count} 筆，失敗: ${data.error_count} 筆`,
          variant: 'destructive',
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

  const handleFileChange = (fileInfos: FileInfo[]) => {
    setFiles(fileInfos)
    // 當檔案改變時，我們需要從 input 重新取得 File 物件
    // 但由於 FileUpload 組件不直接提供 File 物件，我們需要另一種方式
  }

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
      toast({ title: '錯誤', description: '請先選擇檔案', variant: 'destructive' })
      return
    }

    importMutation.mutate(fileObjects[0])
  }

  const handleClose = () => {
    setFiles([])
    setFileObjects([])
    setResult(null)
    onOpenChange(false)
  }

  const handleDownloadTemplate = async (format: 'xlsx' | 'csv' = 'csv') => {
    try {
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
          ? (format === 'csv' ? 'pig_basic_import_template.csv' : 'pig_basic_import_template.xlsx')
          : (format === 'csv' ? 'pig_weight_import_template.csv' : 'pig_weight_import_template.xlsx')
      }

      link.setAttribute('download', filename)
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      window.URL.revokeObjectURL(url)

      toast({
        title: '下載成功',
        description: '範本檔案已開始下載',
      })
    } catch (error: any) {
      toast({
        title: '下載失敗',
        description: error?.response?.data?.error?.message || '無法下載範本檔案',
        variant: 'destructive',
      })
    }
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
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                asChild
                className="border-blue-300 text-blue-700 hover:bg-blue-100"
              >
                {type === 'basic' ? (
                  <a href="/file_imput.csv" download="file_imput.csv">
                    <Download className="h-4 w-4 mr-1" />
                    下載範本 (CSV)
                  </a>
                ) : (
                  <a href="/weight_import.csv" download="weight_import.csv">
                    <Download className="h-4 w-4 mr-1" />
                    下載範本 (CSV)
                  </a>
                )}
              </Button>
            </div>
          </div>

          {/* File Upload */}
          {!result && (
            <div className="space-y-2">
              <Label>選擇檔案</Label>
              <input
                type="file"
                accept=".xlsx,.xls,.csv"
                onChange={handleFileInputChange}
                className="block w-full text-sm text-slate-500
                  file:mr-4 file:py-2 file:px-4
                  file:rounded-lg file:border-0
                  file:text-sm file:font-semibold
                  file:bg-purple-50 file:text-purple-700
                  hover:file:bg-purple-100
                  file:cursor-pointer"
              />
              {files.length > 0 && (
                <div className="mt-2 p-2 bg-slate-50 rounded-lg">
                  <p className="text-sm font-medium">{files[0].file_name}</p>
                  <p className="text-xs text-slate-500">
                    {(files[0].file_size / 1024 / 1024).toFixed(2)} MB
                  </p>
                </div>
              )}
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
                          <th className="px-3 py-2 text-left font-medium">耳號</th>
                          <th className="px-3 py-2 text-left font-medium">錯誤訊息</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.errors.map((error, i) => (
                          <tr key={i} className="border-t">
                            <td className="px-3 py-2">{error.row}</td>
                            <td className="px-3 py-2 font-mono">{error.ear_tag || '-'}</td>
                            <td className="px-3 py-2 text-red-600">{error.error}</td>
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
                    <li>耳號規則：若為數字，系統會自動轉換為三位數（例如 1 轉為 001）</li>
                    <li>品種：miniature/minipig/mini/M (迷你豬)、white/W (白豬)、other (其他)</li>
                    <li>性別：male/M (公)、female/F (母)</li>
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
