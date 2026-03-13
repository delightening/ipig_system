// 4.1.5 實驗終點相關欄位元件

import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/input'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function EndpointsSection({ formData, updateWorkingContent, t }: Props) {
  const { endpoints } = formData.working_content.design

  return (
    <div className="space-y-4">
      <Label>{t('aup.design.endpointsTitle')}</Label>
      <div className="space-y-2">
        <Label>{t('aup.design.experimentalEndpoint')} *</Label>
        <Textarea
          value={endpoints.experimental_endpoint}
          onChange={(e) => updateWorkingContent('design', 'endpoints.experimental_endpoint', e.target.value)}
          placeholder={t('aup.design.placeholders.experimentalEndpoint')}
          rows={3}
        />
      </div>
      <div className="space-y-2">
        <Label>{t('aup.design.humaneEndpoint')} *</Label>
        <Textarea
          value={endpoints.humane_endpoint}
          onChange={(e) => updateWorkingContent('design', 'endpoints.humane_endpoint', e.target.value)}
          placeholder={t('aup.design.placeholders.humaneEndpoint')}
          rows={4}
        />
      </div>
    </div>
  )
}
