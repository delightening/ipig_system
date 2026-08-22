// 4.4 危害性物質使用欄位元件 — multi-select：可同時勾選生物 / 放射 / 化學

import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { FileUpload } from '@/components/ui/file-upload'
import { Plus } from 'lucide-react'
import type { SectionProps } from '../types'

type Props = Pick<SectionProps, 'formData' | 'updateWorkingContent' | 't'>

type HazardType = 'biological' | 'radioactive' | 'chemical'
const HAZARD_TYPES: HazardType[] = ['biological', 'radioactive', 'chemical']

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

  function newMaterial(type: HazardType) {
    return { _uid: crypto.randomUUID(), type, agent_name: '', amount: '', photos: [] }
  }

  function toggleType(type: HazardType, checked: boolean) {
    if (checked) {
      // Add an empty material of this type to "enable" the section
      updateWorkingContent('design', 'hazards.materials', [...hazards.materials, newMaterial(type)])
    } else {
      // Remove all materials of this type
      const materials = hazards.materials.filter(m => m.type !== type)
      updateWorkingContent('design', 'hazards.materials', materials)
    }
  }

  function handleAddMaterial(type: HazardType) {
    updateWorkingContent('design', 'hazards.materials', [...hazards.materials, newMaterial(type)])
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
        <div className="space-y-4 mt-2 pl-6 border-l-2 border-border">
          <div>
            <Label className="text-sm font-medium">{t('aup.design.selectHazardType')}（{t('aup.design.hazardMultiSelectHint')}）</Label>
            <div className="flex flex-wrap gap-4 mt-2">
              {HAZARD_TYPES.map(type => {
                const hasType = hazards.materials.some(m => m.type === type)
                return (
                  <Checkbox
                    key={type}
                    label={t(`aup.design.hazardTypes.${type}`)}
                    checked={hasType}
                    onCheckedChange={(checked) => toggleType(type, checked)}
                  />
                )
              })}
            </div>
          </div>

          {HAZARD_TYPES.map(type => {
            const typeMaterials = hazards.materials.filter(m => m.type === type)
            if (typeMaterials.length === 0) return null
            return (
              <div key={type} className="space-y-3 pl-4 border-l-2 border-purple-200">
                <div className="flex justify-between items-center">
                  <Label className="text-sm font-medium">
                    {t(`aup.design.hazardTypes.${type}`)}
                  </Label>
                  <Button type="button" variant="outline" onClick={() => handleAddMaterial(type)}>
                    <Plus className="h-4 w-4 mr-1" />
                    {t('aup.items.add')}
                  </Button>
                </div>
                {typeMaterials.map(material => {
                  const materialIndex = hazards.materials.findIndex(m => m === material)
                  // Key on _uid (stable across deletes); fallback to index for legacy data without _uid
                  const stableKey = material._uid ?? `idx-${materialIndex}`
                  return (
                    <div key={stableKey} className="space-y-3 relative p-3 border rounded bg-muted">
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        className="absolute right-2 top-2 h-6 w-6 text-destructive"
                        aria-label="刪除"
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
            )
          })}
        </div>
      )}
    </div>
  )
}
