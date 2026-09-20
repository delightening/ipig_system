import { Trans } from 'react-i18next'
import { Card, CardContent } from '@/components/ui/card'
import { AlertTriangle, Undo2 } from 'lucide-react'
import type { Document } from '@/lib/api'
import { formatDate } from '@/lib/utils'

/**
 * R84-5 沖銷關聯提示（雙向）。
 *
 * - 本單**已被沖銷**：顯示沖銷單號與生效時間，並可點進沖銷單。
 * - 本單**是沖銷單**：標示其沖銷的原單，可點回原單。
 *
 * 沖銷單尚未經管理員核准時 `reversed_at` 為空——此時原單上顯示「沖銷處理中」，
 * 避免使用者誤以為帳已經沖掉（實際庫存與會計要等核准才反向）。
 *
 * 句中夾著可點的單號，用 `<Trans>` 而非拼接字串：兩種語言的語序不同，
 * 單號的位置由譯文自己決定。
 */
export function ReversalNotice({
  document,
  navigate,
}: {
  document: Document
  navigate: (path: string) => void
}) {
  const isReversal = !!document.reverses_doc_id
  const reversedBy = document.reversed_by_doc_id

  if (!isReversal && !reversedBy) return null

  if (isReversal) {
    return (
      <Card className="border-status-warning-border">
        <CardContent className="flex items-center gap-3 py-4">
          <Undo2 className="h-4 w-4 shrink-0 text-status-warning-text" />
          <p className="text-sm">
            <Trans
              i18nKey="erpDocs.documents.reversal.isReversal"
              values={{
                docNo: document.reverses_doc_id
                  ? (document.reverses_doc_no ?? document.reverses_doc_id)
                  : (document.reverses_doc_no ?? '-'),
              }}
              components={{
                bold: <span className="font-medium" />,
                docref: document.reverses_doc_id ? (
                  <button
                    type="button"
                    className="text-primary hover:underline"
                    onClick={() => navigate(`/documents/${document.reverses_doc_id}`)}
                  />
                ) : (
                  <span />
                ),
              }}
            />
          </p>
        </CardContent>
      </Card>
    )
  }

  const done = !!document.reversed_at
  const reversedByDocRef = (
    <button
      type="button"
      className="text-primary hover:underline"
      onClick={() => navigate(`/documents/${reversedBy}`)}
    />
  )
  const reversedByDocNo = document.reversed_by_doc_no ?? reversedBy
  return (
    <Card className={done ? 'border-destructive' : 'border-status-warning-border'}>
      <CardContent className="flex items-center gap-3 py-4">
        <AlertTriangle
          className={`h-4 w-4 shrink-0 ${done ? 'text-destructive' : 'text-status-warning-text'}`}
        />
        <p className="text-sm">
          {done ? (
            <Trans
              i18nKey="erpDocs.documents.reversal.done"
              values={{ date: formatDate(document.reversed_at!), docNo: reversedByDocNo }}
              components={{
                bold: <span className="font-medium text-destructive" />,
                docref: reversedByDocRef,
              }}
            />
          ) : (
            <Trans
              i18nKey="erpDocs.documents.reversal.pending"
              values={{ docNo: reversedByDocNo }}
              components={{
                bold: <span className="font-medium text-status-warning-text" />,
                docref: reversedByDocRef,
              }}
            />
          )}
        </p>
      </CardContent>
    </Card>
  )
}
