// 4.1.6 動物最終處置方式元件

import { Label } from '@/components/ui/label'
import { Input, Textarea } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function FinalHandlingSection({ formData, updateWorkingContent, t }: Props) {
  const { final_handling } = formData.working_content.design

  function handleMethodChange(value: string) {
    updateWorkingContent('design', 'final_handling.method', value)
    if (value !== 'euthanasia') {
      updateWorkingContent('design', 'final_handling.euthanasia_type', undefined)
      updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
    }
    if (value !== 'transfer') {
      updateWorkingContent('design', 'final_handling.transfer.recipient_name', '')
      updateWorkingContent('design', 'final_handling.transfer.recipient_org', '')
      updateWorkingContent('design', 'final_handling.transfer.project_name', '')
    }
    if (value !== 'other') {
      updateWorkingContent('design', 'final_handling.other_description', undefined)
    }
  }

  return (
    <div className="space-y-4">
      <Label>{t('aup.design.finalHandlingTitle')} *</Label>
      <div className="space-y-2">
        <Select value={final_handling.method || ''} onValueChange={handleMethodChange}>
          <SelectTrigger>
            <SelectValue placeholder={t('aup.design.selectHandlingMethod')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="euthanasia">{t('aup.design.handlingMethods.euthanasia')}</SelectItem>
            <SelectItem value="transfer">{t('aup.design.handlingMethods.transfer')}</SelectItem>
            <SelectItem value="other">{t('aup.design.handlingMethods.other')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {final_handling.method === 'euthanasia' && (
        <div className="space-y-3 border-l-2 border-slate-200 pl-6">
          <Label className="text-sm font-medium">{t('aup.design.euthanasiaLabel')}</Label>
          <Select
            value={final_handling.euthanasia_type || ''}
            onValueChange={(value) => {
              updateWorkingContent('design', 'final_handling.euthanasia_type', value)
              if (value !== 'other') {
                updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
              }
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('common.pleaseSelect')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="kcl">{t('aup.design.euthanasiaTypes.kcl')}</SelectItem>
              <SelectItem value="electrocution">{t('aup.design.euthanasiaTypes.electrocution')}</SelectItem>
              <SelectItem value="other">{t('aup.design.euthanasiaTypes.other')}</SelectItem>
            </SelectContent>
          </Select>
          {final_handling.euthanasia_type === 'other' && (
            <div className="space-y-2 mt-2">
              <Textarea
                value={final_handling.euthanasia_other_description || ''}
                onChange={(e) => updateWorkingContent('design', 'final_handling.euthanasia_other_description', e.target.value)}
                placeholder={t('aup.design.placeholders.euthanasiaOther')}
                rows={3}
              />
            </div>
          )}
        </div>
      )}

      {final_handling.method === 'transfer' && (
        <div className="space-y-3 border-l-2 border-slate-200 pl-6">
          <Label className="text-sm font-medium">{t('aup.design.transferLabel')}</Label>
          <div className="space-y-3">
            <div className="space-y-2">
              <Label className="text-sm">{t('aup.design.recipientName')}</Label>
              <Input
                value={final_handling.transfer.recipient_name}
                onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_name', e.target.value)}
                placeholder={t('aup.design.recipientNamePlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label className="text-sm">{t('aup.design.recipientOrg')}</Label>
              <Input
                value={final_handling.transfer.recipient_org}
                onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_org', e.target.value)}
                placeholder={t('aup.design.recipientOrgPlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label className="text-sm">{t('aup.design.projectName')}</Label>
              <Input
                value={final_handling.transfer.project_name}
                onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.project_name', e.target.value)}
                placeholder={t('aup.design.projectNamePlaceholder')}
              />
            </div>
          </div>
        </div>
      )}

      {final_handling.method === 'other' && (
        <div className="space-y-3 border-l-2 border-slate-200 pl-6">
          <Label className="text-sm font-medium">{t('aup.design.handlingMethods.other')}: </Label>
          <Textarea
            value={final_handling.other_description || ''}
            onChange={(e) => updateWorkingContent('design', 'final_handling.other_description', e.target.value)}
            placeholder={t('aup.design.otherHandlingPlaceholder')}
            rows={3}
          />
        </div>
      )}
    </div>
  )
}
