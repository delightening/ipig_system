import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { ChoiceList } from './ChoiceList'
import {
  YES_NO_OPTIONS,
  boolSelected,
  oneOf,
  ALT_SEARCH_PLATFORM_OPTIONS,
  DUPLICATE_STATUS_OPTIONS,
  SINGLE_HOUSING_REASON_OPTIONS,
  ANIMAL_REUSE_PLAN_OPTIONS,
} from '@/lib/constants/protocolChoiceOptions'

interface PurposeSectionProps {
  purpose: ProtocolWorkingContent['purpose']
}

export function PurposeSection({ purpose }: PurposeSectionProps) {
  const { t } = useTranslation()

  if (!purpose.significance && !purpose.replacement && !purpose.reduction) {
    return null
  }

  const reduction = purpose.reduction

  return (
    <section className="mb-8 border-t pt-6 section-2" data-section={t('protocols.content.sections.purpose')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.purpose')}</h2>

      {purpose.significance && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.significance')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{purpose.significance}</p>
        </div>
      )}

      {purpose.replacement && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.replacement')}</h3>

          {purpose.replacement.rationale && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.replacementRationale')}</h4>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{purpose.replacement.rationale}</p>
            </div>
          )}

          {purpose.replacement.alt_search && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-2">{t('protocols.content.sections.alternativeSearch')}</h4>
              {/* 替代方案搜尋平台（顯示完整選項 + 已選/未選） */}
              <ChoiceList options={ALT_SEARCH_PLATFORM_OPTIONS} selectedValues={purpose.replacement.alt_search.platforms ?? []} />
              {purpose.replacement.alt_search.platforms?.includes('other') && purpose.replacement.alt_search.other_name && (
                <p className="text-sm mt-1"><strong>{t('aup.purpose.otherPlatformLabel')}: </strong>{purpose.replacement.alt_search.other_name}</p>
              )}
              {purpose.replacement.alt_search.keywords && (
                <p className="text-sm mt-2"><strong>{t('protocols.content.sections.searchKeywords')}: </strong>{purpose.replacement.alt_search.keywords}</p>
              )}
              {purpose.replacement.alt_search.conclusion && (
                <div className="mt-2">
                  <p className="text-sm font-medium mb-1">{t('protocols.content.sections.searchConclusion')}:</p>
                  <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{purpose.replacement.alt_search.conclusion}</p>
                </div>
              )}
            </div>
          )}

          {purpose.duplicate && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-2">{t('protocols.content.sections.duplicate')}</h4>
              {/* 是否為重複試驗（顯示完整 4 選項 + 已選/未選） */}
              <ChoiceList options={DUPLICATE_STATUS_OPTIONS} selectedValues={oneOf(purpose.duplicate.status)} />
              {purpose.duplicate.status === 'not_applicable' && purpose.duplicate.regulation_basis && (
                <p className="text-sm mt-1"><strong>{t('aup.purpose.regulationBasis')}: </strong>{purpose.duplicate.regulation_basis}</p>
              )}
              {purpose.duplicate.status === 'yes_continuation' && purpose.duplicate.previous_iacuc_no && (
                <p className="text-sm mt-1"><strong>{t('aup.purpose.previousIacucNo')}: </strong>{purpose.duplicate.previous_iacuc_no}</p>
              )}
              {purpose.duplicate.status === 'yes_duplicate' && purpose.duplicate.justification && (
                <div className="mt-1">
                  <p className="text-sm font-medium mb-1">{t('protocols.content.sections.duplicateJustification')}</p>
                  <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{purpose.duplicate.justification}</p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {reduction && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.reduction')}</h3>

          {reduction.design && (
            <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded mb-3">{reduction.design}</p>
          )}

          {/* 2.3.1 特殊照護（是/否 + 已選/未選；是→照護內容） */}
          <div className="mb-3">
            <h4 className="text-base font-medium mb-2">{t('aup.purpose.specialCareTitle')}</h4>
            <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(reduction.special_care?.needed)} />
            {reduction.special_care?.needed === true && reduction.special_care.description && (
              <p className="text-sm mt-1 whitespace-pre-wrap bg-muted p-3 rounded">{reduction.special_care.description}</p>
            )}
          </div>

          {/* 2.3.2 單獨飼養（是/否 + 已選/未選；是→原因複選 + 監控明細） */}
          <div className="mb-3">
            <h4 className="text-base font-medium mb-2">{t('aup.purpose.singleHousingTitle')}</h4>
            <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(reduction.single_housing?.required)} />
            {reduction.single_housing?.required === true && (
              <div className="mt-2 pl-4 border-l-2 border-border">
                <p className="text-sm font-medium mb-1">{t('aup.purpose.singleHousingReasons')}</p>
                <ChoiceList options={SINGLE_HOUSING_REASON_OPTIONS} selectedValues={reduction.single_housing.reasons ?? []} />
                {reduction.single_housing.reasons?.includes('b2_metabolic_cage') && reduction.single_housing.metabolic_cage_duration && (
                  <p className="text-sm mt-1"><strong>{t('aup.purpose.metabolicCageDuration')}: </strong>{reduction.single_housing.metabolic_cage_duration}</p>
                )}
                {reduction.single_housing.monitoring_method && (
                  <p className="text-sm mt-1"><strong>{t('aup.purpose.singleHousingMonitoringMethod')}: </strong>{reduction.single_housing.monitoring_method}</p>
                )}
                {reduction.single_housing.estimated_duration && (
                  <p className="text-sm mt-1"><strong>{t('aup.purpose.singleHousingEstimatedDuration')}: </strong>{reduction.single_housing.estimated_duration}</p>
                )}
              </div>
            )}
          </div>

          {/* 2.3.3 動物再應用（是/否 + 已選/未選；是→再應用計畫單選） */}
          <div className="mb-3">
            <h4 className="text-base font-medium mb-2">{t('aup.purpose.animalReuseTitle')}</h4>
            <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(reduction.animal_reuse?.considered)} />
            {reduction.animal_reuse?.considered === true && (
              <div className="mt-2 pl-4 border-l-2 border-border">
                <p className="text-sm font-medium mb-1">{t('aup.purpose.animalReusePlan')}</p>
                <ChoiceList options={ANIMAL_REUSE_PLAN_OPTIONS} selectedValues={oneOf(reduction.animal_reuse.plan)} />
                {reduction.animal_reuse.plan === 'other' && reduction.animal_reuse.plan_other && (
                  <p className="text-sm mt-1 whitespace-pre-wrap bg-muted p-3 rounded">{reduction.animal_reuse.plan_other}</p>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  )
}
