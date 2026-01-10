import { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  CreateProtocolRequest,
  UpdateProtocolRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Slider } from '@/components/ui/slider'
import { toast } from '@/components/ui/use-toast'
import { useAuthStore } from '@/stores/auth'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  ArrowLeft,
  Save,
  Send,
  Loader2,
  FileText,
  User,
  Calendar,
  ClipboardList,
  Beaker,
  Stethoscope,
  Users,
  Paperclip,
  Plus,
} from 'lucide-react'

const formSections = [
  { key: 'basic', label: <>1. 研究資料<br />（Study Information）</>, icon: FileText },
  { key: 'purpose', label: <>2. 研究目的<br />（Study Purpose）</>, icon: ClipboardList },
  { key: 'items', label: <>3. 試驗物質與對照物質<br />（Testing and Control Item）</>, icon: Beaker },
  { key: 'design', label: <>4. 研究設計與方法<br />（Study Design and Methods）</>, icon: ClipboardList },
  { key: 'guidelines', label: <>5. 相關規範及參考文獻<br />（Guidelines and References）</>, icon: FileText },
  { key: 'surgery', label: <>6. 手術計畫書<br />（Animal Surgical Plan）</>, icon: Stethoscope },
  { key: 'animals', label: <>7. 實驗動物資料<br />（Animal Information）</>, icon: User },
  { key: 'personnel', label: <>8. 試驗人員資料<br />（Personnel Working on Animal Study）</>, icon: Users },
  { key: 'attachments', label: <>9. 附件<br />（Attachments）</>, icon: Paperclip },
]

interface FormData {
  title: string
  start_date: string
  end_date: string
  working_content: {
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
      trainings: string[] // 訓練/資格：A, B, C, D, E
      training_certificates: Array<{ // 每個訓練的證書編號列表
        training_code: string // A, B, C, D, E
        certificate_no: string // 證書編號
      }>
    }>
    attachments: Array<{ // Section 9
      name: string
      type: string
    }>
  }
}

