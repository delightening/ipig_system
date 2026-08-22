import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

/**
 * 計畫起訖日。起始日 = 計畫核准通過日（唯讀，自里程碑帶入，不另填）；結束日必填。
 * 手機單欄避免日期框 overflow。
 */
export function DateRangePicker({
  startDate,
  endDate,
  onEndChange,
}: {
  /** 計畫核准通過日（= 起始日，唯讀） */
  startDate: string
  endDate: string
  onEndChange: (v: string) => void
}) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div className="grid gap-2 min-w-0">
        <Label>計畫起始日（= 計畫核准通過日）</Label>
        <Input type="date" className="w-full" value={startDate} readOnly disabled />
        {!startDate && (
          <p className="text-xs text-muted-foreground">填寫「計畫核准通過」里程碑後自動帶入。</p>
        )}
      </div>
      <div className="grid gap-2 min-w-0">
        <Label>計畫結束日 *</Label>
        <Input type="date" className="w-full" value={endDate} onChange={(e) => onEndChange(e.target.value)} />
      </div>
    </div>
  )
}
