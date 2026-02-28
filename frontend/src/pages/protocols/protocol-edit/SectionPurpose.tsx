// Section Purpose 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import type { SectionProps } from './types'

export function SectionPurpose({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section2')}</CardTitle>
        <CardDescription>{t('aup.purpose.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* 2.1 Purpose and Significance */}
        <div className="space-y-2">
          <h3 className="font-semibold">{t('aup.purpose.significance')} *</h3>
          <Textarea
            value={formData.working_content.purpose.significance}
            onChange={(e) => updateWorkingContent('purpose', 'significance', e.target.value)}
            placeholder={t('aup.purpose.placeholders.significance')}
            rows={5}
          />
        </div>

        <div className="h-px bg-border my-4" />

        {/* 2.2 Replacement Principle */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.purpose.replacementPrinciple')}</h3>

          {/* 2.2.1 Live Animal Necessity */}
          <div className="space-y-2">
            <Label>{t('aup.purpose.liveAnimalNecessity')} *</Label>
            <Textarea
              value={formData.working_content.purpose.replacement.rationale}
              onChange={(e) => updateWorkingContent('purpose', 'replacement.rationale', e.target.value)}
              placeholder={t('aup.purpose.placeholders.rationale')}
              rows={4}
            />
          </div>

          {/* 2.2.2 Alternative Methods Search */}
          <div className="space-y-2">
            <Label>{t('aup.purpose.altSearchLabel')} *</Label>
            <div className="space-y-4 pl-4">
              <div className="flex items-start space-x-3 py-2">
                <Checkbox
                  id="search_altbib"
                  checked={formData.working_content.purpose.replacement.alt_search.platforms.includes('altbib')}
                  onCheckedChange={(checked) => {
                    const current = formData.working_content.purpose.replacement.alt_search.platforms
                    const updated = checked
                      ? [...current, 'altbib']
                      : current.filter(p => p !== 'altbib')
                    updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                  }}
                  className="mt-1"
                />
                <Label htmlFor="search_altbib" className="font-normal leading-relaxed flex-1">
                  {t('aup.purpose.altbibLabel')}<br />
                  <a
                    href="https://ntp.niehs.nih.gov/whatwestudy/niceatm/altbib"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline text-sm break-all"
                  >
                    https://ntp.niehs.nih.gov/whatwestudy/niceatm/altbib
                  </a>
                </Label>
              </div>
              <div className="flex items-start space-x-3 py-2">
                <Checkbox
                  id="search_db_alm"
                  checked={formData.working_content.purpose.replacement.alt_search.platforms.includes('db_alm')}
                  onCheckedChange={(checked) => {
                    const current = formData.working_content.purpose.replacement.alt_search.platforms
                    const updated = checked
                      ? [...current, 'db_alm']
                      : current.filter(p => p !== 'db_alm')
                    updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                  }}
                  className="mt-1"
                />
                <Label htmlFor="search_db_alm" className="font-normal leading-relaxed flex-1">
                  {t('aup.purpose.dbAlmLabel')}<br />
                  <a
                    href="https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/EURL-ECVAM/datasets/DBALM/LATEST/online/dbalm.html"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline text-sm break-all"
                  >
                    https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/EURL<br />-ECVAM/datasets/DBALM/LATEST/online/dbalm.html
                  </a>
                </Label>
              </div>
              <div className="flex items-start space-x-3 py-2">
                <Checkbox
                  id="search_re_place"
                  checked={formData.working_content.purpose.replacement.alt_search.platforms.includes('re_place')}
                  onCheckedChange={(checked) => {
                    const current = formData.working_content.purpose.replacement.alt_search.platforms
                    const updated = checked
                      ? [...current, 're_place']
                      : current.filter(p => p !== 're_place')
                    updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                  }}
                  className="mt-1"
                />
                <Label htmlFor="search_re_place" className="font-normal leading-relaxed flex-1">
                  {t('aup.purpose.rePlaceLabel')}<br />
                  <a
                    href="https://www.re-place.be/"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-600 hover:underline text-sm break-all"
                  >
                    https://www.re-place.be/
                  </a>
                </Label>
              </div>
            </div>
            {formData.working_content.purpose.replacement.alt_search.platforms.includes('other') && (
              <Input
                placeholder={t('aup.purpose.placeholders.otherDb')}
                value={formData.working_content.purpose.replacement.alt_search.other_name || ''}
                onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.other_name', e.target.value)}
                className="mt-2"
              />
            )}
          </div>
          <div className="space-y-2">
            <Label>{t('aup.purpose.searchKeywords')} *</Label>
            <Input
              value={formData.working_content.purpose.replacement.alt_search.keywords}
              onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.keywords', e.target.value)}
              placeholder={t('aup.purpose.searchKeywordsPlaceholder')}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.purpose.searchConclusion')} *</Label>
            <Textarea
              value={formData.working_content.purpose.replacement.alt_search.conclusion}
              onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.conclusion', e.target.value)}
              placeholder={t('aup.purpose.placeholders.conclusion')}
              rows={3}
            />
          </div>

          {/* 2.2.3 Duplicate Experiment */}
          <div className="space-y-2">
            <Label>{t('aup.purpose.duplicateExperiment')}</Label>
            <Select
              value={formData.working_content.purpose.duplicate.experiment ? 'yes' : 'no'}
              onValueChange={(value) => {
                const isYes = value === 'yes'
                updateWorkingContent('purpose', 'duplicate.experiment', isYes)
                if (!isYes) {
                  updateWorkingContent('purpose', 'duplicate.justification', '')
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
            {formData.working_content.purpose.duplicate.experiment && (
              <div className="space-y-2 mt-2">
                <Label>{t('aup.purpose.duplicateJustification')} *</Label>
                <Textarea
                  value={formData.working_content.purpose.duplicate.justification}
                  onChange={(e) => updateWorkingContent('purpose', 'duplicate.justification', e.target.value)}
                  placeholder={t('aup.purpose.placeholders.duplicateJustification')}
                  rows={3}
                />
              </div>
            )}
          </div>
        </div>

        <div className="h-px bg-border my-4" />

        {/* 2.3 Reduction Principle */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.purpose.reductionPrinciple')}</h3>
          <div className="space-y-2">
            <Label>{t('aup.purpose.reductionDesign')} *</Label>
            <Textarea
              value={formData.working_content.purpose.reduction.design}
              onChange={(e) => updateWorkingContent('purpose', 'reduction.design', e.target.value)}
              placeholder={t('aup.purpose.placeholders.reductionDesign')}
              rows={6}
            />
          </div>
        </div>

      </CardContent>
    </Card>
  )
}
