import { Button } from '@/components/ui/button'
import { Loader2, AlertCircle, Download } from 'lucide-react'

import type { NoSkuColumnPromptProps } from './importTypes'

export function NoSkuColumnPrompt({
  previewMutationIsPending,
  importIsPending,
  hasDuplicates,
  onSetSkuManually,
  onAutoGenerateSku,
  onDownloadTemplate,
}: NoSkuColumnPromptProps) {
  return (
    <div className="space-y-4 p-4 border border-status-info-border bg-status-info-bg rounded-lg">
      <div className="flex items-center gap-2 text-status-info-text">
        <AlertCircle className="h-5 w-5" />
        <span className="font-medium">此檔案未含 SKU 編碼欄位</span>
      </div>
      <p className="text-sm text-status-info-text">請選擇處理方式：</p>
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          onClick={onSetSkuManually}
          disabled={previewMutationIsPending}
          variant="outline"
          className="border-blue-600 text-status-info-text hover:bg-status-info-bg"
        >
          {previewMutationIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          依序設定 SKU
        </Button>
        <Button
          size="sm"
          onClick={onAutoGenerateSku}
          disabled={importIsPending}
          className="bg-primary hover:bg-primary/90"
        >
          {importIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          由系統自動產生 SKU 並繼續匯入
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onDownloadTemplate}
          className="border-blue-600 text-status-info-text hover:bg-status-info-bg"
        >
          <Download className="h-4 w-4 mr-1" />
          取消，改下載含 SKU 的範本
        </Button>
      </div>
    </div>
  )
}
