import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Save, Send, Loader2 } from 'lucide-react'

interface DocumentFormHeaderProps {
  isEdit: boolean
  docTypeName: string
  onBack: () => void
  onSave: () => void
  onSubmit: () => void
  isSaving: boolean
  isSubmitting: boolean
  hasLines: boolean
  /**
   * 非空即擋下兩個按鈕，字串是擋下的原因（同時當 tooltip 與按鈕下方的說明文字）。
   *
   * 「儲存草稿」也一起擋：盤點單的底稿是**建單當下**由後端一次產生的，
   * 存成草稿的那一刻就已經定案要盤哪些品項，不是送審才決定。
   */
  blockedReason?: string
}

export function DocumentFormHeader({
  isEdit,
  docTypeName,
  onSave,
  onSubmit,
  isSaving,
  isSubmitting,
  hasLines,
  blockedReason,
}: DocumentFormHeaderProps) {
  const blocked = Boolean(blockedReason)

  return (
    <PageHeader
      title={isEdit ? '編輯單據' : '新增單據'}
      description={isEdit
        ? `編輯現有的 ${docTypeName || '單據'}`
        : `建立新的 ${docTypeName || '單據'}`}
      actions={
        <div className="flex flex-col items-end gap-1">
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={onSave}
              disabled={isSaving || isSubmitting || blocked}
              title={blockedReason}
            >
              {isSaving ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              儲存草稿
            </Button>
            <Button
              size="sm"
              onClick={onSubmit}
              disabled={isSaving || isSubmitting || !hasLines || blocked}
              title={blockedReason}
            >
              {isSubmitting ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              儲存並送審
            </Button>
          </div>
          {blocked && (
            <p className="text-xs text-destructive" role="status">
              {blockedReason}
            </p>
          )}
        </div>
      }
    />
  )
}
