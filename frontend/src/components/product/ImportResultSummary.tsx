import { Label } from '@/components/ui/label'
import { AlertCircle, CheckCircle2 } from 'lucide-react'

import type { ImportResultSummaryProps } from './importTypes'

export function ImportResultSummary({ result }: ImportResultSummaryProps) {
  return (
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
  )
}
