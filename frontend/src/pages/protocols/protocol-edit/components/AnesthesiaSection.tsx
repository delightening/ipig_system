// 4.1.1 麻醉相關欄位元件

import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ANESTHESIA_TYPE_OPTIONS } from '@/lib/constants/protocolDesignOptions'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function AnesthesiaSection({ formData, updateWorkingContent, t }: Props) {
  const { anesthesia } = formData.working_content.design
  const { surgery } = formData.working_content

  function handleAnesthesiaToggle(value: string) {
    const isYes = value === 'yes'
    updateWorkingContent('design', 'anesthesia.is_under_anesthesia', isYes as boolean | null)
    if (!isYes) {
      updateWorkingContent('design', 'anesthesia.anesthesia_type', undefined)
      updateWorkingContent('design', 'anesthesia.other_description', undefined)
      const naText = 'N/A'
      updateWorkingContent('surgery', 'surgery_type', naText)
      updateWorkingContent('surgery', 'preop_preparation', naText)
      updateWorkingContent('surgery', 'surgery_description', naText)
      updateWorkingContent('surgery', 'monitoring', naText)
      updateWorkingContent('surgery', 'postop_expected_impact', naText)
      updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
      updateWorkingContent('surgery', 'postop_care', naText)
      updateWorkingContent('surgery', 'postop_care_type', undefined)
      updateWorkingContent('surgery', 'expected_end_point', naText)
      updateWorkingContent('surgery', 'drugs', [])
      updateWorkingContent('surgery', 'aseptic_techniques', [])
    }
  }

  function handleAnesthesiaTypeChange(value: string) {
    updateWorkingContent('design', 'anesthesia.anesthesia_type', value)
    if (value === 'survival_surgery' || value === 'non_survival_surgery') {
      const surgeryType = value === 'survival_surgery' ? 'survival' : 'non_survival'
      updateWorkingContent('surgery', 'surgery_type', surgeryType)
      if (!surgery.preop_preparation || surgery.preop_preparation === '略' || surgery.preop_preparation === t('common.na')) {
        updateWorkingContent('surgery', 'preop_preparation', t('aup.design.templates.preop_preparation'))
      }
      if (!surgery.surgery_description || surgery.surgery_description === '略' || surgery.surgery_description === t('common.na')) {
        updateWorkingContent('surgery', 'surgery_description', t('aup.design.templates.surgery_description'))
      }
      if (!surgery.monitoring) {
        updateWorkingContent('surgery', 'monitoring', t('aup.design.templates.monitoring'))
      }
      updateWorkingContent('surgery', 'aseptic_techniques', [])
    } else if (value && value !== '') {
      const naText = 'N/A'
      updateWorkingContent('surgery', 'surgery_type', naText)
      updateWorkingContent('surgery', 'preop_preparation', naText)
      updateWorkingContent('surgery', 'surgery_description', naText)
      updateWorkingContent('surgery', 'monitoring', naText)
      updateWorkingContent('surgery', 'postop_expected_impact', naText)
      updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
      updateWorkingContent('surgery', 'postop_care', naText)
      updateWorkingContent('surgery', 'postop_care_type', undefined)
      updateWorkingContent('surgery', 'expected_end_point', naText)
      updateWorkingContent('surgery', 'drugs', [])
      updateWorkingContent('surgery', 'aseptic_techniques', [])
    }
  }

  const anesthesiaSelectValue =
    anesthesia.is_under_anesthesia === null ? '' :
    anesthesia.is_under_anesthesia === true ? 'yes' : 'no'

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>{t('aup.design.anesthesiaLabel')} *</Label>
        <Select value={anesthesiaSelectValue} onValueChange={handleAnesthesiaToggle}>
          <SelectTrigger>
            <SelectValue placeholder={t('common.pleaseSelect')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="no">{t('common.no')}</SelectItem>
            <SelectItem value="yes">{t('common.yes')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {anesthesia.is_under_anesthesia === true && (
        <div className="space-y-4 pl-6 border-l-2 border-border">
          <div className="space-y-2">
            <Label>{t('aup.design.selectAnesthesiaType')}</Label>
            <Select value={anesthesia.anesthesia_type || ''} onValueChange={handleAnesthesiaTypeChange}>
              <SelectTrigger>
                <SelectValue placeholder={t('common.pleaseSelect')} />
              </SelectTrigger>
              <SelectContent>
                {ANESTHESIA_TYPE_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>{t(opt.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      )}
    </div>
  )
}
