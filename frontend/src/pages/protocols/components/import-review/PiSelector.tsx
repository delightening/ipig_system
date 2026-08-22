import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

import { ExternalPiFields } from './ExternalPiFields'
import type { ExternalPiData } from './externalPi'
import { PI_OTHER, userLabel, type UserOption } from './users'

/** 計畫主持人選擇：系統內 PI（已排除試驗工作人員）或「其他」外部 PI。 */
export function PiSelector({
  piUserId,
  onPiChange,
  usersLoading,
  piOptions,
  isExternalPi,
  externalPi,
  onExternalPiChange,
}: {
  piUserId: string
  onPiChange: (v: string) => void
  usersLoading: boolean
  piOptions: UserOption[]
  isExternalPi: boolean
  externalPi: ExternalPiData
  onExternalPiChange: (v: ExternalPiData) => void
}) {
  return (
    <div className="grid gap-2">
      <Label>計畫主持人（PI）*</Label>
      <Select value={piUserId} onValueChange={onPiChange} disabled={usersLoading}>
        <SelectTrigger>
          <SelectValue placeholder={usersLoading ? '載入中…' : '選擇 PI'} />
        </SelectTrigger>
        <SelectContent>
          {piOptions.map((u) => (
            <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
          ))}
          <SelectItem value={PI_OTHER}>其他（外部 PI，非系統使用者）</SelectItem>
        </SelectContent>
      </Select>
      {isExternalPi && <ExternalPiFields value={externalPi} onChange={onExternalPiChange} />}
    </div>
  )
}
