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
            <span className="font-medium">本單是沖銷單</span>
            ：沖銷原單{' '}
            {document.reverses_doc_id ? (
              <button
                type="button"
                className="text-primary hover:underline"
                onClick={() => navigate(`/documents/${document.reverses_doc_id}`)}
              >
                {document.reverses_doc_no ?? document.reverses_doc_id}
              </button>
            ) : (
              (document.reverses_doc_no ?? '-')
            )}
            。核准後將反向沖銷原單的庫存與會計帳。
          </p>
        </CardContent>
      </Card>
    )
  }

  const done = !!document.reversed_at
  return (
    <Card className={done ? 'border-destructive' : 'border-status-warning-border'}>
      <CardContent className="flex items-center gap-3 py-4">
        <AlertTriangle
          className={`h-4 w-4 shrink-0 ${done ? 'text-destructive' : 'text-status-warning-text'}`}
        />
        <p className="text-sm">
          {done ? (
            <>
              <span className="font-medium text-destructive">此單已被沖銷</span>
              ：於 {formatDate(document.reversed_at!)} 由沖銷單{' '}
            </>
          ) : (
            <>
              <span className="font-medium text-status-warning-text">沖銷處理中</span>
              ：已建立沖銷單{' '}
            </>
          )}
          <button
            type="button"
            className="text-primary hover:underline"
            onClick={() => navigate(`/documents/${reversedBy}`)}
          >
            {document.reversed_by_doc_no ?? reversedBy}
          </button>
          {done ? ' 沖銷，庫存與會計帳已反向。' : '，待管理員核准後才會反向沖銷庫存與會計帳。'}
        </p>
      </CardContent>
    </Card>
  )
}
