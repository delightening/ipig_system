// ProtocolEditPage 常數定義
// 包含 sectionKeys 和 defaultFormData

import {
    FileText,
    ClipboardList,
    Beaker,
    Stethoscope,
    User,
    Users,
    Paperclip,
} from 'lucide-react'
import { ProtocolFormData } from '@/types/protocol'

export type FormData = ProtocolFormData

export const sectionKeys = [
    { key: 'basic', labelKey: 'aup.section1', icon: FileText },
    { key: 'purpose', labelKey: 'aup.section2', icon: ClipboardList },
    { key: 'items', labelKey: 'aup.section3', icon: Beaker },
    { key: 'design', labelKey: 'aup.section4', icon: ClipboardList },
    { key: 'guidelines', labelKey: 'aup.section5', icon: FileText },
    { key: 'surgery', labelKey: 'aup.section6', icon: Stethoscope },
    { key: 'animals', labelKey: 'aup.section7', icon: User },
    { key: 'personnel', labelKey: 'aup.section8', icon: Users },
    { key: 'attachments', labelKey: 'aup.section9', icon: Paperclip },
    { key: 'signature', labelKey: 'aup.section10', icon: FileText },
]

export const defaultFormData: FormData = {
    title: '',
    start_date: '',
    end_date: '',
    working_content: {
        basic: {
            is_glp: false,
            registration_authorities: [],
            study_title: '',
            apply_study_number: '',
            start_date: '',
            end_date: '',
            project_type: '',
            project_type_other: '',
            project_category: '',
            test_item_type: '',
            tech_categories: [],
            funding_sources: [],
            funding_other: '',
            pi: { name: '', phone: '', email: '', address: '' },
            sponsor: { name: '', contact_person: '', contact_phone: '', contact_email: '' },
            sd: { name: '', email: '' },
            facility: { title: '', address: '' },
            housing_location: ''
        },
        purpose: {
            significance: '',
            replacement: {
                rationale: '',
                alt_search: { platforms: [], keywords: '', conclusion: '' }
            },
            reduction: {
                design: '',
                grouping_plan: []
            },
            duplicate: { experiment: false, justification: '' }
        },
        items: {
            use_test_item: null,
            test_items: [],
            control_items: []
        },
        design: {
            anesthesia: { is_under_anesthesia: null, plan_type: '', premed_option: '' },
            procedures: '',
            route_justifications: [],
            blood_withdrawals: [],
            imaging: [],
            restraint: [],
            pain: { category: '' },
            restrictions: { is_restricted: null, types: [] },
            endpoints: {
                experimental_endpoint: '',
                humane_endpoint: ''
            },
            final_handling: { method: '', transfer: { recipient_name: '', recipient_org: '', project_name: '' } },
            carcass_disposal: {
                method: ''
            },
            non_pharma_grade: { used: null, description: '' },
            hazards: {
                used: null,
                selected_type: undefined,
                materials: [],
                waste_disposal_method: '',
                operation_location_method: '',
                protection_measures: '',
                waste_and_carcass_disposal: ''
            },
            controlled_substances: { used: null, items: [] }
        },
        guidelines: {
            content: '',
            databases: [
                { code: 'A', checked: false, keywords: '' },
                { code: 'B', checked: false, keywords: '' },
                { code: 'C', checked: false, keywords: '' },
                { code: 'D', checked: false, keywords: '' },
                { code: 'E', checked: false, keywords: '' },
                { code: 'F', checked: false },
                { code: 'G', checked: false },
                { code: 'H', checked: false },
                { code: 'I', checked: false },
                { code: 'J', checked: false },
                { code: 'K', checked: false, note: '' },
                { code: 'L', checked: false, note: '' },
            ],
            references: [],
        },
        surgery: {
            surgery_type: '',
            preop_preparation: '',
            aseptic_techniques: [],
            surgery_description: '',
            surgery_steps: [],
            monitoring: '',
            postop_expected_impact: '',
            multiple_surgeries: { used: false, number: 0, reason: '' },
            postop_care_type: undefined,
            postop_care: '',
            drugs: [
                { drug_name: 'Atropine', dose: '1mg/ml', route: 'IM', frequency: '1 time', purpose: 'Anesthesia induction' },
                { drug_name: 'Azeperonum', dose: '0.03-0.5mg/kg', route: 'IM', frequency: '1 time', purpose: 'Anesthesia induction' },
                { drug_name: 'Zoletil®-50', dose: '3-5 mg/kg', route: 'IM', frequency: '1 time', purpose: 'Anesthesia induction' },
                { drug_name: 'Cefazolin', dose: '15-30 mg/kg', route: 'IM', frequency: '1 time pre-op / SID post-op', purpose: 'Pre- and post-operative antibiotics' },
                { drug_name: 'Meloxicam', dose: '0.1-0.4mg/kg', route: 'IM', frequency: '1 time pre-op / SID post-op', purpose: 'Pre- and post-operative analgesics' },
                { drug_name: 'Isoflurane', dose: '0.5-2%', route: 'Inhalation', frequency: 'Intraoperative', purpose: 'Anesthesia maintenance' },
                { drug_name: 'Ketoprofen', dose: '1-3mg/kg', route: 'IM', frequency: 'SID', purpose: 'Post-operative analgesics' },
                { drug_name: 'Penicillin', dose: '0.1-1mL/kg', route: 'IM', frequency: 'SID', purpose: 'Post-operative antibiotics' },
                { drug_name: 'Cephalexin', dose: '30-60mg/kg', route: 'PO', frequency: 'BID', purpose: 'Post-operative antibiotics' },
                { drug_name: 'Amoxicillin', dose: '20-40mg/kg', route: 'PO', frequency: 'BID', purpose: 'Post-operative antibiotics' },
                { drug_name: 'Meloxicam (Oral)', dose: '0.1-0.4mg/kg', route: 'PO', frequency: 'SID', purpose: 'Post-operative analgesics' }
            ],
            expected_end_point: ''
        },
        animals: {
            animals: [{
                species: '',
                species_other: '',
                strain: undefined,
                strain_other: '',
                sex: '',
                number: 0,
                age_min: undefined,
                age_max: undefined,
                age_unlimited: false,
                weight_min: undefined,
                weight_max: undefined,
                weight_unlimited: false,
                housing_location: ''
            }],
            total_animals: 0
        },
        personnel: [
            {
                id: 1,
                name: '許芮蓁',
                position: '',
                roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                roles_other_text: '',
                years_experience: 6,
                trainings: ['C', 'A'],
                training_certificates: [
                    { training_code: 'C', certificate_no: '輻安訓字第1080551' },
                    { training_code: 'C', certificate_no: '111輻協訓繼教證字第0983號' },
                    { training_code: 'A', certificate_no: '111農科實動字第0513號' },
                    { training_code: 'C', certificate_no: '112輻協訓繼教證字第3107號' }
                ]
            },
            {
                id: 2,
                name: '陳怡均',
                position: '',
                roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                roles_other_text: '',
                years_experience: 6,
                trainings: ['C', 'A'],
                training_certificates: [
                    { training_code: 'C', certificate_no: '輻安訓字第1080552' },
                    { training_code: 'A', certificate_no: '110農科實動字第0461號' },
                    { training_code: 'C', certificate_no: '111輻協訓繼教證字第4159號' },
                    { training_code: 'A', certificate_no: '112農科實動字第0213號' }
                ]
            },
            {
                id: 3,
                name: '林莉珊',
                position: '',
                roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                roles_other_text: '',
                years_experience: 4,
                trainings: ['C', 'A'],
                training_certificates: [
                    { training_code: 'C', certificate_no: '輻安訓字第1091274' },
                    { training_code: 'C', certificate_no: '111輻協訓繼教證字第0979號' },
                    { training_code: 'A', certificate_no: '111農科實動字第0512號' },
                    { training_code: 'C', certificate_no: '111輻協訓繼教證字第3105號' }
                ]
            },
            {
                id: 4,
                name: '王永發',
                position: '',
                roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                roles_other_text: '',
                years_experience: 5,
                trainings: ['C', 'A'],
                training_certificates: [
                    { training_code: 'C', certificate_no: '輻安訓字第1090109' },
                    { training_code: 'A', certificate_no: '109農科實動字第0093號' },
                    { training_code: 'C', certificate_no: '111輻協訓繼教證字第0982號' },
                    { training_code: 'A', certificate_no: '111農科實動字第0514號' }
                ]
            },
            {
                id: 5,
                name: '潘映潔',
                position: '',
                roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                roles_other_text: '',
                years_experience: 1,
                trainings: ['C', 'A'],
                training_certificates: [
                    { training_code: 'C', certificate_no: '輻安訓字第1130188' },
                    { training_code: 'A', certificate_no: '113優農實動字第0006號' }
                ]
            }
        ],
        attachments: [],
        signature: [],
    },
}
