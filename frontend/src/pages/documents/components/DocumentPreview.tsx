import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatNumber } from '@/lib/utils'
import type { DocumentFormData } from '../types'

interface DocumentPreviewProps {
  formData: DocumentFormData
  totalAmount: number
  showTotalAmount: boolean
}

export function DocumentPreview({
  formData,
  totalAmount,
  showTotalAmount,
}: DocumentPreviewProps) {
  const totalQty = formData.lines.reduce(
    (sum, l) => sum + (parseFloat(l.qty) || 0),
    0
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>單據摘要</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex justify-between">
          <span className="text-muted-foreground">明細行數</span>
          <span className="font-medium">{formData.lines.length} 項</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">總數量</span>
          <span className="font-medium">{formatNumber(totalQty, 0)}</span>
        </div>
        {showTotalAmount && (
          <div className="flex justify-between text-lg border-t pt-4">
            <span className="font-medium">總金額</span>
            <span className="font-bold">${formatNumber(totalAmount, 2)}</span>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
