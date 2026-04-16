// ProtocolEditPage 驗證邏輯
// 提取自 validateRequiredFields 函數

import type { FormData } from './constants'
import type { TFunction } from 'i18next'

/**
 * 驗證必填字段
 * @returns 錯誤訊息字串，若通過則回傳 null
 */
export function validateRequiredFields(formData: FormData, t: TFunction): string | null {
    const { basic, purpose, items } = formData.working_content

    // 1. 研究名稱
    if (!formData.title.trim()) {
        return t('aup.basic.validation.titleRequired')
    }

    // 2. 預計試驗時程
    if (!formData.start_date || !formData.end_date) {
        return t('aup.basic.validation.periodRequired')
    }
    // 驗證結束日期必須大於開始日期
    if (new Date(formData.end_date) <= new Date(formData.start_date)) {
        return t('aup.basic.validation.periodInvalid')
    }

    // 3. 計畫類型
    if (!basic.project_type?.trim()) {
        return t('aup.basic.validation.projectTypeRequired')
    }

    // 4. 計畫種類
    if (!basic.project_category?.trim()) {
        return t('aup.basic.validation.projectCategoryRequired')
    }
    if (basic.project_category === 'other' && !basic.project_category_other?.trim()) {
        return t('aup.basic.validation.specifyOtherRequired')
    }

    // 5. PI 資訊
    if (!basic.pi.name?.trim()) {
        return t('aup.basic.validation.piNameRequired')
    }
    if (!basic.pi.email?.trim()) {
        return t('aup.basic.validation.piEmailRequired')
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(basic.pi.email.trim())) {
        return t('aup.basic.validation.piEmailInvalid')
    }
    if (!basic.pi.phone?.trim()) {
        return t('aup.basic.validation.piPhoneRequired')
    }
    const piPhoneDigits = basic.pi.phone.trim().replace(/-/g, '')
    if (!/^\d{9,10}$/.test(piPhoneDigits)) {
        return t('aup.basic.validation.piPhoneInvalid')
    }
    if (!basic.pi.address?.trim()) {
        return t('aup.basic.validation.piAddressRequired')
    }

    // 6. Sponsor 資訊
    if (!basic.sponsor.name?.trim()) {
        return t('aup.basic.validation.sponsorNameRequired')
    }
    if (!basic.sponsor.contact_person?.trim()) {
        return t('aup.basic.validation.sponsorContactRequired')
    }
    if (!basic.sponsor.contact_phone?.trim()) {
        return t('aup.basic.validation.sponsorPhoneRequired')
    }
    const sponsorPhoneDigits = basic.sponsor.contact_phone.trim().replace(/-/g, '')
    if (!/^\d{9,10}$/.test(sponsorPhoneDigits)) {
        return t('aup.basic.validation.sponsorPhoneInvalid')
    }
    if (!basic.sponsor.contact_email?.trim()) {
        return t('aup.basic.validation.sponsorEmailRequired')
    }
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(basic.sponsor.contact_email.trim())) {
        return t('aup.basic.validation.sponsorEmailInvalid')
    }

    // 7. 機構名稱
    if (!basic.facility.title?.trim()) {
        return t('aup.basic.validation.facilityRequired')
    }

    // 8. 位置
    if (!basic.housing_location?.trim()) {
        return t('aup.basic.validation.locationRequired')
    }

    // ===================== Section 2 - 研究目的 =====================

    // 2.0 計畫摘要
    if (!purpose.abstract?.trim()) {
        return t('aup.purpose.validation.abstractRequired')
    }

    // 2.1 研究之目的及重要性
    if (!purpose.significance?.trim()) {
        return t('aup.purpose.validation.significanceRequired')
    }

    // 2.2.1 活體動物試驗之必要性
    if (!purpose.replacement.rationale?.trim()) {
        return t('aup.purpose.validation.rationaleRequired')
    }

    // 2.2.2 非動物替代方案搜尋資料庫
    if (!purpose.replacement.alt_search.platforms?.length) {
        return t('aup.purpose.validation.platformsRequired')
    }
    if (!purpose.replacement.alt_search.keywords?.trim()) {
        return t('aup.purpose.validation.keywordsRequired')
    }
    if (!purpose.replacement.alt_search.conclusion?.trim()) {
        return t('aup.purpose.validation.conclusionRequired')
    }

    // 2.2.3 重複試驗
    if (!purpose.duplicate.status) {
        return t('aup.purpose.validation.duplicateStatusRequired')
    }
    if (purpose.duplicate.status === 'not_applicable' && !purpose.duplicate.regulation_basis?.trim()) {
        return t('aup.purpose.validation.regulationBasisRequired')
    }
    if (purpose.duplicate.status === 'yes_continuation' && !purpose.duplicate.previous_iacuc_no?.trim()) {
        return t('aup.purpose.validation.previousIacucNoRequired')
    }
    if (purpose.duplicate.status === 'yes_duplicate' && !purpose.duplicate.justification?.trim()) {
        return t('aup.purpose.validation.duplicateJustificationRequired')
    }

    // 2.3 減量原則 - 實驗設計說明
    if (!purpose.reduction.design?.trim()) {
        return t('aup.purpose.validation.reductionDesignRequired')
    }

    // 2.4 精緻化原則
    if (!purpose.refinement_description?.trim()) {
        return t('aup.purpose.validation.refinementRequired')
    }

    // ===================== Section 3 - 試驗物質與對照物質 =====================

    if (items.use_test_item === null) {
        return t('aup.items.validation.useTestItemRequired')
    }

    if (items.use_test_item === true) {
        for (let i = 0; i < items.test_items.length; i++) {
            const item = items.test_items[i]
            if (!item.name?.trim()) {
                return t('aup.items.validation.testItemNameRequired', { index: i + 1 })
            }
            if (!item.form?.trim()) {
                return t('aup.items.validation.testFormRequired', { index: i + 1 })
            }
            if (!item.purpose?.trim()) {
                return t('aup.items.validation.testPurposeRequired', { index: i + 1 })
            }
            if (!item.storage_conditions?.trim()) {
                return t('aup.items.validation.testStorageRequired', { index: i + 1 })
            }
            if (!item.is_sterile && !item.non_sterile_justification?.trim()) {
                return t('aup.items.validation.testJustificationRequired', { index: i + 1 })
            }
        }

        for (let i = 0; i < items.control_items.length; i++) {
            const item = items.control_items[i]
            if (!item.name?.trim()) {
                return t('aup.items.validation.controlItemNameRequired', { index: i + 1 })
            }
            if (!item.purpose?.trim()) {
                return t('aup.items.validation.controlPurposeRequired', { index: i + 1 })
            }
            if (!item.storage_conditions?.trim()) {
                return t('aup.items.validation.controlStorageRequired', { index: i + 1 })
            }
            if (!item.is_sterile && !item.non_sterile_justification?.trim()) {
                return t('aup.items.validation.controlJustificationRequired', { index: i + 1 })
            }
        }
    }

    // ===================== Section 4 - 研究設計與方法 =====================

    const { design } = formData.working_content

    // 4.1.1 麻醉
    if (design.anesthesia.is_under_anesthesia === true) {
        if (!design.anesthesia.anesthesia_type?.trim()) {
            return t('aup.design.validation.anesthesiaTypeRequired')
        }
        if (design.anesthesia.anesthesia_type === 'other' && !design.anesthesia.other_description?.trim()) {
            return t('aup.design.validation.anesthesiaOtherRequired')
        }
    }

    // 4.1.2 動物試驗流程
    if (!design.procedures?.trim()) {
        return t('aup.design.validation.proceduresRequired')
    }

    // 4.1.3 疼痛等級
    if (!design.pain.category?.trim()) {
        return t('aup.design.validation.painCategoryRequired')
    }
    if (!design.pain.category_items?.length) {
        return t('aup.design.validation.painCategoryItemsRequired')
    }

    // 4.1.4 飲食飲水限制
    if (design.restrictions.is_restricted === true) {
        if (!design.restrictions.restriction_type?.trim()) {
            return t('aup.design.validation.restrictionTypeRequired')
        }
        if (design.restrictions.restriction_type === 'other' && !design.restrictions.other_description?.trim()) {
            return t('aup.design.validation.restrictionOtherRequired')
        }
    }

    // 4.1.5 疼痛症狀
    if (!design.pain.distress_signs?.length) {
        return t('aup.design.validation.distressSignsRequired')
    }

    // 4.1.6 緩解措施
    if (!design.pain.relief_measures?.length) {
        return t('aup.design.validation.reliefMeasuresRequired')
    }
    if (design.pain.relief_measures.includes('anesthesia_analgesia') && !design.pain.relief_drug_name?.trim()) {
        return t('aup.design.validation.reliefDrugNameRequired')
    }
    if (design.pain.relief_measures.includes('no_relief_with_justification') && !design.pain.no_relief_justification?.trim()) {
        return t('aup.design.validation.noReliefJustificationRequired')
    }

    // 4.1.7 實驗終點
    if (!design.endpoints.experimental_endpoint?.trim()) {
        return t('aup.design.validation.experimentalEndpointRequired')
    }
    if (!design.endpoints.humane_endpoint?.trim()) {
        return t('aup.design.validation.humaneEndpointRequired')
    }

    // 4.3 非醫藥級
    if (design.non_pharma_grade.used === true) {
        if (!design.non_pharma_grade.description?.trim()) {
            return t('aup.design.validation.nonPharmaRequired')
        }
    }

    // 4.4 危害性物質
    if (design.hazards.used === true) {
        if (!design.hazards.selected_type?.trim()) {
            return t('aup.design.validation.hazardTypeRequired')
        }
        if (!design.hazards.materials.length || design.hazards.materials.every(m => !m.agent_name?.trim())) {
            return t('aup.design.validation.hazardMaterialsRequired')
        }
        for (let i = 0; i < design.hazards.materials.length; i++) {
            const material = design.hazards.materials[i]
            if (!material.agent_name?.trim()) {
                return t('aup.design.validation.hazardAgentNameRequired', { index: i + 1 })
            }
            if (!material.amount?.trim()) {
                return t('aup.design.validation.hazardAmountRequired', { index: i + 1 })
            }
        }
        if (!design.hazards.operation_location_method?.trim()) {
            return t('aup.design.validation.hazardOpsRequired')
        }
        if (!design.hazards.protection_measures?.trim()) {
            return t('aup.design.validation.hazardProtectionRequired')
        }
        if (!design.hazards.waste_and_carcass_disposal?.trim()) {
            return t('aup.design.validation.hazardWasteRequired')
        }
    }

    // 管制藥品
    if (design.controlled_substances.used === true) {
        if (!design.controlled_substances.items.length) {
            return t('aup.design.validation.controlledSubstancesRequired')
        }
        for (let i = 0; i < design.controlled_substances.items.length; i++) {
            const item = design.controlled_substances.items[i]
            if (!item.drug_name?.trim()) {
                return t('aup.design.validation.drugNameRequired', { index: i + 1 })
            }
            if (!item.approval_no?.trim()) {
                return t('aup.design.validation.approvalNoRequired', { index: i + 1 })
            }
            if (!item.amount?.trim()) {
                return t('aup.design.validation.drugAmountRequired', { index: i + 1 })
            }
            if (!item.authorized_person?.trim()) {
                return t('aup.design.validation.authorizedPersonRequired', { index: i + 1 })
            }
        }
    }

    // ===================== Section 6 - Surgery =====================

    const needsSurgeryPlan = design.anesthesia.is_under_anesthesia === true &&
        (design.anesthesia.anesthesia_type === 'survival_surgery' || design.anesthesia.anesthesia_type === 'non_survival_surgery')

    if (needsSurgeryPlan) {
        const { surgery } = formData.working_content
        if (!surgery.surgery_type?.trim() || surgery.surgery_type === '略') {
            return t('aup.surgery.validation.surgeryTypeRequired')
        }
        if (!surgery.preop_preparation?.trim() || surgery.preop_preparation === '略') {
            return t('aup.surgery.validation.preop_PreparationRequired')
        }
        if (!surgery.surgery_description?.trim() || surgery.surgery_description === '略') {
            return t('aup.surgery.validation.surgeryDescriptionRequired')
        }
        if (!surgery.monitoring?.trim()) {
            return t('aup.surgery.validation.monitoringRequired')
        }
        if (surgery.surgery_type === 'survival') {
            if (!surgery.postop_expected_impact?.trim() || surgery.postop_expected_impact === '略') {
                return t('aup.surgery.validation.expectedImpactRequired')
            }
        }
        if (surgery.multiple_surgeries.used) {
            if (!surgery.multiple_surgeries.number || surgery.multiple_surgeries.number <= 0) {
                return t('aup.surgery.validation.multipleSurgeriesNumberRequired')
            }
            if (!surgery.multiple_surgeries.reason?.trim()) {
                return t('aup.surgery.validation.multipleSurgeriesReasonRequired')
            }
        }
        if (!surgery.postop_care_type?.trim()) {
            return t('aup.surgery.validation.postopCareTypeRequired')
        }
        if (!surgery.postop_care?.trim()) {
            return t('aup.surgery.validation.postopCareRequired')
        }
        if (!surgery.expected_end_point?.trim()) {
            return t('aup.surgery.validation.expectedEndPointRequired')
        }
        if (!surgery.drugs?.length) {
            return t('aup.surgery.validation.drugsRequired')
        }
        for (let i = 0; i < surgery.drugs.length; i++) {
            const drug = surgery.drugs[i]
            if (!drug.drug_name?.trim()) {
                return t('aup.surgery.validation.drugNameRequired', { index: i + 1 })
            }
            if (!drug.dose?.trim()) {
                return t('aup.surgery.validation.drugDoseRequired', { index: i + 1 })
            }
            if (!drug.route?.trim()) {
                return t('aup.surgery.validation.drugRouteRequired', { index: i + 1 })
            }
            if (!drug.frequency?.trim()) {
                return t('aup.surgery.validation.drugFrequencyRequired', { index: i + 1 })
            }
            if (!drug.purpose?.trim()) {
                return t('aup.surgery.validation.drugPurposeRequired', { index: i + 1 })
            }
        }
    }

    // ===================== Section 7 - Experimental animal data =====================

    const { animals } = formData.working_content
    if (!animals.animals?.length) {
        return t('aup.animals.validation.animalsRequired')
    }
    for (let i = 0; i < animals.animals.length; i++) {
        const animal = animals.animals[i]
        if (!animal.species?.trim()) {
            return t('aup.animals.validation.speciesRequired', { index: i + 1 })
        }
        if (animal.species === 'other' && !animal.species_other?.trim()) {
            return t('aup.animals.validation.speciesRequired', { index: i + 1 })
        }
        if (animal.species === 'pig' && !animal.strain) {
            return t('aup.animals.validation.strainRequired', { index: i + 1 })
        }
        if (animal.species === 'other' && !animal.strain_other?.trim()) {
            return t('aup.animals.validation.strainRequired', { index: i + 1 })
        }
        if (!animal.sex?.trim()) {
            return t('aup.animals.validation.sexRequired', { index: i + 1 })
        }
        if (!animal.number || animal.number <= 0) {
            return t('aup.animals.validation.numberRequired', { index: i + 1 })
        }
        if (!animal.age_unlimited) {
            if (animal.age_min === undefined || animal.age_min < 3) {
                return t('aup.animals.validation.ageMinRequired', { index: i + 1 })
            }
            if (animal.age_max === undefined) {
                return t('aup.animals.validation.ageMaxRequired', { index: i + 1 })
            }
            if (animal.age_max <= animal.age_min) {
                return t('aup.animals.validation.ageMaxInvalid', { index: i + 1 })
            }
        }
        if (!animal.weight_unlimited) {
            if (animal.weight_min === undefined || animal.weight_min < 20) {
                return t('aup.animals.validation.weightMinRequired', { index: i + 1 })
            }
            if (animal.weight_max === undefined) {
                return t('aup.animals.validation.weightMaxRequired', { index: i + 1 })
            }
            if (animal.weight_max <= animal.weight_min) {
                return t('aup.animals.validation.weightMaxInvalid', { index: i + 1 })
            }
        }
    }

    return null
}
