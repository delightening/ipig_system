// Section Basic 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import { useAuthStore } from '@/stores/auth'
import type { SectionProps } from './types'

export function SectionBasic({ formData, updateWorkingContent, setFormData, t, isIACUCStaff, isNew }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section1')}</CardTitle>
        <CardDescription>{t('aup.basic.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* 1. GLP & Title */}
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="title">{t('aup.basic.studyTitle')} *</Label>
            <Input
              id="title"
              value={formData.title}
              onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
              placeholder={t('aup.basic.studyTitlePlaceholder')}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.glpAttribute')} *</Label>
            <div className="flex items-center space-x-2 pt-2">
              <Checkbox
                id="is_glp"
                checked={formData.working_content.basic.is_glp}
                onCheckedChange={(checked) => updateWorkingContent('basic', 'is_glp', checked)}
              />
              <Label htmlFor="is_glp">{t('aup.basic.glpCompliant')}</Label>
            </div>
          </div>
        </div>

        {/* Registration Authorities (shown when GLP is checked) */}
        {formData.working_content.basic.is_glp && (
          <div className="space-y-4">
            <Label className="text-base font-semibold">{t('aup.basic.registrationAuthorities')} *</Label>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 pl-4 text-sm">
              {['FDA', 'CE', 'TFDA', 'CFDA', 'other'].map((option) => (
                <div key={option} className="flex items-center space-x-3">
                  <Checkbox
                    id={`reg_auth_${option}`}
                    checked={formData.working_content.basic.registration_authorities.includes(option)}
                    onCheckedChange={(checked) => {
                      const current = formData.working_content.basic.registration_authorities || []
                      let updated: string[]
                      if (checked) {
                        updated = [...current, option]
                      } else {
                        updated = current.filter(s => s !== option)
                      }
                      updateWorkingContent('basic', 'registration_authorities', updated)
                    }}
                  />
                  <Label htmlFor={`reg_auth_${option}`} className="font-normal cursor-pointer">
                    {t(`aup.basic.registrationAuthorityOptions.${option}`)}
                  </Label>
                </div>
              ))}
            </div>
            {formData.working_content.basic.registration_authorities.includes('other') && (
              <div className="pl-4 pt-2 lg:w-1/2">
                <Input
                  placeholder={t('aup.basic.specifyOther')}
                  value={formData.working_content.basic.registration_authority_other || ''}
                  onChange={(e) => updateWorkingContent('basic', 'registration_authority_other', e.target.value)}
                />
              </div>
            )}
          </div>
        )}

        {/* 2. IDs and Dates */}
        <div className={`grid gap-4 ${isNew || !isIACUCStaff ? 'md:grid-cols-1' : 'md:grid-cols-2'}`}>
          {/* Study No: hidden on new page, only editable by IACUC staff on edit page */}
          {(!isNew && isIACUCStaff) && (
            <div className="space-y-2">
              <Label htmlFor="apply_study_number">{t('aup.basic.studyNo')}</Label>
              <Input
                id="apply_study_number"
                value={formData.working_content.basic.apply_study_number || ''}
                onChange={(e) => updateWorkingContent('basic', 'apply_study_number', e.target.value)}
                placeholder={t('aup.basic.studyNoPlaceholder')}
              />
            </div>
          )}
          <div className="space-y-2">
            <Label>{t('aup.basic.expectedPeriod')} *</Label>
            <div className="flex gap-2 items-center">
              <Input
                type="date"
                value={formData.start_date || ''}
                onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                required
              />
              <span className="self-center">{t('aup.basic.to')}</span>
              <Input
                type="date"
                value={formData.end_date || ''}
                onChange={(e) => setFormData(prev => ({ ...prev, end_date: e.target.value }))}
                required
              />
            </div>
          </div>
        </div>

        {/* 3. Types */}
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label>{t('aup.basic.projectType')} *</Label>
            <Select
              value={formData.working_content.basic.project_type}
              onValueChange={(val) => updateWorkingContent('basic', 'project_type', val)}
            >
              <SelectTrigger><SelectValue placeholder={t('aup.basic.selectProjectType')} /></SelectTrigger>
              <SelectContent>
                <SelectItem value="1_basic_research">{t('aup.projectTypes.1_basic_research')}</SelectItem>
                <SelectItem value="2_applied_research">{t('aup.projectTypes.2_applied_research')}</SelectItem>
                <SelectItem value="3_pre_market_testing">{t('aup.projectTypes.3_pre_market_testing')}</SelectItem>
                <SelectItem value="4_educational">{t('aup.projectTypes.4_educational')}</SelectItem>
                <SelectItem value="5_biologics_manufacturing">{t('aup.projectTypes.5_biologics_manufacturing')}</SelectItem>
                <SelectItem value="6_other">{t('aup.projectTypes.6_other')}</SelectItem>
              </SelectContent>
            </Select>
            {formData.working_content.basic.project_type === '6_other' && (
              <div className="pt-2">
                <Input
                  placeholder={t('aup.basic.specifyOther')}
                  value={formData.working_content.basic.project_type_other || ''}
                  onChange={(e) => updateWorkingContent('basic', 'project_type_other', e.target.value)}
                />
              </div>
            )}
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.projectCategory')} *</Label>
            <Select
              value={formData.working_content.basic.project_category}
              onValueChange={(val) => updateWorkingContent('basic', 'project_category', val)}
            >
              <SelectTrigger><SelectValue placeholder={t('aup.basic.selectProjectCategory')} /></SelectTrigger>
              <SelectContent>
                <SelectItem value="1_medical">{t('aup.projectCategories.1_medical')}</SelectItem>
                <SelectItem value="2_agricultural">{t('aup.projectCategories.2_agricultural')}</SelectItem>
                <SelectItem value="3_drugs_vaccines">{t('aup.projectCategories.3_drugs_vaccines')}</SelectItem>
                <SelectItem value="4_supplements">{t('aup.projectCategories.4_supplements')}</SelectItem>
                <SelectItem value="5_food">{t('aup.projectCategories.5_food')}</SelectItem>
                <SelectItem value="6_toxics_chemicals">{t('aup.projectCategories.6_toxics_chemicals')}</SelectItem>
                <SelectItem value="7_medical_materials">{t('aup.projectCategories.7_medical_materials')}</SelectItem>
                <SelectItem value="8_pesticide">{t('aup.projectCategories.8_pesticide')}</SelectItem>
                <SelectItem value="9_animal_drugs_vaccines">{t('aup.projectCategories.9_animal_drugs_vaccines')}</SelectItem>
                <SelectItem value="10_animal_supplements_feed">{t('aup.projectCategories.10_animal_supplements_feed')}</SelectItem>
                <SelectItem value="11_cosmetics">{t('aup.projectCategories.11_cosmetics')}</SelectItem>
                <SelectItem value="12_other">{t('aup.projectCategories.12_other')}</SelectItem>
              </SelectContent>
            </Select>
            {formData.working_content.basic.project_category === '12_other' && (
              <div className="pt-2">
                <Input
                  placeholder={t('aup.basic.specifyOther')}
                  value={formData.working_content.basic.project_category_other || ''}
                  onChange={(e) => updateWorkingContent('basic', 'project_category_other', e.target.value)}
                />
              </div>
            )}
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 資金來源 */}
        <div className="space-y-4">
          <Label className="text-base font-semibold">{t('aup.basic.fundingSources')} (複選)</Label>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 pl-4 text-sm">
            {['moa', 'mohw', 'nstc', 'moe', 'env', 'other'].map((option) => (
              <div key={option} className="flex items-center space-x-3">
                <Checkbox
                  id={`funding_${option}`}
                  checked={formData.working_content.basic.funding_sources.includes(option)}
                  onCheckedChange={(checked) => {
                    const current = formData.working_content.basic.funding_sources || []
                    let updated: string[]
                    if (checked) {
                      updated = [...current, option]
                    } else {
                      updated = current.filter(s => s !== option)
                    }
                    updateWorkingContent('basic', 'funding_sources', updated)
                  }}
                />
                <Label htmlFor={`funding_${option}`} className="font-normal cursor-pointer">
                  {t(`aup.basic.fundingSourceOptions.${option}`)}
                </Label>
              </div>
            ))}
          </div>
          {formData.working_content.basic.funding_sources.includes('other') && (
            <div className="pl-4 pt-2 lg:w-1/2">
              <Input
                placeholder={t('aup.basic.specifyOther')}
                value={formData.working_content.basic.funding_other || ''}
                onChange={(e) => updateWorkingContent('basic', 'funding_other', e.target.value)}
              />
            </div>
          )}
        </div>

        <div className="h-px bg-border my-4" />

        {/* 4. PI Info */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.basic.pi')}</h3>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label>{t('aup.basic.name')} *</Label>
              <Input
                value={formData.working_content.basic.pi.name}
                onChange={(e) => updateWorkingContent('basic', 'pi.name', e.target.value)}
                placeholder={t('aup.basic.piNamePlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.email')} *</Label>
              <Input
                value={formData.working_content.basic.pi.email}
                onChange={(e) => updateWorkingContent('basic', 'pi.email', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.phone')} *</Label>
              <Input
                value={formData.working_content.basic.pi.phone}
                onChange={(e) => updateWorkingContent('basic', 'pi.phone', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.address')} *</Label>
              <Input
                value={formData.working_content.basic.pi.address}
                onChange={(e) => updateWorkingContent('basic', 'pi.address', e.target.value)}
              />
            </div>
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 5. Sponsor Info */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.basic.sponsor')}</h3>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label>{t('aup.basic.organizationName')} *</Label>
              <Input
                value={formData.working_content.basic.sponsor.name}
                onChange={(e) => updateWorkingContent('basic', 'sponsor.name', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.contactPerson')} *</Label>
              <Input
                value={formData.working_content.basic.sponsor.contact_person}
                onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_person', e.target.value)}
                placeholder={t('aup.basic.contactPersonPlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.contactPhone')} *</Label>
              <Input
                value={formData.working_content.basic.sponsor.contact_phone}
                onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_phone', e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.contactEmail')} *</Label>
              <Input
                value={formData.working_content.basic.sponsor.contact_email}
                onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_email', e.target.value)}
              />
            </div>
          </div>
        </div>

        {/* 6. Facility */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.basic.facilityAndLocation')}</h3>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label>{t('aup.basic.facilityName')} *</Label>
              <Input
                value={formData.working_content.basic.facility.title}
                onChange={(e) => updateWorkingContent('basic', 'facility.title', e.target.value)}
                disabled={!useAuthStore.getState().hasRole('admin') && !useAuthStore.getState().hasRole('IACUC_STAFF')}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('aup.basic.location')} *</Label>
              <Input
                value={formData.working_content.basic.housing_location}
                onChange={(e) => updateWorkingContent('basic', 'housing_location', e.target.value)}
                disabled={!useAuthStore.getState().hasRole('admin') && !useAuthStore.getState().hasRole('IACUC_STAFF')}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
