// 4.1.4 飲食限制相關欄位元件

import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function RestrictionsSection({ formData, updateWorkingContent, t }: Props) {
  const { restrictions } = formData.working_content.design

  function handleRestrictionsToggle(value: string) {
    const isYes = value === 'yes'
    updateWorkingContent('design', 'restrictions.is_restricted', isYes as boolean | null)
    if (!isYes) {
      updateWorkingContent('design', 'restrictions.restriction_type', undefined)
      updateWorkingContent('design', 'restrictions.other_description', undefined)
    }
  }

  const restrictionSelectValue =
    restrictions.is_restricted === null ? '' :
    restrictions.is_restricted === true ? 'yes' : 'no'

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>{t('aup.design.restrictionsLabel')} *</Label>
        <Select value={restrictionSelectValue} onValueChange={handleRestrictionsToggle}>
          <SelectTrigger>
            <SelectValue placeholder={t('common.pleaseSelect')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="no">{t('common.no')}</SelectItem>
            <SelectItem value="yes">{t('common.yes')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {restrictions.is_restricted === true && (
        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
          <div className="space-y-2">
            <Label>{t('aup.design.selectRestrictionType')}</Label>
            <Select
              value={restrictions.restriction_type || ''}
              onValueChange={(value) => {
                updateWorkingContent('design', 'restrictions.restriction_type', value)
                if (value !== 'other') {
                  updateWorkingContent('design', 'restrictions.other_description', undefined)
                }
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('common.pleaseSelect')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="fasting_before_anesthesia">{t('aup.design.restrictionTypes.fasting')}</SelectItem>
                <SelectItem value="other">{t('aup.design.restrictionTypes.other')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {restrictions.restriction_type === 'other' && (
            <div className="space-y-2">
              <Label>{t('aup.design.explainRestrictionOther')}</Label>
              <Textarea
                value={restrictions.other_description || ''}
                onChange={(e) => updateWorkingContent('design', 'restrictions.other_description', e.target.value)}
                placeholder={t('aup.design.placeholders.explainRestrictionOther')}
                rows={3}
              />
            </div>
          )}
        </div>
      )}
    </div>
  )
}
