import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

import type { ExternalPiData } from './externalPi'

export function ExternalPiFields({
  value,
  onChange,
}: {
  value: ExternalPiData
  onChange: (v: ExternalPiData) => void
}) {
  const { t } = useTranslation()
  const set = (patch: Partial<ExternalPiData>) => onChange({ ...value, ...patch })
  return (
    <div className="grid gap-3 rounded-md border border-dashed p-3">
      <div className="grid gap-2">
        <Label>{t('protocolPages.importReview.externalPi.name')}</Label>
        <Input value={value.piName} onChange={(e) => set({ piName: e.target.value })} placeholder={t('protocolPages.importReview.externalPi.namePlaceholder')} />
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div className="grid gap-2 min-w-0">
          <Label>PI Email *</Label>
          <Input type="email" value={value.piEmail} onChange={(e) => set({ piEmail: e.target.value })} placeholder={t('protocolPages.importReview.externalPi.emailPlaceholder')} />
        </div>
        <div className="grid gap-2 min-w-0">
          <Label>{t('protocolPages.importReview.externalPi.phone')}</Label>
          <Input type="tel" value={value.piPhone} onChange={(e) => set({ piPhone: e.target.value })} placeholder={t('protocolPages.importReview.externalPi.phonePlaceholder')} />
        </div>
      </div>
      <p className="text-xs text-muted-foreground">{t('protocolPages.importReview.externalPi.sponsorHint')}</p>
    </div>
  )
}
