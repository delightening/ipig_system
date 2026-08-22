import { FileInfo } from '@/components/ui/file-upload'

/** P2-R4-12: Section 8 人員 */
export interface ProtocolPerson {
    id?: number
    name: string
    position: string
    roles: string[]
    roles_other_text?: string
    years_experience: number
    trainings: string[]
    trainings_other_text?: string
    training_certificates: Array<{ training_code: string; certificate_no: string }>
}

/** Section 6 手術用藥 */
export interface ProtocolSurgeryDrugItem {
    drug_name: string
    dose: string
    route: string
    frequency: string
    purpose: string
}

/** Section 7 動物規格（表單用） */
export interface ProtocolAnimalItem {
    species: 'pig' | 'other' | ''
    species_other?: string
    source_id?: string
    source_name?: string
    strain?: 'white_pig' | 'mini_pig' | ''
    strain_other?: string
    sex: string
    number: number
    age_min?: number
    age_max?: number
    age_unlimited: boolean
    weight_min?: number
    weight_max?: number
    weight_unlimited: boolean
    housing_location: string
}

export interface ProtocolWorkingContent {
    basic: { // Section 1
        is_glp: boolean
        registration_authorities: string[]
        registration_authority_other?: string
        study_title: string
        apply_study_number: string
        start_date: string
        end_date: string
        project_type: string[]
        project_type_other?: string
        project_category: string[]
        project_category_other?: string
        test_item_type: string
        test_item_type_other?: string
        tech_categories: string[]
        funding_sources: string[]
        funding_other?: string
        pi: {
            name: string
            position?: string
            phone: string
            phone_ext?: string
            email: string
            address: string
        }
        sponsor: {
            name: string
            contact_person: string
            contact_phone: string
            contact_phone_ext?: string
            contact_email: string
            /** 委託單位地址（C/D 版孤兒欄位；E/F 已移除） */
            address?: string
            /** 勾選「同計畫主持人」：聯絡人三欄繼承自 PI 並唯讀 */
            contact_same_as_pi?: boolean
        }
        sd: {
            name: string
            email: string
        }
        facility: {
            title: string
            address: string
        }
        housing_location: string
        /** C/D 版孤兒：試驗單位 Test Unit 多站塊（委派階段/名稱/地址/PI；E/F 已移除） */
        test_units?: Array<{
            stage: string
            name: string
            address: string
            pi_name: string
        }>
    }
    purpose: { // Section 2
        abstract: string // 2.0 計畫摘要
        significance: string
        replacement: {
            rationale: string
            alt_search: {
                platforms: string[]
                other_name?: string
                keywords: string
                conclusion: string
            }
        }
        duplicate: { // 2.2.3 重複試驗（4 選項）
            status: string // 'no' | 'not_applicable' | 'yes_continuation' | 'yes_duplicate'
            regulation_basis: string
            previous_iacuc_no: string
            justification: string
        }
        reduction: {
            design: string
            sample_size_method?: string
            sample_size_details?: string
            grouping_plan: Array<{
                group_name: string
                n: number
                treatment: string
                timepoints: string
            }>
            // 2.3.1 特殊照護
            special_care: {
                needed: boolean | null
                description: string
            }
            // 2.3.2 單獨飼養
            single_housing: {
                required: boolean | null
                reasons: string[]
                metabolic_cage_duration: string
                monitoring_method: string
                estimated_duration: string
            }
            // 2.3.3 動物再應用
            animal_reuse: {
                considered: boolean | null
                plan: string
                plan_other: string
            }
        }
        refinement_description: string // 2.4 精緻化原則
    }
    items: { // Section 3
        use_test_item: boolean | null // null means not selected, true/false for yes/no
        test_items: Array<{
            name: string
            lot_no?: string
            expiry_date?: string
            is_sterile: boolean
            non_sterile_justification?: string
            purpose: string
            storage_conditions: string
            // C/D 版孤兒欄位（E/F 已精簡移除）：11 欄制的額外 6 欄
            composition?: string
            source?: string
            characteristics?: string
            retention?: string
            retention_location?: string
            coa?: string
            concentration?: string
            form?: string
            hazard_classification?: string
            photos?: FileInfo[]
        }>
        control_items: Array<{
            name: string
            lot_no?: string
            expiry_date?: string
            is_sterile: boolean
            non_sterile_justification?: string
            purpose: string
            storage_conditions: string
            // C/D 版孤兒欄位（E/F 已精簡移除）：11 欄制的額外 6 欄
            composition?: string
            source?: string
            characteristics?: string
            retention?: string
            retention_location?: string
            coa?: string
            concentration?: string
            form?: string
            hazard_classification?: string
            is_sham?: boolean
            is_vehicle?: boolean
            photos?: FileInfo[]
        }>
    }
    design: { // Section 4
        anesthesia: {
            is_under_anesthesia: boolean | null // null means not selected
            anesthesia_type?: string // 'survival_surgery' | 'non_survival_surgery' | 'gas_only' | 'azaperonum_atropine' | 'other'
            other_description?: string
            plan_type: string
            premed_option: string
            custom_text?: string
        }
        procedures: string
        /** C/D 版孤兒：試驗動物飼養環境 SOP 塊（溫濕度/日照/餵飼固定文字；E/F 已移除） */
        housing_environment_sop?: string
        route_justifications: Array<{
            substance_name: string
            route: string
            justification: string
        }>
        blood_withdrawals: Array<{
            timepoint: string
            volume_ml: number
            frequency: string
            site: string
            notes: string
        }>
        imaging: Array<{
            modality: string
            timepoint: string
            anesthesia_required: boolean
            notes: string
        }>
        restraint: Array<{
            method: string
            duration_min: number
            frequency: string
            welfare_notes: string
        }>
        pain: {
            category: string // 4.1.3 單選 B/C/D/E
            category_items: string[] // 4.1.3 依 category 展開的複選細項
            category_item_other_text: string // *_other 說明
            distress_signs: string[] // 4.1.5 疼痛症狀複選
            distress_signs_other_text: string
            relief_measures: string[] // 4.1.6 緩解措施複選
            relief_drug_name: string // if 'anesthesia_analgesia'
            no_relief_justification: string // if 'no_relief_with_justification'
        }
        restrictions: {
            is_restricted: boolean | null // null means not selected
            restriction_type?: string // 'fasting_before_anesthesia' | 'other'
            other_description?: string
            types: string[]
            other_text?: string
        }
        endpoints: {
            experimental_endpoint: string
            humane_endpoint: string
        }
        final_handling: {
            method: string // 'euthanasia' | 'transfer' | 'other'
            euthanasia_type?: string // 'kcl' | 'electrocution' | 'other'
            euthanasia_other_description?: string
            transfer: {
                recipient_name: string
                recipient_org: string
                project_name: string
            }
            other_description?: string
            other_text?: string
        }
        carcass_disposal: {
            method: string
            vendor_name?: string
            vendor_id?: string
        }
        non_pharma_grade: {
            used: boolean | null // null means not selected
            description: string
        }
        hazards: {
            used: boolean | null // null means not selected
            selected_type?: string // 'biological' | 'radioactive' | 'chemical' - 互斥選擇
            materials: Array<{
                _uid?: string // client-side stable id for React keys (multi-select repeater needs stable key surviving deletes); persisted via JSONB pass-through
                type: string // 'biological' | 'radioactive' | 'chemical'
                agent_name: string
                amount: string
                photos?: FileInfo[]
            }>
            waste_disposal_method: string
            operation_location_method: string
            protection_measures: string
            waste_and_carcass_disposal: string
        }
        controlled_substances: {
            used: boolean | null // null means not selected
            items: Array<{
                drug_name: string
                approval_no: string
                amount: string
                authorized_person: string
                photos?: FileInfo[]
            }>
        }
    }
    // C/D 版孤兒（GLP 專用，E/F 已移除）：結果分析 + 研究文件與歸檔
    results_analysis?: {
        acceptance_criteria: string
        statistics: string
        determination: string
    }
    document_archiving?: string
    guidelines: { // Section 5
        content: string
        databases: Array<{
            code: string        // 'A' | 'B' | ... | 'L'
            checked: boolean
            keywords?: string   // A-E 資料庫有關鍵字欄位
            note?: string       // K（同儕聯繫）、L（其他）有備註欄位
        }>
        references: Array<{
            citation: string
            url?: string
        }>
    }
    surgery: { // Section 6
        surgery_type: string
        preop_preparation: string
        aseptic_techniques: string[]
        surgery_description: string
        surgery_steps: Array<{
            step_no: number
            description: string
            estimated_duration_min: number
            key_risks: string
        }>
        monitoring: string
        postop_expected_impact: string
        multiple_surgeries: {
            used: boolean
            number: number
            reason: string
        }
        postop_care_type?: 'orthopedic' | 'non_orthopedic' // 骨科手術或非骨科手術
        postop_care: string
        drugs: ProtocolSurgeryDrugItem[]
        expected_end_point: string
    }
    animals: { // Section 7
        animals: ProtocolAnimalItem[]
        total_animals: number
    }
    personnel: ProtocolPerson[]
    attachments: FileInfo[] // Section 9 - PDF附件
    signature: FileInfo[] // Section 10 - 電子簽名（上傳模式）
    handwriting_svg?: string // Section 10 - 手寫簽名 SVG
    stroke_data?: object[] // Section 10 - 手寫筆跡資料
    signed_at?: string // Section 10 - PI 簽署時間戳（ISO UTC，簽名當下記錄）
}

export interface ProtocolFormData {
    title: string
    start_date: string
    end_date: string
    working_content: ProtocolWorkingContent
}
