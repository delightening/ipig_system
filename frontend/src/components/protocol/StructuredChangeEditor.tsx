import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Plus } from 'lucide-react'

export interface ChangeDetailRow {
  id: string
  section: string
  before: string
  after: string
}

interface StructuredChangeEditorProps {
  purpose: string
  onPurposeChange: (value: string) => void
  rows: ChangeDetailRow[]
  onAddRow: () => void
  onRemoveRow: (idx: number) => void
  onUpdateRow: (idx: number, field: 'section' | 'before' | 'after', value: string) => void
}

/**
 * R71-12 結構化變更明細編輯器：變更目的 + 多列「項次／改動前／改動後」。
 * 狀態由父層持有（送出 / 重設需用），本元件僅負責呈現與事件回拋（presentational）。
 */
export function StructuredChangeEditor({
  purpose,
  onPurposeChange,
  rows,
  onAddRow,
  onRemoveRow,
  onUpdateRow,
}: StructuredChangeEditorProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-2">
      <Label htmlFor="amendment-change-purpose">{t('amendments.decision.changePurpose')}</Label>
      <Textarea
        id="amendment-change-purpose"
        value={purpose}
        onChange={(e) => onPurposeChange(e.target.value)}
        placeholder={t('amendments.decision.changePurposePlaceholder')}
        rows={2}
      />
      <p className="text-xs text-muted-foreground">{t('amendments.decision.changeDetailHint')}</p>
      <div className="space-y-3">
        {rows.map((row, idx) => (
          <div key={row.id} className="rounded-md border p-3 space-y-2">
            <div className="flex items-center gap-2">
              <Input
                value={row.section}
                onChange={(e) => onUpdateRow(idx, 'section', e.target.value)}
                placeholder={t('amendments.decision.sectionPlaceholder')}
                className="max-w-[200px]"
              />
              <Button type="button" variant="ghost" size="sm" onClick={() => onRemoveRow(idx)}>
                {t('amendments.decision.removeRow')}
              </Button>
            </div>
            <div className="grid gap-2 sm:grid-cols-2">
              <Textarea
                value={row.before}
                onChange={(e) => onUpdateRow(idx, 'before', e.target.value)}
                placeholder={t('amendments.decision.before')}
                rows={2}
              />
              <Textarea
                value={row.after}
                onChange={(e) => onUpdateRow(idx, 'after', e.target.value)}
                placeholder={t('amendments.decision.after')}
                rows={2}
              />
            </div>
          </div>
        ))}
      </div>
      <Button type="button" variant="outline" size="sm" onClick={onAddRow}>
        <Plus className="mr-2 h-4 w-4" />
        {t('amendments.decision.addChangeRow')}
      </Button>
    </div>
  )
}