const defaultFormData: FormData = {
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
      project_category: '',
      test_item_type: '',
      tech_categories: [],
      funding_sources: [],
      pi: { name: '', phone: '', email: '', address: '' },
      sponsor: { name: '', contact_person: '', contact_phone: '', contact_email: '' },
      sd: { name: '', email: '' },
      facility: { title: '豬博士動物科技股份有限公司', address: '' },
      housing_location: '苗栗縣後龍鎮外埔里外埔6-15號'
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
        humane_endpoint: '實驗過程中如果動物體重下降超過原體重的20%、食慾不振 (無法進食)、身體虛弱、感染，持續治療或傷口清創後無改善，或其他經獸醫師評估不宜持續實驗之情形，則提早結束實驗，以符合動物福祉。'
      },
      final_handling: { method: '', transfer: { recipient_name: '', recipient_org: '', project_name: '' } },
      carcass_disposal: { 
        method: '委由簽約之合格化製廠商進行化製處理\n (化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)'
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
    guidelines: { content: '', references: [] },
    surgery: {
      surgery_type: '',
      preop_preparation: '1.實驗動物術前禁食至少12小時，不禁水。\n2.試驗豬隻經清洗擦乾後，以畜舒坦( Azeperonum 40 mg/mL)3-5 mg/kg和0.03-0.05 mg/kg阿托平(Atropine® 1 mg/mL)肌肉注射鎮靜。仔細觀察豬隻呼吸頻率。\n3.經20-30分鐘後，以4.4 mg/kg舒泰-50(Zoletil®-50)肌肉注射誘導麻醉\n4.經5-10分鐘後，將豬隻移至手術檯上，以趴姿進行氣管插管，接上氣體麻醉機，以2-3 L/min流速的氧氣混合0.5-2%氣體麻醉劑Isoflurane維持麻醉，隨時注意觀察豬隻麻醉深度。\n5.術前肌肉注射抗生素Cefazolin 15 mg/kg及止痛劑meloxicam 0.4 mg/kg\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行',
      aseptic_techniques: [],
      surgery_description: '請詳述手術流程，包含手術位置、手術方法、切創大小及縫合處理',
      surgery_steps: [],
      monitoring: '手術進行中依試驗豬隻麻醉深度、呼吸頻率及手術需要，調整氧氣、笑氣流速及麻醉氣體濃度，同時注意保溫，接上生理監視器，監控記錄心跳、呼吸及體溫。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行',
      postop_expected_impact: '',
      multiple_surgeries: { used: false, number: 0, reason: '' },
      postop_care_type: undefined,
      postop_care: '',
      drugs: [
        { drug_name: 'Atropine', dose: '1mg/ml', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
        { drug_name: '畜舒坦(Azeperonum)', dose: '0.03-0.5mg/kg', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
        { drug_name: '舒泰Zoletil®-50', dose: '3-5 mg/kg', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
        { drug_name: 'Cefazolin', dose: '15-30 mg/kg', route: 'IM', frequency: '術前1次/術後SID', purpose: '術前及術後抗生素' },
        { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'IM', frequency: '術前1次/術後SID', purpose: '術前及術後止痛藥' },
        { drug_name: 'Isoflurane', dose: '0.5-2%', route: '吸入', frequency: '術中', purpose: '麻醉維持' },
        { drug_name: 'ketoprofen', dose: '1-3mg/kg', route: 'IM', frequency: 'SID', purpose: '術後止痛藥' },
        { drug_name: 'pencillin', dose: '0.1-1mL/kg', route: 'IM', frequency: 'SID', purpose: '術後抗生素' },
        { drug_name: 'cephalexin', dose: '30-60mg/kg', route: 'PO', frequency: 'BID', purpose: '術後抗生素' },
        { drug_name: 'amoxicillin', dose: '20-40mg/kg', route: 'PO', frequency: 'BID', purpose: '術後抗生素' },
        { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'PO', frequency: 'SID', purpose: '術後止痛藥' }
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
        housing_location: '豬博士畜牧場'
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
  },
}

export function ProtocolEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user } = useAuthStore()
  const isNew = !id

  const [activeSection, setActiveSection] = useState('basic')
  const [formData, setFormData] = useState<FormData>(defaultFormData)
  const [isSaving, setIsSaving] = useState(false)

  // 檢查是否為執行秘書角色（IACUC_STAFF）
  const isIACUCStaff = user?.roles?.some(r => ['IACUC_STAFF', 'SYSTEM_ADMIN'].includes(r))

  const { data: protocol, isLoading } = useQuery({
    queryKey: ['protocol', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !isNew,
  })

  useEffect(() => {
    if (protocol) {
      setFormData((prev) => {
        // Use recursive merge for working_content to ensure new fields (like pi, sponsor) 
        // from defaultFormData are preserved if missing in protocol.working_content
        const mergedWorkingContent = protocol.working_content
          ? deepMerge(defaultFormData.working_content, protocol.working_content)
          : defaultFormData.working_content

        // 如果機構名稱或位置為空，使用預設值
        if (mergedWorkingContent.basic) {
          if (!mergedWorkingContent.basic.facility?.title || !mergedWorkingContent.basic.facility.title.trim()) {
            mergedWorkingContent.basic.facility = {
              ...mergedWorkingContent.basic.facility,
              title: '豬博士動物科技股份有限公司'
            }
          }
          if (!mergedWorkingContent.basic.housing_location || !mergedWorkingContent.basic.housing_location.trim()) {
            mergedWorkingContent.basic.housing_location = '苗栗縣後龍鎮外埔里外埔6-15號'
          }
        }

        // 確保 use_test_item 如果是 undefined，則設為 null
        if (mergedWorkingContent.items && mergedWorkingContent.items.use_test_item === undefined) {
          mergedWorkingContent.items.use_test_item = null
        }

        // 確保 test_items 和 control_items 中的 photos 字段存在
        if (mergedWorkingContent.items) {
          if (mergedWorkingContent.items.test_items) {
            mergedWorkingContent.items.test_items = mergedWorkingContent.items.test_items.map((item: any) => ({
              ...item,
              photos: item.photos || []
            }))
          }
          if (mergedWorkingContent.items.control_items) {
            mergedWorkingContent.items.control_items = mergedWorkingContent.items.control_items.map((item: any) => ({
              ...item,
              photos: item.photos || []
            }))
          }
        }

        // 確保人道終點有預設內容
        if (mergedWorkingContent.design && mergedWorkingContent.design.endpoints) {
          if (!mergedWorkingContent.design.endpoints.humane_endpoint || !mergedWorkingContent.design.endpoints.humane_endpoint.trim()) {
            mergedWorkingContent.design.endpoints.humane_endpoint = '實驗過程中如果動物體重下降超過原體重的20%、食慾不振 (無法進食)、身體虛弱、感染，持續治療或傷口清創後無改善，或其他經獸醫師評估不宜持續實驗之情形，則提早結束實驗，以符合動物福祉。'
          }
        }

        // 確保動物屍體處理方法有預設內容
        if (mergedWorkingContent.design && mergedWorkingContent.design.carcass_disposal) {
          if (!mergedWorkingContent.design.carcass_disposal.method || !mergedWorkingContent.design.carcass_disposal.method.trim()) {
            mergedWorkingContent.design.carcass_disposal.method = '委由簽約之合格化製廠商進行化製處理\n(名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)'
          }
        }

        // 確保 hazards.materials 中的 photos 字段存在
        if (mergedWorkingContent.design && mergedWorkingContent.design.hazards && mergedWorkingContent.design.hazards.materials) {
          mergedWorkingContent.design.hazards.materials = mergedWorkingContent.design.hazards.materials.map((item: any) => ({
            ...item,
            photos: item.photos || []
          }))
        }

        // 確保 controlled_substances.items 中的 photos 字段存在
        if (mergedWorkingContent.design && mergedWorkingContent.design.controlled_substances && mergedWorkingContent.design.controlled_substances.items) {
          mergedWorkingContent.design.controlled_substances.items = mergedWorkingContent.design.controlled_substances.items.map((item: any) => ({
            ...item,
            photos: item.photos || []
          }))
        }

        // 確保 personnel 中的 training_certificates 字段存在
        if (mergedWorkingContent.personnel) {
          mergedWorkingContent.personnel = mergedWorkingContent.personnel.map((person: any) => ({
            ...person,
            id: person.id || undefined,
            roles: person.roles || [],
            roles_other_text: person.roles_other_text || '',
            trainings: person.trainings || [],
            training_certificates: person.training_certificates || []
          }))
        }

        // 根據 4.1.1 的選擇自動處理手術計劃書
        if (mergedWorkingContent.design && mergedWorkingContent.design.anesthesia && mergedWorkingContent.surgery) {
          const anesthesiaType = mergedWorkingContent.design.anesthesia.anesthesia_type
          const needsSurgeryPlan = mergedWorkingContent.design.anesthesia.is_under_anesthesia === true &&
            (anesthesiaType === 'survival_surgery' || anesthesiaType === 'non_survival_surgery')
          
          if (needsSurgeryPlan) {
            // 如果需要填寫手術計劃書，自動設置手術種類
            if (anesthesiaType === 'survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'survival'
            } else if (anesthesiaType === 'non_survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'non_survival'
            }
            // 如果術前準備為空，設置預設內容
            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = '1.實驗動物術前禁食至少12小時，不禁水。\n2.試驗豬隻經清洗擦乾後，以畜舒坦( Azeperonum 40 mg/mL)3-5 mg/kg和0.03-0.05 mg/kg阿托平(Atropine® 1 mg/mL)肌肉注射鎮靜。仔細觀察豬隻呼吸頻率。\n3.經20-30分鐘後，以4.4 mg/kg舒泰-50(Zoletil®-50)肌肉注射誘導麻醉\n4.經5-10分鐘後，將豬隻移至手術檯上，以趴姿進行氣管插管，接上氣體麻醉機，以2-3 L/min流速的氧氣混合0.5-2%氣體麻醉劑Isoflurane維持麻醉，隨時注意觀察豬隻麻醉深度。\n5.術前肌肉注射抗生素Cefazolin 15 mg/kg及止痛劑meloxicam 0.4 mg/kg\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行'
            }
            // 如果手術內容說明為空，設置預設內容
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = '請詳述手術流程，包含手術位置、手術方法、切創大小及縫合處理'
            }
            // 如果術中監控為空，設置預設內容
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = '手術進行中依試驗豬隻麻醉深度、呼吸頻率及手術需要，調整氧氣、笑氣流速及麻醉氣體濃度，同時注意保溫，接上生理監視器，監控記錄心跳、呼吸及體溫。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行'
            }
            // 如果術後照護類型未選擇，不自動設置（讓用戶選擇）
          } else {
            // 如果不需要填寫手術計劃書，自動填寫"略"
            if (!mergedWorkingContent.surgery.surgery_type || mergedWorkingContent.surgery.surgery_type.trim() === '') {
              mergedWorkingContent.surgery.surgery_type = '略'
            }
            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = '略'
            }
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = '略'
            }
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = '略'
            }
            if (!mergedWorkingContent.surgery.postop_expected_impact || mergedWorkingContent.surgery.postop_expected_impact.trim() === '') {
              mergedWorkingContent.surgery.postop_expected_impact = '略'
            }
            if (!mergedWorkingContent.surgery.postop_care || mergedWorkingContent.surgery.postop_care.trim() === '') {
              mergedWorkingContent.surgery.postop_care = '1.術後每日評估豬隻健康狀態，依術後狀況進行傷口護理。\n2.術後7日內每日進行疼痛評估依豬隻狀況並依照不同手術給予止痛藥及抗生素\n分為兩項：\n2.1\n骨科手術\n止痛藥\nketoprofen 1-3 mg/kg IM SID (3天)\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14天或長期給予)\n抗生素\ncefazolin 15 mg/kg IM BID (1-7天)\ncephalexin 30 mg/kg PO BID (8-14天或長期給予)\n\n2.2\n非骨科手術\n止痛藥\nmeloxicam 0.1-0.4 mg/kg IM SID (3天)\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14天或長期給予)\n抗生素\npencillin 10000 u/kg IM SID (1-7天)\namoxicillin 20 mg/kg PO BID (8-14天或長期給予)\n\n3.若動物發生異常情形，則依獸醫師指示處理。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行'
            }
            if (!mergedWorkingContent.surgery.expected_end_point || mergedWorkingContent.surgery.expected_end_point.trim() === '') {
              mergedWorkingContent.surgery.expected_end_point = '略'
            }
            // 如果不需要填寫手術計劃書，清空用藥資訊
            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = []
            }
            // 如果手術用藥資訊為空，設置預設內容
            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = [
                { drug_name: 'Atropine', dose: '1mg/ml', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
                { drug_name: '畜舒坦(Azeperonum)', dose: '0.03-0.5mg/kg', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
                { drug_name: '舒泰Zoletil®-50', dose: '3-5 mg/kg', route: 'IM', frequency: '1次', purpose: '麻醉誘導' },
                { drug_name: 'Cefazolin', dose: '15-30 mg/kg', route: 'IM', frequency: '術前1次/術後SID', purpose: '術前及術後抗生素' },
                { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'IM', frequency: '術前1次/術後SID', purpose: '術前及術後止痛藥' },
                { drug_name: 'Isoflurane', dose: '0.5-2%', route: '吸入', frequency: '術中', purpose: '麻醉維持' },
                { drug_name: 'ketoprofen', dose: '1-3mg/kg', route: 'IM', frequency: 'SID', purpose: '術後止痛藥' },
                { drug_name: 'pencillin', dose: '0.1-1mL/kg', route: 'IM', frequency: 'SID', purpose: '術後抗生素' },
                { drug_name: 'cephalexin', dose: '30-60mg/kg', route: 'PO', frequency: 'BID', purpose: '術後抗生素' },
                { drug_name: 'amoxicillin', dose: '20-40mg/kg', route: 'PO', frequency: 'BID', purpose: '術後抗生素' },
                { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'PO', frequency: 'SID', purpose: '術後止痛藥' }
              ]
            }
          }
        }

        return {
          title: protocol.title,
          start_date: protocol.start_date || '',
          end_date: protocol.end_date || '',
          working_content: mergedWorkingContent as FormData['working_content'],
        }
      })
    }
  }, [protocol])

  // Helper for deep merging objects
  function deepMerge(target: any, source: any): any {
    if (typeof target !== 'object' || target === null ||
      typeof source !== 'object' || source === null) {
      return source;
    }

    if (Array.isArray(target) && Array.isArray(source)) {
      // For arrays, we generally prefer the source (database) value, 
      // unless we want to merge items which is rare/dangerous for lists.
      // Here we just take the source array.
      return source;
    }

    if (Array.isArray(target) || Array.isArray(source)) {
      return source; // Mismatched types, take source
    }

    const output = { ...target };
    Object.keys(source).forEach(key => {
      if (Object.prototype.hasOwnProperty.call(source, key)) {
        if (key in target) {
          output[key] = deepMerge(target[key], source[key]);
        } else {
          output[key] = source[key];
        }
      }
    });
    return output;
  }

  const createMutation = useMutation({
    mutationFn: async (data: CreateProtocolRequest) => api.post('/protocols', data),
    onSuccess: (response) => {
      toast({
        title: '成功',
        description: '計畫書已建立',
      })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
      navigate(`/protocols/${response.data.id}`)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '建立失敗',
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async (data: UpdateProtocolRequest) => api.put(`/protocols/${id}`, data),
    onSuccess: () => {
      toast({
        title: '成功',
        description: '計畫書已儲存',
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '儲存失敗',
        variant: 'destructive',
      })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/submit`),
    onSuccess: () => {
      toast({
        title: '成功',
        description: '計畫書已提交審查',
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      navigate(`/protocols/${id}`)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '提交失敗',
        variant: 'destructive',
      })
    },
  })

  // 驗證必填字段（Section 1 - 研究資料）
  const validateRequiredFields = (): string | null => {
    const { basic, purpose } = formData.working_content

    // 1. 研究名稱
    if (!formData.title.trim()) {
      return '請填寫研究名稱 (Study Title)'
    }

    // 2. 預計試驗時程
    if (!formData.start_date || !formData.end_date) {
      return '請填寫預計試驗時程'
    }

    // 3. 計畫類型
    if (!basic.project_type || !basic.project_type.trim()) {
      return '請選擇計畫類型'
    }

    // 4. 計畫種類
    if (!basic.project_category || !basic.project_category.trim()) {
      return '請選擇計畫種類'
    }
    if (basic.project_category === 'other' && (!basic.project_category_other || !basic.project_category_other.trim())) {
      return '請填寫其他計畫種類說明'
    }

    // 5. PI 資訊
    if (!basic.pi.name || !basic.pi.name.trim()) {
      return '請填寫計畫主持人姓名'
    }
    if (!basic.pi.email || !basic.pi.email.trim()) {
      return '請填寫計畫主持人 Email'
    }
    if (!basic.pi.phone || !basic.pi.phone.trim()) {
      return '請填寫計畫主持人電話'
    }
    if (!basic.pi.address || !basic.pi.address.trim()) {
      return '請填寫計畫主持人地址'
    }

    // 6. Sponsor 資訊
    if (!basic.sponsor.name || !basic.sponsor.name.trim()) {
      return '請填寫委託單位名稱'
    }
    if (!basic.sponsor.contact_person || !basic.sponsor.contact_person.trim()) {
      return '請填寫委託單位聯絡人'
    }
    if (!basic.sponsor.contact_phone || !basic.sponsor.contact_phone.trim()) {
      return '請填寫委託單位聯絡電話'
    }
    if (!basic.sponsor.contact_email || !basic.sponsor.contact_email.trim()) {
      return '請填寫委託單位聯絡 Email'
    }

    // 7. 機構名稱
    if (!basic.facility.title || !basic.facility.title.trim()) {
      return '請填寫機構名稱'
    }

    // 8. 位置
    if (!basic.housing_location || !basic.housing_location.trim()) {
      return '請填寫位置'
    }

    // Section 2 - 研究目的
    // 2.1 研究之目的及重要性
    if (!purpose.significance || !purpose.significance.trim()) {
      return '請填寫研究之目的及重要性'
    }

    // 2.2.1 活體動物試驗之必要性
    if (!purpose.replacement.rationale || !purpose.replacement.rationale.trim()) {
      return '請說明活體動物試驗之必要性，以及選擇此動物種別的原因'
    }

    // 2.2.2 非動物替代方案搜尋資料庫
    if (!purpose.replacement.alt_search.platforms || purpose.replacement.alt_search.platforms.length === 0) {
      return '請至少選擇一個非動物性替代方案搜尋資料庫'
    }
    if (!purpose.replacement.alt_search.keywords || !purpose.replacement.alt_search.keywords.trim()) {
      return '請填寫搜尋關鍵字'
    }
    if (!purpose.replacement.alt_search.conclusion || !purpose.replacement.alt_search.conclusion.trim()) {
      return '請填寫搜尋結果與結論'
    }

    // 2.2.3 重複試驗理由（如果選擇"是"）
    if (purpose.duplicate.experiment && (!purpose.duplicate.justification || !purpose.duplicate.justification.trim())) {
      return '請說明重複進行之科學理由'
    }

    // 2.3 減量原則 - 實驗設計說明
    if (!purpose.reduction.design || !purpose.reduction.design.trim()) {
      return '請填寫實驗設計說明（包括動物分組方法、訂定使用動物數量之理由等）'
    }

    // Section 4 - 研究設計與方法
    const { design } = formData.working_content
    // 4.1.1 如果選擇"是"（進行麻醉），必須選擇麻醉類型
    if (design.anesthesia.is_under_anesthesia === true) {
      if (!design.anesthesia.anesthesia_type || !design.anesthesia.anesthesia_type.trim()) {
        return '請選擇麻醉類型'
      }
      // 如果選擇"其他"，必須填寫說明
      if (design.anesthesia.anesthesia_type === 'other' && (!design.anesthesia.other_description || !design.anesthesia.other_description.trim())) {
        return '請填寫其他麻醉方式的說明'
      }
    }
    // 4.1.2 詳細敘述動物試驗內容及流程
    if (!design.procedures || !design.procedures.trim()) {
      return '請詳細敘述動物試驗內容及流程'
    }
    // 4.1.3 實驗動物等級評估
    if (!design.pain.category || !design.pain.category.trim()) {
      return '請選擇實驗動物等級評估'
    }
    // 4.1.4 如果選擇"是"（限制飲食或飲水），必須選擇限制類型
    if (design.restrictions.is_restricted === true) {
      if (!design.restrictions.restriction_type || !design.restrictions.restriction_type.trim()) {
        return '請選擇限制類型'
      }
      // 如果選擇"其他"，必須填寫說明
      if (design.restrictions.restriction_type === 'other' && (!design.restrictions.other_description || !design.restrictions.other_description.trim())) {
        return '請填寫其他限制方式的說明'
      }
    }
    // 4.1.5 實驗預期結束之時機
    if (!design.endpoints.experimental_endpoint || !design.endpoints.experimental_endpoint.trim()) {
      return '請填寫實驗終點'
    }
    if (!design.endpoints.humane_endpoint || !design.endpoints.humane_endpoint.trim()) {
      return '請填寫人道終點'
    }
    // 4.3 如果選擇"是"（使用非醫藥級化學藥品），必須填寫說明
    if (design.non_pharma_grade.used === true) {
      if (!design.non_pharma_grade.description || !design.non_pharma_grade.description.trim()) {
        return '請說明物質性質、安全性及使用之科學理由'
      }
    }
    // 4.4 如果選擇"是"（使用危害性物質材料），必須選擇類型並填寫材料資訊
    if (design.hazards.used === true) {
      if (!design.hazards.selected_type || !design.hazards.selected_type.trim()) {
        return '請選擇危害性物質類型'
      }
      if (design.hazards.materials.length === 0 || design.hazards.materials.every(m => !m.agent_name || !m.agent_name.trim())) {
        return '請至少填寫一個危害性物質的名稱'
      }
      // 驗證每個材料都有名稱和用量
      for (let i = 0; i < design.hazards.materials.length; i++) {
        const material = design.hazards.materials[i]
        if (!material.agent_name || !material.agent_name.trim()) {
          return `請填寫第 ${i + 1} 個危害性物質的名稱`
        }
        if (!material.amount || !material.amount.trim()) {
          return `請填寫第 ${i + 1} 個危害性物質的所需用量`
        }
      }
      // 4.5 危害性物質及其廢棄物處理方式（如果 4.4 為"是"）
      if (!design.hazards.operation_location_method || !design.hazards.operation_location_method.trim()) {
        return '請填寫施用方法、途徑與使用場所'
      }
      if (!design.hazards.protection_measures || !design.hazards.protection_measures.trim()) {
        return '請填寫保護措施'
      }
      if (!design.hazards.waste_and_carcass_disposal || !design.hazards.waste_and_carcass_disposal.trim()) {
        return '請填寫實驗廢棄物與屍體之處理方式'
      }
    }
    // 4.6 或 4.5（當 4.4 為"否"時）是否使用管制藥品
    if (design.controlled_substances.used === true) {
      if (design.controlled_substances.items.length === 0) {
        return '請至少添加一個管制藥品'
      }
      // 驗證每個管制藥品的必填字段
      for (let i = 0; i < design.controlled_substances.items.length; i++) {
        const item = design.controlled_substances.items[i]
        if (!item.drug_name || !item.drug_name.trim()) {
          return `請填寫第 ${i + 1} 個管制藥品的藥品名稱`
        }
        if (!item.approval_no || !item.approval_no.trim()) {
          return `請填寫第 ${i + 1} 個管制藥品的核准編號`
        }
        if (!item.amount || !item.amount.trim()) {
          return `請填寫第 ${i + 1} 個管制藥品的所需用量`
        }
        if (!item.authorized_person || !item.authorized_person.trim()) {
          return `請填寫第 ${i + 1} 個管制藥品的管制藥品管理人`
        }
      }
    }

    // 6. 手術計劃書 - 根據 4.1.1 的選擇判斷是否需要填寫
    const needsSurgeryPlan = design.anesthesia.is_under_anesthesia === true &&
      (design.anesthesia.anesthesia_type === 'survival_surgery' || design.anesthesia.anesthesia_type === 'non_survival_surgery')
    
    if (needsSurgeryPlan) {
      const { surgery } = formData.working_content
      if (!surgery.surgery_type || !surgery.surgery_type.trim() || surgery.surgery_type === '略') {
        return '請填寫手術種類'
      }
      if (!surgery.preop_preparation || !surgery.preop_preparation.trim() || surgery.preop_preparation === '略') {
        return '請填寫術前準備'
      }
      if (!surgery.surgery_description || !surgery.surgery_description.trim() || surgery.surgery_description === '略') {
        return '請填寫手術內容說明'
      }
      if (!surgery.monitoring || !surgery.monitoring.trim()) {
        return '請填寫術中監控'
      }
      // 6.6 只在存活手術時需要填寫
      if (surgery.surgery_type === 'survival') {
        if (!surgery.postop_expected_impact || !surgery.postop_expected_impact.trim() || surgery.postop_expected_impact === '略') {
          return '請填寫存活手術預期術後可能對實驗動物造成之影響'
        }
      }
      if (surgery.multiple_surgeries.used) {
        if (!surgery.multiple_surgeries.number || surgery.multiple_surgeries.number <= 0) {
          return '請填寫多次手術的數量'
        }
        if (!surgery.multiple_surgeries.reason || !surgery.multiple_surgeries.reason.trim()) {
          return '請填寫多次手術的原因'
        }
      }
      if (!surgery.postop_care_type || !surgery.postop_care_type.trim()) {
        return '請選擇手術類型（骨科手術或非骨科手術）'
      }
      if (!surgery.postop_care || !surgery.postop_care.trim()) {
        return '請填寫動物術後照護及止痛給藥方法'
      }
      if (!surgery.expected_end_point || !surgery.expected_end_point.trim()) {
        return '請填寫實驗預期結束之時機'
      }
      // 6.10 手術用藥資訊
      if (!surgery.drugs || surgery.drugs.length === 0) {
        return '請至少添加一個手術用藥資訊'
      }
      for (let i = 0; i < surgery.drugs.length; i++) {
        const drug = surgery.drugs[i]
        if (!drug.drug_name || !drug.drug_name.trim()) {
          return `請填寫第 ${i + 1} 個用藥的藥品名稱`
        }
        if (!drug.dose || !drug.dose.trim()) {
          return `請填寫第 ${i + 1} 個用藥的劑量`
        }
        if (!drug.route || !drug.route.trim()) {
          return `請填寫第 ${i + 1} 個用藥的投與途徑`
        }
        if (!drug.frequency || !drug.frequency.trim()) {
          return `請填寫第 ${i + 1} 個用藥的頻率`
        }
        if (!drug.purpose || !drug.purpose.trim()) {
          return `請填寫第 ${i + 1} 個用藥的給藥目的`
        }
      }
    }

    // Section 3 - 試驗物質與對照物質
    const { items } = formData.working_content
    if (items.use_test_item === null) {
      return '請選擇是否投予「試驗物質」於動物'
    }

    // 如果選擇"是"，驗證試驗物質和對照物質的必填字段
    if (items.use_test_item === true) {
      // 驗證試驗物質
      for (let i = 0; i < items.test_items.length; i++) {
        const item = items.test_items[i]
        if (!item.name || !item.name.trim()) {
          return `請填寫第 ${i + 1} 個試驗物質的名稱`
        }
        // 如果選擇"否"（非無菌製備），必須填寫說明
        if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
          return `請填寫第 ${i + 1} 個試驗物質的非無菌製備說明`
        }
      }

      // 驗證對照物質
      for (let i = 0; i < items.control_items.length; i++) {
        const item = items.control_items[i]
        if (!item.name || !item.name.trim()) {
          return `請填寫第 ${i + 1} 個對照物質的名稱`
        }
        // 如果選擇"否"（非無菌製備），必須填寫說明
        if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
          return `請填寫第 ${i + 1} 個對照物質的非無菌製備說明`
        }
      }
    }

    // Section 7 - 實驗動物資料
    const { animals } = formData.working_content
    if (!animals.animals || animals.animals.length === 0) {
      return '請至少添加一個實驗動物資料'
    }
    for (let i = 0; i < animals.animals.length; i++) {
      const animal = animals.animals[i]
      // 驗證物種
      if (!animal.species || !animal.species.trim()) {
        return `請選擇第 ${i + 1} 個動物的物種`
      }
      if (animal.species === 'other' && (!animal.species_other || !animal.species_other.trim())) {
        return `請填寫第 ${i + 1} 個動物的物種`
      }
      // 驗證品系
      if (animal.species === 'pig' && !animal.strain) {
        return `請選擇第 ${i + 1} 個動物的品系`
      }
      if (animal.species === 'other' && (!animal.strain_other || !animal.strain_other.trim())) {
        return `請填寫第 ${i + 1} 個動物的品系`
      }
      // 驗證性別
      if (!animal.sex || !animal.sex.trim()) {
        return `請選擇第 ${i + 1} 個動物的性別`
      }
      // 驗證數量
      if (!animal.number || animal.number <= 0) {
        return `請填寫第 ${i + 1} 個動物的數量（必須大於0）`
      }
      // 驗證年齡（如果不是"不限"）
      if (!animal.age_unlimited) {
        if (animal.age_min === undefined || animal.age_min < 3) {
          return `第 ${i + 1} 個動物的最小月齡必須至少為3個月`
        }
        if (animal.age_max === undefined) {
          return `請填寫第 ${i + 1} 個動物的最大月齡`
        }
        if (animal.age_max <= animal.age_min) {
          return `第 ${i + 1} 個動物的最大月齡必須大於最小月齡`
        }
      }
      // 驗證體重（如果不是"不限"）
      if (!animal.weight_unlimited) {
        if (animal.weight_min === undefined || animal.weight_min < 20) {
          return `第 ${i + 1} 個動物的最小體重必須至少為20公斤`
        }
        if (animal.weight_max === undefined) {
          return `請填寫第 ${i + 1} 個動物的最大體重`
        }
        if (animal.weight_max <= animal.weight_min) {
          return `第 ${i + 1} 個動物的最大體重必須大於最小體重`
        }
      }
    }

    return null
  }

  const handleSave = async () => {
    // 驗證必填字段
    const validationError = validateRequiredFields()
    if (validationError) {
      toast({
        title: '錯誤',
        description: validationError,
        variant: 'destructive',
      })
      return
    }

    setIsSaving(true)
    try {
      // 如果不是執行秘書，確保試驗編號為空
      const basicContent = {
        ...formData.working_content.basic,
        study_title: formData.title,
        start_date: formData.start_date,
        end_date: formData.end_date,
      }
      
      // 如果不是 IACUC_STAFF，清空試驗編號
      if (!isIACUCStaff) {
        basicContent.apply_study_number = ''
      }

      const data = {
        title: formData.title,
        working_content: {
          ...formData.working_content,
          basic: basicContent,
        },
        start_date: formData.start_date || undefined,
        end_date: formData.end_date || undefined,
      }

      if (isNew) {
        await createMutation.mutateAsync(data)
      } else {
        await updateMutation.mutateAsync(data)
      }
    } finally {
      setIsSaving(false)
    }
  }

  const handleSubmit = async () => {
    if (!id) return
    await handleSave()

    if (confirm('確認要提交此計畫書進行審查嗎？提交後將無法直接修改')) {
      submitMutation.mutate()
    }
  }

  const updateWorkingContent = (section: keyof FormData['working_content'], path: string, value: any) => {
    setFormData((prev) => {
      const newContent = { ...prev.working_content }
      // @ts-ignore
      const sectionData = { ...(newContent[section] as any) }

      if (path.includes('.')) {
        const parts = path.split('.')
        let current = sectionData
        for (let i = 0; i < parts.length - 1; i++) {
          current[parts[i]] = { ...current[parts[i]] }
          current = current[parts[i]]
        }
        current[parts[parts.length - 1]] = value
      } else {
        sectionData[path] = value
      }

      newContent[section] = sectionData
      return { ...prev, working_content: newContent }
    })
  }

  if (!isNew && isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate(-1)}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-bold">
              {isNew ? '新增計畫書' : '編輯計畫書'}
            </h1>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleSave} disabled={isSaving}>
            {isSaving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            儲存草稿
          </Button>
          {!isNew && (
            <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              提交審查
            </Button>
          )}
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[250px_1fr]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-lg">章節</CardTitle>
          </CardHeader>
          <CardContent className="p-2">
            <nav className="space-y-1">
              {formSections.map((section) => (
                <button
                  key={section.key}
                  onClick={() => setActiveSection(section.key)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors ${activeSection === section.key
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-slate-600 hover:bg-slate-100'
                    }`}
                >
                  <section.icon className="h-4 w-4" />
                  <span className="text-sm font-medium">{section.label}</span>
                </button>
              ))}
            </nav>
          </CardContent>
        </Card>

        <div className="space-y-6">
          {activeSection === 'basic' && (
            <Card>
              <CardHeader>
                <CardTitle>1. 研究資料<br />(Study Information)</CardTitle>
                <CardDescription>填寫研究基本資訊、試驗機構與主持人資料</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 1. GLP & Title */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>GLP 屬性 *</Label>
                    <div className="flex items-center space-x-2 pt-2">
                      <Checkbox
                        id="is_glp"
                        checked={formData.working_content.basic.is_glp}
                        onChange={(e) => updateWorkingContent('basic', 'is_glp', e.target.checked)}
                      />
                      <Label htmlFor="is_glp">符合 GLP 規範</Label>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="title">研究名稱 (Study Title) *</Label>
                    <Input
                      id="title"
                      value={formData.title}
                      onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
                      placeholder="請輸入研究名稱"
                    />
                  </div>
                </div>

                {/* 2. IDs and Dates */}
                <div className={`grid gap-4 ${isNew || !isIACUCStaff ? 'md:grid-cols-1' : 'md:grid-cols-2'}`}>
                  {/* 試驗編號：新增頁面隱藏，編輯頁面只有執行秘書可編輯 */}
                  {(!isNew && isIACUCStaff) && (
                    <div className="space-y-2">
                      <Label htmlFor="apply_study_number">試驗編號 (Study No.)</Label>
                      <Input
                        id="apply_study_number"
                        value={formData.working_content.basic.apply_study_number || ''}
                        onChange={(e) => updateWorkingContent('basic', 'apply_study_number', e.target.value)}
                        placeholder="由執行秘書填寫"
                      />
                    </div>
                  )}
                  <div className="space-y-2">
                    <Label>預計試驗時程 *</Label>
                    <div className="flex gap-2">
                      <Input
                        type="date"
                        value={formData.start_date}
                        onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                        required
                      />
                      <span className="self-center">至</span>
                      <Input
                        type="date"
                        value={formData.end_date}
                        onChange={(e) => setFormData(prev => ({ ...prev, end_date: e.target.value }))}
                        required
                      />
                    </div>
                  </div>
                </div>

                {/* 3. Types */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>計畫類型 *</Label>
                    <Select
                      value={formData.working_content.basic.project_type}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_type', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="選擇計畫類型" /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="basic_research">基礎研究</SelectItem>
                        <SelectItem value="applied_research">應用研究</SelectItem>
                        <SelectItem value="pre_market_testing">上市前試驗</SelectItem>
                        <SelectItem value="teaching_training">教學訓練</SelectItem>
                        <SelectItem value="biologics_manufacturing">生物製劑製造</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label>計畫種類 *</Label>
                    <Select
                      value={formData.working_content.basic.project_category}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="選擇計畫種類" /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="medical">醫藥</SelectItem>
                        <SelectItem value="agricultural">農業</SelectItem>
                        <SelectItem value="drug_herbal">藥用植物</SelectItem>
                        <SelectItem value="health_food">健康食品</SelectItem>
                        <SelectItem value="food">食品</SelectItem>
                        <SelectItem value="toxic_chemical">毒性化學物質</SelectItem>
                        <SelectItem value="medical_device">醫療器材</SelectItem>
                        <SelectItem value="other">其他</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.basic.project_category === 'other' && (
                      <div className="pt-2">
                        <Input
                          placeholder="請說明其他種類"
                          value={formData.working_content.basic.project_category_other || ''}
                          onChange={(e) => updateWorkingContent('basic', 'project_category_other', e.target.value)}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4. PI Info */}
                <div className="space-y-4">
                  <h3 className="font-semibold">計畫主持人 (Principal Investigator)</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>姓名 *</Label>
                      <Input
                        value={formData.working_content.basic.pi.name}
                        onChange={(e) => updateWorkingContent('basic', 'pi.name', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>Email *</Label>
                      <Input
                        value={formData.working_content.basic.pi.email}
                        onChange={(e) => updateWorkingContent('basic', 'pi.email', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>電話 *</Label>
                      <Input
                        value={formData.working_content.basic.pi.phone}
                        onChange={(e) => updateWorkingContent('basic', 'pi.phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>地址 *</Label>
                      <Input
                        value={formData.working_content.basic.pi.address}
                        onChange={(e) => updateWorkingContent('basic', 'pi.address', e.target.value)}
                      />
                    </div>
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 5. Sponsor Info */}
                <div className="space-y-4">
                  <h3 className="font-semibold">委託單位 (Sponsor)</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>單位名稱 *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.name}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.name', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>聯絡人 *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_person}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_person', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>聯絡電話 *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_phone}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>聯絡 Email *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_email}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_email', e.target.value)}
                      />
                    </div>
                  </div>
                </div>

                {/* 6. Facility */}
                <div className="space-y-4">
                  <h3 className="font-semibold">試驗機構與設施</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>機構名稱 *</Label>
                      <Input
                        value={formData.working_content.basic.facility.title}
                        onChange={(e) => updateWorkingContent('basic', 'facility.title', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>位置 *</Label>
                      <Input
                        value={formData.working_content.basic.housing_location}
                        onChange={(e) => updateWorkingContent('basic', 'housing_location', e.target.value)}
                      />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'purpose' && (
            <Card>
              <CardHeader>
                <CardTitle>2. 研究目的<br />(Study Purpose)</CardTitle>
                <CardDescription>說明研究目的、重要性與 3Rs 替代、減量原則</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 2.1 研究之目的及重要性 */}
                <div className="space-y-2">
                  <Label>2.1 研究之目的及重要性 *</Label>
                  <Textarea
                    value={formData.working_content.purpose.significance}
                    onChange={(e) => updateWorkingContent('purpose', 'significance', e.target.value)}
                    placeholder="請說明研究背景、臨床或科學重要性及其預期成果"
                    rows={5}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.2 替代原則 */}
                <div className="space-y-4">
                  <h3 className="font-semibold">2.2 請以動物試驗應用3Rs之替代原則，說明本動物試驗之合理性:</h3>
                  
                  {/* 2.2.1 活體動物試驗之必要性 */}
                  <div className="space-y-2">
                    <Label>2.2.1 請說明活體動物試驗之必要性，以及選擇此動物種別的原因: *</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.rationale}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.rationale', e.target.value)}
                      placeholder="請說明活體動物試驗之必要性，以及選擇此動物種別的原因"
                      rows={4}
                    />
                  </div>

                  {/* 2.2.2 非動物性替代方案搜尋資料庫 */}
                  <div className="space-y-2">
                    <Label>2.2.2 請於下列網站搜尋非動物性替代方案 *</Label>
                    <div className="space-y-4 pl-4">
                      <div className="flex items-start space-x-3 py-2">
                        <Checkbox
                          id="search_altbib"
                          checked={formData.working_content.purpose.replacement.alt_search.platforms.includes('altbib')}
                          onChange={(e) => {
                            const checked = e.target.checked
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 'altbib']
                              : current.filter(p => p !== 'altbib')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_altbib" className="font-normal leading-relaxed flex-1">
                          1. ALTBIB-非動物性替代方法參考文獻搜索工具<br />
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
                          onChange={(e) => {
                            const checked = e.target.checked
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 'db_alm']
                              : current.filter(p => p !== 'db_alm')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_db_alm" className="font-normal leading-relaxed flex-1">
                          2. DB-ALM動物試驗替代方法資料庫<br />
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
                          onChange={(e) => {
                            const checked = e.target.checked
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 're_place']
                              : current.filter(p => p !== 're_place')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_re_place" className="font-normal leading-relaxed flex-1">
                          3. 歐洲動物替代試驗資源平台<br />
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
                        placeholder="請說明其他資料庫"
                        value={formData.working_content.purpose.replacement.alt_search.other_name || ''}
                        onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.other_name', e.target.value)}
                        className="mt-2"
                      />
                    )}
                  </div>
                  <div className="space-y-2">
                    <Label>搜尋關鍵字 *</Label>
                    <Input
                      value={formData.working_content.purpose.replacement.alt_search.keywords}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.keywords', e.target.value)}
                      placeholder="例如：minipig, cardiovascular, replacement"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>搜尋結果與結論 *</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.alt_search.conclusion}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.conclusion', e.target.value)}
                      placeholder="說明搜尋結果是否發現可替代方案"
                      rows={3}
                    />
                  </div>

                  {/* 2.2.3 是否為重複他人試驗 */}
                  <div className="space-y-2">
                    <Label>2.2.3 是否為重複他人試驗</Label>
                    <Select
                      value={formData.working_content.purpose.duplicate.experiment ? 'yes' : 'no'}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('purpose', 'duplicate.experiment', isYes)
                        // 如果選擇"否"，清空說明欄位
                        if (!isYes) {
                          updateWorkingContent('purpose', 'duplicate.justification', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">否</SelectItem>
                        <SelectItem value="yes">是</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.purpose.duplicate.experiment && (
                      <div className="space-y-2 mt-2">
                        <Label>請說明重複進行之科學理由 *</Label>
                        <Textarea
                          value={formData.working_content.purpose.duplicate.justification}
                          onChange={(e) => updateWorkingContent('purpose', 'duplicate.justification', e.target.value)}
                          placeholder="請說明重複進行之科學理由"
                          rows={3}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.3 減量原則 */}
                <div className="space-y-4">
                  <h3 className="font-semibold">2.3 請以實驗動物應用3Rs之減量原則，說明動物試驗設計，包括動物分組方法、訂定使用動物數量之理由等:</h3>
                  <div className="space-y-2">
                    <Label>實驗設計說明 *</Label>
                    <Textarea
                      value={formData.working_content.purpose.reduction.design}
                      onChange={(e) => updateWorkingContent('purpose', 'reduction.design', e.target.value)}
                      placeholder="請說明動物分組方法、統計假設、納入排除標準、減少變異之方法，以及訂定使用動物數量之理由"
                      rows={6}
                    />
                  </div>
                </div>

              </CardContent>
            </Card>
          )}

          {activeSection === 'items' && (
            <Card>
              <CardHeader>
                <CardTitle>3. 試驗物質與對照物質<br />(Testing and Control Item)</CardTitle>
                <CardDescription>填寫試驗物質與對照物質資訊</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>本計畫是否投予「試驗物質」於動物 *</Label>
                  <Select
                    value={formData.working_content.items.use_test_item === null ? '' : (formData.working_content.items.use_test_item ? 'yes' : 'no')}
                    onValueChange={(value) => {
                      const isYes = value === 'yes'
                      updateWorkingContent('items', 'use_test_item', isYes)
                      // 如果選擇"否"，清空物質列表
                      if (!isYes) {
                        updateWorkingContent('items', 'test_items', [])
                        updateWorkingContent('items', 'control_items', [])
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="請選擇" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="no">否</SelectItem>
                      <SelectItem value="yes">是</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {formData.working_content.items.use_test_item === true && (
                  <>
                    {/* 試驗物質列表 */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">試驗物質</h3>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            const newItems = [...formData.working_content.items.test_items, {
                              name: '', is_sterile: true, purpose: '', storage_conditions: '', photos: []
                            }]
                            updateWorkingContent('items', 'test_items', newItems)
                          }}
                        >
                          <Plus className="h-4 w-4 mr-1" />
                          新增
                        </Button>
                      </div>
                      {formData.working_content.items.test_items.map((item, index) => (
                        <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="absolute right-2 top-2 h-6 w-6 text-red-500"
                            onClick={() => {
                              const newItems = [...formData.working_content.items.test_items]
                              newItems.splice(index, 1)
                              updateWorkingContent('items', 'test_items', newItems)
                            }}
                          >
                            X
                          </Button>
                          <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                              <Label>物質名稱 *</Label>
                              <Input
                                value={item.name}
                                onChange={(e) => {
                                  const newItems = [...formData.working_content.items.test_items]
                                  newItems[index].name = e.target.value
                                  updateWorkingContent('items', 'test_items', newItems)
                                }}
                              />
                            </div>
                            <div className="space-y-2">
                              <Label>劑型</Label>
                              <Input
                                value={item.form || ''}
                                onChange={(e) => {
                                  const newItems = [...formData.working_content.items.test_items]
                                  newItems[index].form = e.target.value
                                  updateWorkingContent('items', 'test_items', newItems)
                                }}
                                placeholder="如：液體、粉末"
                              />
                            </div>
                          </div>
                          <div className="space-y-2">
                            <Label>用途</Label>
                            <Input
                              value={item.purpose}
                              onChange={(e) => {
                                const newItems = [...formData.working_content.items.test_items]
                                newItems[index].purpose = e.target.value
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>保存環境</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newItems = [...formData.working_content.items.test_items]
                                newItems[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                              placeholder="請填寫保存環境"
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>本物質是否為無菌製備</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newItems = [...formData.working_content.items.test_items]
                                const isYes = value === 'yes'
                                newItems[index].is_sterile = isYes
                                // 如果選擇"是"，清空說明欄位
                                if (isYes) {
                                  newItems[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="請選擇" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">否</SelectItem>
                                <SelectItem value="yes">是</SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>請說明 *</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newItems = [...formData.working_content.items.test_items]
                                    newItems[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'test_items', newItems)
                                  }}
                                  placeholder="請說明為何本物質非無菌製備"
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* 照片上傳 */}
                          <div className="space-y-2">
                            <Label>照片</Label>
                            <FileUpload
                              value={item.photos || []}
                              onChange={(photos) => {
                                const newItems = [...formData.working_content.items.test_items]
                                newItems[index].photos = photos
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                              accept="image/*"
                              multiple={true}
                              maxSize={10}
                              maxFiles={10}
                              placeholder="拖曳照片到此處，或點擊選擇照片"
                            />
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* 對照物質列表 */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">對照物質</h3>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            const newControls = [...formData.working_content.items.control_items, {
                              name: '', is_sterile: true, purpose: '', storage_conditions: '', photos: []
                            }]
                            updateWorkingContent('items', 'control_items', newControls)
                          }}
                        >
                          <Plus className="h-4 w-4 mr-1" />
                          新增
                        </Button>
                      </div>
                      {formData.working_content.items.control_items.map((item, index) => (
                        <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="absolute right-2 top-2 h-6 w-6 text-red-500"
                            onClick={() => {
                              const newControls = [...formData.working_content.items.control_items]
                              newControls.splice(index, 1)
                              updateWorkingContent('items', 'control_items', newControls)
                            }}
                          >
                            X
                          </Button>
                          <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                              <Label>對照名稱 *</Label>
                              <Input
                                value={item.name}
                                onChange={(e) => {
                                  const newControls = [...formData.working_content.items.control_items]
                                  newControls[index].name = e.target.value
                                  updateWorkingContent('items', 'control_items', newControls)
                                }}
                                placeholder="若無對照請填寫 N/A"
                              />
                            </div>
                            <div className="space-y-2">
                              <Label>用途</Label>
                              <Input
                                value={item.purpose}
                                onChange={(e) => {
                                  const newControls = [...formData.working_content.items.control_items]
                                  newControls[index].purpose = e.target.value
                                  updateWorkingContent('items', 'control_items', newControls)
                                }}
                              />
                            </div>
                          </div>
                          <div className="space-y-2">
                            <Label>保存環境</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newControls = [...formData.working_content.items.control_items]
                                newControls[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                              placeholder="請填寫保存環境"
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>本物質是否為無菌製備</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newControls = [...formData.working_content.items.control_items]
                                const isYes = value === 'yes'
                                newControls[index].is_sterile = isYes
                                // 如果選擇"是"，清空說明欄位
                                if (isYes) {
                                  newControls[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="請選擇" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">否</SelectItem>
                                <SelectItem value="yes">是</SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>請說明 *</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newControls = [...formData.working_content.items.control_items]
                                    newControls[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'control_items', newControls)
                                  }}
                                  placeholder="請說明為何本物質非無菌製備"
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* 照片上傳 */}
                          <div className="space-y-2">
                            <Label>照片</Label>
                            <FileUpload
                              value={item.photos || []}
                              onChange={(photos) => {
                                const newControls = [...formData.working_content.items.control_items]
                                newControls[index].photos = photos
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                              accept="image/*"
                              multiple={true}
                              maxSize={10}
                              maxFiles={10}
                              placeholder="拖曳照片到此處，或點擊選擇照片"
                            />
                          </div>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
          )}

          {activeSection === 'design' && (
            <Card>
              <CardHeader>
                <CardTitle>4. 研究設計與方法<br />(Study Design and Methods)</CardTitle>
                <CardDescription>描述研究設計、實驗流程、麻醉與人道終點</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 4.1 標題 */}
                <div className="space-y-2">
                  <h3 className="text-lg font-semibold">4.1 請以實驗動物應用3Rs之精緻化原則，詳細說明實驗中所進行之動物試驗內容。使實驗動物照護及使用委員會委員了解動物試驗所有過程:</h3>
                </div>

                {/* 4.1.1 是否於麻醉下進行試驗 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.1 是否於麻醉下進行試驗</Label>
                    <Select
                      value={formData.working_content.design.anesthesia.is_under_anesthesia === null ? '' : (formData.working_content.design.anesthesia.is_under_anesthesia === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'anesthesia.is_under_anesthesia', isYes as boolean | null)
                        // 如果選擇"否"，清空相關欄位
                        if (!isYes) {
                          updateWorkingContent('design', 'anesthesia.anesthesia_type', undefined)
                          updateWorkingContent('design', 'anesthesia.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">否</SelectItem>
                        <SelectItem value="yes">是</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.anesthesia.is_under_anesthesia === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>請選擇麻醉類型 *</Label>
                        <Select
                          value={formData.working_content.design.anesthesia.anesthesia_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'anesthesia.anesthesia_type', value)
                            // 如果選擇的不是"其他"，清空其他說明
                            if (value !== 'other') {
                              updateWorkingContent('design', 'anesthesia.anesthesia_other_description', undefined)
                            }
                            // 根據選擇自動處理手術計劃書
                            if (value === 'survival_surgery') {
                              // 如果是存活手術，自動設置手術種類為"存活手術"
                              updateWorkingContent('surgery', 'surgery_type', 'survival')
                              // 如果術前準備是"略"或為空，設置預設內容
                              if (formData.working_content.surgery.preop_preparation === '略' || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', '1.實驗動物術前禁食至少12小時，不禁水。\n2.試驗豬隻經清洗擦乾後，以畜舒坦( Azeperonum 40 mg/mL)3-5 mg/kg和0.03-0.05 mg/kg阿托平(Atropine® 1 mg/mL)肌肉注射鎮靜。仔細觀察豬隻呼吸頻率。\n3.經20-30分鐘後，以4.4 mg/kg舒泰-50(Zoletil®-50)肌肉注射誘導麻醉\n4.經5-10分鐘後，將豬隻移至手術檯上，以趴姿進行氣管插管，接上氣體麻醉機，以2-3 L/min流速的氧氣混合0.5-2%氣體麻醉劑Isoflurane維持麻醉，隨時注意觀察豬隻麻醉深度。\n5.術前肌肉注射抗生素Cefazolin 15 mg/kg及止痛劑meloxicam 0.4 mg/kg\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                              }
                              // 如果手術內容說明是"略"或為空，設置預設內容
                              if (formData.working_content.surgery.surgery_description === '略' || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', '請詳述手術流程，包含手術位置、手術方法、切創大小及縫合處理')
                              }
                              // 如果術中監控為空，設置預設內容
                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', '手術進行中依試驗豬隻麻醉深度、呼吸頻率及手術需要，調整氧氣、笑氣流速及麻醉氣體濃度，同時注意保溫，接上生理監視器，監控記錄心跳、呼吸及體溫。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                              }
                              // 術後照護類型由用戶選擇，不自動設置
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value === 'non_survival_surgery') {
                              // 如果是非存活手術，自動設置手術種類為"非存活手術"
                              updateWorkingContent('surgery', 'surgery_type', 'non_survival')
                              // 如果術前準備是"略"或為空，設置預設內容
                              if (formData.working_content.surgery.preop_preparation === '略' || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', '1.實驗動物術前禁食至少12小時，不禁水。\n2.試驗豬隻經清洗擦乾後，以畜舒坦( Azeperonum 40 mg/mL)3-5 mg/kg和0.03-0.05 mg/kg阿托平(Atropine® 1 mg/mL)肌肉注射鎮靜。仔細觀察豬隻呼吸頻率。\n3.經20-30分鐘後，以4.4 mg/kg舒泰-50(Zoletil®-50)肌肉注射誘導麻醉\n4.經5-10分鐘後，將豬隻移至手術檯上，以趴姿進行氣管插管，接上氣體麻醉機，以2-3 L/min流速的氧氣混合0.5-2%氣體麻醉劑Isoflurane維持麻醉，隨時注意觀察豬隻麻醉深度。\n5.術前肌肉注射抗生素Cefazolin 15 mg/kg及止痛劑meloxicam 0.4 mg/kg\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                              }
                              // 如果手術內容說明是"略"或為空，設置預設內容
                              if (formData.working_content.surgery.surgery_description === '略' || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', '請詳述手術流程，包含手術位置、手術方法、切創大小及縫合處理')
                              }
                              // 如果術中監控為空，設置預設內容
                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', '手術進行中依試驗豬隻麻醉深度、呼吸頻率及手術需要，調整氧氣、笑氣流速及麻醉氣體濃度，同時注意保溫，接上生理監視器，監控記錄心跳、呼吸及體溫。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                              }
                              // 術後照護類型由用戶選擇，不自動設置
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value && value !== '') {
                              // 如果不是存活手術或非存活手術，自動填寫"略"
                              updateWorkingContent('surgery', 'surgery_type', '略')
                              updateWorkingContent('surgery', 'preop_preparation', '略')
                              updateWorkingContent('surgery', 'surgery_description', '略')
                              updateWorkingContent('surgery', 'monitoring', '略')
                              updateWorkingContent('surgery', 'postop_expected_impact', '略')
                              updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                              updateWorkingContent('surgery', 'postop_care', '略')
                              updateWorkingContent('surgery', 'postop_care_type', undefined)
                              updateWorkingContent('surgery', 'expected_end_point', '略')
                              updateWorkingContent('surgery', 'drugs', [])
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="請選擇" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="survival_surgery">1. 存活手術（請填寫6. 手術計劃書）</SelectItem>
                            <SelectItem value="non_survival_surgery">2. 非存活手術（請填寫6. 手術計劃書）</SelectItem>
                            <SelectItem value="gas_only">3. 非侵入式試驗，僅使用氣體麻醉(Isoflurane 1-2%)誘導後再進行實驗( Isoflurane inhalation before experiment)</SelectItem>
                            <SelectItem value="azeperonum_atropine">4. 使用畜舒坦( Azeperonum 40 mg/mL)3-5 mg/kg和0.03-0.05 mg/kg阿托平(Atropine® 1 mg/mL)肌肉注射鎮靜後，氣體麻醉(Isoflurane 1-2%)再進行實驗 (Using Azeperonum and Atropine IM with Isoflurane inhalation before experiment)</SelectItem>
                            <SelectItem value="other">5. 其他</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.anesthesia.anesthesia_type === 'other' && (
                        <div className="space-y-2">
                          <Label>請說明 *</Label>
                          <Textarea
                            value={formData.working_content.design.anesthesia.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'anesthesia.other_description', e.target.value)}
                            placeholder="請說明其他麻醉方式"
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.2 詳細敘述動物試驗內容及流程 */}
                <div className="space-y-2">
                  <Label>4.1.2 請詳細敘述動物試驗內容及流程 (Animal experiment procedures)、試驗投予物質 (experimental injections or inoculations) 、投予途徑及選擇該途徑之理由、採血 (blood withdrawals)、影像觀察 (CT, MRI, X-ray)、保定 (methods of restraint)、頻率 (frequency) *</Label>
                  <p className="text-sm text-muted-foreground mb-2">手術相關內容請於項次6. 手術計畫書中說明 (surgical procedures fill in surgical plan)</p>
                  <Textarea
                    value={formData.working_content.design.procedures}
                    onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
                    placeholder="請詳細描述動物試驗內容及流程"
                    rows={8}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.3 實驗動物等級評估 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.3 實驗動物等級評估 *</Label>
                    <Select
                      value={formData.working_content.design.pain.category}
                      onValueChange={(val) => updateWorkingContent('design', 'pain.category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="請選擇等級" /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="B">Category B 繁殖、觀察</SelectItem>
                        <SelectItem value="C">Category C 動物進行不會造成痛苦或緊迫的操作。動物進行只造成短暫或輕微痛苦及緊迫的操作。這些操作不需使用到止痛藥。</SelectItem>
                        <SelectItem value="D">Category D 動物進行可能產生疼痛或緊迫的操作，且會給予適當之止痛、麻醉或鎮定藥。</SelectItem>
                        <SelectItem value="E">Category E 動物進行可能產生疼痛或緊迫的操作，且不會給予止痛、麻醉或鎮定藥。</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.4 是否限制實驗動物飲食或飲水 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.4 是否限制實驗動物飲食或飲水</Label>
                    <Select
                      value={formData.working_content.design.restrictions.is_restricted === null ? '' : (formData.working_content.design.restrictions.is_restricted === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'restrictions.is_restricted', isYes as boolean | null)
                        // 如果選擇"否"，清空相關欄位
                        if (!isYes) {
                          updateWorkingContent('design', 'restrictions.restriction_type', undefined)
                          updateWorkingContent('design', 'restrictions.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">否</SelectItem>
                        <SelectItem value="yes">是</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.restrictions.is_restricted === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>請選擇限制類型 *</Label>
                        <Select
                          value={formData.working_content.design.restrictions.restriction_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'restrictions.restriction_type', value)
                            // 如果選擇的不是"其他"，清空其他說明
                            if (value !== 'other') {
                              updateWorkingContent('design', 'restrictions.other_description', undefined)
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="請選擇" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="fasting_before_anesthesia">麻醉前禁食</SelectItem>
                            <SelectItem value="other">其他</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.restrictions.restriction_type === 'other' && (
                        <div className="space-y-2">
                          <Label>請說明 *</Label>
                          <Textarea
                            value={formData.working_content.design.restrictions.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'restrictions.other_description', e.target.value)}
                            placeholder="請說明其他限制方式"
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.5 實驗預期結束之時機 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.1.5 實驗預期結束之時機，以及動物出現何種異常與痛苦症狀時，應提前終止試驗</Label>
                  </div>
                  <div className="space-y-2">
                    <Label>實驗終點：</Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.experimental_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.experimental_endpoint', e.target.value)}
                      placeholder="請說明實驗預期結束之時機"
                      rows={3}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>人道終點：</Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.humane_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.humane_endpoint', e.target.value)}
                      placeholder="請說明人道終點"
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.6 動物安樂死或最終處置方式 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.1.6 動物安樂死或最終處置方式</Label>
                    <Select
                      value={formData.working_content.design.final_handling.method || ''}
                      onValueChange={(value) => {
                        updateWorkingContent('design', 'final_handling.method', value)
                        // 清空其他選項的內容
                        if (value !== 'euthanasia') {
                          updateWorkingContent('design', 'final_handling.euthanasia_type', undefined)
                          updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
                        }
                        if (value !== 'transfer') {
                          updateWorkingContent('design', 'final_handling.transfer.recipient_name', '')
                          updateWorkingContent('design', 'final_handling.transfer.recipient_org', '')
                          updateWorkingContent('design', 'final_handling.transfer.project_name', '')
                        }
                        if (value !== 'other') {
                          updateWorkingContent('design', 'final_handling.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇處置方式" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="euthanasia">安樂死</SelectItem>
                        <SelectItem value="transfer">轉讓</SelectItem>
                        <SelectItem value="other">其他</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {/* 1. 安樂死 */}
                  {formData.working_content.design.final_handling.method === 'euthanasia' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">安樂死：</Label>
                      <Select
                        value={formData.working_content.design.final_handling.euthanasia_type || ''}
                        onValueChange={(value) => {
                          updateWorkingContent('design', 'final_handling.euthanasia_type', value)
                          // 如果選擇的不是"其他"，清空其他說明
                          if (value !== 'other') {
                            updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
                          }
                        }}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="請選擇" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="kcl">麻醉下(Zoletil®-50 4.4 mg/kg)，以KCl 安樂死後放血。依照「AD-04-03-00試驗豬隻安樂死規範標準作業程序書」執行</SelectItem>
                          <SelectItem value="electrocution">麻醉下(Zoletil®-50 4.4 mg/kg)，以220V電擊後放血。依照「AD-04-03-00試驗豬隻安樂死規範標準作業程序書」執行</SelectItem>
                          <SelectItem value="other">其他：</SelectItem>
                        </SelectContent>
                      </Select>
                      {formData.working_content.design.final_handling.euthanasia_type === 'other' && (
                        <div className="space-y-2 mt-2">
                          <Textarea
                            value={formData.working_content.design.final_handling.euthanasia_other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.euthanasia_other_description', e.target.value)}
                            placeholder="請說明其他安樂死方式"
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}

                  {/* 2. 轉讓 */}
                  {formData.working_content.design.final_handling.method === 'transfer' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">轉讓</Label>
                      <div className="space-y-3">
                        <div className="space-y-2">
                          <Label className="text-sm">接受者姓名：</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_name', e.target.value)}
                            placeholder="請填寫接受者姓名"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">接受者單位：</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_org}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_org', e.target.value)}
                            placeholder="請填寫接受者單位"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">計畫名稱：</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.project_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.project_name', e.target.value)}
                            placeholder="請填寫計畫名稱"
                          />
                        </div>
                      </div>
                    </div>
                  )}

                  {/* 3. 其他 */}
                  {formData.working_content.design.final_handling.method === 'other' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">其他：</Label>
                      <Textarea
                        value={formData.working_content.design.final_handling.other_description || ''}
                        onChange={(e) => updateWorkingContent('design', 'final_handling.other_description', e.target.value)}
                        placeholder="請說明其他處置方式"
                        rows={3}
                      />
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.2 動物屍體處理方法 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.2 動物屍體處理方法</Label>
                    <Textarea
                      value={formData.working_content.design.carcass_disposal.method}
                      onChange={(e) => updateWorkingContent('design', 'carcass_disposal.method', e.target.value)}
                      placeholder="請說明動物屍體處理方法"
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.3 是否使用非醫藥級化學藥品或其他物質 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.3 是否使用非醫藥級化學藥品或其他物質</Label>
                    <Select
                      value={formData.working_content.design.non_pharma_grade.used === null ? '' : (formData.working_content.design.non_pharma_grade.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'non_pharma_grade.used', isYes as boolean | null)
                        // 如果選擇"否"，清空說明欄位
                        if (!isYes) {
                          updateWorkingContent('design', 'non_pharma_grade.description', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">否</SelectItem>
                        <SelectItem value="yes">是</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.non_pharma_grade.used === true && (
                      <div className="space-y-2 mt-2">
                        <Label>請說明物質性質、安全性及使用之科學理由 *</Label>
                        <Textarea
                          value={formData.working_content.design.non_pharma_grade.description}
                          onChange={(e) => updateWorkingContent('design', 'non_pharma_grade.description', e.target.value)}
                          placeholder="請說明物質性質、安全性及使用之科學理由"
                          rows={4}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.4 是否使用危害性物質材料 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.4 是否使用危害性物質材料</Label>
                    <Select
                      value={formData.working_content.design.hazards.used === null ? '' : (formData.working_content.design.hazards.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'hazards.used', isYes as boolean | null)
                        // 如果選擇"否"，清空相關欄位
                        if (!isYes) {
                          updateWorkingContent('design', 'hazards.materials', [])
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="請選擇" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">否</SelectItem>
                        <SelectItem value="yes">是</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.hazards.used === true && (
                      <div className="space-y-4 mt-2 pl-6 border-l-2 border-slate-200">
                        <div className="space-y-3">
                          <div className="space-y-2">
                            <Label className="text-sm font-medium">請選擇危害性物質類型：</Label>
                            <Select
                              value={formData.working_content.design.hazards.selected_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('design', 'hazards.selected_type', value)
                                // 清空其他類型的材料，只保留當前類型的材料
                                const currentMaterials = formData.working_content.design.hazards.materials.filter(m => m.type === value)
                                updateWorkingContent('design', 'hazards.materials', currentMaterials)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="請選擇類型" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="biological">1. 生物性材料</SelectItem>
                                <SelectItem value="radioactive">2. 放射性</SelectItem>
                                <SelectItem value="chemical">3. 危險性化學藥品</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>

                          {/* 顯示選中類型的材料列表 */}
                          {formData.working_content.design.hazards.selected_type && (
                            <div className="space-y-3">
                              <div className="flex justify-between items-center">
                                <Label className="text-sm font-medium">
                                  {formData.working_content.design.hazards.selected_type === 'biological' && '1. 生物性材料'}
                                  {formData.working_content.design.hazards.selected_type === 'radioactive' && '2. 放射性'}
                                  {formData.working_content.design.hazards.selected_type === 'chemical' && '3. 危險性化學藥品'}
                                </Label>
                                <Button
                                  variant="outline"
                                  size="sm"
                                  onClick={() => {
                                    const materials = [...formData.working_content.design.hazards.materials]
                                    materials.push({ 
                                      type: formData.working_content.design.hazards.selected_type!, 
                                      agent_name: '', 
                                      amount: '',
                                      photos: []
                                    })
                                    updateWorkingContent('design', 'hazards.materials', materials)
                                  }}
                                >
                                  <Plus className="h-4 w-4 mr-1" />
                                  新增
                                </Button>
                              </div>
                              {formData.working_content.design.hazards.materials
                                .filter(m => m.type === formData.working_content.design.hazards.selected_type)
                                .map((material, index) => {
                                  const materialIndex = formData.working_content.design.hazards.materials.findIndex(m => m === material)
                                  return (
                                    <div key={materialIndex} className="space-y-3 relative p-3 border rounded bg-slate-50">
                                      <Button
                                        variant="ghost"
                                        size="icon"
                                        className="absolute right-2 top-2 h-6 w-6 text-red-500"
                                        onClick={() => {
                                          const materials = [...formData.working_content.design.hazards.materials]
                                          materials.splice(materialIndex, 1)
                                          updateWorkingContent('design', 'hazards.materials', materials)
                                        }}
                                      >
                                        X
                                      </Button>
                                      <div className="grid grid-cols-2 gap-3">
                                        <Input
                                          placeholder="名稱"
                                          value={material.agent_name}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].agent_name = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                        <Input
                                          placeholder="所需用量"
                                          value={material.amount}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].amount = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                      </div>
                                      {/* 照片上傳 */}
                                      <div className="space-y-2">
                                        <Label className="text-sm">照片</Label>
                                        <FileUpload
                                          value={material.photos || []}
                                          onChange={(photos) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].photos = photos
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                          accept="image/*"
                                          multiple={true}
                                          maxSize={10}
                                          maxFiles={10}
                                          placeholder="拖曳照片到此處，或點擊選擇照片"
                                        />
                                      </div>
                                    </div>
                                  )
                                })}
                            </div>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {/* 条件显示：如果 4.4 为"是"，显示 4.5 和 4.6；如果 4.4 为"否"，显示 4.5（管制药品） */}
                {formData.working_content.design.hazards.used === true && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 危害性物質及其廢棄物處理方式 */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label className="text-base font-semibold">4.5 危害性物質及其廢棄物處理方式（請附上主觀機關認可之證明文件）</Label>
                      </div>

                      {/* 4.5.1 施用方法、途徑與使用場所 */}
                      <div className="space-y-2">
                        <Label>4.5.1 施用方法、途徑與使用場所</Label>
                        <Textarea
                          value={formData.working_content.design.hazards.operation_location_method}
                          onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                          placeholder="請說明施用方法、途徑與使用場所"
                          rows={4}
                        />
                      </div>

                      {/* 4.5.2 保護措施 */}
                      <div className="space-y-2">
                        <Label>4.5.2 保護措施</Label>
                        <p className="text-sm text-muted-foreground mb-2">針對試驗人員、實驗動物以及飼養環境所採行之保護措施:</p>
                        <Textarea
                          value={formData.working_content.design.hazards.protection_measures}
                          onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                          placeholder="請說明保護措施"
                          rows={4}
                        />
                      </div>

                      {/* 4.5.3 實驗廢棄物與屍體之處理方式 */}
                      <div className="space-y-2">
                        <Label>4.5.3 實驗廢棄物與屍體之處理方式</Label>
                        <Textarea
                          value={formData.working_content.design.hazards.waste_and_carcass_disposal}
                          onChange={(e) => updateWorkingContent('design', 'hazards.waste_and_carcass_disposal', e.target.value)}
                          placeholder="請說明實驗廢棄物與屍體之處理方式"
                          rows={4}
                        />
                      </div>
                    </div>

                    <div className="h-px bg-border my-4" />

                    {/* 4.6 是否使用管制藥品 */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label>4.6 是否使用管制藥品</Label>
                        <Select
                          value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                          onValueChange={(value) => {
                            const isYes = value === 'yes'
                            updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                            // 如果選擇"否"，清空相關欄位
                            if (!isYes) {
                              updateWorkingContent('design', 'controlled_substances.items', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="請選擇" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">否</SelectItem>
                            <SelectItem value="yes">是</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">管制藥品列表</Label>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                const items = [...formData.working_content.design.controlled_substances.items, {
                                  drug_name: '',
                                  approval_no: '',
                                  amount: '',
                                  authorized_person: '',
                                  photos: []
                                }]
                                updateWorkingContent('design', 'controlled_substances.items', items)
                              }}
                            >
                              <Plus className="h-4 w-4 mr-1" />
                              新增
                            </Button>
                          </div>
                          {formData.working_content.design.controlled_substances.items.map((item, index) => (
                            <div key={index} className="space-y-3 relative p-3 border rounded bg-slate-50">
                              <Button
                                variant="ghost"
                                size="icon"
                                className="absolute right-2 top-2 h-6 w-6 text-red-500"
                                onClick={() => {
                                  const items = [...formData.working_content.design.controlled_substances.items]
                                  items.splice(index, 1)
                                  updateWorkingContent('design', 'controlled_substances.items', items)
                                }}
                              >
                                X
                              </Button>
                              <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-2">
                                  <Label className="text-sm">藥品名稱 *</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫藥品名稱"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">核准編號 *</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫核准編號"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">所需用量 *</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫所需用量"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">管制藥品管理人 *</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫管制藥品管理人"
                                  />
                                </div>
                              </div>
                              {/* 照片上傳 */}
                              <div className="space-y-2">
                                <Label className="text-sm">照片</Label>
                                <FileUpload
                                  value={item.photos || []}
                                  onChange={(photos) => {
                                    const items = [...formData.working_content.design.controlled_substances.items]
                                    items[index].photos = photos
                                    updateWorkingContent('design', 'controlled_substances.items', items)
                                  }}
                                  accept="image/*"
                                  multiple={true}
                                  maxSize={10}
                                  maxFiles={10}
                                  placeholder="拖曳照片到此處，或點擊選擇照片"
                                />
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </>
                )}

                {/* 如果 4.4 为"否"，显示 4.5（管制药品） */}
                {formData.working_content.design.hazards.used === false && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 是否使用管制藥品 */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label>4.5 是否使用管制藥品</Label>
                        <Select
                          value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                          onValueChange={(value) => {
                            const isYes = value === 'yes'
                            updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                            // 如果選擇"否"，清空相關欄位
                            if (!isYes) {
                              updateWorkingContent('design', 'controlled_substances.items', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="請選擇" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">否</SelectItem>
                            <SelectItem value="yes">是</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">管制藥品列表</Label>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => {
                                const items = [...formData.working_content.design.controlled_substances.items, {
                                  drug_name: '',
                                  approval_no: '',
                                  amount: '',
                                  authorized_person: '',
                                  photos: []
                                }]
                                updateWorkingContent('design', 'controlled_substances.items', items)
                              }}
                            >
                              <Plus className="h-4 w-4 mr-1" />
                              新增
                            </Button>
                          </div>
                          {formData.working_content.design.controlled_substances.items.map((item, index) => (
                            <div key={index} className="space-y-3 relative p-3 border rounded bg-slate-50">
                              <Button
                                variant="ghost"
                                size="icon"
                                className="absolute right-2 top-2 h-6 w-6 text-red-500"
                                onClick={() => {
                                  const items = [...formData.working_content.design.controlled_substances.items]
                                  items.splice(index, 1)
                                  updateWorkingContent('design', 'controlled_substances.items', items)
                                }}
                              >
                                X
                              </Button>
                              <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-2">
                                  <Label className="text-sm">藥品名稱 *</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫藥品名稱"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">核准編號 *</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫核准編號"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">所需用量 *</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫所需用量"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">管制藥品管理人 *</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="請填寫管制藥品管理人"
                                  />
                                </div>
                              </div>
                              {/* 照片上傳 */}
                              <div className="space-y-2">
                                <Label className="text-sm">照片</Label>
                                <FileUpload
                                  value={item.photos || []}
                                  onChange={(photos) => {
                                    const items = [...formData.working_content.design.controlled_substances.items]
                                    items[index].photos = photos
                                    updateWorkingContent('design', 'controlled_substances.items', items)
                                  }}
                                  accept="image/*"
                                  multiple={true}
                                  maxSize={10}
                                  maxFiles={10}
                                  placeholder="拖曳照片到此處，或點擊選擇照片"
                                />
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
          )}

          {activeSection === 'guidelines' && (
            <Card>
              <CardHeader>
                <CardTitle>5. 相關規範及參考文獻<br />(Guidelines and References)</CardTitle>
                <CardDescription>填寫本計畫參考之法規、指引或文獻</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>相關規範說明</Label>
                  <Textarea
                    value={formData.working_content.guidelines.content}
                    onChange={(e) => updateWorkingContent('guidelines', 'content', e.target.value)}
                    placeholder="例如：本計畫遵循動物保護法及實驗動物照護及使用指引"
                    rows={5}
                  />
                </div>
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">參考文獻列表</h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const newRefs = [...formData.working_content.guidelines.references, { citation: '', url: '' }]
                        updateWorkingContent('guidelines', 'references', newRefs)
                      }}
                    >
                      新增文獻
                    </Button>
                  </div>
                  {formData.working_content.guidelines.references.map((ref, index) => (
                    <div key={index} className="grid w-full gap-2 relative">
                      <div className="flex gap-2 items-start">
                        <div className="grid gap-2 flex-1">
                          <Input
                            placeholder="文獻引用 (Citation)"
                            value={ref.citation}
                            onChange={(e) => {
                              const newRefs = [...formData.working_content.guidelines.references]
                              newRefs[index].citation = e.target.value
                              updateWorkingContent('guidelines', 'references', newRefs)
                            }}
                          />
                          <Input
                            placeholder="URL (Optional)"
                            value={ref.url || ''}
                            onChange={(e) => {
                              const newRefs = [...formData.working_content.guidelines.references]
                              newRefs[index].url = e.target.value
                              updateWorkingContent('guidelines', 'references', newRefs)
                            }}
                          />
                        </div>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-red-500 mt-1"
                          onClick={() => {
                            const newRefs = [...formData.working_content.guidelines.references]
                            newRefs.splice(index, 1)
                            updateWorkingContent('guidelines', 'references', newRefs)
                          }}
                        >
                          X
                        </Button>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'surgery' && (
            <Card>
              <CardHeader>
                <CardTitle>6. 手術計畫書<br />(Animal Surgical Plan)</CardTitle>
                <CardDescription>填寫手術種類、術前準備、無菌措施與術後照護</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {(() => {
                  const needsSurgeryPlan = formData.working_content.design.anesthesia.is_under_anesthesia === true &&
                    (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
                     formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery')
                  
                  if (!needsSurgeryPlan) {
                    // 如果不需要填寫手術計劃書，顯示"略"
                    return (
                      <div className="space-y-4">
                        <div className="space-y-2">
                          <Label>6.1 手術種類</Label>
                          <Input value="略" disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.2 術前準備</Label>
                          <Textarea value="略" disabled rows={3} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.3 無菌措施 (Aseptic Techniques)</Label>
                          <Input value="略" disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.4 手術內容說明</Label>
                          <Textarea value="略" disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.5 術中監控</Label>
                          <Textarea value="略" disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.6 存活手術，請說明預期術後可能對實驗動物造成之影響:</Label>
                          <Textarea value="略" disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.7 動物是否會接受一次以上的手術，若有，則寫出數量及原因:</Label>
                          <Input value="略" disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.8 請說明動物術後照護及止痛給藥方法:</Label>
                          <div className="space-y-4">
                            <div className="space-y-2">
                              <Label>手術類型</Label>
                              <Input value="略" disabled />
                            </div>
                            <div className="space-y-2">
                              <Label>詳細內容</Label>
                              <Textarea value="略" disabled rows={5} />
                            </div>
                          </div>
                        </div>
                        <div className="space-y-2">
                          <Label>6.9 請說明實驗預期結束之時機(Please specify expected experimental end point):</Label>
                          <Textarea value="略" disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.10 手術用藥資訊，投與麻醉前導、鎮靜、止痛或其他藥物資訊:</Label>
                          <Input value="略" disabled />
                        </div>
                      </div>
                    )
                  }
                  
                  // 如果需要填寫手術計劃書，顯示正常表單
                  return (
                    <>
                      <div className="space-y-2">
                        <Label>6.1 手術種類 *</Label>
                        <Input
                          value={formData.working_content.surgery.surgery_type === 'survival' ? '存活手術' : 
                                 formData.working_content.surgery.surgery_type === 'non_survival' ? '非存活手術' : 
                                 formData.working_content.surgery.surgery_type || ''}
                          disabled
                          className="bg-slate-50"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.2 術前準備 *</Label>
                        <Textarea
                          value={formData.working_content.surgery.preop_preparation}
                          onChange={(e) => updateWorkingContent('surgery', 'preop_preparation', e.target.value)}
                          placeholder="說明禁食禁水時間、備皮、消毒等"
                          rows={8}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.3 無菌措施 (Aseptic Techniques) *</Label>
                        <div className="space-y-2">
                          {[
                            { value: 'surgical_site_disinfection', label: '動物術部消毒(Surgical site disinfection)' },
                            { value: 'instrument_disinfection', label: '器械消毒(Instrument disinfection)' },
                            { value: 'sterilized_gowns_gloves', label: '無菌手術衣及手套(Sterilizd surgical gowns and gloves)' },
                            { value: 'sterilized_drapes', label: '無菌手術覆布Sterilized surgical drapes' },
                            { value: 'surgical_hand_disinfection', label: '術者刷手Surgical hand disinfection' }
                          ].map(item => (
                            <div key={item.value} className="flex items-center space-x-2">
                              <Checkbox
                                id={`aseptic_${item.value}`}
                                checked={formData.working_content.surgery.aseptic_techniques.includes(item.value)}
                                onChange={(e) => {
                                  const checked = e.target.checked
                                  const current = formData.working_content.surgery.aseptic_techniques
                                  const updated = checked
                                    ? [...current, item.value]
                                    : current.filter(i => i !== item.value)
                                  updateWorkingContent('surgery', 'aseptic_techniques', updated)
                                }}
                              />
                              <Label htmlFor={`aseptic_${item.value}`} className="font-normal cursor-pointer">{item.label}</Label>
                            </div>
                          ))}
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>6.4 手術內容說明 *</Label>
                        <Textarea
                          value={formData.working_content.surgery.surgery_description}
                          onChange={(e) => updateWorkingContent('surgery', 'surgery_description', e.target.value)}
                          placeholder="請詳述手術流程，包含手術位置、手術方法、切創大小及縫合處理"
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.5 術中監控 *</Label>
                        <Textarea
                          value={formData.working_content.surgery.monitoring}
                          onChange={(e) => updateWorkingContent('surgery', 'monitoring', e.target.value)}
                          placeholder="手術進行中依試驗豬隻麻醉深度、呼吸頻率及手術需要，調整氧氣、笑氣流速及麻醉氣體濃度，同時注意保溫，接上生理監視器，監控記錄心跳、呼吸及體溫。依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行"
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.6 存活手術，請說明預期術後可能對實驗動物造成之影響:</Label>
                        <Textarea
                          value={formData.working_content.surgery.postop_expected_impact}
                          onChange={(e) => updateWorkingContent('surgery', 'postop_expected_impact', e.target.value)}
                          placeholder="請說明預期術後可能對實驗動物造成之影響"
                          rows={4}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.7 動物是否會接受一次以上的手術，若有，則寫出數量及原因:</Label>
                        <div className="space-y-4">
                          <div className="flex items-center space-x-2">
                            <Checkbox
                              id="multiple_surgeries"
                              checked={formData.working_content.surgery.multiple_surgeries.used}
                              onChange={(e) => {
                                const checked = e.target.checked
                                updateWorkingContent('surgery', 'multiple_surgeries.used', checked)
                                if (!checked) {
                                  updateWorkingContent('surgery', 'multiple_surgeries.number', 0)
                                  updateWorkingContent('surgery', 'multiple_surgeries.reason', '')
                                }
                              }}
                            />
                            <Label htmlFor="multiple_surgeries" className="font-normal cursor-pointer">是</Label>
                          </div>
                          {formData.working_content.surgery.multiple_surgeries.used && (
                            <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                              <div className="space-y-2">
                                <Label>數量 *</Label>
                                <Input
                                  type="number"
                                  value={formData.working_content.surgery.multiple_surgeries.number || ''}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.number', parseInt(e.target.value) || 0)}
                                  placeholder="請輸入手術次數"
                                />
                              </div>
                              <div className="space-y-2">
                                <Label>原因 *</Label>
                                <Textarea
                                  value={formData.working_content.surgery.multiple_surgeries.reason}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.reason', e.target.value)}
                                  placeholder="請說明原因"
                                  rows={3}
                                />
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>6.8 請說明動物術後照護及止痛給藥方法: *</Label>
                        <div className="space-y-4">
                          <div className="space-y-2">
                            <Label>手術類型 *</Label>
                            <Select
                              value={formData.working_content.surgery.postop_care_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('surgery', 'postop_care_type', value as 'orthopedic' | 'non_orthopedic')
                                // 根據選擇自動設置對應的預設內容
                                if (value === 'orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', '1.術後每日評估豬隻健康狀態，依術後狀況進行傷口護理。\n\n2.術後7日內每日進行疼痛評估依豬隻狀況並依照不同手術給予止痛藥及抗生素\n骨科手術\n止痛藥\nketoprofen 1-3 mg/kg IM SID (3天)\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14天或長期給予)\n抗生素\ncefazolin 15 mg/kg IM BID (1-7天)\ncephalexin 30 mg/kg PO BID (8-14天或長期給予)\n\n3.若動物發生異常情形，則依獸醫師指示處理。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                                } else if (value === 'non_orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', '1.術後每日評估豬隻健康狀態，依術後狀況進行傷口護理。\n\n2.術後7日內每日進行疼痛評估依豬隻狀況並依照不同手術給予止痛藥及抗生素\n非骨科手術\n止痛藥\nmeloxicam 0.1-0.4 mg/kg IM SID (3天)\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14天或長期給予)\n抗生素\npencillin 10000 u/kg IM SID (1-7天)\namoxicillin 20 mg/kg PO BID (8-14天或長期給予)\n\n3.若動物發生異常情形，則依獸醫師指示處理。\n依「TU-03-09-00試驗豬隻外科手術標準作業程序書」進行')
                                }
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder="請選擇手術類型" /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="orthopedic">骨科手術</SelectItem>
                                <SelectItem value="non_orthopedic">非骨科手術</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                          <div className="space-y-2">
                            <Label>詳細內容 *</Label>
                            <Textarea
                              value={formData.working_content.surgery.postop_care}
                              onChange={(e) => updateWorkingContent('surgery', 'postop_care', e.target.value)}
                              placeholder="請說明動物術後照護及止痛給藥方法"
                              rows={15}
                            />
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>6.9 請說明實驗預期結束之時機(Please specify expected experimental end point): *</Label>
                        <Textarea
                          value={formData.working_content.surgery.expected_end_point}
                          onChange={(e) => updateWorkingContent('surgery', 'expected_end_point', e.target.value)}
                          placeholder="請說明實驗預期結束之時機"
                          rows={4}
                        />
                      </div>
                      <div className="space-y-4">
                        <div className="flex justify-between items-center">
                          <Label>6.10 手術用藥資訊，投與麻醉前導、鎮靜、止痛或其他藥物資訊 *</Label>
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={() => {
                              const currentDrugs = formData.working_content.surgery.drugs || []
                              const newDrugs = [...currentDrugs, {
                                drug_name: '',
                                dose: '',
                                route: '',
                                frequency: '',
                                purpose: ''
                              }]
                              updateWorkingContent('surgery', 'drugs', newDrugs)
                            }}
                          >
                            + 新增
                          </Button>
                        </div>
                        <div className="border rounded-md overflow-hidden">
                          <div className="overflow-x-auto">
                            <table className="w-full border-collapse">
                              <thead>
                                <tr className="bg-slate-100">
                                  <th className="border p-2 text-left text-sm font-semibold">藥品名稱</th>
                                  <th className="border p-2 text-left text-sm font-semibold">劑量</th>
                                  <th className="border p-2 text-left text-sm font-semibold">投與途徑</th>
                                  <th className="border p-2 text-left text-sm font-semibold">頻率</th>
                                  <th className="border p-2 text-left text-sm font-semibold">給藥目的</th>
                                  <th className="border p-2 text-center text-sm font-semibold w-16">操作</th>
                                </tr>
                              </thead>
                              <tbody>
                                {(formData.working_content.surgery.drugs || []).map((drug: any, index: number) => (
                                  <tr key={index} className="hover:bg-slate-50">
                                    <td className="border p-2">
                                      <Input
                                        value={drug.drug_name || ''}
                                        onChange={(e) => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs[index].drug_name = e.target.value
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                        placeholder="藥品名稱"
                                        className="border-0 focus-visible:ring-0"
                                      />
                                    </td>
                                    <td className="border p-2">
                                      <Input
                                        value={drug.dose || ''}
                                        onChange={(e) => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs[index].dose = e.target.value
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                        placeholder="劑量"
                                        className="border-0 focus-visible:ring-0"
                                      />
                                    </td>
                                    <td className="border p-2">
                                      <Input
                                        value={drug.route || ''}
                                        onChange={(e) => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs[index].route = e.target.value
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                        placeholder="投與途徑"
                                        className="border-0 focus-visible:ring-0"
                                      />
                                    </td>
                                    <td className="border p-2">
                                      <Input
                                        value={drug.frequency || ''}
                                        onChange={(e) => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs[index].frequency = e.target.value
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                        placeholder="頻率"
                                        className="border-0 focus-visible:ring-0"
                                      />
                                    </td>
                                    <td className="border p-2">
                                      <Input
                                        value={drug.purpose || ''}
                                        onChange={(e) => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs[index].purpose = e.target.value
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                        placeholder="給藥目的"
                                        className="border-0 focus-visible:ring-0"
                                      />
                                    </td>
                                    <td className="border p-2 text-center">
                                      <Button
                                        type="button"
                                        variant="ghost"
                                        size="icon"
                                        className="h-8 w-8 text-red-500"
                                        onClick={() => {
                                          const newDrugs = [...formData.working_content.surgery.drugs]
                                          newDrugs.splice(index, 1)
                                          updateWorkingContent('surgery', 'drugs', newDrugs)
                                        }}
                                      >
                                        X
                                      </Button>
                                    </td>
                                  </tr>
                                ))}
                                {(!formData.working_content.surgery.drugs || formData.working_content.surgery.drugs.length === 0) && (
                                  <tr>
                                    <td colSpan={6} className="border p-4 text-center text-muted-foreground">
                                      暫無用藥資訊，請點擊「+ 新增」添加
                                    </td>
                                  </tr>
                                )}
                              </tbody>
                            </table>
                          </div>
                        </div>
                      </div>
                    </>
                  )
                })()}
              </CardContent>
            </Card>
          )}

          {activeSection === 'animals' && (
            <Card>
              <CardHeader>
                <CardTitle>7. 實驗動物資料<br />(Animal Information)</CardTitle>
                <CardDescription>填寫實驗動物物種、來源與飼養資訊</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">動物清單（依序添加公母或性別不限）</h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const currentList = formData.working_content.animals.animals || []
                        const newAnimals = [...currentList, {
                          species: '' as '' | 'pig' | 'other',
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
                          housing_location: '豬博士畜牧場'
                        }]
                        updateWorkingContent('animals', 'animals', newAnimals) // Special case for direct array update if passing null key or just replace 'animals'
                      }}
                    >
                      ＋新增動物
                    </Button>
                  </div>
                  {/* Helper to update entire animals array */}
                  {(formData.working_content.animals.animals || []).map((animal: any, index: number) => (
                    <div key={index} className="grid gap-4 p-4 border rounded relative bg-slate-50 mb-4">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="absolute right-2 top-2 h-6 w-6 text-red-500"
                        onClick={() => {
                          const newAnimals = [...formData.working_content.animals.animals]
                          newAnimals.splice(index, 1)
                          updateWorkingContent('animals', 'animals', newAnimals)
                        }}
                      >
                        X
                      </Button>
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>物種 *</Label>
                          <Select
                            value={animal.species || ''}
                            onValueChange={(value) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              newAnimals[index].species = value as 'pig' | 'other'
                              if (value !== 'other') {
                                newAnimals[index].species_other = ''
                              }
                              if (value !== 'pig') {
                                newAnimals[index].strain = undefined
                                newAnimals[index].strain_other = ''
                              }
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                          >
                            <SelectTrigger><SelectValue placeholder="請選擇物種" /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="pig">豬</SelectItem>
                              <SelectItem value="other">其他</SelectItem>
                            </SelectContent>
                          </Select>
                          {animal.species === 'other' && (
                            <Input
                              value={animal.species_other || ''}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].species_other = e.target.value
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                              placeholder="請填寫物種"
                            />
                          )}
                        </div>
                        <div className="space-y-2">
                          <Label>品系</Label>
                          {animal.species === 'pig' ? (
                            <Select
                              value={animal.strain || ''}
                              onValueChange={(value) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].strain = value as 'white_pig' | 'mini_pig'
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder="請選擇品系" /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="white_pig">白豬</SelectItem>
                                <SelectItem value="mini_pig">迷你豬</SelectItem>
                              </SelectContent>
                            </Select>
                          ) : animal.species === 'other' ? (
                            <Input
                              value={animal.strain_other || ''}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].strain_other = e.target.value
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                              placeholder="請填寫品系"
                            />
                          ) : (
                            <Input disabled placeholder="請先選擇物種" />
                          )}
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>性別 *</Label>
                          <Select
                            value={animal.sex || ''}
                            onValueChange={(value) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              newAnimals[index].sex = value
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                          >
                            <SelectTrigger><SelectValue placeholder="請選擇性別" /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="male">公</SelectItem>
                              <SelectItem value="female">母</SelectItem>
                              <SelectItem value="unlimited">不限</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="space-y-2">
                          <Label>數量 *</Label>
                          <Input
                            type="number"
                            min="0"
                            step="1"
                            value={animal.number || ''}
                            onChange={(e) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              const value = parseInt(e.target.value) || 0
                              newAnimals[index].number = value >= 0 ? value : 0
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                            placeholder="請輸入數量"
                          />
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>年齡：</Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.age_unlimited && (
                              <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                  <Label>最小年齡 (月齡)</Label>
                                  <Input
                                    type="number"
                                    min="3"
                                    step="1"
                                    value={animal.age_min || ''}
                                    onChange={(e) => {
                                      const newAnimals = [...formData.working_content.animals.animals]
                                      const value = parseInt(e.target.value)
                                      if (value >= 3) {
                                        newAnimals[index].age_min = value
                                        // 如果最大月齡小於等於新的最小月齡，自動調整最大月齡
                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= value) {
                                          newAnimals[index].age_max = value + 1
                                        }
                                      } else if (value < 3 && value > 0) {
                                        newAnimals[index].age_min = 3
                                        // 如果最大月齡小於等於3，自動調整最大月齡
                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= 3) {
                                          newAnimals[index].age_max = 4
                                        }
                                      } else {
                                        newAnimals[index].age_min = undefined
                                      }
                                      updateWorkingContent('animals', 'animals', newAnimals)
                                    }}
                                    placeholder="最小月齡（至少3）"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label>最大年齡 (月齡)</Label>
                                  <Input
                                    type="number"
                                    min={animal.age_min !== undefined ? animal.age_min + 1 : 4}
                                    step="1"
                                    value={animal.age_max || ''}
                                    onChange={(e) => {
                                      const newAnimals = [...formData.working_content.animals.animals]
                                      const value = parseInt(e.target.value)
                                      const minAge = newAnimals[index].age_min || 3
                                      if (value > minAge) {
                                        newAnimals[index].age_max = value
                                      } else if (value <= minAge && value > 0) {
                                        newAnimals[index].age_max = minAge + 1
                                      } else {
                                        newAnimals[index].age_max = undefined
                                      }
                                      updateWorkingContent('animals', 'animals', newAnimals)
                                    }}
                                    placeholder="最大月齡"
                                  />
                                </div>
                              </div>
                            )}
                          </div>
                          <div className="flex items-center gap-2">
                            <Checkbox
                              id={`age_unlimited_${index}`}
                              checked={animal.age_unlimited || false}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].age_unlimited = e.target.checked
                                if (e.target.checked) {
                                  newAnimals[index].age_min = undefined
                                  newAnimals[index].age_max = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            />
                            <Label htmlFor={`age_unlimited_${index}`} className="font-normal cursor-pointer">不限</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>體重範圍：</Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.weight_unlimited && (
                              <div className="space-y-4">
                                <div className="grid grid-cols-2 gap-4">
                                  <div className="space-y-2">
                                    <Label>最小體重 (kg)</Label>
                                    <Input
                                      type="number"
                                      min="20"
                                      step="5"
                                      value={animal.weight_min || ''}
                                      onChange={(e) => {
                                        const newAnimals = [...formData.working_content.animals.animals]
                                        const value = parseFloat(e.target.value)
                                        if (value >= 20) {
                                          const roundedValue = Math.round(value / 5) * 5
                                          newAnimals[index].weight_min = roundedValue >= 20 ? roundedValue : 20
                                          // 如果最大體重小於等於新的最小體重，自動調整最大體重
                                          if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= roundedValue) {
                                            newAnimals[index].weight_max = roundedValue + 5
                                          }
                                        } else if (value < 20 && value > 0) {
                                          newAnimals[index].weight_min = 20
                                          // 如果最大體重小於等於20，自動調整最大體重
                                          if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= 20) {
                                            newAnimals[index].weight_max = 25
                                          }
                                        } else {
                                          newAnimals[index].weight_min = undefined
                                        }
                                        updateWorkingContent('animals', 'animals', newAnimals)
                                      }}
                                      placeholder="最小體重（至少20kg）"
                                    />
                                    <p className="text-xs text-muted-foreground">每五公斤一個區間</p>
                                  </div>
                                  <div className="space-y-2">
                                    <Label>最大體重 (kg)</Label>
                                    <Input
                                      type="number"
                                      min={animal.weight_min !== undefined ? animal.weight_min + 5 : 25}
                                      step="5"
                                      value={animal.weight_max || ''}
                                      onChange={(e) => {
                                        const newAnimals = [...formData.working_content.animals.animals]
                                        const value = parseFloat(e.target.value)
                                        const minWeight = newAnimals[index].weight_min || 20
                                        if (value > minWeight) {
                                          newAnimals[index].weight_max = Math.round(value / 5) * 5
                                          // 確保最大體重大於最小體重
                                          if (newAnimals[index].weight_max <= minWeight) {
                                            newAnimals[index].weight_max = minWeight + 5
                                          }
                                        } else if (value <= minWeight && value > 0) {
                                          newAnimals[index].weight_max = minWeight + 5
                                        } else {
                                          newAnimals[index].weight_max = undefined
                                        }
                                        updateWorkingContent('animals', 'animals', newAnimals)
                                      }}
                                      placeholder="最大體重（必須大於最小體重）"
                                    />
                                  </div>
                                </div>
                                {animal.weight_min !== undefined && animal.weight_max !== undefined && (
                                  <div className="space-y-4">
                                    <div className="space-y-2">
                                      <Label>調整區間 - 最小體重</Label>
                                      <Slider
                                        min={20}
                                        max={Math.max(200, (animal.weight_max || 0) + 50)}
                                        step={5}
                                        value={animal.weight_min ?? 20}
                                        onChange={(value: number) => {
                                          const newAnimals = [...formData.working_content.animals.animals]
                                          const minValue = Math.max(20, Math.round(value / 5) * 5)
                                          newAnimals[index].weight_min = minValue
                                          // 確保最小值不大於最大值，且最大體重必須大於最小體重
                                          if (minValue >= (animal.weight_max || 0)) {
                                            newAnimals[index].weight_max = minValue + 5
                                          }
                                          updateWorkingContent('animals', 'animals', newAnimals)
                                        }}
                                        className="w-full"
                                      />
                                      <p className="text-xs text-muted-foreground text-center">{animal.weight_min || 20} kg</p>
                                    </div>
                                    <div className="space-y-2">
                                      <Label>調整區間 - 最大體重</Label>
                                      <Slider
                                        min={Math.max(25, (animal.weight_min || 20) + 5)}
                                        max={200}
                                        step={5}
                                        value={animal.weight_max ?? 25}
                                        onChange={(value: number) => {
                                          const newAnimals = [...formData.working_content.animals.animals]
                                          const maxValue = Math.round(value / 5) * 5
                                          const minWeight = newAnimals[index].weight_min || 20
                                          // 確保最大值大於最小值
                                          if (maxValue > minWeight) {
                                            newAnimals[index].weight_max = maxValue
                                          } else {
                                            newAnimals[index].weight_max = minWeight + 5
                                          }
                                          updateWorkingContent('animals', 'animals', newAnimals)
                                        }}
                                        className="w-full"
                                      />
                                      <p className="text-xs text-muted-foreground text-center">{animal.weight_max || 25} kg</p>
                                    </div>
                                  </div>
                                )}
                              </div>
                            )}
                          </div>
                          <div className="flex items-center gap-2">
                            <Checkbox
                              id={`weight_unlimited_${index}`}
                              checked={animal.weight_unlimited || false}
                              onChange={(e) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].weight_unlimited = e.target.checked
                                if (e.target.checked) {
                                  newAnimals[index].weight_min = undefined
                                  newAnimals[index].weight_max = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            />
                            <Label htmlFor={`weight_unlimited_${index}`} className="font-normal cursor-pointer">不限</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>飼養環境：豬博士畜牧場</Label>
                      </div>
                    </div>
                  ))}
                  <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md">
                    <h4 className="font-semibold text-sm mb-2">備註：</h4>
                    <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
                      <li>※豬/白豬，來源：國內繁殖場(豬博士畜牧場)；豬/迷你豬，來源：國內繁殖場(豬博士畜牧場)</li>
                      <li>※性別：公、母或不限。</li>
                      <li>※年齡：大約月齡範圍(以上)或不限。</li>
                      <li>※體重：大約體重範圍或不限。</li>
                      <li>※白豬成長快速，7個月齡可達100公斤，觀察期超過3個月建議使用迷你豬進行試驗。</li>
                    </ul>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'personnel' && (
            <Card>
              <CardHeader>
                <CardTitle>8. 試驗人員資料<br />(Personnel Working on Animal Study)</CardTitle>
                <CardDescription>參與本計畫之試驗人員清單與資格</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">8.1 負責進行動物試驗之相關人員資料</h3>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const currentPersonnel = formData.working_content.personnel || []
                        const maxId = currentPersonnel.length > 0 
                          ? Math.max(...currentPersonnel.map((p: any) => p.id || 0))
                          : 0
                        const newPersonnel = [...currentPersonnel, {
                          id: maxId + 1,
                          name: '',
                          position: '',
                          roles: [],
                          roles_other_text: '',
                          years_experience: 0,
                          trainings: [],
                          training_certificates: []
                        }]
                        setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                      }}
                    >
                      + 新增
                    </Button>
                  </div>
                  <div className="border rounded-md overflow-hidden">
                    <div className="overflow-x-auto">
                      <table className="w-full border-collapse">
                        <thead>
                          <tr className="bg-slate-100">
                            <th className="border p-2 text-left text-sm font-semibold">編號</th>
                            <th className="border p-2 text-left text-sm font-semibold">姓名</th>
                            <th className="border p-2 text-left text-sm font-semibold">職稱</th>
                            <th className="border p-2 text-left text-sm font-semibold">工作內容</th>
                            <th className="border p-2 text-left text-sm font-semibold">參與動物試驗年數</th>
                            <th className="border p-2 text-left text-sm font-semibold">訓練/資格/取得時間</th>
                            <th className="border p-2 text-center text-sm font-semibold w-16">操作</th>
                          </tr>
                        </thead>
                        <tbody>
                          {(formData.working_content.personnel || []).map((person: any, index: number) => (
                            <tr key={index} className="hover:bg-slate-50">
                              <td className="border p-2">
                                <Input
                                  type="number"
                                  min="1"
                                  value={person.id || index + 1}
                                  onChange={(e) => {
                                    const newPersonnel = [...formData.working_content.personnel]
                                    newPersonnel[index].id = parseInt(e.target.value) || index + 1
                                    setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                  }}
                                  className="border-0 focus-visible:ring-0 w-16"
                                />
                              </td>
                              <td className="border p-2">
                                <Input
                                  value={person.name || ''}
                                  onChange={(e) => {
                                    const newPersonnel = [...formData.working_content.personnel]
                                    newPersonnel[index].name = e.target.value
                                    setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                  }}
                                  placeholder="姓名"
                                  className="border-0 focus-visible:ring-0"
                                />
                              </td>
                              <td className="border p-2">
                                <Input
                                  value={person.position || ''}
                                  onChange={(e) => {
                                    const newPersonnel = [...formData.working_content.personnel]
                                    newPersonnel[index].position = e.target.value
                                    setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                  }}
                                  placeholder="職稱"
                                  className="border-0 focus-visible:ring-0"
                                />
                              </td>
                              <td className="border p-2">
                                <div className="space-y-2 min-w-[300px]">
                                  <div className="flex flex-wrap gap-2">
                                    {[
                                      { value: 'a', label: 'a.計畫督導(Supervision)' },
                                      { value: 'b', label: 'b.飼養照顧(Animal care)' },
                                      { value: 'c', label: 'c.保定(Restraint)' },
                                      { value: 'd', label: 'd.麻醉止痛(Anesthesia and analgesia)' },
                                      { value: 'e', label: 'e.手術(Surgery)' },
                                      { value: 'f', label: 'f.手術支援(Surgery assistance)' },
                                      { value: 'g', label: 'g.觀察監測(Monitoring)' },
                                      { value: 'h', label: 'h.安樂死(Euthanasia)' },
                                      { value: 'i', label: 'i.其他(Other)' }
                                    ].map(role => (
                                      <div key={role.value} className="flex items-center space-x-1">
                                        <Checkbox
                                          id={`role_${index}_${role.value}`}
                                          checked={(person.roles || []).includes(role.value)}
                                          onChange={(e) => {
                                            const newPersonnel = [...formData.working_content.personnel]
                                            const currentRoles = newPersonnel[index].roles || []
                                            if (e.target.checked) {
                                              newPersonnel[index].roles = [...currentRoles, role.value]
                                            } else {
                                              newPersonnel[index].roles = currentRoles.filter((r: string) => r !== role.value)
                                              if (role.value === 'i') {
                                                newPersonnel[index].roles_other_text = ''
                                              }
                                            }
                                            setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                          }}
                                        />
                                        <Label htmlFor={`role_${index}_${role.value}`} className="text-xs font-normal cursor-pointer">{role.label}</Label>
                                      </div>
                                    ))}
                                  </div>
                                  {(person.roles || []).includes('i') && (
                                    <Input
                                      value={person.roles_other_text || ''}
                                      onChange={(e) => {
                                        const newPersonnel = [...formData.working_content.personnel]
                                        newPersonnel[index].roles_other_text = e.target.value
                                        setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                      }}
                                      placeholder="請說明其他工作內容 *"
                                      className="mt-2"
                                    />
                                  )}
                                </div>
                              </td>
                              <td className="border p-2">
                                <Input
                                  type="number"
                                  min="0"
                                  step="1"
                                  value={person.years_experience || ''}
                                  onChange={(e) => {
                                    const newPersonnel = [...formData.working_content.personnel]
                                    newPersonnel[index].years_experience = parseInt(e.target.value) || 0
                                    setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                  }}
                                  placeholder="年數"
                                  className="border-0 focus-visible:ring-0 w-20"
                                />
                              </td>
                              <td className="border p-2">
                                <div className="space-y-2 min-w-[400px]">
                                  <div className="flex flex-wrap gap-2 mb-2">
                                    {[
                                      { value: 'A', label: 'A. 實驗動物照護及使用委員會或小組成員訓練班' },
                                      { value: 'B', label: 'B. IACUC教育訓練研討會' },
                                      { value: 'C', label: 'C. 輻射安全訓練班' },
                                      { value: 'D', label: 'D. 生醫產業用畜禽應用研習會及技術研習會' },
                                      { value: 'E', label: 'E. 實驗動物法規及照護管理班' }
                                    ].map(training => (
                                      <div key={training.value} className="flex items-center space-x-1">
                                        <Checkbox
                                          id={`training_${index}_${training.value}`}
                                          checked={(person.trainings || []).includes(training.value)}
                                          onChange={(e) => {
                                            const newPersonnel = [...formData.working_content.personnel]
                                            const currentTrainings = newPersonnel[index].trainings || []
                                            if (e.target.checked) {
                                              newPersonnel[index].trainings = [...currentTrainings, training.value]
                                            } else {
                                              newPersonnel[index].trainings = currentTrainings.filter((t: string) => t !== training.value)
                                              // 移除該訓練的所有證書
                                              newPersonnel[index].training_certificates = (newPersonnel[index].training_certificates || []).filter(
                                                (cert: any) => cert.training_code !== training.value
                                              )
                                            }
                                            setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                          }}
                                        />
                                        <Label htmlFor={`training_${index}_${training.value}`} className="text-xs font-normal cursor-pointer">{training.label}</Label>
                                      </div>
                                    ))}
                                  </div>
                                  {/* 顯示每個已選訓練的證書編號列表 */}
                                  {(person.trainings || []).map((trainingCode: string) => {
                                    const certificates = (person.training_certificates || []).filter((cert: any) => cert.training_code === trainingCode)
                                    return (
                                      <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                                        <Label className="text-xs font-semibold">{trainingCode}:</Label>
                                        {certificates.map((cert: any, certIndex: number) => {
                                          // 找到該證書在完整 training_certificates 數組中的索引
                                          const allCerts = person.training_certificates || []
                                          let globalCertIndex = -1
                                          let count = 0
                                          for (let i = 0; i < allCerts.length; i++) {
                                            if (allCerts[i].training_code === trainingCode) {
                                              if (count === certIndex) {
                                                globalCertIndex = i
                                                break
                                              }
                                              count++
                                            }
                                          }
                                          
                                          return (
                                            <div key={certIndex} className="flex items-center gap-2">
                                              <Input
                                                value={cert.certificate_no || ''}
                                                onChange={(e) => {
                                                  const newPersonnel = [...formData.working_content.personnel]
                                                  const certs = [...(newPersonnel[index].training_certificates || [])]
                                                  if (globalCertIndex >= 0 && globalCertIndex < certs.length) {
                                                    certs[globalCertIndex].certificate_no = e.target.value
                                                    newPersonnel[index].training_certificates = certs
                                                    setFormData((prev) => ({
                                                      ...prev,
                                                      working_content: {
                                                        ...prev.working_content,
                                                        personnel: newPersonnel
                                                      }
                                                    }))
                                                  }
                                                }}
                                                placeholder="證書編號"
                                                className="text-xs h-7"
                                              />
                                              <Button
                                                type="button"
                                                variant="ghost"
                                                size="icon"
                                                className="h-6 w-6 text-red-500"
                                                onClick={() => {
                                                  const newPersonnel = [...formData.working_content.personnel]
                                                  const certs = [...(newPersonnel[index].training_certificates || [])]
                                                  if (globalCertIndex >= 0 && globalCertIndex < certs.length) {
                                                    certs.splice(globalCertIndex, 1)
                                                    newPersonnel[index].training_certificates = certs
                                                    setFormData((prev) => ({
                                                      ...prev,
                                                      working_content: {
                                                        ...prev.working_content,
                                                        personnel: newPersonnel
                                                      }
                                                    }))
                                                  }
                                                }}
                                              >
                                                X
                                              </Button>
                                            </div>
                                          )
                                        })}
                                        <Button
                                          type="button"
                                          variant="outline"
                                          size="sm"
                                          className="h-7 text-xs"
                                          onClick={() => {
                                            const newPersonnel = [...formData.working_content.personnel]
                                            if (!newPersonnel[index].training_certificates) {
                                              newPersonnel[index].training_certificates = []
                                            }
                                            newPersonnel[index].training_certificates.push({
                                              training_code: trainingCode,
                                              certificate_no: ''
                                            })
                                            setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                          }}
                                        >
                                          + 新增證書
                                        </Button>
                                      </div>
                                    )
                                  })}
                                </div>
                              </td>
                              <td className="border p-2 text-center">
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="icon"
                                  className="h-8 w-8 text-red-500"
                                  onClick={() => {
                                    const newPersonnel = [...formData.working_content.personnel]
                                    newPersonnel.splice(index, 1)
                                    setFormData((prev) => ({
                          ...prev,
                          working_content: {
                            ...prev.working_content,
                            personnel: newPersonnel
                          }
                        }))
                                  }}
                                >
                                  X
                                </Button>
                              </td>
                            </tr>
                          ))}
                          {(!formData.working_content.personnel || formData.working_content.personnel.length === 0) && (
                            <tr>
                              <td colSpan={7} className="border p-4 text-center text-muted-foreground">
                                暫無人員資料，請點擊「+ 新增」添加
                              </td>
                            </tr>
                          )}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'attachments' && (
            <Card>
              <CardHeader>
                <CardTitle>9. 附件<br />(Attachments)</CardTitle>
                <CardDescription>上傳相關附件與文件</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="text-center py-8 text-muted-foreground">
                  <Paperclip className="h-12 w-12 mx-auto mb-2" />
                  <p>附件上傳功能尚未開放</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
