import { useTranslation } from 'react-i18next'

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
  const { t } = useTranslation()
  return (
    <div className="grid gap-2">
      <Label>{t('protocolPages.importReview.piSelector.label')}</Label>
      <Select value={piUserId} onValueChange={onPiChange} disabled={usersLoading}>
        <SelectTrigger>
          <SelectValue placeholder={usersLoading ? t('protocolPages.shared.loadingEllipsis') : t('protocolPages.importReview.piSelector.placeholder')} />
        </SelectTrigger>
        <SelectContent>
          {piOptions.map((u) => (
            <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
          ))}
          <SelectItem value={PI_OTHER}>{t('protocolPages.importReview.piSelector.other')}</SelectItem>
        </SelectContent>
      </Select>
      {isExternalPi && <ExternalPiFields value={externalPi} onChange={onExternalPiChange} />}
    </div>
  )
}
