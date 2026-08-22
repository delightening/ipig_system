import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

import { PI_OTHER, userLabel, type UserOption } from './users'
import type { ReviewerValue } from './reviewer'

/**
 * 審查者選擇：依角色過濾的系統使用者下拉 + 「其他」（院外，自行填姓名）。
 * 選系統使用者 → reviewer_id；選「其他」→ 顯示姓名輸入框填 reviewer_name。
 * 預設（無 id）即「其他」狀態，可改選系統使用者。
 */
export function ReviewerSelect({
  label,
  role,
  users,
  value,
  onChange,
  disabled,
}: {
  label: string
  role: string
  users: UserOption[]
  value: ReviewerValue
  onChange: (v: ReviewerValue) => void
  disabled?: boolean
}) {
  const options = users.filter((u) => (u.roles ?? []).includes(role))
  const isOther = value.reviewer_id === null
  const selected = value.reviewer_id ?? PI_OTHER

  const onSelect = (v: string) => {
    if (v === PI_OTHER) onChange({ reviewer_id: null, reviewer_name: value.reviewer_name })
    else onChange({ reviewer_id: v, reviewer_name: '' })
  }

  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      <Select value={selected} onValueChange={onSelect} disabled={disabled}>
        <SelectTrigger>
          <SelectValue placeholder={disabled ? '載入中…' : '選擇'} />
        </SelectTrigger>
        <SelectContent>
          {options.map((u) => (
            <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
          ))}
          <SelectItem value={PI_OTHER}>其他（院外，自行填寫姓名）</SelectItem>
        </SelectContent>
      </Select>
      {isOther && (
        <Input
          value={value.reviewer_name}
          onChange={(e) => onChange({ reviewer_id: null, reviewer_name: e.target.value })}
          placeholder="院外審查者姓名"
        />
      )}
    </div>
  )
}
