import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
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
import { getApiErrorMessage } from '@/lib/validation'
import {
  Loader2,
  Upload,
  Download,
  FileSpreadsheet,
  AlertCircle,
  CheckCircle2,
  AlertTriangle,
} from 'lucide-react'

interface ProductImportErrorDetail {
  row: number
  sku?: string
  error: string
}

interface ProductImportResult {
  success_count: number
  error_count: number
  errors?: ProductImportErrorDetail[]
}

/** 規則一：匯入預檢重複項目 */
interface ProductImportDuplicateItem {
  row: number
  name: string
  spec?: string
  existing_sku: string
  existing_id: string
}

interface ProductImportCheckResult {
  total_rows: number
  duplicate_count: number
  duplicates: ProductImportDuplicateItem[]
  /** 檔案是否包含 SKU 編碼欄位 */
  has_sku_column: boolean
}

/** 匯入預覽列（依序設定 SKU 用） */
interface ProductImportPreviewRow {
  row: number
  name: string
  spec?: string
  category_code?: string
  subcategory_code?: string
  base_uom: string
  track_batch: boolean
  track_expiry: boolean
  safety_stock?: number
  remark?: string
}

interface ProductImportPreviewResult {
  rows: ProductImportPreviewRow[]
  has_sku_column: boolean
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ProductImportDialog({ open, onOpenChange }: Props) {
  const queryClient = useQueryClient()
  const [file, setFile] = useState<File | null>(null)
  const [result, setResult] = useState<ProductImportResult | null>(null)
  const [checkResult, setCheckResult] = useState<ProductImportCheckResult | null>(null)
  /** 使用者選擇「由系統自動產生 SKU 並繼續」後，用於顯示重複處理選項 */
  const [userAcceptedNoSku, setUserAcceptedNoSku] = useState(false)
  /** 依序設定 SKU：預覽列與使用者輸入的 SKU */
  const [previewRows, setPreviewRows] = useState<ProductImportPreviewRow[] | null>(null)
  const [skuOverrides, setSkuOverrides] = useState<Record<number, string>>({})

  const checkMutation = useMutation({
    mutationFn: async (f: File) => {
      const formData = new FormData()
      formData.append('file', f)
      const res = await api.post<ProductImportCheckResult>('/products/import/check', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      return res.data
    },
    onSuccess: (data, f) => {
      setCheckResult(data)
      // 僅在「有 SKU 欄位」且無重複時自動匯入；無 SKU 時改由使用者選擇
      if (data.duplicate_count === 0 && data.has_sku_column && f) {
        doImport(f, false)
      }
    },
    onError: (error: unknown) => {
      toast({
        title: '預檢失敗',
        description: getApiErrorMessage(error, '無法檢查重複'),
        variant: 'destructive',
      })
    },
  })

  const previewMutation = useMutation({
    mutationFn: async (f: File) => {
      const formData = new FormData()
      formData.append('file', f)
      const res = await api.post<ProductImportPreviewResult>(
        '/products/import/preview',
        formData,
        { headers: { 'Content-Type': 'multipart/form-data' } }
      )
      return res.data
    },
    onSuccess: (data) => {
      setPreviewRows(data.rows)
      setSkuOverrides({})
    },
    onError: (error: unknown) => {
      toast({
        title: '預覽失敗',
        description: getApiErrorMessage(error, '無法解析檔案'),
        variant: 'destructive',
      })
    },
  })

  const importMutation = useMutation({
    mutationFn: async ({
      f,
      skipDuplicates,
      regenerateSkuForDuplicates,
    }: {
      f: File
      skipDuplicates: boolean
      regenerateSkuForDuplicates: boolean
    }) => {
      const formData = new FormData()
      formData.append('file', f)
      formData.append('skip_duplicates', String(skipDuplicates))
      formData.append('regenerate_sku_for_duplicates', String(regenerateSkuForDuplicates))
      const res = await api.post<ProductImportResult>('/products/import', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      return res.data
    },
    onSuccess: (data) => {
      setResult(data)
      setCheckResult(null)
      setPreviewRows(null)
      setSkuOverrides({})
      queryClient.invalidateQueries({ queryKey: ['products'] })
      if (data.error_count === 0) {
        toast({
          title: '匯入成功',
          description: `成功匯入 ${data.success_count} 筆產品`,
        })
      } else {
        toast({
          title: '匯入完成（部分失敗）',
          description: `成功: ${data.success_count} 筆，失敗: ${data.error_count} 筆`,
          variant: 'destructive',
        })
      }
    },
    onError: (error: unknown) => {
      toast({
        title: '匯入失敗',
        description: getApiErrorMessage(error, '發生未知錯誤'),
        variant: 'destructive',
      })
    },
  })

  const doImport = (
    f: File,
    skipDuplicates: boolean,
    regenerateSkuForDuplicates = false
  ) => {
    importMutation.mutate({ f, skipDuplicates, regenerateSkuForDuplicates })
  }

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (selectedFiles && selectedFiles.length > 0) {
      setFile(selectedFiles[0])
      setCheckResult(null)
      setResult(null)
      setUserAcceptedNoSku(false)
    }
  }

  const handleImport = () => {
    if (!file) {
      toast({ title: '錯誤', description: '請先選擇檔案', variant: 'destructive' })
      return
    }
    setCheckResult(null)
    checkMutation.mutate(file)
  }

  const handleSkipDuplicates = () => {
    if (file) doImport(file, true, false)
  }

  const handleImportAnyway = () => {
    if (file) doImport(file, false, false)
  }

  const handleImportWithNewSku = () => {
    if (file) doImport(file, false, true)
  }

  const handleClose = () => {
    setFile(null)
    setResult(null)
    setCheckResult(null)
    setUserAcceptedNoSku(false)
    setPreviewRows(null)
    setSkuOverrides({})
    onOpenChange(false)
  }

  /** 將預覽列與使用者輸入的 SKU 組成 CSV（含 BOM），供匯入 API 使用 */
  const buildCsvWithSku = (): File => {
    if (!previewRows?.length) throw new Error('無預覽資料')
    const escape = (v: string) => {
      const s = String(v ?? '').trim()
      if (s.includes(',') || s.includes('"') || s.includes('\n') || s.includes('\r')) {
        return '"' + s.replace(/"/g, '""') + '"'
      }
      return s
    }
    const header =
      'SKU編碼,名稱,規格,品類代碼,子類代碼,單位,追蹤批號,追蹤效期,安全庫存,備註'
    const lines = previewRows.map((r) => {
      const sku = skuOverrides[r.row]?.trim() ?? ''
      return [
        escape(sku),
        escape(r.name),
        escape(r.spec ?? ''),
        escape(r.category_code ?? ''),
        escape(r.subcategory_code ?? ''),
        escape(r.base_uom),
        r.track_batch ? 'true' : 'false',
        r.track_expiry ? 'true' : 'false',
        escape(r.safety_stock != null ? String(r.safety_stock) : ''),
        escape(r.remark ?? ''),
      ].join(',')
    })
    const csv = '\uFEFF' + header + '\n' + lines.join('\n')
    return new File([new Blob([csv], { type: 'text/csv;charset=utf-8' })], 'product_import_with_sku.csv')
  }

  const handleConfirmImportWithSku = () => {
    try {
      const f = buildCsvWithSku()
      doImport(f, false, false)
    } catch (e) {
      toast({
        title: '無法產生匯入檔',
        description: e instanceof Error ? e.message : '請稍後再試',
        variant: 'destructive',
      })
    }
  }

  const handleDownloadTemplate = async () => {
    try {
      const response = await api.get('/products/import/template', {
        responseType: 'blob',
      })

      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url

      const contentDisposition = response.headers['content-disposition']
      let filename = 'product_import_template.xlsx'
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

      toast({
        title: '下載成功',
        description: '範本檔案已開始下載',
      })
    } catch (error: unknown) {
      toast({
        title: '下載失敗',
        description: getApiErrorMessage(error, '無法下載範本檔案'),
        variant: 'destructive',
      })
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className={previewRows?.length ? 'max-w-4xl' : 'max-w-lg'}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            匯入產品
          </DialogTitle>
          <DialogDescription>
            支援 Excel (.xlsx, .xls) 或 CSV 格式，批次匯入多筆產品資料
          </DialogDescription>
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
              className="border-blue-300 text-blue-700 hover:bg-blue-100"
              onClick={handleDownloadTemplate}
            >
              <Download className="h-4 w-4 mr-1" />
              下載範本 (XLSX)
            </Button>
          </div>

          {/* File Upload */}
          {!result && (
            <label className="block space-y-2">
              <span className="block text-sm font-medium leading-none">選擇檔案</span>
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
              {file && (
                <div className="mt-2 p-2 bg-slate-50 rounded-lg">
                  <p className="text-sm font-medium">{file.name}</p>
                  <p className="text-xs text-slate-500">
                    {(file.size / 1024).toFixed(1)} KB
                  </p>
                </div>
              )}
            </label>
          )}

          {/* 檔案未含 SKU 欄位：讓使用者選擇 */}
          {checkResult && !checkResult.has_sku_column && !result && !previewRows && (checkResult.duplicate_count === 0 || !userAcceptedNoSku) && (
            <div className="space-y-4 p-4 border border-blue-200 bg-blue-50 rounded-lg">
              <div className="flex items-center gap-2 text-blue-800">
                <AlertCircle className="h-5 w-5" />
                <span className="font-medium">此檔案未含 SKU 編碼欄位</span>
              </div>
              <p className="text-sm text-blue-700">
                請選擇處理方式：
              </p>
              <div className="flex flex-wrap gap-2">
                <Button
                  size="sm"
                  onClick={() => {
                    if (file) previewMutation.mutate(file)
                  }}
                  disabled={previewMutation.isPending}
                  variant="outline"
                  className="border-blue-600 text-blue-700 hover:bg-blue-100"
                >
                  {previewMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  依序設定 SKU
                </Button>
                <Button
                  size="sm"
                  onClick={() => {
                    if (checkResult.duplicate_count === 0 && file) {
                      doImport(file, false)
                    } else {
                      setUserAcceptedNoSku(true)
                    }
                  }}
                  disabled={importMutation.isPending}
                  className="bg-blue-600 hover:bg-blue-700"
                >
                  {importMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  由系統自動產生 SKU 並繼續匯入
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    handleDownloadTemplate()
                    setCheckResult(null)
                    setUserAcceptedNoSku(false)
                  }}
                  className="border-blue-600 text-blue-700 hover:bg-blue-100"
                >
                  <Download className="h-4 w-4 mr-1" />
                  取消，改下載含 SKU 的範本
                </Button>
              </div>
            </div>
          )}

          {/* 依序設定 SKU：預覽表格 */}
          {previewRows && previewRows.length > 0 && !result && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium text-slate-700">
                  請為以下商品設定 SKU（留空則由系統自動產生），共 {previewRows.length} 筆
                </p>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setPreviewRows(null)
                    setSkuOverrides({})
                  }}
                >
                  返回
                </Button>
              </div>
              <div className="max-h-64 overflow-auto rounded-lg border border-slate-200">
                <table className="w-full text-sm">
                  <thead className="bg-slate-100 sticky top-0">
                    <tr>
                      <th className="px-3 py-2 text-left font-medium w-14">列</th>
                      <th className="px-3 py-2 text-left font-medium">名稱</th>
                      <th className="px-3 py-2 text-left font-medium min-w-[80px]">規格</th>
                      <th className="px-3 py-2 text-left font-medium w-16">單位</th>
                      <th className="px-3 py-2 text-left font-medium w-20">安全庫存</th>
                      <th className="px-3 py-2 text-left font-medium min-w-[120px]">SKU 編碼</th>
                    </tr>
                  </thead>
                  <tbody>
                    {previewRows.map((r) => (
                      <tr key={r.row} className="border-t border-slate-100 hover:bg-slate-50">
                        <td className="px-3 py-1.5">{r.row}</td>
                        <td className="px-3 py-1.5">{r.name}</td>
                        <td className="px-3 py-1.5 text-slate-600">{r.spec ?? '-'}</td>
                        <td className="px-3 py-1.5">{r.base_uom}</td>
                        <td className="px-3 py-1.5">{r.safety_stock ?? '-'}</td>
                        <td className="px-3 py-1.5">
                          <input
                            type="text"
                            value={skuOverrides[r.row] ?? ''}
                            onChange={(e) =>
                              setSkuOverrides((prev) => ({ ...prev, [r.row]: e.target.value }))
                            }
                            placeholder="留空自動產生"
                            className="w-full min-w-[100px] rounded border border-slate-300 px-2 py-1 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                          />
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  onClick={handleConfirmImportWithSku}
                  disabled={importMutation.isPending}
                  className="bg-purple-600 hover:bg-purple-700"
                >
                  {importMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  確認匯入
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setPreviewRows(null)
                    setSkuOverrides({})
                  }}
                >
                  返回
                </Button>
              </div>
            </div>
          )}

          {/* 規則一：重複警示確認（有 SKU 欄位時，或使用者已選擇由系統產生 SKU 時顯示；依序設定 SKU 步驟不顯示） */}
          {checkResult && checkResult.duplicate_count > 0 && (checkResult.has_sku_column || userAcceptedNoSku) && !result && !previewRows && (
            <div className="space-y-4 p-4 border border-amber-200 bg-amber-50 rounded-lg">
              <div className="flex items-center gap-2 text-amber-800">
                <AlertTriangle className="h-5 w-5" />
                <span className="font-medium">發現 {checkResult.duplicate_count} 筆與既有產品重複</span>
              </div>
              <p className="text-sm text-amber-700">
                下列產品的「名稱+規格」已存在於資料庫中，請選擇處理方式：
              </p>
              <div className="max-h-32 overflow-y-auto border border-amber-200 rounded bg-white">
                <table className="w-full text-sm">
                  <thead className="bg-amber-100 sticky top-0">
                    <tr>
                      <th className="px-3 py-2 text-left font-medium">列</th>
                      <th className="px-3 py-2 text-left font-medium">名稱</th>
                      <th className="px-3 py-2 text-left font-medium">規格</th>
                      <th className="px-3 py-2 text-left font-medium">既有 SKU</th>
                    </tr>
                  </thead>
                  <tbody>
                    {checkResult.duplicates.map((d, i) => (
                      <tr key={i} className="border-t">
                        <td className="px-3 py-2">{d.row}</td>
                        <td className="px-3 py-2">{d.name}</td>
                        <td className="px-3 py-2">{d.spec ?? '-'}</td>
                        <td className="px-3 py-2 font-mono">{d.existing_sku}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleSkipDuplicates}
                  disabled={importMutation.isPending}
                  className="border-amber-600 text-amber-700 hover:bg-amber-100"
                >
                  {importMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                  略過重複列（僅匯入不重複者）
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleImportWithNewSku}
                  disabled={importMutation.isPending}
                  className="border-amber-600 text-amber-700 hover:bg-amber-100"
                >
                  匯入，但更改流水號
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleImportAnyway}
                  disabled={importMutation.isPending}
                >
                  仍要匯入（可能產生重複產品）
                </Button>
              </div>
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
                          <th className="px-3 py-2 text-left font-medium">SKU</th>
                          <th className="px-3 py-2 text-left font-medium">錯誤訊息</th>
                        </tr>
                      </thead>
                      <tbody>
                        {result.errors.map((err, i) => (
                          <tr key={i} className="border-t">
                            <td className="px-3 py-2">{err.row}</td>
                            <td className="px-3 py-2 font-mono">{err.sku || '-'}</td>
                            <td className="px-3 py-2 text-red-600">{err.error}</td>
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
                <li>名稱為必填欄位</li>
                <li>單位為必填欄位（預設 PCS）</li>
                <li>品類代碼、子類代碼可選，未填時預設為 GEN-OTH</li>
                <li>追蹤批號、追蹤效期：true/false 或 是/否</li>
                <li>CSV 欄位順序：SKU編碼、名稱、規格、品類代碼、子類代碼、單位、追蹤批號、追蹤效期、安全庫存、備註（SKU 可留空由系統自動產生）</li>
              </ul>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            {result ? '關閉' : '取消'}
          </Button>
          {!result && !checkResult && !previewRows && (
            <Button
              onClick={handleImport}
              disabled={checkMutation.isPending || importMutation.isPending || !file}
              className="bg-purple-600 hover:bg-purple-700"
            >
              {(checkMutation.isPending || importMutation.isPending) && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              開始匯入
            </Button>
          )}
          {result && result.error_count === 0 && (
            <Button onClick={handleClose} className="bg-green-600 hover:bg-green-700">
              完成
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
