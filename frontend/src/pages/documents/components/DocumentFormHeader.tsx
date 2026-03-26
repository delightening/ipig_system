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
}

export function DocumentFormHeader({
  isEdit,
  docTypeName,
  onSave,
  onSubmit,
  isSaving,
  isSubmitting,
  hasLines,
}: DocumentFormHeaderProps) {
  return (
    <PageHeader
      title={isEdit ? '編輯單據' : '新增單據'}
      description={isEdit
        ? `編輯現有的 ${docTypeName || '單據'}`
        : `建立新的 ${docTypeName || '單據'}`}
      actions={
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={onSave}
            disabled={isSaving || isSubmitting}
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
            disabled={isSaving || isSubmitting || !hasLines}
          >
            {isSubmitting ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Send className="mr-2 h-4 w-4" />
            )}
            儲存並送審
          </Button>
        </div>
      }
    />
  )
}
