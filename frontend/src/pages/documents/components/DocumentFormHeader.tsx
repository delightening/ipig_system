import { Button } from '@/components/ui/button'
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
    <div className="flex items-center justify-between">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">
          {isEdit ? '編輯單據' : '新增單據'}
        </h1>
        <p className="text-muted-foreground">
          {isEdit ? `編輯 ${docTypeName}` : `建立新的${docTypeName}`}
        </p>
      </div>
      <div className="flex gap-2">
        <Button
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
    </div>
  )
}
