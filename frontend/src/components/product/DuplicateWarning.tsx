import { Button } from '@/components/ui/button'
import { Loader2, AlertTriangle } from 'lucide-react'

import type { DuplicateWarningProps } from './importTypes'

export function DuplicateWarning({
  checkResult,
  importIsPending,
  onSkipDuplicates,
  onImportWithNewSku,
  onImportAnyway,
}: DuplicateWarningProps) {
  return (
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
              <tr key={`dup-${d.row}-${i}`} className="border-t">
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
          onClick={onSkipDuplicates}
          disabled={importIsPending}
          className="border-amber-600 text-amber-700 hover:bg-amber-100"
        >
          {importIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          略過重複列（僅匯入不重複者）
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onImportWithNewSku}
          disabled={importIsPending}
          className="border-amber-600 text-amber-700 hover:bg-amber-100"
        >
          匯入，但更改流水號
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onImportAnyway}
          disabled={importIsPending}
        >
          仍要匯入（可能產生重複產品）
        </Button>
      </div>
    </div>
  )
}
