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
    if (!basic.project_type || !basic.project_type.trim()) {
        return t('aup.basic.validation.projectTypeRequired')
    }

    // 4. 計畫種類
    if (!basic.project_category || !basic.project_category.trim()) {
        return t('aup.basic.validation.projectCategoryRequired')
    }
    if (basic.project_category === 'other' && (!basic.project_category_other || !basic.project_category_other.trim())) {
        return t('aup.basic.validation.specifyOtherRequired')
    }

    // 5. PI 資訊
    if (!basic.pi.name || !basic.pi.name.trim()) {
        return t('aup.basic.validation.piNameRequired')
    }
    if (!basic.pi.email || !basic.pi.email.trim()) {
        return t('aup.basic.validation.piEmailRequired')
    }
    // 驗證 PI Email 格式
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(basic.pi.email.trim())) {
        return t('aup.basic.validation.piEmailInvalid')
    }
    if (!basic.pi.phone || !basic.pi.phone.trim()) {
        return t('aup.basic.validation.piPhoneRequired')
    }
    // 驗證 PI 電話格式 (9-10 碼數字，允許中間有 -)
    const piPhoneDigits = basic.pi.phone.trim().replace(/-/g, '')
    if (!/^\d{9,10}$/.test(piPhoneDigits)) {
        return t('aup.basic.validation.piPhoneInvalid')
    }
    if (!basic.pi.address || !basic.pi.address.trim()) {
        return t('aup.basic.validation.piAddressRequired')
    }

    // 6. Sponsor 資訊
    if (!basic.sponsor.name || !basic.sponsor.name.trim()) {
        return t('aup.basic.validation.sponsorNameRequired')
    }
    if (!basic.sponsor.contact_person || !basic.sponsor.contact_person.trim()) {
        return t('aup.basic.validation.sponsorContactRequired')
    }
    if (!basic.sponsor.contact_phone || !basic.sponsor.contact_phone.trim()) {
        return t('aup.basic.validation.sponsorPhoneRequired')
    }
    // 驗證委託單位電話格式 (9-10 碼數字，允許中間有 -)
    const sponsorPhoneDigits = basic.sponsor.contact_phone.trim().replace(/-/g, '')
    if (!/^\d{9,10}$/.test(sponsorPhoneDigits)) {
        return t('aup.basic.validation.sponsorPhoneInvalid')
    }
    if (!basic.sponsor.contact_email || !basic.sponsor.contact_email.trim()) {
        return t('aup.basic.validation.sponsorEmailRequired')
    }
    // 驗證委託單位 Email 格式
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(basic.sponsor.contact_email.trim())) {
        return t('aup.basic.validation.sponsorEmailInvalid')
    }

    // 7. 機構名稱
    if (!basic.facility.title || !basic.facility.title.trim()) {
        return t('aup.basic.validation.facilityRequired')
    }

    // 8. 位置
    if (!basic.housing_location || !basic.housing_location.trim()) {
        return t('aup.basic.validation.locationRequired')
    }

    // Section 2 - 研究目的
    // 2.1 研究之目的及重要性
    if (!purpose.significance || !purpose.significance.trim()) {
        return t('aup.purpose.validation.significanceRequired')
    }

    // 2.2.1 活體動物試驗之必要性
    if (!purpose.replacement.rationale || !purpose.replacement.rationale.trim()) {
        return t('aup.purpose.validation.rationaleRequired')
    }

    // 2.2.2 非動物替代方案搜尋資料庫
    if (!purpose.replacement.alt_search.platforms || purpose.replacement.alt_search.platforms.length === 0) {
        return t('aup.purpose.validation.platformsRequired')
    }
    if (!purpose.replacement.alt_search.keywords || !purpose.replacement.alt_search.keywords.trim()) {
        return t('aup.purpose.validation.keywordsRequired')
    }
    if (!purpose.replacement.alt_search.conclusion || !purpose.replacement.alt_search.conclusion.trim()) {
        return t('aup.purpose.validation.conclusionRequired')
    }

    // 2.2.3 重複試驗理由（如果選擇"是"）
    if (purpose.duplicate.experiment && (!purpose.duplicate.justification || !purpose.duplicate.justification.trim())) {
        return t('aup.purpose.validation.duplicateJustificationRequired')
    }

    // 2.3 減量原則 - 實驗設計說明
    if (!purpose.reduction.design || !purpose.reduction.design.trim()) {
        return t('aup.purpose.validation.reductionDesignRequired')
    }

    // Section 3 - 試驗物質與對照物質
    if (items.use_test_item === null) {
        return t('aup.items.validation.useTestItemRequired')
    }

    // 如果選擇"是"，驗證試驗物質和對照物質的必填字段
    if (items.use_test_item === true) {
        // 驗證試驗物質
        for (let i = 0; i < items.test_items.length; i++) {
            const item = items.test_items[i]
            if (!item.name || !item.name.trim()) {
                return t('aup.items.validation.testItemNameRequired', { index: i + 1 })
            }
            if (!item.form || !item.form.trim()) {
                return t('aup.items.validation.testFormRequired', { index: i + 1 })
            }
            if (!item.purpose || !item.purpose.trim()) {
                return t('aup.items.validation.testPurposeRequired', { index: i + 1 })
            }
            if (!item.storage_conditions || !item.storage_conditions.trim()) {
                return t('aup.items.validation.testStorageRequired', { index: i + 1 })
            }
            if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
                return t('aup.items.validation.testJustificationRequired', { index: i + 1 })
            }
        }

        // 驗證對照物質
        for (let i = 0; i < items.control_items.length; i++) {
            const item = items.control_items[i]
            if (!item.name || !item.name.trim()) {
                return t('aup.items.validation.controlItemNameRequired', { index: i + 1 })
            }
            if (!item.purpose || !item.purpose.trim()) {
                return t('aup.items.validation.controlPurposeRequired', { index: i + 1 })
            }
            if (!item.storage_conditions || !item.storage_conditions.trim()) {
                return t('aup.items.validation.controlStorageRequired', { index: i + 1 })
            }
            if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
                return t('aup.items.validation.controlJustificationRequired', { index: i + 1 })
            }
        }
    }

    // Section 4 - 研究設計與方法
    const { design } = formData.working_content
    // 4.1.1 如果選擇"是"（進行麻醉），必須選擇麻醉類型
    if (design.anesthesia.is_under_anesthesia === true) {
        if (!design.anesthesia.anesthesia_type || !design.anesthesia.anesthesia_type.trim()) {
            return t('aup.design.validation.anesthesiaTypeRequired')
        }
        // 如果選擇"其他"，必須填寫說明
        if (design.anesthesia.anesthesia_type === 'other' && (!design.anesthesia.other_description || !design.anesthesia.other_description.trim())) {
            return t('aup.design.validation.anesthesiaOtherRequired')
        }
    }
    // 4.1.2 詳細敘述動物試驗內容及流程
    if (!design.procedures || !design.procedures.trim()) {
        return t('aup.design.validation.proceduresRequired')
    }
    // 4.1.3 實驗動物等級評估
    if (!design.pain.category || !design.pain.category.trim()) {
        return t('aup.design.validation.painCategoryRequired')
    }
    // 4.1.4 如果選擇"是"（限制飲食或飲水），必須選擇限制類型
    if (design.restrictions.is_restricted === true) {
        if (!design.restrictions.restriction_type || !design.restrictions.restriction_type.trim()) {
            return t('aup.design.validation.restrictionTypeRequired')
        }
        // 如果選擇"其他"，必須填寫說明
        if (design.restrictions.restriction_type === 'other' && (!design.restrictions.other_description || !design.restrictions.other_description.trim())) {
            return t('aup.design.validation.restrictionOtherRequired')
        }
    }
    // 4.1.5 實驗預期結束之時機
    if (!design.endpoints.experimental_endpoint || !design.endpoints.experimental_endpoint.trim()) {
        return t('aup.design.validation.experimentalEndpointRequired')
    }
    if (!design.endpoints.humane_endpoint || !design.endpoints.humane_endpoint.trim()) {
        return t('aup.design.validation.humaneEndpointRequired')
    }
    // 4.3 如果選擇"是"（使用非醫藥級化學藥品），必須填寫說明
    if (design.non_pharma_grade.used === true) {
        if (!design.non_pharma_grade.description || !design.non_pharma_grade.description.trim()) {
            return t('aup.design.validation.nonPharmaRequired')
        }
    }
    // 4.4 如果選擇"是"（使用危害性物質材料），必須選擇類型並填寫材料資訊
    if (design.hazards.used === true) {
        if (!design.hazards.selected_type || !design.hazards.selected_type.trim()) {
            return t('aup.design.validation.hazardTypeRequired')
        }
        if (design.hazards.materials.length === 0 || design.hazards.materials.every(m => !m.agent_name || !m.agent_name.trim())) {
            return t('aup.design.validation.hazardMaterialsRequired')
        }
        // 驗證每個材料都有名稱和用量
        for (let i = 0; i < design.hazards.materials.length; i++) {
            const material = design.hazards.materials[i]
            if (!material.agent_name || !material.agent_name.trim()) {
                return t('aup.design.validation.hazardAgentNameRequired', { index: i + 1 })
            }
            if (!material.amount || !material.amount.trim()) {
                return t('aup.design.validation.hazardAmountRequired', { index: i + 1 })
            }
        }
        // 4.5 危害性物質及其廢棄物處理方式（如果 4.4 為"是"）
        if (!design.hazards.operation_location_method || !design.hazards.operation_location_method.trim()) {
            return t('aup.design.validation.hazardOpsRequired')
        }
        if (!design.hazards.protection_measures || !design.hazards.protection_measures.trim()) {
            return t('aup.design.validation.hazardProtectionRequired')
        }
        if (!design.hazards.waste_and_carcass_disposal || !design.hazards.waste_and_carcass_disposal.trim()) {
            return t('aup.design.validation.hazardWasteRequired')
        }
    }
    // 4.6 或 4.5（當 4.4 為"否"時）是否使用管制藥品
    if (design.controlled_substances.used === true) {
        if (design.controlled_substances.items.length === 0) {
            return t('aup.design.validation.controlledSubstancesRequired')
        }
        // 驗證每個管制藥品的必填字段
        for (let i = 0; i < design.controlled_substances.items.length; i++) {
            const item = design.controlled_substances.items[i]
            if (!item.drug_name || !item.drug_name.trim()) {
                return t('aup.design.validation.drugNameRequired', { index: i + 1 })
            }
            if (!item.approval_no || !item.approval_no.trim()) {
                return t('aup.design.validation.approvalNoRequired', { index: i + 1 })
            }
            if (!item.amount || !item.amount.trim()) {
                return t('aup.design.validation.drugAmountRequired', { index: i + 1 })
            }
            if (!item.authorized_person || !item.authorized_person.trim()) {
                return t('aup.design.validation.authorizedPersonRequired', { index: i + 1 })
            }
        }
    }

    // 6. Surgery Plan - check if needed based on 4.1.1 selection
    const needsSurgeryPlan = design.anesthesia.is_under_anesthesia === true &&
        (design.anesthesia.anesthesia_type === 'survival_surgery' || design.anesthesia.anesthesia_type === 'non_survival_surgery')

    if (needsSurgeryPlan) {
        const { surgery } = formData.working_content
        if (!surgery.surgery_type || !surgery.surgery_type.trim() || surgery.surgery_type === '略') {
            return t('aup.surgery.validation.surgeryTypeRequired')
        }
        if (!surgery.preop_preparation || !surgery.preop_preparation.trim() || surgery.preop_preparation === '略') {
            return t('aup.surgery.validation.preop_PreparationRequired')
        }
        if (!surgery.surgery_description || !surgery.surgery_description.trim() || surgery.surgery_description === '略') {
            return t('aup.surgery.validation.surgeryDescriptionRequired')
        }
        if (!surgery.monitoring || !surgery.monitoring.trim()) {
            return t('aup.surgery.validation.monitoringRequired')
        }
        // 6.6 Only required for survival surgery
        if (surgery.surgery_type === 'survival') {
            if (!surgery.postop_expected_impact || !surgery.postop_expected_impact.trim() || surgery.postop_expected_impact === '略') {
                return t('aup.surgery.validation.expectedImpactRequired')
            }
        }
        if (surgery.multiple_surgeries.used) {
            if (!surgery.multiple_surgeries.number || surgery.multiple_surgeries.number <= 0) {
                return t('aup.surgery.validation.multipleSurgeriesNumberRequired')
            }
            if (!surgery.multiple_surgeries.reason || !surgery.multiple_surgeries.reason.trim()) {
                return t('aup.surgery.validation.multipleSurgeriesReasonRequired')
            }
        }
        if (!surgery.postop_care_type || !surgery.postop_care_type.trim()) {
            return t('aup.surgery.validation.postopCareTypeRequired')
        }
        if (!surgery.postop_care || !surgery.postop_care.trim()) {
            return t('aup.surgery.validation.postopCareRequired')
        }
        if (!surgery.expected_end_point || !surgery.expected_end_point.trim()) {
            return t('aup.surgery.validation.expectedEndPointRequired')
        }
        // 6.10 Surgical medication information
        if (!surgery.drugs || surgery.drugs.length === 0) {
            return t('aup.surgery.validation.drugsRequired')
        }
        for (let i = 0; i < surgery.drugs.length; i++) {
            const drug = surgery.drugs[i]
            if (!drug.drug_name || !drug.drug_name.trim()) {
                return t('aup.surgery.validation.drugNameRequired', { index: i + 1 })
            }
            if (!drug.dose || !drug.dose.trim()) {
                return t('aup.surgery.validation.drugDoseRequired', { index: i + 1 })
            }
            if (!drug.route || !drug.route.trim()) {
                return t('aup.surgery.validation.drugRouteRequired', { index: i + 1 })
            }
            if (!drug.frequency || !drug.frequency.trim()) {
                return t('aup.surgery.validation.drugFrequencyRequired', { index: i + 1 })
            }
            if (!drug.purpose || !drug.purpose.trim()) {
                return t('aup.surgery.validation.drugPurposeRequired', { index: i + 1 })
            }
        }
    }

    // Section 7 - Experimental animal data
    const { animals } = formData.working_content
    if (!animals.animals || animals.animals.length === 0) {
        return t('aup.animals.validation.animalsRequired')
    }
    for (let i = 0; i < animals.animals.length; i++) {
        const animal = animals.animals[i]
        // Validate species
        if (!animal.species || !animal.species.trim()) {
            return t('aup.animals.validation.speciesRequired', { index: i + 1 })
        }
        if (animal.species === 'other' && (!animal.species_other || !animal.species_other.trim())) {
            return t('aup.animals.validation.speciesRequired', { index: i + 1 })
        }
        // Validate strain
        if (animal.species === 'pig' && !animal.strain) {
            return t('aup.animals.validation.strainRequired', { index: i + 1 })
        }
        if (animal.species === 'other' && (!animal.strain_other || !animal.strain_other.trim())) {
            return t('aup.animals.validation.strainRequired', { index: i + 1 })
        }
        // Validate sex
        if (!animal.sex || !animal.sex.trim()) {
            return t('aup.animals.validation.sexRequired', { index: i + 1 })
        }
        // Validate quantity
        if (!animal.number || animal.number <= 0) {
            return t('aup.animals.validation.numberRequired', { index: i + 1 })
        }
        // Validate age (if not "unlimited")
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
        // Validate weight (if not "unlimited")
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
