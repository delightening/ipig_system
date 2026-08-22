import type { ImportVetReview } from '@/lib/api/protocol'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Check, X, Minus, Plus, Trash2, ClipboardList } from 'lucide-react'

import { ReviewerSelect } from './ReviewerSelect'
import { VET, type UserOption } from './users'
import { DEFAULT_VET_REVIEW_ITEMS } from './constants'

type VetItem = ImportVetReview['items'][number]

/** 補登：獸醫師評比與意見。獸醫師以 VET 角色下拉 + 其他選擇。 */
export function VetReviewSection({
  value,
  onChange,
  users,
  usersLoading,
}: {
  value: ImportVetReview
  onChange: (v: ImportVetReview) => void
  users: UserOption[]
  usersLoading: boolean
}) {
  const items = value.items

  function updateItem(index: number, patch: Partial<VetItem>) {
    const next = items.map((item, i) => (i === index ? { ...item, ...patch } : item))
    onChange({ ...value, items: next })
  }

  function addItem() {
    onChange({ ...value, items: [...items, { item_name: '', compliance: '-', comment: '', pi_reply: '' }] })
  }

  function removeItem(index: number) {
    onChange({ ...value, items: items.filter((_, i) => i !== index) })
  }

  function loadTemplate() {
    if (items.length > 0 && !window.confirm('確定要載入標準範本嗎？這將會覆蓋目前已填寫的評比項目。')) {
      return
    }
    onChange({ ...value, items: DEFAULT_VET_REVIEW_ITEMS.map((item) => ({ ...item })) })
  }

  return (
    <div className="grid gap-3">
      <ReviewerSelect
        label="獸醫師"
        role={VET}
        users={users}
        value={{ reviewer_id: value.vet_id ?? null, reviewer_name: value.vet_name ?? '' }}
        onChange={(r) => onChange({ ...value, vet_id: r.reviewer_id, vet_name: r.reviewer_name })}
        disabled={usersLoading}
      />
      <div className="grid gap-1">
        <Label>評比結論（選填）</Label>
        <Input
          value={value.decision ?? ''}
          onChange={(e) => onChange({ ...value, decision: e.target.value })}
          placeholder="例：APPROVED / 同意"
        />
      </div>

      <div className="grid gap-2">
        <div className="flex items-center justify-between">
          <Label className="text-sm text-muted-foreground">評比項目</Label>
          <Button type="button" variant="outline" size="sm" onClick={loadTemplate}>
            <ClipboardList className="mr-1.5 h-3.5 w-3.5" />
            從標準範本填入
          </Button>
        </div>

        <div className="overflow-x-auto rounded-md border">
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="bg-muted text-muted-foreground font-semibold">
                <th className="p-2.5 text-left border-b w-8">#</th>
                <th className="p-2.5 text-left border-b">審查項目</th>
                <th className="p-2.5 text-center border-b w-32 whitespace-nowrap">符合性 (V/X/-)</th>
                <th className="p-2.5 text-left border-b">審查意見</th>
                <th className="p-2.5 border-b w-10" />
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {items.length === 0 ? (
                <tr>
                  <td colSpan={5} className="p-6 text-center text-muted-foreground">
                    尚無評比項目，請點選「從標準範本填入」或手動新增
                  </td>
                </tr>
              ) : (
                items.map((item, index) => (
                  <tr key={index} className="hover:bg-muted/30">
                    <td className="p-2.5 text-muted-foreground font-medium">{index + 1}</td>
                    <td className="p-2.5">
                      <Input
                        value={item.item_name}
                        onChange={(e) => updateItem(index, { item_name: e.target.value })}
                        placeholder="審查項目名稱"
                        className="border-0 shadow-none focus-visible:ring-1 bg-transparent"
                      />
                    </td>
                    <td className="p-2.5">
                      <Select
                        value={item.compliance}
                        onValueChange={(v) => updateItem(index, { compliance: v })}
                      >
                        <SelectTrigger className={`w-28 mx-auto font-bold ${
                          item.compliance === 'V'
                            ? 'text-status-success-text border-status-success-border bg-status-success-bg'
                            : item.compliance === 'X'
                              ? 'text-status-error-text border-status-error-border bg-status-error-bg'
                              : 'text-muted-foreground'
                        }`}>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="V" className="text-status-success-text">
                            <div className="flex items-center"><Check className="mr-2 h-4 w-4" />符合 (V)</div>
                          </SelectItem>
                          <SelectItem value="X" className="text-status-error-text">
                            <div className="flex items-center"><X className="mr-2 h-4 w-4" />不符 (X)</div>
                          </SelectItem>
                          <SelectItem value="-" className="text-muted-foreground">
                            <div className="flex items-center"><Minus className="mr-2 h-4 w-4" />不適用 (-)</div>
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </td>
                    <td className="p-2.5">
                      <Textarea
                        rows={2}
                        value={item.comment ?? ''}
                        onChange={(e) => updateItem(index, { comment: e.target.value })}
                        placeholder="獸醫師意見（選填）"
                        className="min-h-[60px] resize-y border-0 shadow-none focus-visible:ring-1 bg-transparent"
                      />
                    </td>
                    <td className="p-2.5">
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-destructive"
                        onClick={() => removeItem(index)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        <Button type="button" variant="outline" size="sm" className="w-fit" onClick={addItem}>
          <Plus className="mr-1.5 h-3.5 w-3.5" />
          新增項目
        </Button>
      </div>
    </div>
  )
}
