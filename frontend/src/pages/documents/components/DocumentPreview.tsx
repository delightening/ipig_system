import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
  const totalQty = formData.lines.reduce(
    (sum, l) => sum + (parseFloat(l.qty) || 0),
    0
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('erpDocs.documents.summary.title')}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex justify-between">
          <span className="text-muted-foreground">{t('erpDocs.documents.summary.lineCount')}</span>
          <span className="font-medium">{t('erpDocs.documents.summary.lineCountValue', { count: formData.lines.length })}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-muted-foreground">{t('erpDocs.documents.summary.totalQty')}</span>
          <span className="font-medium">{formatNumber(totalQty, 0)}</span>
        </div>
        {showTotalAmount && (
          <div className="flex justify-between text-lg border-t pt-4">
            <span className="font-medium">{t('erpDocs.documents.summary.totalAmount')}</span>
            <span className="font-bold">${formatNumber(totalAmount, 2)}</span>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
