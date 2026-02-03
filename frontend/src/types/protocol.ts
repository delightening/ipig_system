import { FileInfo } from '@/components/ui/file-upload'

export interface ProtocolWorkingContent {
    basic: { // Section 1
        is_glp: boolean
        registration_authorities: string[]
        registration_authority_other?: string
        study_title: string
        apply_study_number: string
        start_date: string
        end_date: string
        project_type: string
        project_category: string
        project_category_other?: string
        test_item_type: string
        test_item_type_other?: string
        tech_categories: string[]
        funding_sources: string[]
        funding_other?: string
        pi: {
            name: string
            phone: string
            email: string
            address: string
        }
        sponsor: {
            name: string
            contact_person: string
            contact_phone: string
            contact_email: string
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
    }
    purpose: { // Section 2
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
        }
        duplicate: {
            experiment: boolean
            justification: string
        }
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
            anesthesia_type?: string // 'survival_surgery' | 'non_survival_surgery' | 'gas_only' | 'azeperonum_atropine' | 'other'
            other_description?: string
            plan_type: string
            premed_option: string
            custom_text?: string
        }
        procedures: string
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
            category: string
            management_plan?: string
            no_analgesia_justification?: string
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
    guidelines: { // Section 5
        content: string
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
        drugs: Array<{
            drug_name: string
            dose: string
            route: string
            frequency: string
            purpose: string
        }>
        expected_end_point: string
    }
    animals: { // Section 7
        animals: Array<{
            species: 'pig' | 'other' | ''
            species_other?: string
            source_id?: string
            source_name?: string
            strain?: 'white_pig' | 'mini_pig' | ''
            strain_other?: string
            sex: string // 單一性別選擇
            number: number
            age_min?: number
            age_max?: number
            age_unlimited: boolean
            weight_min?: number
            weight_max?: number
            weight_unlimited: boolean
            housing_location: string
        }>
        total_animals: number
    }
    personnel: Array<{ // Section 8
        id?: number // 編號
        name: string
        position: string
        roles: string[] // 工作內容：a, b, c, d, e, f, g, h, i
        roles_other_text?: string // 如果選擇 i.其他，需要填寫說明
        years_experience: number // 參與動物試驗年數
        trainings: string[] // 訓練/資格：A, B, C, D, E, F
        trainings_other_text?: string // 如果選擇 F.其他，需要填寫說明
        training_certificates: Array<{ // 每個訓練的證書編號列表
            training_code: string // A, B, C, D, E
            certificate_no: string // 證書編號
        }>
    }>
    attachments: FileInfo[] // Section 9 - PDF附件
    signature: FileInfo[] // Section 10 - 電子簽名
}

export interface ProtocolFormData {
    title: string
    start_date: string
    end_date: string
    working_content: ProtocolWorkingContent
}
