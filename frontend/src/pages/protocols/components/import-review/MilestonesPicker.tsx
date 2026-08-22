import { useState } from 'react'
import { Plus, X } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

import { MILESTONES, type MilestoneKey, type MilestoneState } from './milestones'

/**
 * 補登審查里程碑日期格。必填里程碑（標 *）恆顯示；
 * 選填里程碑（補件 / 修訂退回、委員第二次審查）預設隱藏，按「新增」後才出現
 * （並非每件計劃都發生這些階段）。手機單欄避免日期框 overflow。
 */
export function MilestonesPicker({
  value,
  onChange,
}: {
  value: MilestoneState
  onChange: (updater: (s: MilestoneState) => MilestoneState) => void
}) {
  // 已加入的選填里程碑：初始含已有值者（編輯既有資料時）
  const [added, setAdded] = useState<Set<MilestoneKey>>(
    () => new Set(MILESTONES.filter((m) => !m.required && value[m.key]).map((m) => m.key)),
  )

  const visible = MILESTONES.filter((m) => m.required || added.has(m.key) || !!value[m.key])
  const addable = MILESTONES.filter((m) => !m.required && !added.has(m.key) && !value[m.key])

  const addMilestone = (key: MilestoneKey) => setAdded((s) => new Set(s).add(key))
  const removeMilestone = (key: MilestoneKey) => {
    setAdded((s) => {
      const next = new Set(s)
      next.delete(key)
      return next
    })
    onChange((s) => ({ ...s, [key]: '' }))
  }

  return (
    <div className="grid gap-2">
      <Label>審查里程碑日期</Label>
      <p className="text-sm text-muted-foreground">依時序填寫歷史日期（標 * 為必填）；補件 / 委員二審等非必然階段請按下方按鈕新增。</p>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        {visible.map((m) => (
          <div key={m.key} className="grid gap-2 min-w-0">
            <div className="flex items-center justify-between gap-2">
              <Label className="text-sm font-normal text-muted-foreground">{m.label}{m.required ? ' *' : ''}</Label>
              {!m.required && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeMilestone(m.key)}
                  className="h-auto px-1 py-0 text-xs text-muted-foreground hover:text-foreground"
                >
                  <X className="h-3 w-3 mr-0.5" />移除
                </Button>
              )}
            </div>
            <Input
              type="date"
              className="w-full"
              value={value[m.key]}
              onChange={(e) => onChange((s) => ({ ...s, [m.key]: e.target.value }))}
            />
          </div>
        ))}
      </div>
      {addable.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {addable.map((m) => (
            <Button key={m.key} type="button" variant="outline" size="sm" onClick={() => addMilestone(m.key)}>
              <Plus className="h-4 w-4 mr-1" />新增「{m.label}」
            </Button>
          ))}
        </div>
      )}
    </div>
  )
}
