// Section Surgery 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { Button } from '@/components/ui/button'
import type { ProtocolSurgeryDrugItem } from '@/types/protocol'
import type { SectionProps } from './types'

export function SectionSurgery({ formData, updateWorkingContent, setFormData, t, isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <div className="flex justify-between items-center">
          <div>
            <CardTitle>{t('aup.section6')}</CardTitle>
            <CardDescription>{t('aup.surgery.subtitle')}</CardDescription>
          </div>
          {formData.working_content.design.anesthesia.is_under_anesthesia === true &&
            (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
              formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery') && (
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  // Load default values for surgery section
                  // 6.2 Pre-operative
                  updateWorkingContent('surgery', 'preop_preparation', t('aup.defaults.preop_Preparation'))
                  // 6.3 Aseptic (unbox all)
                  updateWorkingContent('surgery', 'aseptic_techniques', [])
                  // 6.4 Surgery description (clear)
                  updateWorkingContent('surgery', 'surgery_description', '')
                  // 6.5 Monitoring
                  updateWorkingContent('surgery', 'monitoring', t('aup.defaults.monitoring'))
                  // 6.6 Post-operative impact (clear)
                  updateWorkingContent('surgery', 'postop_expected_impact', '')
                  // 6.7 Multiple surgeries (set to "No")
                  updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                  // 6.8 Orthopedic/Non-orthopedic (deselect)
                  updateWorkingContent('surgery', 'postop_care_type', undefined)
                  updateWorkingContent('surgery', 'postop_care', '')
                  // 6.9 Expected end point (clear)
                  updateWorkingContent('surgery', 'expected_end_point', '')

                  // 6.10 Load default drugs
                  const defaultDrugs = t('aup.defaults.drugDefaults', { returnObjects: true }) as any[]
                  if (Array.isArray(defaultDrugs)) {
                    const formattedDrugs = defaultDrugs.map(drug => ({
                      drug_name: drug.name,
                      dose: drug.dose,
                      route: drug.route,
                      frequency: drug.frequency,
                      purpose: drug.purpose
                    }))
                    updateWorkingContent('surgery', 'drugs', formattedDrugs)
                  }
                }}

              >
                {t('aup.surgery.loadDefaults')}
              </Button>
            )}
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {(() => {
          const needsSurgeryPlan = formData.working_content.design.anesthesia.is_under_anesthesia === true &&
            (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
              formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery')

          if (!needsSurgeryPlan) {
            // If surgery plan is not required, show "N/A"
            return (
              <div className="space-y-4">
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.surgeryType')}</h3>
                  <Input value={t('common.na')} disabled />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.preop_Preparation')}</h3>
                  <Textarea value={t('common.na')} disabled rows={3} />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.asepticTechniques')}</h3>
                  <Input value={t('common.na')} disabled />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.surgeryDescription')}</h3>
                  <Textarea value={t('common.na')} disabled rows={5} />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.monitoring')}</h3>
                  <Textarea value={t('common.na')} disabled rows={5} />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.expectedImpact')}</h3>
                  <Textarea value={t('common.na')} disabled rows={4} />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.multipleSurgeries')}</h3>
                  <Input value={t('common.na')} disabled />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.postopCare')}</h3>
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label>{t('aup.surgery.labels.postopCareType')}</Label>
                      <Input value={t('common.na')} disabled />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.surgery.labels.postopCareDetail')}</Label>
                      <Textarea value={t('common.na')} disabled rows={5} />
                    </div>
                  </div>
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.expectedEndPoint')}</h3>
                  <Textarea value={t('common.na')} disabled rows={4} />
                </div>
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.surgery.labels.drugInfo')}</h3>
                  <Input value={t('common.na')} disabled />
                </div>
              </div>
            )
          }

          // If surgery plan is required, show normal form
          return (
            <>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.surgeryType')}</h3>
                <Input
                  value={formData.working_content.surgery.surgery_type === 'survival' ? t('aup.surgery.types.survival') :
                    formData.working_content.surgery.surgery_type === 'non_survival' ? t('aup.surgery.types.non_survival') :
                      formData.working_content.surgery.surgery_type || ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.preop_Preparation')}</h3>
                <Textarea
                  value={formData.working_content.surgery.preop_preparation}
                  onChange={(e) => updateWorkingContent('surgery', 'preop_preparation', e.target.value)}
                  placeholder={t('aup.surgery.placeholders.preop_Preparation')}
                  rows={8}
                />
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.asepticTechniques')}</h3>
                <div className="space-y-2">
                  {[
                    { value: 'surgical_site_disinfection', label: t('aup.surgery.asepticTechniques.surgical_site_disinfection') },
                    { value: 'instrument_disinfection', label: t('aup.surgery.asepticTechniques.instrument_disinfection') },
                    { value: 'sterilized_gowns_gloves', label: t('aup.surgery.asepticTechniques.sterilized_gowns_gloves') },
                    { value: 'sterilized_drapes', label: t('aup.surgery.asepticTechniques.sterilized_drapes') },
                    { value: 'surgical_hand_disinfection', label: t('aup.surgery.asepticTechniques.surgical_hand_disinfection') }
                  ].map(item => (
                    <div key={item.value} className="flex items-center space-x-2">
                      <Checkbox
                        id={`aseptic_${item.value}`}
                        checked={formData.working_content.surgery.aseptic_techniques.includes(item.value)}
                        onCheckedChange={(checked) => {
                          const current = formData.working_content.surgery.aseptic_techniques
                          const updated = checked
                            ? [...current, item.value]
                            : current.filter(i => i !== item.value)
                          updateWorkingContent('surgery', 'aseptic_techniques', updated)
                        }}
                      />
                      <Label htmlFor={`aseptic_${item.value}`} className="font-normal cursor-pointer">{item.label}</Label>
                    </div>
                  ))}
                </div>
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.surgeryDescription')}</h3>
                <Textarea
                  value={formData.working_content.surgery.surgery_description}
                  onChange={(e) => updateWorkingContent('surgery', 'surgery_description', e.target.value)}
                  placeholder={t('aup.surgery.placeholders.surgeryDescription')}
                  rows={5}
                />
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.monitoring')}</h3>
                <Textarea
                  value={formData.working_content.surgery.monitoring}
                  onChange={(e) => updateWorkingContent('surgery', 'monitoring', e.target.value)}
                  placeholder={t('aup.surgery.placeholders.monitoring')}
                  rows={5}
                />
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.expectedImpact')}</h3>
                <Textarea
                  value={formData.working_content.surgery.postop_expected_impact}
                  onChange={(e) => updateWorkingContent('surgery', 'postop_expected_impact', e.target.value)}
                  placeholder={t('aup.surgery.placeholders.expectedImpact')}
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.multipleSurgeries')}</h3>
                <div className="space-y-4">
                  <Select
                    value={formData.working_content.surgery.multiple_surgeries.used === true ? 'yes' : formData.working_content.surgery.multiple_surgeries.used === false ? 'no' : ''}
                    onValueChange={(value) => {
                      const isYes = value === 'yes'
                      updateWorkingContent('surgery', 'multiple_surgeries.used', isYes)
                      if (!isYes) {
                        updateWorkingContent('surgery', 'multiple_surgeries.number', 0)
                        updateWorkingContent('surgery', 'multiple_surgeries.reason', '')
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
                  {formData.working_content.surgery.multiple_surgeries.used && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>{t('aup.items.amount')} *</Label>
                        <Input
                          type="number"
                          value={formData.working_content.surgery.multiple_surgeries.number || ''}
                          onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.number', parseInt(e.target.value) || 0)}
                          placeholder={t('aup.items.amount')}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>{t('aup.items.reason')} *</Label>
                        <Textarea
                          value={formData.working_content.surgery.multiple_surgeries.reason}
                          onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.reason', e.target.value)}
                          placeholder={t('aup.items.reason')}
                          rows={3}
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.postopCare')}</h3>
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>{t('aup.surgery.labels.postopCareType')} *</Label>
                    <Select
                      value={formData.working_content.surgery.postop_care_type || ''}
                      onValueChange={(value) => {
                        updateWorkingContent('surgery', 'postop_care_type', value as 'orthopedic' | 'non_orthopedic')
                        // Auto-set corresponding default content based on selection
                        if (value === 'orthopedic') {
                          updateWorkingContent('surgery', 'postop_care', t('aup.surgery.postOpTemplates.orthopedic'))
                        } else if (value === 'non_orthopedic') {
                          updateWorkingContent('surgery', 'postop_care', t('aup.surgery.postOpTemplates.non_orthopedic'))
                        }
                      }}
                    >
                      <SelectTrigger><SelectValue placeholder={t('aup.surgery.labels.postopCareType')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="orthopedic">{t('aup.surgery.postOpTypes.orthopedic')}</SelectItem>
                        <SelectItem value="non_orthopedic">{t('aup.surgery.postOpTypes.non_orthopedic')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.surgery.labels.postopCareDetail')} *</Label>
                    <Textarea
                      value={formData.working_content.surgery.postop_care}
                      onChange={(e) => updateWorkingContent('surgery', 'postop_care', e.target.value)}
                      placeholder={t('aup.surgery.placeholders.postopCare')}
                      rows={15}
                    />
                  </div>
                </div>
              </div>
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.surgery.labels.expectedEndPoint')}</h3>
                <Textarea
                  value={formData.working_content.surgery.expected_end_point}
                  onChange={(e) => updateWorkingContent('surgery', 'expected_end_point', e.target.value)}
                  placeholder={t('aup.surgery.placeholders.expectedEndPoint')}
                  rows={4}
                />
              </div>
              <div className="space-y-4">
                <div className="flex justify-between items-center">
                  <h3 className="font-semibold">{t('aup.surgery.labels.drugInfo')}</h3>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      const currentDrugs = formData.working_content.surgery.drugs || []
                      const newDrugs = [...currentDrugs, {
                        drug_name: '',
                        dose: '',
                        route: '',
                        frequency: '',
                        purpose: ''
                      }]
                      updateWorkingContent('surgery', 'drugs', newDrugs)
                    }}
                  >
                    + {t('aup.items.add')}
                  </Button>
                </div>
                <div className="border rounded-md overflow-hidden">
                  <div className="overflow-x-auto">
                    <table className="w-full border-collapse">
                      <thead>
                        <tr className="bg-slate-100">
                          <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.name')}</th>
                          <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.dose')}</th>
                          <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.route')}</th>
                          <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.frequency')}</th>
                          <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.purpose')}</th>
                          <th className="border p-2 text-center text-sm font-semibold w-16">{t('common.actions')}</th>
                        </tr>
                      </thead>
                      <tbody>
                        {(formData.working_content.surgery.drugs || []).map((drug: ProtocolSurgeryDrugItem, index: number) => (
                          <tr key={index} className="hover:bg-slate-50">
                            <td className="border p-2">
                              <Input
                                value={drug.drug_name || ''}
                                onChange={(e) => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs[index].drug_name = e.target.value
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                                placeholder={t('aup.surgery.placeholders.drugName')}
                                className="border-0 focus-visible:ring-0"
                              />
                            </td>
                            <td className="border p-2">
                              <Input
                                value={drug.dose || ''}
                                onChange={(e) => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs[index].dose = e.target.value
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                                placeholder={t('aup.surgery.placeholders.drugDose')}
                                className="border-0 focus-visible:ring-0"
                              />
                            </td>
                            <td className="border p-2">
                              <Input
                                value={drug.route || ''}
                                onChange={(e) => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs[index].route = e.target.value
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                                placeholder={t('aup.surgery.placeholders.drugRoute')}
                                className="border-0 focus-visible:ring-0"
                              />
                            </td>
                            <td className="border p-2">
                              <Input
                                value={drug.frequency || ''}
                                onChange={(e) => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs[index].frequency = e.target.value
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                                placeholder={t('aup.surgery.placeholders.drugFrequency')}
                                className="border-0 focus-visible:ring-0"
                              />
                            </td>
                            <td className="border p-2">
                              <Input
                                value={drug.purpose || ''}
                                onChange={(e) => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs[index].purpose = e.target.value
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                                placeholder={t('aup.surgery.placeholders.drugPurpose')}
                                className="border-0 focus-visible:ring-0"
                              />
                            </td>
                            <td className="border p-2 text-center">
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8 text-red-500"
                                onClick={() => {
                                  const newDrugs = [...formData.working_content.surgery.drugs]
                                  newDrugs.splice(index, 1)
                                  updateWorkingContent('surgery', 'drugs', newDrugs)
                                }}
                              >
                                X
                              </Button>
                            </td>
                          </tr>
                        ))}
                        {(!formData.working_content.surgery.drugs || formData.working_content.surgery.drugs.length === 0) && (
                          <tr>
                            <td colSpan={6} className="border p-4 text-center text-muted-foreground">
                              {t('aup.surgery.noDrugs')}
                            </td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            </>
          )
        })()}
      </CardContent>
    </Card>
  )
}
