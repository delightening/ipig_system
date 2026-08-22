// Section Design 元件
// 自動從 ProtocolEditPage.tsx 提取

import { fieldVisibleForVersion } from '@/lib/constants/protocolVersionManifests'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { AutoGrowTextarea } from '@/components/ui/autoGrowTextarea'
import { Label } from '@/components/ui/label'
import type { SectionProps } from './types'
import { AnesthesiaSection } from './components/AnesthesiaSection'
import { PainCategorySection } from './components/PainCategorySection'
import { RestrictionsSection } from './components/RestrictionsSection'
import { EndpointsSection } from './components/EndpointsSection'
import { FinalHandlingSection } from './components/FinalHandlingSection'
import { NonPharmaSection } from './components/NonPharmaSection'
import { HazardsSection } from './components/HazardsSection'
import { ControlledSubstancesSection } from './components/ControlledSubstancesSection'

const Divider = () => <div className="h-px bg-border my-4" />

export function SectionDesign({ formData, updateWorkingContent, setFormData: _setFormData, t, isIACUCStaff: _isIACUCStaff, formVersion }: SectionProps) {
  const sharedProps = { formData, updateWorkingContent, t, formVersion }
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
          <AutoGrowTextarea
            value={design.procedures}
            onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
            placeholder={t('aup.design.placeholders.procedures')}
            rows={8}
          />
        </div>

        <Divider />

        {/* 4.1.3 疼痛等級 + 4.1.4 疼痛症狀 + 4.1.5 緩解措施 */}
        <PainCategorySection {...sharedProps} />

        <Divider />

        {/* 4.1.6 是否限制實驗動物飲食或飲水 */}
        <RestrictionsSection {...sharedProps} />

        <Divider />

        {/* 4.1.7 預期實驗完成時機 */}
        <EndpointsSection {...sharedProps} />

        <Divider />

        {/* 4.1.8 動物安樂死或最終處置方式 */}
        <FinalHandlingSection {...sharedProps} />

        <Divider />

        {/* 4.2 動物屍體處理方式 */}
        <div className="space-y-4">
          <h3 className="font-semibold">{t('aup.design.carcassDisposalLabel')} *</h3>
          <div className="space-y-2">
            <AutoGrowTextarea
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
                <AutoGrowTextarea
                  value={design.hazards.operation_location_method}
                  onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <Label>{t('aup.design.protectionMeasuresLabel')}</Label>
                <p className="text-sm text-muted-foreground mb-2">{t('aup.design.protectionMeasuresSubtitle')}</p>
                <AutoGrowTextarea
                  value={design.hazards.protection_measures}
                  onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                  rows={4}
                />
              </div>
              <div className="space-y-2">
                <Label>{t('aup.design.wasteDisposalLabel')}</Label>
                <AutoGrowTextarea
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

        {/* C/D 版孤兒（E/F 已移除）：飼養環境 SOP + GLP 結果分析 + 文件歸檔 */}
        {fieldVisibleForVersion('design.housingEnvironmentSop', formVersion) && (
          <>
            <Divider />
            <div className="space-y-2">
              <Label>{t('aup.design.housingEnvironmentSop')}</Label>
              <AutoGrowTextarea
                value={design.housing_environment_sop || ''}
                onChange={(e) => updateWorkingContent('design', 'housing_environment_sop', e.target.value)}
                rows={3}
              />
            </div>
          </>
        )}
        {/* 結果分析/文件歸檔＝GLP 適用內容：版本 C/D 有此欄 且 本案為 GLP 才顯示（與註冊機關同慣例） */}
        {fieldVisibleForVersion('resultsAnalysis', formVersion) && formData.working_content.basic.is_glp && (
          <>
            <Divider />
            <div className="space-y-4 border p-4 rounded-md">
              <h3 className="font-semibold">{t('aup.design.resultsAnalysisTitle')}</h3>
              {([
                ['acceptance_criteria', 'aup.design.acceptanceCriteria'],
                ['statistics', 'aup.design.statistics'],
                ['determination', 'aup.design.determination'],
              ] as const).map(([field, labelKey]) => (
                <div key={field} className="space-y-2">
                  <Label>{t(labelKey)}</Label>
                  <AutoGrowTextarea
                    value={formData.working_content.results_analysis?.[field] || ''}
                    onChange={(e) => updateWorkingContent('results_analysis', field, e.target.value)}
                    rows={2}
                  />
                </div>
              ))}
              <div className="space-y-2">
                <Label>{t('aup.design.documentArchiving')}</Label>
                <AutoGrowTextarea
                  value={formData.working_content.document_archiving || ''}
                  onChange={(e) => updateWorkingContent('document_archiving', '', e.target.value)}
                  rows={2}
                />
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
