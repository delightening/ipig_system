// 4.1.7 實驗終點相關欄位元件（原 4.1.5）

import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/input'
import type { SectionProps } from '../types'

const DEFAULT_HUMANE_ENDPOINT = '實驗過程中如果動物體重下降超過原體重的20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，或其他經獸醫師評估不宜持續實驗之情形，則提早結束實驗，以符合動物福祉。'

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
        <div className="flex items-center justify-between">
          <Label>{t('aup.design.humaneEndpoint')} *</Label>
          {!endpoints.humane_endpoint.trim() && (
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => updateWorkingContent('design', 'endpoints.humane_endpoint', DEFAULT_HUMANE_ENDPOINT)}
            >
              {t('aup.design.insertDefaultHumaneEndpoint')}
            </Button>
          )}
        </div>
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
