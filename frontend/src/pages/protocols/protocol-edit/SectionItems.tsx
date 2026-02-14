// Section Items 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import { Plus } from 'lucide-react'
import type { SectionProps } from './types'

export function SectionItems({ formData, updateWorkingContent, setFormData, t, isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section3')}</CardTitle>
        <CardDescription>{t('aup.items.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <Label>{t('aup.items.useTestItemLabel')} *</Label>
          <Select
            value={formData.working_content.items.use_test_item === null ? '' : (formData.working_content.items.use_test_item ? 'yes' : 'no')}
            onValueChange={(value) => {
              const isYes = value === 'yes'
              updateWorkingContent('items', 'use_test_item', isYes)
              // If "No" is selected, clear the substance list
              if (!isYes) {
                updateWorkingContent('items', 'test_items', [])
                updateWorkingContent('items', 'control_items', [])
              }
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder={t('common.pleaseSelect')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="no">{t('common.no')}</SelectItem>
              <SelectItem value="yes">{t('common.yes')}</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {formData.working_content.items.use_test_item === false && (
          <p className="text-muted-foreground italic">{t('aup.items.skipped')}</p>
        )}

        {formData.working_content.items.use_test_item === true && (
          <>
            {/* Test Items */}
            <div className="space-y-4 border p-4 rounded-md">
              <div className="flex justify-between items-center">
                <h3 className="font-semibold">{t('aup.items.testItems')}</h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    const newItems = [...formData.working_content.items.test_items, {
                      name: '', is_sterile: true, purpose: '', storage_conditions: '', photos: []
                    }]
                    updateWorkingContent('items', 'test_items', newItems)
                  }}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  {t('aup.items.add')}
                </Button>
              </div>
              {formData.working_content.items.test_items.map((item, index) => (
                <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="absolute right-2 top-2 h-6 w-6 text-red-500"
                    onClick={() => {
                      const newItems = [...formData.working_content.items.test_items]
                      newItems.splice(index, 1)
                      updateWorkingContent('items', 'test_items', newItems)
                    }}
                  >
                    X
                  </Button>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>{t('aup.items.itemName')} *</Label>
                      <Input
                        value={item.name}
                        onChange={(e) => {
                          const newItems = [...formData.working_content.items.test_items]
                          newItems[index].name = e.target.value
                          updateWorkingContent('items', 'test_items', newItems)
                        }}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.items.dosageForm')} *</Label>
                      <Input
                        value={item.form || ''}
                        onChange={(e) => {
                          const newItems = [...formData.working_content.items.test_items]
                          newItems[index].form = e.target.value
                          updateWorkingContent('items', 'test_items', newItems)
                        }}
                        placeholder={t('aup.items.placeholders.dosageForm')}
                      />
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.items.purpose')} *</Label>
                    <Input
                      value={item.purpose}
                      onChange={(e) => {
                        const newItems = [...formData.working_content.items.test_items]
                        newItems[index].purpose = e.target.value
                        updateWorkingContent('items', 'test_items', newItems)
                      }}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.items.storageConditions')} *</Label>
                    <Input
                      value={item.storage_conditions || ''}
                      onChange={(e) => {
                        const newItems = [...formData.working_content.items.test_items]
                        newItems[index].storage_conditions = e.target.value
                        updateWorkingContent('items', 'test_items', newItems)
                      }}
                      placeholder={t('aup.items.storageConditionsPlaceholder')}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.items.isSterile')} *</Label>
                    <Select
                      value={item.is_sterile ? 'yes' : 'no'}
                      onValueChange={(value) => {
                        const newItems = [...formData.working_content.items.test_items]
                        const isYes = value === 'yes'
                        newItems[index].is_sterile = isYes
                        // If "Yes" is selected, clear the explanation field
                        if (isYes) {
                          newItems[index].non_sterile_justification = ''
                        }
                        updateWorkingContent('items', 'test_items', newItems)
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {!item.is_sterile && (
                      <div className="space-y-2 mt-2">
                        <Label>{t('aup.items.nonSterileJustification')} *</Label>
                        <Textarea
                          value={item.non_sterile_justification || ''}
                          onChange={(e) => {
                            const newItems = [...formData.working_content.items.test_items]
                            newItems[index].non_sterile_justification = e.target.value
                            updateWorkingContent('items', 'test_items', newItems)
                          }}
                          placeholder={t('aup.items.placeholders.nonSterile')}
                          rows={3}
                        />
                      </div>
                    )}
                  </div>
                  {/* Photos Upload */}
                  <div className="space-y-2">
                    <Label>{t('aup.items.photos')}</Label>
                    <FileUpload
                      value={item.photos || []}
                      onChange={(photos) => {
                        const newItems = [...formData.working_content.items.test_items]
                        newItems[index].photos = photos
                        updateWorkingContent('items', 'test_items', newItems)
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

            {/* Control Items */}
            <div className="space-y-4 border p-4 rounded-md">
              <div className="flex justify-between items-center">
                <h3 className="font-semibold">{t('aup.items.controlItems')}</h3>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    const newControls = [...formData.working_content.items.control_items, {
                      name: '', is_sterile: true, purpose: '', storage_conditions: '', photos: []
                    }]
                    updateWorkingContent('items', 'control_items', newControls)
                  }}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  {t('aup.items.add')}
                </Button>
              </div>
              {formData.working_content.items.control_items.map((item, index) => (
                <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="absolute right-2 top-2 h-6 w-6 text-red-500"
                    onClick={() => {
                      const newControls = [...formData.working_content.items.control_items]
                      newControls.splice(index, 1)
                      updateWorkingContent('items', 'control_items', newControls)
                    }}
                  >
                    X
                  </Button>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label>{t('aup.items.controlName')} *</Label>
                      <Input
                        value={item.name}
                        onChange={(e) => {
                          const newControls = [...formData.working_content.items.control_items]
                          newControls[index].name = e.target.value
                          updateWorkingContent('items', 'control_items', newControls)
                        }}
                        placeholder={t('aup.items.placeholders.controlName')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.items.purpose')} *</Label>
                      <Input
                        value={item.purpose}
                        onChange={(e) => {
                          const newControls = [...formData.working_content.items.control_items]
                          newControls[index].purpose = e.target.value
                          updateWorkingContent('items', 'control_items', newControls)
                        }}
                      />
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.items.storageConditions')} *</Label>
                    <Input
                      value={item.storage_conditions || ''}
                      onChange={(e) => {
                        const newControls = [...formData.working_content.items.control_items]
                        newControls[index].storage_conditions = e.target.value
                        updateWorkingContent('items', 'control_items', newControls)
                      }}
                      placeholder={t('aup.items.storageConditionsPlaceholder')}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.items.isSterile')} *</Label>
                    <Select
                      value={item.is_sterile ? 'yes' : 'no'}
                      onValueChange={(value) => {
                        const newControls = [...formData.working_content.items.control_items]
                        const isYes = value === 'yes'
                        newControls[index].is_sterile = isYes
                        // If "Yes" is selected, clear the explanation field
                        if (isYes) {
                          newControls[index].non_sterile_justification = ''
                        }
                        updateWorkingContent('items', 'control_items', newControls)
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {!item.is_sterile && (
                      <div className="space-y-2 mt-2">
                        <Label>{t('aup.items.nonSterileJustification')} *</Label>
                        <Textarea
                          value={item.non_sterile_justification || ''}
                          onChange={(e) => {
                            const newControls = [...formData.working_content.items.control_items]
                            newControls[index].non_sterile_justification = e.target.value
                            updateWorkingContent('items', 'control_items', newControls)
                          }}
                          placeholder={t('aup.items.placeholders.nonSterile')}
                          rows={3}
                        />
                      </div>
                    )}
                  </div>
                  {/* Photos Upload */}
                  <div className="space-y-2">
                    <Label>{t('aup.items.photos')}</Label>
                    <FileUpload
                      value={item.photos || []}
                      onChange={(photos) => {
                        const newControls = [...formData.working_content.items.control_items]
                        newControls[index].photos = photos
                        updateWorkingContent('items', 'control_items', newControls)
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
          </>
        )}
      </CardContent>
    </Card>
  )
}
