// Section Design 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { FileUpload } from '@/components/ui/file-upload'
import { Plus } from 'lucide-react'
import type { SectionProps } from './types'

export function SectionDesign({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section4')}</CardTitle>
        <CardDescription>{t('aup.design.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* 4.1 Title */}
        <div className="space-y-2">
          <h3 className="font-semibold">{t('aup.design.title4_1')}</h3>
        </div>

        {/* 4.1.1 Is experiment conducted under anesthesia */}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t('aup.design.anesthesiaLabel')} *</Label>
            <Select
              value={formData.working_content.design.anesthesia.is_under_anesthesia === null ? '' : (formData.working_content.design.anesthesia.is_under_anesthesia === true ? 'yes' : 'no')}
              onValueChange={(value) => {
                const isYes = value === 'yes'
                updateWorkingContent('design', 'anesthesia.is_under_anesthesia', isYes as boolean | null)
                // If "No" is selected, clear related fields and auto-fill surgery plan with "N/A"
                if (!isYes) {
                  updateWorkingContent('design', 'anesthesia.anesthesia_type', undefined)
                  updateWorkingContent('design', 'anesthesia.other_description', undefined)
                  // Auto-fill surgery plan with "N/A" when not under anesthesia
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

          {formData.working_content.design.anesthesia.is_under_anesthesia === true && (
            <div className="space-y-4 pl-6 border-l-2 border-slate-200">
              <div className="space-y-2">
                <Label>{t('aup.design.selectAnesthesiaType')}</Label>
                <Select
                  value={formData.working_content.design.anesthesia.anesthesia_type || ''}
                  onValueChange={(value) => {
                    updateWorkingContent('design', 'anesthesia.anesthesia_type', value)
                    // If not "Other", clear other description
                    if (value !== 'other') {
                      updateWorkingContent('design', 'anesthesia.anesthesia_other_description', undefined)
                    }
                    // Auto-set surgery plan based on selection
                    if (value === 'survival_surgery') {
                      // If survival surgery, auto-set surgery type to "Survival"
                      updateWorkingContent('surgery', 'surgery_type', 'survival')
                      // If pre-op preparation is "N/A" or empty, set default content
                      if (formData.working_content.surgery.preop_preparation === '略' || formData.working_content.surgery.preop_preparation === t('common.na') || !formData.working_content.surgery.preop_preparation) {
                        updateWorkingContent('surgery', 'preop_preparation', t('aup.design.templates.preop_preparation'))
                      }
                      // If surgery description is "N/A" or empty, set default content
                      if (formData.working_content.surgery.surgery_description === '略' || formData.working_content.surgery.surgery_description === t('common.na') || !formData.working_content.surgery.surgery_description) {
                        updateWorkingContent('surgery', 'surgery_description', t('aup.design.templates.surgery_description'))
                      }
                      // If monitoring is empty, set default content
                      if (!formData.working_content.surgery.monitoring) {
                        updateWorkingContent('surgery', 'monitoring', t('aup.design.templates.monitoring'))
                      }
                      // Post-op care type is user-selected, not auto-set
                      updateWorkingContent('surgery', 'aseptic_techniques', [])
                    } else if (value === 'non_survival_surgery') {
                      // If non-survival surgery, auto-set surgery type to "Non-survival"
                      updateWorkingContent('surgery', 'surgery_type', 'non_survival')
                      // If pre-op preparation is "N/A" or empty, set default content
                      if (formData.working_content.surgery.preop_preparation === '略' || formData.working_content.surgery.preop_preparation === t('common.na') || !formData.working_content.surgery.preop_preparation) {
                        updateWorkingContent('surgery', 'preop_preparation', t('aup.design.templates.preop_preparation'))
                      }
                      // If surgery description is "N/A" or empty, set default content
                      if (formData.working_content.surgery.surgery_description === '略' || formData.working_content.surgery.surgery_description === t('common.na') || !formData.working_content.surgery.surgery_description) {
                        updateWorkingContent('surgery', 'surgery_description', t('aup.design.templates.surgery_description'))
                      }
                      // If monitoring is empty, set default content
                      if (!formData.working_content.surgery.monitoring) {
                        updateWorkingContent('surgery', 'monitoring', t('aup.design.templates.monitoring'))
                      }
                      // Post-op care type is user-selected, not auto-set
                      updateWorkingContent('surgery', 'aseptic_techniques', [])
                    } else if (value && value !== '') {
                      // If not survival or non-survival surgery, auto-fill "N/A"
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
                  }}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('common.pleaseSelect')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="survival_surgery">{t('aup.design.anesthesiaTypes.survival')}</SelectItem>
                    <SelectItem value="non_survival_surgery">{t('aup.design.anesthesiaTypes.non_survival')}</SelectItem>
                    <SelectItem value="gas_only">{t('aup.design.anesthesiaTypes.gas_only')}</SelectItem>
                    <SelectItem value="azeperonum_atropine">{t('aup.design.anesthesiaTypes.azeperonum_atropine')}</SelectItem>
                    <SelectItem value="other">{t('aup.design.anesthesiaTypes.other')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {formData.working_content.design.anesthesia.anesthesia_type === 'other' && (
                <div className="space-y-2">
                  <Label>{t('aup.design.explainOther')}</Label>
                  <Textarea
                    value={formData.working_content.design.anesthesia.other_description || ''}
                    onChange={(e) => updateWorkingContent('design', 'anesthesia.other_description', e.target.value)}
                    placeholder={t('aup.design.placeholders.explainOther')}
                    rows={3}
                  />
                </div>
              )}
            </div>
          )}
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.1.2 Detailed narrative of animal experiment content and procedures */}
        <div className="space-y-2">
          <Label>{t('aup.design.proceduresLabel')}</Label>
          <p className="text-sm text-muted-foreground mb-2">{t('aup.design.proceduresNote')}</p>
          <Textarea
            value={formData.working_content.design.procedures}
            onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
            placeholder={t('aup.design.placeholders.procedures')}
            rows={8}
          />
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.1.3 Assessment of experimental animal levels */}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t('aup.design.painCategoryLabel')}</Label>
            <Select
              value={formData.working_content.design.pain.category}
              onValueChange={(val) => updateWorkingContent('design', 'pain.category', val)}
            >
              <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
              <SelectContent>
                <SelectItem value="B">{t('aup.design.painCategories.B')}</SelectItem>
                <SelectItem value="C">{t('aup.design.painCategories.C')}</SelectItem>
                <SelectItem value="D">{t('aup.design.painCategories.D')}</SelectItem>
                <SelectItem value="E">{t('aup.design.painCategories.E')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.1.4 Whether to restrict diet or water for experimental animals */}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t('aup.design.restrictionsLabel')} *</Label>
            <Select
              value={formData.working_content.design.restrictions.is_restricted === null ? '' : (formData.working_content.design.restrictions.is_restricted === true ? 'yes' : 'no')}
              onValueChange={(value) => {
                const isYes = value === 'yes'
                updateWorkingContent('design', 'restrictions.is_restricted', isYes as boolean | null)
                // If "No" is selected, clear related fields
                if (!isYes) {
                  updateWorkingContent('design', 'restrictions.restriction_type', undefined)
                  updateWorkingContent('design', 'restrictions.other_description', undefined)
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

          {formData.working_content.design.restrictions.is_restricted === true && (
            <div className="space-y-4 pl-6 border-l-2 border-slate-200">
              <div className="space-y-2">
                <Label>{t('aup.design.selectRestrictionType')}</Label>
                <Select
                  value={formData.working_content.design.restrictions.restriction_type || ''}
                  onValueChange={(value) => {
                    updateWorkingContent('design', 'restrictions.restriction_type', value)
                    // If not "Other", clear other description
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

              {formData.working_content.design.restrictions.restriction_type === 'other' && (
                <div className="space-y-2">
                  <Label>{t('aup.design.explainRestrictionOther')}</Label>
                  <Textarea
                    value={formData.working_content.design.restrictions.other_description || ''}
                    onChange={(e) => updateWorkingContent('design', 'restrictions.other_description', e.target.value)}
                    placeholder={t('aup.design.placeholders.explainRestrictionOther')}
                    rows={3}
                  />
                </div>
              )}
            </div>
          )}
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.1.5 Expected timing of experiment completion */}
        <div className="space-y-4">
          <Label>{t('aup.design.endpointsTitle')}</Label>
          <div className="space-y-2">
            <Label>{t('aup.design.experimentalEndpoint')} *</Label>
            <Textarea
              value={formData.working_content.design.endpoints.experimental_endpoint}
              onChange={(e) => updateWorkingContent('design', 'endpoints.experimental_endpoint', e.target.value)}
              placeholder={t('aup.design.placeholders.experimentalEndpoint')}
              rows={3}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.design.humaneEndpoint')} *</Label>
            <Textarea
              value={formData.working_content.design.endpoints.humane_endpoint}
              onChange={(e) => updateWorkingContent('design', 'endpoints.humane_endpoint', e.target.value)}
              placeholder={t('aup.design.placeholders.humaneEndpoint')}
              rows={4}
            />
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.1.6 Animal euthanasia or final disposal method */}
        <div className="space-y-4">
          <Label>{t('aup.design.finalHandlingTitle')} *</Label>
          <div className="space-y-2">
            <Select
              value={formData.working_content.design.final_handling.method || ''}
              onValueChange={(value) => {
                updateWorkingContent('design', 'final_handling.method', value)
                // Clear other options content
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
              }}
            >
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

          {/* 1. Euthanasia */}
          {formData.working_content.design.final_handling.method === 'euthanasia' && (
            <div className="space-y-3 border-l-2 border-slate-200 pl-6">
              <Label className="text-sm font-medium">{t('aup.design.euthanasiaLabel')}</Label>
              <Select
                value={formData.working_content.design.final_handling.euthanasia_type || ''}
                onValueChange={(value) => {
                  updateWorkingContent('design', 'final_handling.euthanasia_type', value)
                  // If not "Other", clear other description
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
              {formData.working_content.design.final_handling.euthanasia_type === 'other' && (
                <div className="space-y-2 mt-2">
                  <Textarea
                    value={formData.working_content.design.final_handling.euthanasia_other_description || ''}
                    onChange={(e) => updateWorkingContent('design', 'final_handling.euthanasia_other_description', e.target.value)}
                    placeholder={t('aup.design.placeholders.euthanasiaOther')}
                    rows={3}
                  />
                </div>
              )}
            </div>
          )}

          {/* 2. Transfer */}
          {formData.working_content.design.final_handling.method === 'transfer' && (
            <div className="space-y-3 border-l-2 border-slate-200 pl-6">
              <Label className="text-sm font-medium">{t('aup.design.transferLabel')}</Label>
              <div className="space-y-3">
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.recipientName')}</Label>
                  <Input
                    value={formData.working_content.design.final_handling.transfer.recipient_name}
                    onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_name', e.target.value)}
                    placeholder={t('aup.design.recipientNamePlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.recipientOrg')}</Label>
                  <Input
                    value={formData.working_content.design.final_handling.transfer.recipient_org}
                    onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_org', e.target.value)}
                    placeholder={t('aup.design.recipientOrgPlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-sm">{t('aup.design.projectName')}</Label>
                  <Input
                    value={formData.working_content.design.final_handling.transfer.project_name}
                    onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.project_name', e.target.value)}
                    placeholder={t('aup.design.projectNamePlaceholder')}
                  />
                </div>
              </div>
            </div>
          )}

          {/* 3. Other */}
          {formData.working_content.design.final_handling.method === 'other' && (
            <div className="space-y-3 border-l-2 border-slate-200 pl-6">
              <Label className="text-sm font-medium">{t('aup.design.handlingMethods.other')}: </Label>
              <Textarea
                value={formData.working_content.design.final_handling.other_description || ''}
                onChange={(e) => updateWorkingContent('design', 'final_handling.other_description', e.target.value)}
                placeholder={t('aup.design.otherHandlingPlaceholder')}
                rows={3}
              />
            </div>
          )}
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.2 Animal carcass disposal method */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.design.carcassDisposalLabel')} *</h3>
          <div className="space-y-2">
            <Textarea
              value={formData.working_content.design.carcass_disposal.method}
              onChange={(e) => updateWorkingContent('design', 'carcass_disposal.method', e.target.value)}
              placeholder={t('aup.design.carcassDisposalPlaceholder')}
              rows={4}
            />
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.3 Use of non-pharmaceutical chemical drugs or other substances */}
        <div className="space-y-4">
          <div className="space-y-2">
            <h3 className="font-semibold">{t('aup.design.nonPharmaLabel')} *</h3>
            <Select
              value={formData.working_content.design.non_pharma_grade.used === null ? '' : (formData.working_content.design.non_pharma_grade.used === true ? 'yes' : 'no')}
              onValueChange={(value) => {
                const isYes = value === 'yes'
                updateWorkingContent('design', 'non_pharma_grade.used', isYes as boolean | null)
                // If "No" is selected, clear description field
                if (!isYes) {
                  updateWorkingContent('design', 'non_pharma_grade.description', '')
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
            {formData.working_content.design.non_pharma_grade.used === true && (
              <div className="space-y-2 mt-2">
                <Label>{t('aup.design.nonPharmaExplain')}</Label>
                <Textarea
                  value={formData.working_content.design.non_pharma_grade.description}
                  onChange={(e) => updateWorkingContent('design', 'non_pharma_grade.description', e.target.value)}
                  rows={4}
                />
              </div>
            )}
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4.4 Use of hazardous materials */}
        <div className="space-y-4">
          <div className="space-y-2">
            <h3 className="font-semibold">{t('aup.design.hazardsLabel')} *</h3>
            <Select
              value={formData.working_content.design.hazards.used === null ? '' : (formData.working_content.design.hazards.used === true ? 'yes' : 'no')}
              onValueChange={(value) => {
                const isYes = value === 'yes'
                updateWorkingContent('design', 'hazards.used', isYes as boolean | null)
                // If "No" is selected, clear related fields
                if (!isYes) {
                  updateWorkingContent('design', 'hazards.materials', [])
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
            {formData.working_content.design.hazards.used === true && (
              <div className="space-y-4 mt-2 pl-6 border-l-2 border-slate-200">
                <div className="space-y-3">
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">{t('aup.design.selectHazardType')}</Label>
                    <Select
                      value={formData.working_content.design.hazards.selected_type || ''}
                      onValueChange={(value) => {
                        updateWorkingContent('design', 'hazards.selected_type', value)
                        // Clear materials of other types, keep only materials of current type
                        const currentMaterials = formData.working_content.design.hazards.materials.filter(m => m.type === value)
                        updateWorkingContent('design', 'hazards.materials', currentMaterials)
                      }}
                    >
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

                  {/* Show material list for selected type */}
                  {formData.working_content.design.hazards.selected_type && (
                    <div className="space-y-3">
                      <div className="flex justify-between items-center">
                        <Label className="text-sm font-medium">
                          {formData.working_content.design.hazards.selected_type === 'biological' && t('aup.design.hazardTypes.biological')}
                          {formData.working_content.design.hazards.selected_type === 'radioactive' && t('aup.design.hazardTypes.radioactive')}
                          {formData.working_content.design.hazards.selected_type === 'chemical' && t('aup.design.hazardTypes.chemical')}
                        </Label>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            const materials = [...formData.working_content.design.hazards.materials]
                            materials.push({
                              type: formData.working_content.design.hazards.selected_type!,
                              agent_name: '',
                              amount: '',
                              photos: []
                            })
                            updateWorkingContent('design', 'hazards.materials', materials)
                          }}
                        >
                          <Plus className="h-4 w-4 mr-1" />
                          {t('aup.items.add')}
                        </Button>
                      </div>
                      {formData.working_content.design.hazards.materials
                        .filter(m => m.type === formData.working_content.design.hazards.selected_type)
                        .map((material, _index) => {
                          const materialIndex = formData.working_content.design.hazards.materials.findIndex(m => m === material)
                          return (
                            <div key={materialIndex} className="space-y-3 relative p-3 border rounded bg-slate-50">
                              <Button
                                variant="ghost"
                                size="icon"
                                className="absolute right-2 top-2 h-6 w-6 text-red-500"
                                onClick={() => {
                                  const materials = [...formData.working_content.design.hazards.materials]
                                  materials.splice(materialIndex, 1)
                                  updateWorkingContent('design', 'hazards.materials', materials)
                                }}
                              >
                                X
                              </Button>
                              <div className="grid grid-cols-2 gap-3">
                                <Input
                                  placeholder={t('aup.design.agentNamePlaceholder')}
                                  value={material.agent_name}
                                  onChange={(e) => {
                                    const materials = [...formData.working_content.design.hazards.materials]
                                    materials[materialIndex].agent_name = e.target.value
                                    updateWorkingContent('design', 'hazards.materials', materials)
                                  }}
                                />
                                <Input
                                  placeholder={t('aup.design.amountPlaceholder')}
                                  value={material.amount}
                                  onChange={(e) => {
                                    const materials = [...formData.working_content.design.hazards.materials]
                                    materials[materialIndex].amount = e.target.value
                                    updateWorkingContent('design', 'hazards.materials', materials)
                                  }}
                                />
                              </div>
                              {/* Photo Upload */}
                              <div className="space-y-2">
                                <Label className="text-sm">{t('aup.items.photos')}</Label>
                                <FileUpload
                                  value={material.photos || []}
                                  onChange={(photos) => {
                                    const materials = [...formData.working_content.design.hazards.materials]
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
        </div>

        {/* Conditional Visibility: if 4.4 is "Yes", show 4.5 and 4.6; if 4.4 is "No", show 4.5 (Controlled Substances) */}
        {formData.working_content.design.hazards.used === true && (
          <>
            <div className="h-px bg-border my-4" />

            {/* 4.5 Hazardous substances and waste disposal methods */}
            <div className="space-y-4">
              <h3 className="font-semibold">{t('aup.design.hazardsWasteLabel')}</h3>

              {/* 4.5.1 Administration method, route and place of use */}
              <div className="space-y-2">
                <Label>{t('aup.design.operationLocationLabel')}</Label>
                <Textarea
                  value={formData.working_content.design.hazards.operation_location_method}
                  onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                  rows={4}
                />
              </div>

              {/* 4.5.2 Protection measures */}
              <div className="space-y-2">
                <Label>{t('aup.design.protectionMeasuresLabel')}</Label>
                <p className="text-sm text-muted-foreground mb-2">{t('aup.design.protectionMeasuresSubtitle')}</p>
                <Textarea
                  value={formData.working_content.design.hazards.protection_measures}
                  onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                  rows={4}
                />
              </div>

              {/* 4.5.3 Disposal of experimental waste and carcasses */}
              <div className="space-y-2">
                <Label>{t('aup.design.wasteDisposalLabel')}</Label>
                <Textarea
                  value={formData.working_content.design.hazards.waste_and_carcass_disposal}
                  onChange={(e) => updateWorkingContent('design', 'hazards.waste_and_carcass_disposal', e.target.value)}
                  rows={4}
                />
              </div>
            </div>

            <div className="h-px bg-border my-4" />

            {/* 4.6 Use of controlled substances */}
            <div className="space-y-4">
              <div className="space-y-2">
                <h3 className="font-semibold">{t('aup.design.controlledSubstancesLabel.section4_6')}</h3>
                <Select
                  value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                  onValueChange={(value) => {
                    const isYes = value === 'yes'
                    updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                    // 如果選擇"否"，清空相關欄位
                    if (!isYes) {
                      updateWorkingContent('design', 'controlled_substances.items', [])
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

              {formData.working_content.design.controlled_substances.used === true && (
                <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                  <div className="flex justify-between items-center">
                    <Label className="text-sm font-medium">{t('aup.design.controlledSubstancesList')}</Label>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const items = [...formData.working_content.design.controlled_substances.items, {
                          drug_name: '',
                          approval_no: '',
                          amount: '',
                          authorized_person: '',
                          photos: []
                        }]
                        updateWorkingContent('design', 'controlled_substances.items', items)
                      }}
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t('aup.items.add')}
                    </Button>
                  </div>
                  {formData.working_content.design.controlled_substances.items.map((item, index) => (
                    <div key={index} className="space-y-3 relative p-3 border rounded bg-slate-50">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="absolute right-2 top-2 h-6 w-6 text-red-500"
                        onClick={() => {
                          const items = [...formData.working_content.design.controlled_substances.items]
                          items.splice(index, 1)
                          updateWorkingContent('design', 'controlled_substances.items', items)
                        }}
                      >
                        X
                      </Button>
                      <div className="grid grid-cols-2 gap-3">
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.drugNameLabel')}</Label>
                          <Input
                            value={item.drug_name}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].drug_name = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.drugNamePlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.approvalNoLabel')}</Label>
                          <Input
                            value={item.approval_no}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].approval_no = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.approvalNoPlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.drugAmountLabel')}</Label>
                          <Input
                            value={item.amount}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].amount = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.drugAmountPlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.authorizedPersonLabel')}</Label>
                          <Input
                            value={item.authorized_person}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].authorized_person = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.authorizedPersonPlaceholder')}
                          />
                        </div>
                      </div>
                      {/* Photo Upload */}
                      <div className="space-y-2">
                        <Label className="text-sm">{t('aup.items.photos')}</Label>
                        <FileUpload
                          value={item.photos || []}
                          onChange={(photos) => {
                            const items = [...formData.working_content.design.controlled_substances.items]
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
          </>
        )}

        {/* If 4.4 is "No", show 4.5 (Controlled Substances) */}
        {formData.working_content.design.hazards.used === false && (
          <>
            <div className="h-px bg-border my-4" />

            {/* 4.5 Use of controlled substances */}
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>{t('aup.design.controlledSubstancesLabel.section4_5')}</Label>
                <Select
                  value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                  onValueChange={(value) => {
                    const isYes = value === 'yes'
                    updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                    // 如果選擇"否"，清空相關欄位
                    if (!isYes) {
                      updateWorkingContent('design', 'controlled_substances.items', [])
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

              {formData.working_content.design.controlled_substances.used === true && (
                <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                  <div className="flex justify-between items-center">
                    <Label className="text-sm font-medium">{t('aup.design.controlledSubstancesList')}</Label>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const items = [...formData.working_content.design.controlled_substances.items, {
                          drug_name: '',
                          approval_no: '',
                          amount: '',
                          authorized_person: '',
                          photos: []
                        }]
                        updateWorkingContent('design', 'controlled_substances.items', items)
                      }}
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t('aup.items.add')}
                    </Button>
                  </div>
                  {formData.working_content.design.controlled_substances.items.map((item, index) => (
                    <div key={index} className="space-y-3 relative p-3 border rounded bg-slate-50">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="absolute right-2 top-2 h-6 w-6 text-red-500"
                        onClick={() => {
                          const items = [...formData.working_content.design.controlled_substances.items]
                          items.splice(index, 1)
                          updateWorkingContent('design', 'controlled_substances.items', items)
                        }}
                      >
                        X
                      </Button>
                      <div className="grid grid-cols-2 gap-3">
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.drugNameLabel')}</Label>
                          <Input
                            value={item.drug_name}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].drug_name = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.drugNamePlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.approvalNoLabel')}</Label>
                          <Input
                            value={item.approval_no}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].approval_no = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.approvalNoPlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.drugAmountLabel')}</Label>
                          <Input
                            value={item.amount}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].amount = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.drugAmountPlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.authorizedPersonLabel')}</Label>
                          <Input
                            value={item.authorized_person}
                            onChange={(e) => {
                              const items = [...formData.working_content.design.controlled_substances.items]
                              items[index].authorized_person = e.target.value
                              updateWorkingContent('design', 'controlled_substances.items', items)
                            }}
                            placeholder={t('aup.design.authorizedPersonPlaceholder')}
                          />
                        </div>
                      </div>
                      {/* Photo Upload */}
                      <div className="space-y-2">
                        <Label className="text-sm">{t('aup.items.photos')}</Label>
                        <FileUpload
                          value={item.photos || []}
                          onChange={(photos) => {
                            const items = [...formData.working_content.design.controlled_substances.items]
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
          </>
        )}
      </CardContent>
    </Card>
  )
}
