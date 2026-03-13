// 4.4 危害性物質使用欄位元件

import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { FileUpload } from '@/components/ui/file-upload'
import { Plus } from 'lucide-react'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

export function HazardsSection({ formData, updateWorkingContent, t }: Props) {
  const { hazards } = formData.working_content.design

  const selectValue =
    hazards.used === null ? '' :
    hazards.used === true ? 'yes' : 'no'

  function handleToggle(value: string) {
    const isYes = value === 'yes'
    updateWorkingContent('design', 'hazards.used', isYes as boolean | null)
    if (!isYes) {
      updateWorkingContent('design', 'hazards.materials', [])
    }
  }

  function handleTypeChange(value: string) {
    updateWorkingContent('design', 'hazards.selected_type', value)
    const currentMaterials = hazards.materials.filter(m => m.type === value)
    updateWorkingContent('design', 'hazards.materials', currentMaterials)
  }

  function handleAddMaterial() {
    const materials = [...hazards.materials, {
      type: hazards.selected_type!,
      agent_name: '',
      amount: '',
      photos: []
    }]
    updateWorkingContent('design', 'hazards.materials', materials)
  }

  function handleRemoveMaterial(materialIndex: number) {
    const materials = [...hazards.materials]
    materials.splice(materialIndex, 1)
    updateWorkingContent('design', 'hazards.materials', materials)
  }

  function handleMaterialFieldChange(materialIndex: number, field: string, value: string) {
    const materials = [...hazards.materials]
    ;(materials[materialIndex] as Record<string, unknown>)[field] = value
    updateWorkingContent('design', 'hazards.materials', materials)
  }

  const filteredMaterials = hazards.materials.filter(
    m => m.type === hazards.selected_type
  )

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <h3 className="font-semibold">{t('aup.design.hazardsLabel')} *</h3>
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

      {hazards.used === true && (
        <div className="space-y-4 mt-2 pl-6 border-l-2 border-slate-200">
          <div className="space-y-3">
            <div className="space-y-2">
              <Label className="text-sm font-medium">{t('aup.design.selectHazardType')}</Label>
              <Select value={hazards.selected_type || ''} onValueChange={handleTypeChange}>
                <SelectTrigger>
                  <SelectValue placeholder={t('aup.design.selectHazardTypePlaceholder')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="biological">{t('aup.design.hazardTypes.biological')}</SelectItem>
                  <SelectItem value="radioactive">{t('aup.design.hazardTypes.radioactive')}</SelectItem>
                  <SelectItem value="chemical">{t('aup.design.hazardTypes.chemical')}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {hazards.selected_type && (
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <Label className="text-sm font-medium">
                    {hazards.selected_type === 'biological' && t('aup.design.hazardTypes.biological')}
                    {hazards.selected_type === 'radioactive' && t('aup.design.hazardTypes.radioactive')}
                    {hazards.selected_type === 'chemical' && t('aup.design.hazardTypes.chemical')}
                  </Label>
                  <Button variant="outline" size="sm" onClick={handleAddMaterial}>
                    <Plus className="h-4 w-4 mr-1" />
                    {t('aup.items.add')}
                  </Button>
                </div>
                {filteredMaterials.map((material, _index) => {
                  const materialIndex = hazards.materials.findIndex(m => m === material)
                  return (
                    <div key={materialIndex} className="space-y-3 relative p-3 border rounded bg-slate-50">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="absolute right-2 top-2 h-6 w-6 text-red-500"
                        onClick={() => handleRemoveMaterial(materialIndex)}
                      >
                        X
                      </Button>
                      <div className="grid grid-cols-2 gap-3">
                        <Input
                          placeholder={t('aup.design.agentNamePlaceholder')}
                          value={material.agent_name}
                          onChange={(e) => handleMaterialFieldChange(materialIndex, 'agent_name', e.target.value)}
                        />
                        <Input
                          placeholder={t('aup.design.amountPlaceholder')}
                          value={material.amount}
                          onChange={(e) => handleMaterialFieldChange(materialIndex, 'amount', e.target.value)}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label className="text-sm">{t('aup.items.photos')}</Label>
                        <FileUpload
                          value={material.photos || []}
                          onChange={(photos) => {
                            const materials = [...hazards.materials]
                            materials[materialIndex].photos = photos
                            updateWorkingContent('design', 'hazards.materials', materials)
                          }}
                          accept="image/*"
                          multiple={true}
                          maxSize={10}
                          maxFiles={10}
                          placeholder={t('aup.items.placeholders.photos')}
                        />
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
