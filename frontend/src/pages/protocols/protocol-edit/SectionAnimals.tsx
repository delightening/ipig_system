// Section Animals 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Button } from '@/components/ui/button'
import type { ProtocolAnimalItem } from '@/types/protocol'
import type { SectionProps } from './types'

export function SectionAnimals({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section7')}</CardTitle>
        <CardDescription>{t('aup.animals.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4 border p-4 rounded-md">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">{t('aup.animals.listHeader')}</h3>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                const currentList = formData.working_content.animals.animals || []
                const newAnimals = [...currentList, {
                  species: '' as '' | 'pig' | 'other',
                  species_other: '',
                  strain: undefined,
                  strain_other: '',
                  sex: '',
                  number: 0,
                  age_min: undefined,
                  age_max: undefined,
                  age_unlimited: false,
                  weight_min: undefined,
                  weight_max: undefined,
                  weight_unlimited: false,
                  housing_location: ''
                }]
                updateWorkingContent('animals', 'animals', newAnimals)
              }}
            >
              + {t('aup.animals.addAnimal')}
            </Button>
          </div>
          {/* Helper to update entire animals array */}
          {(formData.working_content.animals.animals || []).map((animal: ProtocolAnimalItem, index: number) => (
            <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50 mb-4">
              <Button
                variant="ghost"
                size="icon"
                className="absolute right-2 top-2 h-6 w-6 text-red-500"
                onClick={() => {
                  const newAnimals = [...formData.working_content.animals.animals]
                  newAnimals.splice(index, 1)
                  updateWorkingContent('animals', 'animals', newAnimals)
                }}
              >
                X
              </Button>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>{t('aup.animals.labels.species')} *</Label>
                  <Select
                    value={animal.species || ''}
                    onValueChange={(value) => {
                      const newAnimals = [...formData.working_content.animals.animals]
                      newAnimals[index].species = value as 'pig' | 'other'
                      if (value !== 'other') {
                        newAnimals[index].species_other = ''
                      }
                      if (value !== 'pig') {
                        newAnimals[index].strain = undefined
                        newAnimals[index].strain_other = ''
                      }
                      updateWorkingContent('animals', 'animals', newAnimals)
                    }}
                  >
                    <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.species')} /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="pig">{t('aup.animals.species.pig')}</SelectItem>
                      <SelectItem value="other">{t('aup.animals.species.other')}</SelectItem>
                    </SelectContent>
                  </Select>
                  {animal.species === 'other' && (
                    <Input
                      value={animal.species_other || ''}
                      onChange={(e) => {
                        const newAnimals = [...formData.working_content.animals.animals]
                        newAnimals[index].species_other = e.target.value
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                      placeholder={t('aup.animals.placeholders.speciesOther')}
                    />
                  )}
                </div>
                <div className="space-y-2">
                  <Label>{t('aup.animals.labels.strain')}</Label>
                  {animal.species === 'pig' ? (
                    <Select
                      value={animal.strain || ''}
                      onValueChange={(value) => {
                        const newAnimals = [...formData.working_content.animals.animals]
                        newAnimals[index].strain = value as 'white_pig' | 'mini_pig'
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                    >
                      <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.strain')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="white_pig">{t('aup.animals.strains.white_pig')}</SelectItem>
                        <SelectItem value="mini_pig">{t('aup.animals.strains.mini_pig')}</SelectItem>
                      </SelectContent>
                    </Select>
                  ) : animal.species === 'other' ? (
                    <Input
                      value={animal.strain_other || ''}
                      onChange={(e) => {
                        const newAnimals = [...formData.working_content.animals.animals]
                        newAnimals[index].strain_other = e.target.value
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                      placeholder={t('aup.animals.placeholders.strainOther')}
                    />
                  ) : (
                    <Input disabled placeholder={t('aup.animals.placeholders.selectSpeciesFirst')} />
                  )}
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>{t('aup.animals.labels.sex')} *</Label>
                  <Select
                    value={animal.sex || ''}
                    onValueChange={(value) => {
                      const newAnimals = [...formData.working_content.animals.animals]
                      newAnimals[index].sex = value
                      updateWorkingContent('animals', 'animals', newAnimals)
                    }}
                  >
                    <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.sex')} /></SelectTrigger>
                    <SelectContent>
                      <SelectItem value="male">{t('aup.animals.sexTypes.male')}</SelectItem>
                      <SelectItem value="female">{t('aup.animals.sexTypes.female')}</SelectItem>
                      <SelectItem value="unlimited">{t('aup.animals.sexTypes.unlimited')}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>{t('aup.animals.labels.number')} *</Label>
                  <Input
                    type="number"
                    min="0"
                    step="1"
                    value={animal.number || ''}
                    onChange={(e) => {
                      const newAnimals = [...formData.working_content.animals.animals]
                      const value = parseInt(e.target.value) || 0
                      newAnimals[index].number = value >= 0 ? value : 0
                      updateWorkingContent('animals', 'animals', newAnimals)
                    }}
                    placeholder={t('aup.animals.placeholders.number')}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label>{t('aup.animals.labels.age')} </Label>
                <div className="flex gap-4 items-start">
                  <div className="flex-1">
                    {!animal.age_unlimited && (
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>{t('aup.animals.labels.minAge')}</Label>
                          <Input
                            type="number"
                            min="3"
                            step="1"
                            value={animal.age_min || ''}
                            onChange={(e) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              const value = parseInt(e.target.value)
                              if (value >= 3) {
                                newAnimals[index].age_min = value
                                // If max age is less than or equal to new min age, auto-adjust max age
                                if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= value) {
                                  newAnimals[index].age_max = value + 1
                                }
                              } else if (value < 3 && value > 0) {
                                newAnimals[index].age_min = 3
                                // If max age is less than or equal to 3, auto-adjust max age
                                if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= 3) {
                                  newAnimals[index].age_max = 4
                                }
                              } else {
                                newAnimals[index].age_min = undefined
                              }
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                            placeholder={t('aup.animals.placeholders.minAge')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>{t('aup.animals.labels.maxAge')}</Label>
                          <Input
                            type="number"
                            min={animal.age_min !== undefined ? animal.age_min + 1 : 4}
                            step="1"
                            value={animal.age_max || ''}
                            onChange={(e) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              const value = parseInt(e.target.value)
                              const minAge = newAnimals[index].age_min || 3
                              if (value > minAge) {
                                newAnimals[index].age_max = value
                              } else if (value <= minAge && value > 0) {
                                newAnimals[index].age_max = minAge + 1
                              } else {
                                newAnimals[index].age_max = undefined
                              }
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                            placeholder={t('aup.animals.placeholders.maxAge')}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <Checkbox
                      id={`age_unlimited_${index}`}
                      checked={animal.age_unlimited || false}
                      onCheckedChange={(checked) => {
                        const newAnimals = [...formData.working_content.animals.animals]
                        newAnimals[index].age_unlimited = checked
                        if (checked) {
                          newAnimals[index].age_min = undefined
                          newAnimals[index].age_max = undefined
                        }
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                    />
                    <Label htmlFor={`age_unlimited_${index}`} className="font-normal cursor-pointer">{t('aup.animals.labels.unlimited')}</Label>
                  </div>
                </div>
              </div>
              <div className="space-y-2">
                <Label>{t('aup.animals.labels.weight')} </Label>
                <div className="flex gap-4 items-start">
                  <div className="flex-1">
                    {!animal.weight_unlimited && (
                      <div className="space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                          <div className="space-y-2">
                            <Label>{t('aup.animals.labels.minWeight')}</Label>
                            <Input
                              type="number"
                              min="20"
                              step="5"
                              value={animal.weight_min || ''}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                const value = parseFloat(e.target.value)
                                if (value >= 20) {
                                  const roundedValue = Math.round(value / 5) * 5
                                  newAnimals[index].weight_min = roundedValue >= 20 ? roundedValue : 20
                                  // If max weight is less than or equal to new min weight, auto-adjust max weight
                                  if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= roundedValue) {
                                    newAnimals[index].weight_max = roundedValue + 5
                                  }
                                } else if (value < 20 && value > 0) {
                                  newAnimals[index].weight_min = 20
                                  // 如果最大體重小於等於20，自動調整最大體重
                                  if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= 20) {
                                    newAnimals[index].weight_max = 25
                                  }
                                } else {
                                  newAnimals[index].weight_min = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                              placeholder={t('aup.animals.placeholders.minWeight')}
                            />
                            <p className="text-xs text-muted-foreground">{t('aup.animals.labels.weightInterval')}</p>
                          </div>
                          <div className="space-y-2">
                            <Label>{t('aup.animals.labels.maxWeight')}</Label>
                            <Input
                              type="number"
                              min={animal.weight_min !== undefined ? animal.weight_min + 5 : 25}
                              step="5"
                              value={animal.weight_max || ''}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                const value = parseFloat(e.target.value)
                                const minWeight = newAnimals[index].weight_min || 20
                                if (value > minWeight) {
                                  newAnimals[index].weight_max = Math.round(value / 5) * 5
                                  // Ensure max weight is greater than min weight
                                  if (newAnimals[index].weight_max <= minWeight) {
                                    newAnimals[index].weight_max = minWeight + 5
                                  }
                                } else if (value <= minWeight && value > 0) {
                                  newAnimals[index].weight_max = minWeight + 5
                                } else {
                                  newAnimals[index].weight_max = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                              placeholder={t('aup.animals.placeholders.maxWeight')}
                            />
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                  <div className="flex items-center gap-2">
                    <Checkbox
                      id={`weight_unlimited_${index}`}
                      checked={animal.weight_unlimited || false}
                      onCheckedChange={(checked) => {
                        const newAnimals = [...formData.working_content.animals.animals]
                        newAnimals[index].weight_unlimited = checked
                        if (checked) {
                          newAnimals[index].weight_min = undefined
                          newAnimals[index].weight_max = undefined
                        }
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                    />
                    <Label htmlFor={`weight_unlimited_${index}`} className="font-normal cursor-pointer">{t('aup.animals.labels.unlimited')}</Label>
                  </div>
                </div>
              </div>
              <div className="space-y-2">
                <Label>{t('aup.animals.labels.housing')}</Label>
              </div>
            </div>
          ))}
          <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md">
            <h4 className="font-semibold text-sm mb-2">{t('aup.animals.notes.title')}</h4>
            <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
              <li>{t('aup.animals.notes.item1')}</li>
              <li>{t('aup.animals.notes.item2')}</li>
              <li>{t('aup.animals.notes.item3')}</li>
              <li>{t('aup.animals.notes.item4')}</li>
              <li>{t('aup.animals.notes.item5')}</li>
            </ul>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
