import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { ChoiceList } from './ChoiceList'
import {
  ANESTHESIA_TYPE_OPTIONS,
  PAIN_CATEGORY_OPTIONS,
  RELIEF_MEASURE_OPTIONS,
  RESTRICTION_TYPE_OPTIONS,
  HANDLING_METHOD_OPTIONS,
  EUTHANASIA_TYPE_OPTIONS,
} from '@/lib/constants/protocolDesignOptions'
import { YES_NO_OPTIONS, boolSelected, oneOf } from '@/lib/constants/protocolChoiceOptions'

interface DesignSectionProps {
  design: ProtocolWorkingContent['design']
}

export function DesignSection({ design }: DesignSectionProps) {
  const { t } = useTranslation()

  const hasAny = design.procedures
    || design.anesthesia
    || design.pain
    || design.endpoints
    || design.restrictions
    || design.final_handling
    || design.carcass_disposal?.method
    || design.non_pharma_grade
    || design.hazards
    || design.controlled_substances

  if (!hasAny) {
    return null
  }

  const controlledSectionNumber = design.hazards?.used === true ? '4_6' : '4_5'

  return (
    <section className="mb-8 border-t pt-6 section-4" data-section={t('protocols.content.sections.design')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.design')}</h2>

      {/* 4.1.1 麻醉 */}
      {design.anesthesia && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.anesthesia')}</h3>
          <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(design.anesthesia.is_under_anesthesia)} />
          {design.anesthesia.is_under_anesthesia === true && (
            <div className="mt-2">
              <p className="text-sm font-medium mb-1">{t('protocols.content.sections.anesthesiaType')}</p>
              <ChoiceList options={ANESTHESIA_TYPE_OPTIONS} selectedValues={oneOf(design.anesthesia.anesthesia_type)} />
            </div>
          )}
        </div>
      )}

      {/* 4.1.2 程序敘述 */}
      {design.procedures && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.procedures')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.procedures}</p>
        </div>
      )}

      {/* 4.1.3 + 4.1.5 疼痛分級 + 緩解措施 */}
      {design.pain && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.painCategory')}</h3>
          <ChoiceList options={PAIN_CATEGORY_OPTIONS} selectedValues={oneOf(design.pain.category)} />
          <div className="mt-2">
            <p className="text-sm font-medium mb-1">{t('protocols.content.sections.painManagement')}</p>
            <ChoiceList options={RELIEF_MEASURE_OPTIONS} selectedValues={design.pain.relief_measures ?? []} />
            {design.pain.relief_drug_name && (
              <p className="text-sm mt-1">{design.pain.relief_drug_name}</p>
            )}
            {design.pain.no_relief_justification && (
              <p className="text-sm mt-1">{design.pain.no_relief_justification}</p>
            )}
          </div>
        </div>
      )}

      {/* 4.1.4 痛苦症狀 */}
      {design.pain?.distress_signs && design.pain.distress_signs.length > 0 && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.distressSigns')}</h3>
          <ul className="text-sm list-disc pl-6">
            {design.pain.distress_signs.map(sign => (
              <li key={sign}>{t(`aup.design.distressSigns.${sign}`, sign)}</li>
            ))}
          </ul>
          {design.pain.distress_signs_other_text && (
            <p className="text-sm mt-1">{t('protocols.content.sections.other')}: {design.pain.distress_signs_other_text}</p>
          )}
        </div>
      )}

      {/* 4.1.6 飲食/飲水限制 */}
      {design.restrictions && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.restrictions')}</h3>
          <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(design.restrictions.is_restricted)} />
          {design.restrictions.is_restricted === true && (
            <div className="mt-2">
              <ChoiceList options={RESTRICTION_TYPE_OPTIONS} selectedValues={oneOf(design.restrictions.restriction_type)} />
              {design.restrictions.other_description && (
                <p className="text-sm mt-1">{t('protocols.content.sections.otherText')}: {design.restrictions.other_description}</p>
              )}
            </div>
          )}
        </div>
      )}

      {/* 4.1.7 終點 */}
      {design.endpoints && (
        <div className="mb-4">
          {design.endpoints.experimental_endpoint && (
            <div className="mb-3">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.experimentalEndpoint')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.endpoints.experimental_endpoint}</p>
            </div>
          )}
          {design.endpoints.humane_endpoint && (
            <div>
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.humaneEndpoint')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.endpoints.humane_endpoint}</p>
            </div>
          )}
        </div>
      )}

      {/* 4.1.8 最終處置 */}
      {design.final_handling && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.finalHandling')}</h3>
          <ChoiceList options={HANDLING_METHOD_OPTIONS} selectedValues={oneOf(design.final_handling.method)} />
          {design.final_handling.method === 'euthanasia' && (
            <div className="mt-2">
              <ChoiceList options={EUTHANASIA_TYPE_OPTIONS} selectedValues={oneOf(design.final_handling.euthanasia_type)} />
              {design.final_handling.euthanasia_type === 'other' && design.final_handling.euthanasia_other_description && (
                <p className="text-sm mt-1">{design.final_handling.euthanasia_other_description}</p>
              )}
            </div>
          )}
          {design.final_handling.method === 'transfer' && design.final_handling.transfer && (
            <div className="mt-1 text-sm space-y-1">
              <p>{t('protocols.content.sections.transferRecipient')}: {design.final_handling.transfer.recipient_name}</p>
              <p>{t('protocols.content.sections.transferOrg')}: {design.final_handling.transfer.recipient_org}</p>
              <p>{t('protocols.content.sections.transferProject')}: {design.final_handling.transfer.project_name}</p>
            </div>
          )}
          {design.final_handling.method === 'other' && (design.final_handling.other_description || design.final_handling.other_text) && (
            <p className="text-sm mt-1">{design.final_handling.other_description || design.final_handling.other_text}</p>
          )}
        </div>
      )}

      {/* 4.2 屍體處理 */}
      {design.carcass_disposal?.method && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.carcassDisposal')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.carcass_disposal.method}</p>
          {design.carcass_disposal.vendor_name && (
            <p className="text-sm mt-1">{design.carcass_disposal.vendor_name}{design.carcass_disposal.vendor_id ? `（${design.carcass_disposal.vendor_id}）` : ''}</p>
          )}
        </div>
      )}

      {/* 4.3 非醫藥級化學藥品 */}
      {design.non_pharma_grade && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.nonPharma')}</h3>
          <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(design.non_pharma_grade.used)} />
          {design.non_pharma_grade.used === true && design.non_pharma_grade.description && (
            <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded mt-1">{design.non_pharma_grade.description}</p>
          )}
        </div>
      )}

      {/* 4.4 危害性物質 — 是/否 + 已選明細（materials by type） */}
      {design.hazards && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.hazardsTitle')}</h3>
          <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(design.hazards.used)} />
          {design.hazards.used === true && (['biological', 'radioactive', 'chemical'] as const).map(type => {
            const typeMaterials = design.hazards.materials?.filter(m => m.type === type) ?? []
            if (typeMaterials.length === 0) return null
            return (
              <div key={type} className="mt-2">
                <p className="text-sm font-medium">{t(`aup.design.hazardTypes.${type}`, type)}</p>
                <ul className="text-sm list-disc pl-6 mt-1">
                  {typeMaterials.map((m, i) => (
                    <li key={i}>{m.agent_name}（{m.amount}）</li>
                  ))}
                </ul>
              </div>
            )
          })}
        </div>
      )}

      {/* 4.5 危害廢棄物處置（僅 4.4=true 顯示） */}
      {design.hazards?.used === true && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.hazardsWasteTitle')}</h3>
          {design.hazards.operation_location_method && (
            <div className="mb-2">
              <p className="text-sm font-medium">{t('protocols.content.sections.hazardsOperation')}</p>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded mt-1">{design.hazards.operation_location_method}</p>
            </div>
          )}
          {design.hazards.protection_measures && (
            <div className="mb-2">
              <p className="text-sm font-medium">{t('protocols.content.sections.hazardsProtection')}</p>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded mt-1">{design.hazards.protection_measures}</p>
            </div>
          )}
          {design.hazards.waste_and_carcass_disposal && (
            <div>
              <p className="text-sm font-medium">{t('protocols.content.sections.hazardsWaste')}</p>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded mt-1">{design.hazards.waste_and_carcass_disposal}</p>
            </div>
          )}
        </div>
      )}

      {/* 4.5 / 4.6 管制藥品 — 動態編號 */}
      {design.controlled_substances && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t(`protocols.content.sections.controlledSubstances${controlledSectionNumber}`)}</h3>
          <ChoiceList options={YES_NO_OPTIONS} selectedValues={boolSelected(design.controlled_substances.used)} />
          {design.controlled_substances.used === true && design.controlled_substances.items && design.controlled_substances.items.length > 0 && (
            <table className="w-full text-sm border-collapse mt-2">
              <thead>
                <tr className="bg-muted">
                  <th className="border p-2 text-left">{t('protocols.content.sections.controlledDrugName')}</th>
                  <th className="border p-2 text-left">{t('protocols.content.sections.controlledApprovalNo')}</th>
                  <th className="border p-2 text-left">{t('protocols.content.sections.controlledAmount')}</th>
                  <th className="border p-2 text-left">{t('protocols.content.sections.controlledAuthorizedPerson')}</th>
                </tr>
              </thead>
              <tbody>
                {design.controlled_substances.items.map((d, i) => (
                  <tr key={i}>
                    <td className="border p-2">{d.drug_name}</td>
                    <td className="border p-2">{d.approval_no}</td>
                    <td className="border p-2">{d.amount}</td>
                    <td className="border p-2">{d.authorized_person}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </section>
  )
}
