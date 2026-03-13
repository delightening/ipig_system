// 4.3 非藥用等級化學藥品欄位元件

import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function NonPharmaSection({ formData, updateWorkingContent, t }: Props) {
  const { non_pharma_grade } = formData.working_content.design

  const selectValue =
    non_pharma_grade.used === null ? '' :
    non_pharma_grade.used === true ? 'yes' : 'no'

  function handleToggle(value: string) {
    const isYes = value === 'yes'
    updateWorkingContent('design', 'non_pharma_grade.used', isYes as boolean | null)
    if (!isYes) {
      updateWorkingContent('design', 'non_pharma_grade.description', '')
    }
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <h3 className="font-semibold">{t('aup.design.nonPharmaLabel')} *</h3>
        <Select value={selectValue} onValueChange={handleToggle}>
          <SelectTrigger>
            <SelectValue placeholder={t('common.pleaseSelect')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="no">{t('common.no')}</SelectItem>
            <SelectItem value="yes">{t('common.yes')}</SelectItem>
          </SelectContent>
        </Select>
        {non_pharma_grade.used === true && (
          <div className="space-y-2 mt-2">
            <Label>{t('aup.design.nonPharmaExplain')}</Label>
            <Textarea
              value={non_pharma_grade.description}
              onChange={(e) => updateWorkingContent('design', 'non_pharma_grade.description', e.target.value)}
              rows={4}
            />
          </div>
        )}
      </div>
    </div>
  )
}
