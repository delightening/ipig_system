// 管制藥品使用欄位元件（4.5 / 4.6）

import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { FileUpload } from '@/components/ui/file-upload'
import { Plus } from 'lucide-react'
import type { SectionProps } from '../types'

interface Props extends Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'> {
  labelKey: string
}

export function ControlledSubstancesSection({ formData, updateWorkingContent, t, labelKey }: Props) {
  const { controlled_substances } = formData.working_content.design

  const selectValue =
    controlled_substances.used === null ? '' :
    controlled_substances.used === true ? 'yes' : 'no'

  function handleToggle(value: string) {
    const isYes = value === 'yes'
    updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
    if (!isYes) {
      updateWorkingContent('design', 'controlled_substances.items', [])
    }
  }

  function handleAddItem() {
    const items = [...controlled_substances.items, {
      drug_name: '',
      approval_no: '',
      amount: '',
      authorized_person: '',
      photos: []
    }]
    updateWorkingContent('design', 'controlled_substances.items', items)
  }

  function handleRemoveItem(index: number) {
    const items = [...controlled_substances.items]
    items.splice(index, 1)
    updateWorkingContent('design', 'controlled_substances.items', items)
  }

  function handleItemChange(index: number, field: string, value: string) {
    const items = [...controlled_substances.items]
    ;(items[index] as Record<string, unknown>)[field] = value
    updateWorkingContent('design', 'controlled_substances.items', items)
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label>{t(labelKey)}</Label>
        <Select value={selectValue} onValueChange={handleToggle}>
          <SelectTrigger>
            <SelectValue placeholder={t('common.pleaseSelect')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="no">{t('common.no')}</SelectItem>
            <SelectItem value="yes">{t('common.yes')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {controlled_substances.used === true && (
        <div className="space-y-4 pl-6 border-l-2 border-border">
          <div className="flex justify-between items-center">
            <Label className="text-sm font-medium">{t('aup.design.controlledSubstancesList')}</Label>
            <Button variant="outline" onClick={handleAddItem}>
              <Plus className="h-4 w-4 mr-1" />
              {t('aup.items.add')}
            </Button>
          </div>
          {controlled_substances.items.map((item, index) => (
            <div key={index} className="space-y-3 relative p-3 border rounded bg-muted">
              <Button
                variant="ghost"
                size="icon"
                className="absolute right-2 top-2 h-6 w-6 text-destructive"
                aria-label="刪除"
                onClick={() => handleRemoveItem(index)}
              >
                X
              </Button>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.drugNameLabel')}</Label>
                  <Input
                    value={item.drug_name}
                    onChange={(e) => handleItemChange(index, 'drug_name', e.target.value)}
                    placeholder={t('aup.design.drugNamePlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.approvalNoLabel')}</Label>
                  <Input
                    value={item.approval_no}
                    onChange={(e) => handleItemChange(index, 'approval_no', e.target.value)}
                    placeholder={t('aup.design.approvalNoPlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.drugAmountLabel')}</Label>
                  <Input
                    value={item.amount}
                    onChange={(e) => handleItemChange(index, 'amount', e.target.value)}
                    placeholder={t('aup.design.drugAmountPlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.authorizedPersonLabel')}</Label>
                  <Input
                    value={item.authorized_person}
                    onChange={(e) => handleItemChange(index, 'authorized_person', e.target.value)}
                    placeholder={t('aup.design.authorizedPersonPlaceholder')}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label className="text-sm">{t('aup.items.photos')}</Label>
                <FileUpload
                  value={item.photos || []}
                  onChange={(photos) => {
                    const items = [...controlled_substances.items]
                    items[index].photos = photos
                    updateWorkingContent('design', 'controlled_substances.items', items)
                  }}
                  accept="image/*"
                  multiple={true}
                  maxSize={10}
                  maxFiles={10}
                  placeholder={t('aup.items.photosPlaceholder')}
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
