import type { ImportReviewComment } from '@/lib/api/protocol'
import { Repeater } from '@/components/ui/repeater'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'

/** 補登：一組「審查意見 + 申請人回覆」可重複列（執秘 / 委員一審 / 委員二審共用） */
export function CommentRows({
  value,
  onChange,
  addLabel,
}: {
  value: ImportReviewComment[]
  onChange: (v: ImportReviewComment[]) => void
  addLabel: string
}) {
  return (
    <Repeater<ImportReviewComment>
      value={value}
      onChange={onChange}
      defaultItem={() => ({ content: '', reply: '', section_no: '' })}
      addLabel={addLabel}
      maxItems={30}
      renderItem={(item, _idx, onItemChange) => (
        <div className="grid gap-2 rounded-md border p-3">
          <div className="grid gap-1">
            <Label className="text-sm font-normal text-muted-foreground">項次（選填，對應計畫書，如 4.1.2）</Label>
            <Input
              value={item.section_no ?? ''}
              onChange={(e) => onItemChange({ ...item, section_no: e.target.value })}
              placeholder="如 4.1.2"
              className="max-w-[12rem]"
            />
          </div>
          <div className="grid gap-1">
            <Label className="text-sm font-normal text-muted-foreground">審查意見</Label>
            <Textarea
              rows={2}
              value={item.content}
              onChange={(e) => onItemChange({ ...item, content: e.target.value })}
              placeholder="輸入該條審查意見"
            />
          </div>
          <div className="grid gap-1">
            <Label className="text-sm font-normal text-muted-foreground">申請人 / 客戶回覆（選填）</Label>
            <Textarea
              rows={2}
              value={item.reply ?? ''}
              onChange={(e) => onItemChange({ ...item, reply: e.target.value })}
              placeholder="輸入申請人對此意見的回覆"
            />
          </div>
        </div>
      )}
    />
  )
}
