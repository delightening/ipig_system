// Section Design 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { SectionProps } from './types'
import { AnesthesiaSection } from './components/AnesthesiaSection'
import { RestrictionsSection } from './components/RestrictionsSection'
import { EndpointsSection } from './components/EndpointsSection'
import { FinalHandlingSection } from './components/FinalHandlingSection'
import { NonPharmaSection } from './components/NonPharmaSection'
import { HazardsSection } from './components/HazardsSection'
import { ControlledSubstancesSection } from './components/ControlledSubstancesSection'

const Divider = () => <div className="h-px bg-border my-4" />

export function SectionDesign({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff }: SectionProps) {
  const sharedProps = { formData, updateWorkingContent, t }
  const { design } = formData.working_content

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

        {/* 4.1.1 是否在麻醉下進行實驗 */}
        <AnesthesiaSection {...sharedProps} />

        <Divider />

        {/* 4.1.2 動物實驗內容及程序的詳細敘述 */}
        <div className="space-y-2">
          <Label>{t('aup.design.proceduresLabel')}</Label>
          <p className="text-sm text-muted-foreground mb-2">{t('aup.design.proceduresNote')}</p>
          <Textarea
            value={design.procedures}
            onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
            placeholder={t('aup.design.placeholders.procedures')}
            rows={8}
          />
        </div>

        <Divider />

        {/* 4.1.3 實驗動物等級評估 */}
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>{t('aup.design.painCategoryLabel')}</Label>
            <Select
              value={design.pain.category}
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

        <Divider />

        {/* 4.1.4 是否限制實驗動物飲食或飲水 */}
        <RestrictionsSection {...sharedProps} />

        <Divider />

        {/* 4.1.5 預期實驗完成時機 */}
        <EndpointsSection {...sharedProps} />

        <Divider />

        {/* 4.1.6 動物安樂死或最終處置方式 */}
        <FinalHandlingSection {...sharedProps} />

        <Divider />

        {/* 4.2 動物屍體處理方式 */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.design.carcassDisposalLabel')} *</h3>
          <div className="space-y-2">
            <Textarea
              value={design.carcass_disposal.method}
              onChange={(e) => updateWorkingContent('design', 'carcass_disposal.method', e.target.value)}
              placeholder={t('aup.design.carcassDisposalPlaceholder')}
              rows={4}
            />
          </div>
        </div>

        <Divider />

        {/* 4.3 使用非藥用等級化學藥品或其他物質 */}
        <NonPharmaSection {...sharedProps} />

        <Divider />

        {/* 4.4 使用危害性物質 */}
        <HazardsSection {...sharedProps} />

        {/* 4.4 為「是」時：4.5 危害性廢棄物處置 + 4.6 管制藥品 */}
        {design.hazards.used === true && (
          <>
            <Divider />
            <div className="space-y-4">
              <h3 className="font-semibold">{t('aup.design.hazardsWasteLabel')}</h3>
              <div className="space-y-2">
                <Label>{t('aup.design.operationLocationLabel')}</Label>
                <Textarea
                  value={design.hazards.operation_location_method}
                  onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <Label>{t('aup.design.protectionMeasuresLabel')}</Label>
                <p className="text-sm text-muted-foreground mb-2">{t('aup.design.protectionMeasuresSubtitle')}</p>
                <Textarea
                  value={design.hazards.protection_measures}
                  onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <Label>{t('aup.design.wasteDisposalLabel')}</Label>
                <Textarea
                  value={design.hazards.waste_and_carcass_disposal}
                  onChange={(e) => updateWorkingContent('design', 'hazards.waste_and_carcass_disposal', e.target.value)}
                  rows={4}
                />
              </div>
            </div>
            <Divider />
            <div className="space-y-4">
              <h3 className="font-semibold">{t('aup.design.controlledSubstancesLabel.section4_6')}</h3>
              <ControlledSubstancesSection
                {...sharedProps}
                labelKey="aup.design.controlledSubstancesLabel.section4_6"
              />
            </div>
          </>
        )}

        {/* 4.4 為「否」時：4.5 管制藥品 */}
        {design.hazards.used === false && (
          <>
            <Divider />
            <ControlledSubstancesSection
              {...sharedProps}
              labelKey="aup.design.controlledSubstancesLabel.section4_5"
            />
          </>
        )}

      </CardContent>
    </Card>
  )
}
