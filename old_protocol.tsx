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
  { key: 'basic', label: <>1. ?弦鞈?<br />嚗tudy Information嚗?/>, icon: FileText },
  { key: 'purpose', label: <>2. ?弦?桃?<br />嚗tudy Purpose嚗?/>, icon: ClipboardList },
  { key: 'items', label: <>3. 閰阡??抵釭???抒鞈?br />嚗esting and Control Item嚗?/>, icon: Beaker },
  { key: 'design', label: <>4. ?弦閮剛??瘜?br />嚗tudy Design and Methods嚗?/>, icon: ClipboardList },
  { key: 'guidelines', label: <>5. ?賊?閬???????br />嚗uidelines and References嚗?/>, icon: FileText },
  { key: 'surgery', label: <>6. ??閮??br />嚗nimal Surgical Plan嚗?/>, icon: Stethoscope },
  { key: 'animals', label: <>7. 撖阡??鞈?<br />嚗nimal Information嚗?/>, icon: User },
  { key: 'personnel', label: <>8. 閰阡?鈭箏鞈?<br />嚗ersonnel Working on Animal Study嚗?/>, icon: Users },
  { key: 'attachments', label: <>9. ?辣<br />嚗ttachments嚗?/>, icon: Paperclip },
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
        selected_type?: string // 'biological' | 'radioactive' | 'chemical' - 鈭?豢?
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
      postop_care_type?: 'orthopedic' | 'non_orthopedic' // 撉函?????撉函???
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
        sex: string // ?桐??批?豢?
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
      id?: number // 蝺刻?
      name: string
      position: string
      roles: string[] // 撌乩??批捆嚗, b, c, d, e, f, g, h, i
      roles_other_text?: string // 憒??豢? i.?嗡?嚗?閬‵撖怨牧??      years_experience: number // ???閰阡?撟湔
      trainings: string[] // 閮毀/鞈嚗, B, C, D, E
      training_certificates: Array<{ // 瘥?蝺渡?霅蝺刻??”
        training_code: string // A, B, C, D, E
        certificate_no: string // 霅蝺刻?
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
      facility: { title: '鞊砍?憯怠??拍???∩遢???砍', address: '' },
      housing_location: '??蝮??樴憭?????-15??
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
        humane_endpoint: '撖阡???銝剖????拚???????擃???0%???曆???(?⊥??脤?)?澈擃?撘晞?????瘝餌??????萄??⊥???隞??賊撣怨?隡唬?摰?蝥祕撽??耦嚗??蝯?撖阡?嚗誑蝚血??蝳???
      },
      final_handling: { method: '', transfer: { recipient_name: '', recipient_org: '', project_name: '' } },
      carcass_disposal: { 
        method: '憪蝪賜?銋??澆?鋆賢??脰??ˊ??\n (?ˊ撱??迂嚗?瘚琿??蝘??∩遢???砍嚗?鋆賢?蝞∠楊嚗6001213)'
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
      preop_preparation: '1.撖阡??銵?蝳??喳?12撠?嚗?蝳偌?n2.閰阡?鞊祇蝬?瘣銋曉?嚗誑???? Azeperonum 40 mg/mL)3-5 mg/kg??.03-0.05 mg/kg?踵?撟?Atropine簧 1 mg/mL)??瘜典??桅???蝝啗?撖惇?餃?賊?n3.蝬?0-30??敺?隞?.4 mg/kg?陸-50(Zoletil簧-50)??瘜典?隤?暻駁?\n4.蝬?-10??敺?撠惇?餌宏?單?銵狗銝?隞亥韌憪輸脰?瘞?恣?恣嚗銝除擃獄??嚗誑2-3 L/min瘚?瘞扳除瘛瑕?0.5-2%瘞??暻駁??soflurane蝬剜?暻駁?嚗?釣??撖惇?駁獄?楛摨艾n5.銵???瘜典???蝝efazolin 15 mg/kg?迫??meloxicam 0.4 mg/kg\n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?',
      aseptic_techniques: [],
      surgery_description: '隢底餈唳?銵?蝔????雿蔭??銵瘜??萄之撠?蝮怠???',
      surgery_steps: [],
      monitoring: '???脰?銝凋?閰阡?鞊祇暻駁?瘛勗漲??賊?????閬?隤踵瘞扳除??瘞????暻駁?瘞??瞈漲嚗??釣??皞恬??乩???????剁???閮?敹歲??詨?擃澈?n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?',
      postop_expected_impact: '',
      multiple_surgeries: { used: false, number: 0, reason: '' },
      postop_care_type: undefined,
      postop_care: '',
      drugs: [
        { drug_name: 'Atropine', dose: '1mg/ml', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
        { drug_name: '????Azeperonum)', dose: '0.03-0.5mg/kg', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
        { drug_name: '?陸Zoletil簧-50', dose: '3-5 mg/kg', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
        { drug_name: 'Cefazolin', dose: '15-30 mg/kg', route: 'IM', frequency: '銵?1甈?銵?SID', purpose: '銵???敺???' },
        { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'IM', frequency: '銵?1甈?銵?SID', purpose: '銵???敺迫?' },
        { drug_name: 'Isoflurane', dose: '0.5-2%', route: '?詨', frequency: '銵葉', purpose: '暻駁?蝬剜?' },
        { drug_name: 'ketoprofen', dose: '1-3mg/kg', route: 'IM', frequency: 'SID', purpose: '銵?甇Ｙ??? },
        { drug_name: 'pencillin', dose: '0.1-1mL/kg', route: 'IM', frequency: 'SID', purpose: '銵???蝝? },
        { drug_name: 'cephalexin', dose: '30-60mg/kg', route: 'PO', frequency: 'BID', purpose: '銵???蝝? },
        { drug_name: 'amoxicillin', dose: '20-40mg/kg', route: 'PO', frequency: 'BID', purpose: '銵???蝝? },
        { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'PO', frequency: 'SID', purpose: '銵?甇Ｙ??? }
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
        housing_location: '鞊砍?憯怎??批'
      }], 
      total_animals: 0 
    },
    personnel: [
      {
        id: 1,
        name: '閮梯??,
        position: '',
        roles: ['b', 'c', 'd', 'f', 'g', 'h'],
        roles_other_text: '',
        years_experience: 6,
        trainings: ['C', 'A'],
        training_certificates: [
          { training_code: 'C', certificate_no: '頛餃?閮?蝚?080551' },
          { training_code: 'C', certificate_no: '111頛餃?閮匱??摮洵0983?? },
          { training_code: 'A', certificate_no: '111颲脩?撖血?摮洵0513?? },
          { training_code: 'C', certificate_no: '112頛餃?閮匱??摮洵3107?? }
        ]
      },
      {
        id: 2,
        name: '?單∪?',
        position: '',
        roles: ['b', 'c', 'd', 'f', 'g', 'h'],
        roles_other_text: '',
        years_experience: 6,
        trainings: ['C', 'A'],
        training_certificates: [
          { training_code: 'C', certificate_no: '頛餃?閮?蝚?080552' },
          { training_code: 'A', certificate_no: '110颲脩?撖血?摮洵0461?? },
          { training_code: 'C', certificate_no: '111頛餃?閮匱??摮洵4159?? },
          { training_code: 'A', certificate_no: '112颲脩?撖血?摮洵0213?? }
        ]
      },
      {
        id: 3,
        name: '????,
        position: '',
        roles: ['b', 'c', 'd', 'f', 'g', 'h'],
        roles_other_text: '',
        years_experience: 4,
        trainings: ['C', 'A'],
        training_certificates: [
          { training_code: 'C', certificate_no: '頛餃?閮?蝚?091274' },
          { training_code: 'C', certificate_no: '111頛餃?閮匱??摮洵0979?? },
          { training_code: 'A', certificate_no: '111颲脩?撖血?摮洵0512?? },
          { training_code: 'C', certificate_no: '111頛餃?閮匱??摮洵3105?? }
        ]
      },
      {
        id: 4,
        name: '?偶??,
        position: '',
        roles: ['b', 'c', 'd', 'f', 'g', 'h'],
        roles_other_text: '',
        years_experience: 5,
        trainings: ['C', 'A'],
        training_certificates: [
          { training_code: 'C', certificate_no: '頛餃?閮?蝚?090109' },
          { training_code: 'A', certificate_no: '109颲脩?撖血?摮洵0093?? },
          { training_code: 'C', certificate_no: '111頛餃?閮匱??摮洵0982?? },
          { training_code: 'A', certificate_no: '111颲脩?撖血?摮洵0514?? }
        ]
      },
      {
        id: 5,
        name: '瞏?瞏?,
        position: '',
        roles: ['b', 'c', 'd', 'f', 'g', 'h'],
        roles_other_text: '',
        years_experience: 1,
        trainings: ['C', 'A'],
        training_certificates: [
          { training_code: 'C', certificate_no: '頛餃?閮?蝚?130188' },
          { training_code: 'A', certificate_no: '113?芾噙撖血?摮洵0006?? }
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

  // 瑼Ｘ?臬?箏銵??貉??莎?IACUC_STAFF嚗?  const isIACUCStaff = user?.roles?.some(r => ['IACUC_STAFF', 'SYSTEM_ADMIN'].includes(r))

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

        // 憒?璈??迂??蝵桃蝛綽?雿輻?身??        if (mergedWorkingContent.basic) {
          if (!mergedWorkingContent.basic.facility?.title || !mergedWorkingContent.basic.facility.title.trim()) {
            mergedWorkingContent.basic.facility = {
              ...mergedWorkingContent.basic.facility,
              title: '鞊砍?憯怠??拍???∩遢???砍'
            }
          }
          if (!mergedWorkingContent.basic.housing_location || !mergedWorkingContent.basic.housing_location.trim()) {
            mergedWorkingContent.basic.housing_location = '??蝮??樴憭?????-15??
          }
        }

        // 蝣箔? use_test_item 憒???undefined嚗?閮剔 null
        if (mergedWorkingContent.items && mergedWorkingContent.items.use_test_item === undefined) {
          mergedWorkingContent.items.use_test_item = null
        }

        // 蝣箔? test_items ??control_items 銝剔? photos 摮挾摮
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

        // 蝣箔?鈭粹?蝯???閮剖摰?        if (mergedWorkingContent.design && mergedWorkingContent.design.endpoints) {
          if (!mergedWorkingContent.design.endpoints.humane_endpoint || !mergedWorkingContent.design.endpoints.humane_endpoint.trim()) {
            mergedWorkingContent.design.endpoints.humane_endpoint = '撖阡???銝剖????拚???????擃???0%???曆???(?⊥??脤?)?澈擃?撘晞?????瘝餌??????萄??⊥???隞??賊撣怨?隡唬?摰?蝥祕撽??耦嚗??蝯?撖阡?嚗誑蝚血??蝳???
          }
        }

        // 蝣箔??撅????寞???閮剖摰?        if (mergedWorkingContent.design && mergedWorkingContent.design.carcass_disposal) {
          if (!mergedWorkingContent.design.carcass_disposal.method || !mergedWorkingContent.design.carcass_disposal.method.trim()) {
            mergedWorkingContent.design.carcass_disposal.method = '憪蝪賜?銋??澆?鋆賢??脰??ˊ??\n(?迂嚗?瘚琿??蝘??∩遢???砍嚗?鋆賢?蝞∠楊嚗6001213)'
          }
        }

        // 蝣箔? hazards.materials 銝剔? photos 摮挾摮
        if (mergedWorkingContent.design && mergedWorkingContent.design.hazards && mergedWorkingContent.design.hazards.materials) {
          mergedWorkingContent.design.hazards.materials = mergedWorkingContent.design.hazards.materials.map((item: any) => ({
            ...item,
            photos: item.photos || []
          }))
        }

        // 蝣箔? controlled_substances.items 銝剔? photos 摮挾摮
        if (mergedWorkingContent.design && mergedWorkingContent.design.controlled_substances && mergedWorkingContent.design.controlled_substances.items) {
          mergedWorkingContent.design.controlled_substances.items = mergedWorkingContent.design.controlled_substances.items.map((item: any) => ({
            ...item,
            photos: item.photos || []
          }))
        }

        // 蝣箔? personnel 銝剔? training_certificates 摮挾摮
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

        // ?寞? 4.1.1 ??????銵??
        if (mergedWorkingContent.design && mergedWorkingContent.design.anesthesia && mergedWorkingContent.surgery) {
          const anesthesiaType = mergedWorkingContent.design.anesthesia.anesthesia_type
          const needsSurgeryPlan = mergedWorkingContent.design.anesthesia.is_under_anesthesia === true &&
            (anesthesiaType === 'survival_surgery' || anesthesiaType === 'non_survival_surgery')
          
          if (needsSurgeryPlan) {
            // 憒??閬‵撖急?銵??嚗?身蝵格?銵車憿?            if (anesthesiaType === 'survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'survival'
            } else if (anesthesiaType === 'non_survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'non_survival'
            }
            // 憒?銵?皞??箇征嚗身蝵桅?閮剖摰?            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = '1.撖阡??銵?蝳??喳?12撠?嚗?蝳偌?n2.閰阡?鞊祇蝬?瘣銋曉?嚗誑???? Azeperonum 40 mg/mL)3-5 mg/kg??.03-0.05 mg/kg?踵?撟?Atropine簧 1 mg/mL)??瘜典??桅???蝝啗?撖惇?餃?賊?n3.蝬?0-30??敺?隞?.4 mg/kg?陸-50(Zoletil簧-50)??瘜典?隤?暻駁?\n4.蝬?-10??敺?撠惇?餌宏?單?銵狗銝?隞亥韌憪輸脰?瘞?恣?恣嚗銝除擃獄??嚗誑2-3 L/min瘚?瘞扳除瘛瑕?0.5-2%瘞??暻駁??soflurane蝬剜?暻駁?嚗?釣??撖惇?駁獄?楛摨艾n5.銵???瘜典???蝝efazolin 15 mg/kg?迫??meloxicam 0.4 mg/kg\n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?'
            }
            // 憒????批捆隤芣??箇征嚗身蝵桅?閮剖摰?            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = '隢底餈唳?銵?蝔????雿蔭??銵瘜??萄之撠?蝮怠???'
            }
            // 憒?銵葉???箇征嚗身蝵桅?閮剖摰?            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = '???脰?銝凋?閰阡?鞊祇暻駁?瘛勗漲??賊?????閬?隤踵瘞扳除??瘞????暻駁?瘞??瞈漲嚗??釣??皞恬??乩???????剁???閮?敹歲??詨?擃澈?n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?'
            }
            // 憒?銵??扯風憿??芷??銝?身蝵殷?霈?園??
          } else {
            // 憒?銝?閬‵撖急?銵??嚗?‵撖???
            if (!mergedWorkingContent.surgery.surgery_type || mergedWorkingContent.surgery.surgery_type.trim() === '') {
              mergedWorkingContent.surgery.surgery_type = '??
            }
            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = '??
            }
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = '??
            }
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = '??
            }
            if (!mergedWorkingContent.surgery.postop_expected_impact || mergedWorkingContent.surgery.postop_expected_impact.trim() === '') {
              mergedWorkingContent.surgery.postop_expected_impact = '??
            }
            if (!mergedWorkingContent.surgery.postop_care || mergedWorkingContent.surgery.postop_care.trim() === '') {
              mergedWorkingContent.surgery.postop_care = '1.銵?瘥閰摯鞊祇?亙熒???靘?敺?瘜脰??瑕霅瑞??n2.銵?7?亙瘥?脰??潛?閰摯靘惇?餌?瘜蒂靘銝???蝯虫?甇Ｙ??亙???蝝n??拚?嚗n2.1\n撉函???\n甇Ｙ??功nketoprofen 1-3 mg/kg IM SID (3憭?\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14憭拇??瑟?蝯虫?)\n??蝝ncefazolin 15 mg/kg IM BID (1-7憭?\ncephalexin 30 mg/kg PO BID (8-14憭拇??瑟?蝯虫?)\n\n2.2\n?爸蝘?銵n甇Ｙ??功nmeloxicam 0.1-0.4 mg/kg IM SID (3憭?\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14憭拇??瑟?蝯虫?)\n??蝝npencillin 10000 u/kg IM SID (1-7憭?\namoxicillin 20 mg/kg PO BID (8-14憭拇??瑟?蝯虫?)\n\n3.?亙??拍?撣豢?敶ｇ????賊撣急?蝷箄??n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?'
            }
            if (!mergedWorkingContent.surgery.expected_end_point || mergedWorkingContent.surgery.expected_end_point.trim() === '') {
              mergedWorkingContent.surgery.expected_end_point = '??
            }
            // 憒?銝?閬‵撖急?銵??嚗?蝛箇?亥?閮?            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = []
            }
            // 憒????刻鞈??箇征嚗身蝵桅?閮剖摰?            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = [
                { drug_name: 'Atropine', dose: '1mg/ml', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
                { drug_name: '????Azeperonum)', dose: '0.03-0.5mg/kg', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
                { drug_name: '?陸Zoletil簧-50', dose: '3-5 mg/kg', route: 'IM', frequency: '1甈?, purpose: '暻駁?隤?' },
                { drug_name: 'Cefazolin', dose: '15-30 mg/kg', route: 'IM', frequency: '銵?1甈?銵?SID', purpose: '銵???敺???' },
                { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'IM', frequency: '銵?1甈?銵?SID', purpose: '銵???敺迫?' },
                { drug_name: 'Isoflurane', dose: '0.5-2%', route: '?詨', frequency: '銵葉', purpose: '暻駁?蝬剜?' },
                { drug_name: 'ketoprofen', dose: '1-3mg/kg', route: 'IM', frequency: 'SID', purpose: '銵?甇Ｙ??? },
                { drug_name: 'pencillin', dose: '0.1-1mL/kg', route: 'IM', frequency: 'SID', purpose: '銵???蝝? },
                { drug_name: 'cephalexin', dose: '30-60mg/kg', route: 'PO', frequency: 'BID', purpose: '銵???蝝? },
                { drug_name: 'amoxicillin', dose: '20-40mg/kg', route: 'PO', frequency: 'BID', purpose: '銵???蝝? },
                { drug_name: 'meloxicam', dose: '0.1-0.4mg/kg', route: 'PO', frequency: 'SID', purpose: '銵?甇Ｙ??? }
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
        title: '??',
        description: '閮?詨歇撱箇?',
      })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
      navigate(`/protocols/${response.data.id}`)
    },
    onError: (error: any) => {
      toast({
        title: '?航炊',
        description: error?.response?.data?.error?.message || '撱箇?憭望?',
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async (data: UpdateProtocolRequest) => api.put(`/protocols/${id}`, data),
    onSuccess: () => {
      toast({
        title: '??',
        description: '閮?詨歇?脣?',
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: any) => {
      toast({
        title: '?航炊',
        description: error?.response?.data?.error?.message || '?脣?憭望?',
        variant: 'destructive',
      })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/submit`),
    onSuccess: () => {
      toast({
        title: '??',
        description: '閮?詨歇?漱撖拇',
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      navigate(`/protocols/${id}`)
    },
    onError: (error: any) => {
      toast({
        title: '?航炊',
        description: error?.response?.data?.error?.message || '?漱憭望?',
        variant: 'destructive',
      })
    },
  })

  // 撽?敹‵摮挾嚗ection 1 - ?弦鞈?嚗?  const validateRequiredFields = (): string | null => {
    const { basic, purpose } = formData.working_content

    // 1. ?弦?迂
    if (!formData.title.trim()) {
      return '隢‵撖怎?蝛嗅?蝔?(Study Title)'
    }

    // 2. ??閰阡???
    if (!formData.start_date || !formData.end_date) {
      return '隢‵撖恍?閮岫撽?蝔?
    }

    // 3. 閮憿?
    if (!basic.project_type || !basic.project_type.trim()) {
      return '隢???恍???
    }

    // 4. 閮蝔桅?
    if (!basic.project_category || !basic.project_category.trim()) {
      return '隢???怎車憿?
    }
    if (basic.project_category === 'other' && (!basic.project_category_other || !basic.project_category_other.trim())) {
      return '隢‵撖怠隞??怎車憿牧??
    }

    // 5. PI 鞈?
    if (!basic.pi.name || !basic.pi.name.trim()) {
      return '隢‵撖怨??思蜓?犖憪?'
    }
    if (!basic.pi.email || !basic.pi.email.trim()) {
      return '隢‵撖怨??思蜓?犖 Email'
    }
    if (!basic.pi.phone || !basic.pi.phone.trim()) {
      return '隢‵撖怨??思蜓?犖?餉店'
    }
    if (!basic.pi.address || !basic.pi.address.trim()) {
      return '隢‵撖怨??思蜓?犖?啣?'
    }

    // 6. Sponsor 鞈?
    if (!basic.sponsor.name || !basic.sponsor.name.trim()) {
      return '隢‵撖怠?閮雿?蝔?
    }
    if (!basic.sponsor.contact_person || !basic.sponsor.contact_person.trim()) {
      return '隢‵撖怠?閮雿蝯∩犖'
    }
    if (!basic.sponsor.contact_phone || !basic.sponsor.contact_phone.trim()) {
      return '隢‵撖怠?閮雿蝯⊿閰?
    }
    if (!basic.sponsor.contact_email || !basic.sponsor.contact_email.trim()) {
      return '隢‵撖怠?閮雿蝯?Email'
    }

    // 7. 璈??迂
    if (!basic.facility.title || !basic.facility.title.trim()) {
      return '隢‵撖急?瑽?蝔?
    }

    // 8. 雿蔭
    if (!basic.housing_location || !basic.housing_location.trim()) {
      return '隢‵撖思?蝵?
    }

    // Section 2 - ?弦?桃?
    // 2.1 ?弦銋??????    if (!purpose.significance || !purpose.significance.trim()) {
      return '隢‵撖怎?蝛嗡??桃???閬?
    }

    // 2.2.1 瘣駁??閰阡?銋?閬?    if (!purpose.replacement.rationale || !purpose.replacement.rationale.trim()) {
      return '隢牧?暑擃??抵岫撽?敹??改?隞亙??豢?甇文??拍車?亦???'
    }

    // 2.2.2 ???拇隞?獢?撠??澈
    if (!purpose.replacement.alt_search.platforms || purpose.replacement.alt_search.platforms.length === 0) {
      return '隢撠??????扳隞?獢?撠??澈'
    }
    if (!purpose.replacement.alt_search.keywords || !purpose.replacement.alt_search.keywords.trim()) {
      return '隢‵撖急?撠??萄?'
    }
    if (!purpose.replacement.alt_search.conclusion || !purpose.replacement.alt_search.conclusion.trim()) {
      return '隢‵撖急?撠???蝯?'
    }

    // 2.2.3 ??閰阡??嚗??????嚗?    if (purpose.duplicate.experiment && (!purpose.duplicate.justification || !purpose.duplicate.justification.trim())) {
      return '隢牧??銴脰?銋?摮貊???
    }

    // 2.3 皜??? - 撖阡?閮剛?隤芣?
    if (!purpose.reduction.design || !purpose.reduction.design.trim()) {
      return '隢‵撖怠祕撽身閮牧???????寞???摰蝙?典??拇???蝑?'
    }

    // Section 4 - ?弦閮剛??瘜?    const { design } = formData.working_content
    // 4.1.1 憒??豢?"??嚗脰?暻駁?嚗?敹??豢?暻駁?憿?
    if (design.anesthesia.is_under_anesthesia === true) {
      if (!design.anesthesia.anesthesia_type || !design.anesthesia.anesthesia_type.trim()) {
        return '隢?獄????
      }
      // 憒??豢?"?嗡?"嚗??‵撖怨牧??      if (design.anesthesia.anesthesia_type === 'other' && (!design.anesthesia.other_description || !design.anesthesia.other_description.trim())) {
        return '隢‵撖怠隞獄?撘?隤芣?'
      }
    }
    // 4.1.2 閰喟敦?膩?閰阡??批捆??蝔?    if (!design.procedures || !design.procedures.trim()) {
      return '隢底蝝唳?餈啣??抵岫撽摰孵?瘚?'
    }
    // 4.1.3 撖阡??蝑?閰摯
    if (!design.pain.category || !design.pain.category.trim()) {
      return '隢?祕撽??拍?蝝?隡?
    }
    // 4.1.4 憒??豢?"??嚗??園ㄡ憌?憌脫偌嚗?敹??豢??憿?
    if (design.restrictions.is_restricted === true) {
      if (!design.restrictions.restriction_type || !design.restrictions.restriction_type.trim()) {
        return '隢???園???
      }
      // 憒??豢?"?嗡?"嚗??‵撖怨牧??      if (design.restrictions.restriction_type === 'other' && (!design.restrictions.other_description || !design.restrictions.other_description.trim())) {
        return '隢‵撖怠隞??嗆撘?隤芣?'
      }
    }
    // 4.1.5 撖阡???蝯?銋?璈?    if (!design.endpoints.experimental_endpoint || !design.endpoints.experimental_endpoint.trim()) {
      return '隢‵撖怠祕撽?暺?
    }
    if (!design.endpoints.humane_endpoint || !design.endpoints.humane_endpoint.trim()) {
      return '隢‵撖思犖??暺?
    }
    // 4.3 憒??豢?"??嚗蝙?券??怨蝝?摮貉??嚗??‵撖怨牧??    if (design.non_pharma_grade.used === true) {
      if (!design.non_pharma_grade.description || !design.non_pharma_grade.description.trim()) {
        return '隢牧?鞈芣扯釭???冽批?雿輻銋?摮貊???
      }
    }
    // 4.4 憒??豢?"??嚗蝙?典摰單抒鞈芣???嚗?????蒂憛怠神??鞈?
    if (design.hazards.used === true) {
      if (!design.hazards.selected_type || !design.hazards.selected_type.trim()) {
        return '隢?摰單抒鞈芷???
      }
      if (design.hazards.materials.length === 0 || design.hazards.materials.every(m => !m.agent_name || !m.agent_name.trim())) {
        return '隢撠‵撖思??摰單抒鞈芰??迂'
      }
      // 撽?瘥????蝔勗??券?
      for (let i = 0; i < design.hazards.materials.length; i++) {
        const material = design.hazards.materials[i]
        if (!material.agent_name || !material.agent_name.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?摰單抒鞈芰??迂`
        }
        if (!material.amount || !material.amount.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?摰單抒鞈芰????券?`
        }
      }
      // 4.5 ?勗拿?抒鞈芸??嗅誥璉???孵?嚗???4.4 ????嚗?      if (!design.hazards.operation_location_method || !design.hazards.operation_location_method.trim()) {
        return '隢‵撖急?冽瘜??蝙?典?'
      }
      if (!design.hazards.protection_measures || !design.hazards.protection_measures.trim()) {
        return '隢‵撖思?霅瑟??
      }
      if (!design.hazards.waste_and_carcass_disposal || !design.hazards.waste_and_carcass_disposal.trim()) {
        return '隢‵撖怠祕撽誥璉??擃????孵?'
      }
    }
    // 4.6 ??4.5嚗 4.4 ???????臬雿輻蝞∪?亙?
    if (design.controlled_substances.used === true) {
      if (design.controlled_substances.items.length === 0) {
        return '隢撠溶???恣?嗉??
      }
      // 撽?瘥恣?嗉??敹‵摮挾
      for (let i = 0; i < design.controlled_substances.items.length; i++) {
        const item = design.controlled_substances.items[i]
        if (!item.drug_name || !item.drug_name.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?恣?嗉???亙??迂`
        }
        if (!item.approval_no || !item.approval_no.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?恣?嗉???詨?蝺刻?`
        }
        if (!item.amount || !item.amount.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?恣?嗉?????券?`
        }
        if (!item.authorized_person || !item.authorized_person.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?恣?嗉??蝞∪?亙?蝞∠?鈭槁
        }
      }
    }

    // 6. ??閮???- ?寞? 4.1.1 ???瑟?阡?閬‵撖?    const needsSurgeryPlan = design.anesthesia.is_under_anesthesia === true &&
      (design.anesthesia.anesthesia_type === 'survival_surgery' || design.anesthesia.anesthesia_type === 'non_survival_surgery')
    
    if (needsSurgeryPlan) {
      const { surgery } = formData.working_content
      if (!surgery.surgery_type || !surgery.surgery_type.trim() || surgery.surgery_type === '??) {
        return '隢‵撖急?銵車憿?
      }
      if (!surgery.preop_preparation || !surgery.preop_preparation.trim() || surgery.preop_preparation === '??) {
        return '隢‵撖怨?????
      }
      if (!surgery.surgery_description || !surgery.surgery_description.trim() || surgery.surgery_description === '??) {
        return '隢‵撖急?銵摰寡牧??
      }
      if (!surgery.monitoring || !surgery.monitoring.trim()) {
        return '隢‵撖怨?銝剔??
      }
      // 6.6 ?芸摮暑????閬‵撖?      if (surgery.surgery_type === 'survival') {
        if (!surgery.postop_expected_impact || !surgery.postop_expected_impact.trim() || surgery.postop_expected_impact === '??) {
          return '隢‵撖怠?瘣餅?銵???敺?賢?撖阡????銋蔣??
        }
      }
      if (surgery.multiple_surgeries.used) {
        if (!surgery.multiple_surgeries.number || surgery.multiple_surgeries.number <= 0) {
          return '隢‵撖怠?甈⊥?銵??賊?'
        }
        if (!surgery.multiple_surgeries.reason || !surgery.multiple_surgeries.reason.trim()) {
          return '隢‵撖怠?甈⊥?銵???'
        }
      }
      if (!surgery.postop_care_type || !surgery.postop_care_type.trim()) {
        return '隢??銵???撉函?????撉函???嚗?
      }
      if (!surgery.postop_care || !surgery.postop_care.trim()) {
        return '隢‵撖怠??抵?敺霅瑕?甇Ｙ?蝯西?寞?'
      }
      if (!surgery.expected_end_point || !surgery.expected_end_point.trim()) {
        return '隢‵撖怠祕撽???????'
      }
      // 6.10 ???刻鞈?
      if (!surgery.drugs || surgery.drugs.length === 0) {
        return '隢撠溶????銵?亥?閮?
      }
      for (let i = 0; i < surgery.drugs.length; i++) {
        const drug = surgery.drugs[i]
        if (!drug.drug_name || !drug.drug_name.trim()) {
          return `隢‵撖怎洵 ${i + 1} ??亦??亙??迂`
        }
        if (!drug.dose || !drug.dose.trim()) {
          return `隢‵撖怎洵 ${i + 1} ??亦???`
        }
        if (!drug.route || !drug.route.trim()) {
          return `隢‵撖怎洵 ${i + 1} ??亦?????`
        }
        if (!drug.frequency || !drug.frequency.trim()) {
          return `隢‵撖怎洵 ${i + 1} ??亦??餌?`
        }
        if (!drug.purpose || !drug.purpose.trim()) {
          return `隢‵撖怎洵 ${i + 1} ??亦?蝯西?桃?`
        }
      }
    }

    // Section 3 - 閰阡??抵釭???抒鞈?    const { items } = formData.working_content
    if (items.use_test_item === null) {
      return '隢??行?鈭岫撽鞈芥?'
    }

    // 憒??豢?"??嚗?霅岫撽鞈芸?撠?抵釭??憛怠?畾?    if (items.use_test_item === true) {
      // 撽?閰阡??抵釭
      for (let i = 0; i < items.test_items.length; i++) {
        const item = items.test_items[i]
        if (!item.name || !item.name.trim()) {
          return `隢‵撖怎洵 ${i + 1} ?岫撽鞈芰??迂`
        }
        // 憒??豢?"??嚗??∟?鋆賢?嚗?敹?憛怠神隤芣?
        if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
          return `隢‵撖怎洵 ${i + 1} ?岫撽鞈芰???ˊ?牧?
        }
      }

      // 撽?撠?抵釭
      for (let i = 0; i < items.control_items.length; i++) {
        const item = items.control_items[i]
        if (!item.name || !item.name.trim()) {
          return `隢‵撖怎洵 ${i + 1} ???抒鞈芰??迂`
        }
        // 憒??豢?"??嚗??∟?鋆賢?嚗?敹?憛怠神隤芣?
        if (!item.is_sterile && (!item.non_sterile_justification || !item.non_sterile_justification.trim())) {
          return `隢‵撖怎洵 ${i + 1} ???抒鞈芰???ˊ?牧?
        }
      }
    }

    // Section 7 - 撖阡??鞈?
    const { animals } = formData.working_content
    if (!animals.animals || animals.animals.length === 0) {
      return '隢撠溶???祕撽??抵???
    }
    for (let i = 0; i < animals.animals.length; i++) {
      const animal = animals.animals[i]
      // 撽??拍車
      if (!animal.species || !animal.species.trim()) {
        return `隢?洵 ${i + 1} ???拍??拍車`
      }
      if (animal.species === 'other' && (!animal.species_other || !animal.species_other.trim())) {
        return `隢‵撖怎洵 ${i + 1} ???拍??拍車`
      }
      // 撽??頂
      if (animal.species === 'pig' && !animal.strain) {
        return `隢?洵 ${i + 1} ???拍??頂`
      }
      if (animal.species === 'other' && (!animal.strain_other || !animal.strain_other.trim())) {
        return `隢‵撖怎洵 ${i + 1} ???拍??頂`
      }
      // 撽??批
      if (!animal.sex || !animal.sex.trim()) {
        return `隢?洵 ${i + 1} ???拍??批`
      }
      // 撽??賊?
      if (!animal.number || animal.number <= 0) {
        return `隢‵撖怎洵 ${i + 1} ???拍??賊?嚗??之??嚗
      }
      // 撽?撟湧翩嚗?????銝?"嚗?      if (!animal.age_unlimited) {
        if (animal.age_min === undefined || animal.age_min < 3) {
          return `蝚?${i + 1} ???拍??撠?朣∪??撠3??`
        }
        if (animal.age_max === undefined) {
          return `隢‵撖怎洵 ${i + 1} ???拍??憭扳?朣︶
        }
        if (animal.age_max <= animal.age_min) {
          return `蝚?${i + 1} ???拍??憭扳?朣∪??之?潭?撠?朣︶
        }
      }
      // 撽?擃?嚗?????銝?"嚗?      if (!animal.weight_unlimited) {
        if (animal.weight_min === undefined || animal.weight_min < 20) {
          return `蝚?${i + 1} ???拍??撠????撠20?祆`
        }
        if (animal.weight_max === undefined) {
          return `隢‵撖怎洵 ${i + 1} ???拍??憭折??
        }
        if (animal.weight_max <= animal.weight_min) {
          return `蝚?${i + 1} ???拍??憭折????之?潭?撠??
        }
      }
    }

    return null
  }

  const handleSave = async () => {
    // 撽?敹‵摮挾
    const validationError = validateRequiredFields()
    if (validationError) {
      toast({
        title: '?航炊',
        description: validationError,
        variant: 'destructive',
      })
      return
    }

    setIsSaving(true)
    try {
      // 憒?銝?瑁?蝘嚗Ⅱ靽岫撽楊?蝛?      const basicContent = {
        ...formData.working_content.basic,
        study_title: formData.title,
        start_date: formData.start_date,
        end_date: formData.end_date,
      }
      
      // 憒?銝 IACUC_STAFF嚗?蝛箄岫撽楊??      if (!isIACUCStaff) {
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

    if (confirm('蝣箄?閬?鈭斗迨閮?賊脰?撖拇???漱敺??⊥??湔靽格')) {
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
              {isNew ? '?啣?閮?? : '蝺刻摩閮??}
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
            ?脣??阮
          </Button>
          {!isNew && (
            <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              ?漱撖拇
            </Button>
          )}
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[250px_1fr]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-lg">蝡?</CardTitle>
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
                <CardTitle>1. ?弦鞈?<br />(Study Information)</CardTitle>
                <CardDescription>憛怠神?弦?箸鞈??岫撽?瑽?銝餅?鈭箄???/CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 1. GLP & Title */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>GLP 撅祆?*</Label>
                    <div className="flex items-center space-x-2 pt-2">
                      <Checkbox
                        id="is_glp"
                        checked={formData.working_content.basic.is_glp}
                        onChange={(e) => updateWorkingContent('basic', 'is_glp', e.target.checked)}
                      />
                      <Label htmlFor="is_glp">蝚血? GLP 閬?</Label>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="title">?弦?迂 (Study Title) *</Label>
                    <Input
                      id="title"
                      value={formData.title}
                      onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
                      placeholder="隢撓?亦?蝛嗅?蝔?
                    />
                  </div>
                </div>

                {/* 2. IDs and Dates */}
                <div className={`grid gap-4 ${isNew || !isIACUCStaff ? 'md:grid-cols-1' : 'md:grid-cols-2'}`}>
                  {/* 閰阡?蝺刻?嚗憓??ａ??蝺刻摩??芣??瑁?蝘?舐楊頛?*/}
                  {(!isNew && isIACUCStaff) && (
                    <div className="space-y-2">
                      <Label htmlFor="apply_study_number">閰阡?蝺刻? (Study No.)</Label>
                      <Input
                        id="apply_study_number"
                        value={formData.working_content.basic.apply_study_number || ''}
                        onChange={(e) => updateWorkingContent('basic', 'apply_study_number', e.target.value)}
                        placeholder="?勗銵??詨‵撖?
                      />
                    </div>
                  )}
                  <div className="space-y-2">
                    <Label>??閰阡??? *</Label>
                    <div className="flex gap-2">
                      <Input
                        type="date"
                        value={formData.start_date}
                        onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                        required
                      />
                      <span className="self-center">??/span>
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
                    <Label>閮憿? *</Label>
                    <Select
                      value={formData.working_content.basic.project_type}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_type', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="?豢?閮憿?" /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="basic_research">?箇??弦</SelectItem>
                        <SelectItem value="applied_research">??弦</SelectItem>
                        <SelectItem value="pre_market_testing">銝??岫撽?/SelectItem>
                        <SelectItem value="teaching_training">?飛閮毀</SelectItem>
                        <SelectItem value="biologics_manufacturing">?鋆賢?鋆賡?/SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label>閮蝔桅? *</Label>
                    <Select
                      value={formData.working_content.basic.project_category}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="?豢?閮蝔桅?" /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="medical">?怨</SelectItem>
                        <SelectItem value="agricultural">颲脫平</SelectItem>
                        <SelectItem value="drug_herbal">?亦璊</SelectItem>
                        <SelectItem value="health_food">?亙熒憌?</SelectItem>
                        <SelectItem value="food">憌?</SelectItem>
                        <SelectItem value="toxic_chemical">瘥批?摮貊鞈?/SelectItem>
                        <SelectItem value="medical_device">?怎??冽?</SelectItem>
                        <SelectItem value="other">?嗡?</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.basic.project_category === 'other' && (
                      <div className="pt-2">
                        <Input
                          placeholder="隢牧?隞車憿?
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
                  <h3 className="font-semibold">閮銝餅?鈭?(Principal Investigator)</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>憪? *</Label>
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
                      <Label>?餉店 *</Label>
                      <Input
                        value={formData.working_content.basic.pi.phone}
                        onChange={(e) => updateWorkingContent('basic', 'pi.phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>?啣? *</Label>
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
                  <h3 className="font-semibold">憪??桐? (Sponsor)</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>?桐??迂 *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.name}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.name', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>?舐窗鈭?*</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_person}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_person', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>?舐窗?餉店 *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_phone}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>?舐窗 Email *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_email}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_email', e.target.value)}
                      />
                    </div>
                  </div>
                </div>

                {/* 6. Facility */}
                <div className="space-y-4">
                  <h3 className="font-semibold">閰阡?璈??身??/h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>璈??迂 *</Label>
                      <Input
                        value={formData.working_content.basic.facility.title}
                        onChange={(e) => updateWorkingContent('basic', 'facility.title', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>雿蔭 *</Label>
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
                <CardTitle>2. ?弦?桃?<br />(Study Purpose)</CardTitle>
                <CardDescription>隤芣??弦?桃???閬扯? 3Rs ?蹂誨??????/CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 2.1 ?弦銋??????*/}
                <div className="space-y-2">
                  <Label>2.1 ?弦銋??????*</Label>
                  <Textarea
                    value={formData.working_content.purpose.significance}
                    onChange={(e) => updateWorkingContent('purpose', 'significance', e.target.value)}
                    placeholder="隢牧??蝛嗉??胯摨?蝘飛???批??園?????
                    rows={5}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.2 ?蹂誨?? */}
                <div className="space-y-4">
                  <h3 className="font-semibold">2.2 隢誑?閰阡??3Rs銋隞????隤芣??砍??抵岫撽?????</h3>
                  
                  {/* 2.2.1 瘣駁??閰阡?銋?閬?*/}
                  <div className="space-y-2">
                    <Label>2.2.1 隢牧?暑擃??抵岫撽?敹??改?隞亙??豢?甇文??拍車?亦???: *</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.rationale}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.rationale', e.target.value)}
                      placeholder="隢牧?暑擃??抵岫撽?敹??改?隞亙??豢?甇文??拍車?亦???"
                      rows={4}
                    />
                  </div>

                  {/* 2.2.2 ???拇扳隞?獢?撠??澈 */}
                  <div className="space-y-2">
                    <Label>2.2.2 隢銝?蝬脩??????拇扳隞?獢?*</Label>
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
                          1. ALTBIB-???拇扳隞?瘜????餅?蝝Ｗ極??br />
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
                          2. DB-ALM?閰阡??蹂誨?寞?鞈?摨?br />
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
                          3. 甇散??蹂誨閰阡?鞈?撟喳<br />
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
                        placeholder="隢牧?隞??澈"
                        value={formData.working_content.purpose.replacement.alt_search.other_name || ''}
                        onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.other_name', e.target.value)}
                        className="mt-2"
                      />
                    )}
                  </div>
                  <div className="space-y-2">
                    <Label>???摮?*</Label>
                    <Input
                      value={formData.working_content.purpose.replacement.alt_search.keywords}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.keywords', e.target.value)}
                      placeholder="靘?嚗inipig, cardiovascular, replacement"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>??蝯???隢?*</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.alt_search.conclusion}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.conclusion', e.target.value)}
                      placeholder="隤芣???蝯??臬?潛?舀隞?獢?
                      rows={3}
                    />
                  </div>

                  {/* 2.2.3 ?臬?粹?銴?鈭箄岫撽?*/}
                  <div className="space-y-2">
                    <Label>2.2.3 ?臬?粹?銴?鈭箄岫撽?/Label>
                    <Select
                      value={formData.working_content.purpose.duplicate.experiment ? 'yes' : 'no'}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('purpose', 'duplicate.experiment', isYes)
                        // 憒??豢?"??嚗?蝛箄牧??雿?                        if (!isYes) {
                          updateWorkingContent('purpose', 'duplicate.justification', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="隢?? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">??/SelectItem>
                        <SelectItem value="yes">??/SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.purpose.duplicate.experiment && (
                      <div className="space-y-2 mt-2">
                        <Label>隢牧??銴脰?銋?摮貊???*</Label>
                        <Textarea
                          value={formData.working_content.purpose.duplicate.justification}
                          onChange={(e) => updateWorkingContent('purpose', 'duplicate.justification', e.target.value)}
                          placeholder="隢牧??銴脰?銋?摮貊???
                          rows={3}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.3 皜??? */}
                <div className="space-y-4">
                  <h3 className="font-semibold">2.3 隢誑撖阡???3Rs銋?????隤芣??閰阡?閮剛?嚗??砍??拙?蝯瘜?摰蝙?典??拇???蝑?</h3>
                  <div className="space-y-2">
                    <Label>撖阡?閮剛?隤芣? *</Label>
                    <Textarea
                      value={formData.working_content.purpose.reduction.design}
                      onChange={(e) => updateWorkingContent('purpose', 'reduction.design', e.target.value)}
                      placeholder="隢牧???拙?蝯瘜絞閮?閮准??交??斗?皞?撠??唬??寞?嚗誑??摰蝙?典??拇???"
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
                <CardTitle>3. 閰阡??抵釭???抒鞈?br />(Testing and Control Item)</CardTitle>
                <CardDescription>憛怠神閰阡??抵釭???抒鞈芾?閮?/CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>?祈??急?行?鈭岫撽鞈芥? *</Label>
                  <Select
                    value={formData.working_content.items.use_test_item === null ? '' : (formData.working_content.items.use_test_item ? 'yes' : 'no')}
                    onValueChange={(value) => {
                      const isYes = value === 'yes'
                      updateWorkingContent('items', 'use_test_item', isYes)
                      // 憒??豢?"??嚗?蝛箇鞈芸?銵?                      if (!isYes) {
                        updateWorkingContent('items', 'test_items', [])
                        updateWorkingContent('items', 'control_items', [])
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="隢?? />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="no">??/SelectItem>
                      <SelectItem value="yes">??/SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {formData.working_content.items.use_test_item === true && (
                  <>
                    {/* 閰阡??抵釭?” */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">閰阡??抵釭</h3>
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
                          ?啣?
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
                              <Label>?抵釭?迂 *</Label>
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
                              <Label>??</Label>
                              <Input
                                value={item.form || ''}
                                onChange={(e) => {
                                  const newItems = [...formData.working_content.items.test_items]
                                  newItems[index].form = e.target.value
                                  updateWorkingContent('items', 'test_items', newItems)
                                }}
                                placeholder="憒?瘨脤?????
                              />
                            </div>
                          </div>
                          <div className="space-y-2">
                            <Label>?券?/Label>
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
                            <Label>靽??啣?</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newItems = [...formData.working_content.items.test_items]
                                newItems[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                              placeholder="隢‵撖思?摮憓?
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>?祉鞈芣?衣?∟?鋆賢?</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newItems = [...formData.working_content.items.test_items]
                                const isYes = value === 'yes'
                                newItems[index].is_sterile = isYes
                                // 憒??豢?"??嚗?蝛箄牧??雿?                                if (isYes) {
                                  newItems[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="隢?? />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">??/SelectItem>
                                <SelectItem value="yes">??/SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>隢牧??*</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newItems = [...formData.working_content.items.test_items]
                                    newItems[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'test_items', newItems)
                                  }}
                                  placeholder="隢牧?雿?抵釭??ˊ??
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* ?抒?銝 */}
                          <div className="space-y-2">
                            <Label>?抒?</Label>
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
                              placeholder="??抒??唳迨????????
                            />
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* 撠?抵釭?” */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">撠?抵釭</h3>
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
                          ?啣?
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
                              <Label>撠?迂 *</Label>
                              <Input
                                value={item.name}
                                onChange={(e) => {
                                  const newControls = [...formData.working_content.items.control_items]
                                  newControls[index].name = e.target.value
                                  updateWorkingContent('items', 'control_items', newControls)
                                }}
                                placeholder="?亦撠隢‵撖?N/A"
                              />
                            </div>
                            <div className="space-y-2">
                              <Label>?券?/Label>
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
                            <Label>靽??啣?</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newControls = [...formData.working_content.items.control_items]
                                newControls[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                              placeholder="隢‵撖思?摮憓?
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>?祉鞈芣?衣?∟?鋆賢?</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newControls = [...formData.working_content.items.control_items]
                                const isYes = value === 'yes'
                                newControls[index].is_sterile = isYes
                                // 憒??豢?"??嚗?蝛箄牧??雿?                                if (isYes) {
                                  newControls[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="隢?? />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">??/SelectItem>
                                <SelectItem value="yes">??/SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>隢牧??*</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newControls = [...formData.working_content.items.control_items]
                                    newControls[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'control_items', newControls)
                                  }}
                                  placeholder="隢牧?雿?抵釭??ˊ??
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* ?抒?銝 */}
                          <div className="space-y-2">
                            <Label>?抒?</Label>
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
                              placeholder="??抒??唳迨????????
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
                <CardTitle>4. ?弦閮剛??瘜?br />(Study Design and Methods)</CardTitle>
                <CardDescription>?膩?弦閮剛??祕撽?蝔獄??鈭粹?蝯?</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 4.1 璅? */}
                <div className="space-y-2">
                  <h3 className="text-lg font-semibold">4.1 隢誑撖阡???3Rs銋移蝺餃???嚗底蝝啗牧?祕撽葉??脰?銋??抵岫撽摰嫘蝙撖阡???扯風?蝙?典??⊥?憪鈭圾?閰阡????蝔?</h3>
                </div>

                {/* 4.1.1 ?臬?潮獄???脰?閰阡? */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.1 ?臬?潮獄???脰?閰阡?</Label>
                    <Select
                      value={formData.working_content.design.anesthesia.is_under_anesthesia === null ? '' : (formData.working_content.design.anesthesia.is_under_anesthesia === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'anesthesia.is_under_anesthesia', isYes as boolean | null)
                        // 憒??豢?"??嚗?蝛箇??雿?                        if (!isYes) {
                          updateWorkingContent('design', 'anesthesia.anesthesia_type', undefined)
                          updateWorkingContent('design', 'anesthesia.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="隢?? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">??/SelectItem>
                        <SelectItem value="yes">??/SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.anesthesia.is_under_anesthesia === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>隢?獄????*</Label>
                        <Select
                          value={formData.working_content.design.anesthesia.anesthesia_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'anesthesia.anesthesia_type', value)
                            // 憒??豢??????嗡?"嚗?蝛箏隞牧??                            if (value !== 'other') {
                              updateWorkingContent('design', 'anesthesia.anesthesia_other_description', undefined)
                            }
                            // ?寞??豢??芸?????閮???                            if (value === 'survival_surgery') {
                              // 憒??臬?瘣餅?銵??芸?閮剔蔭??蝔桅???摮暑??"
                              updateWorkingContent('surgery', 'surgery_type', 'survival')
                              // 憒?銵?皞??????蝛綽?閮剔蔭?身?批捆
                              if (formData.working_content.surgery.preop_preparation === '?? || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', '1.撖阡??銵?蝳??喳?12撠?嚗?蝳偌?n2.閰阡?鞊祇蝬?瘣銋曉?嚗誑???? Azeperonum 40 mg/mL)3-5 mg/kg??.03-0.05 mg/kg?踵?撟?Atropine簧 1 mg/mL)??瘜典??桅???蝝啗?撖惇?餃?賊?n3.蝬?0-30??敺?隞?.4 mg/kg?陸-50(Zoletil簧-50)??瘜典?隤?暻駁?\n4.蝬?-10??敺?撠惇?餌宏?單?銵狗銝?隞亥韌憪輸脰?瘞?恣?恣嚗銝除擃獄??嚗誑2-3 L/min瘚?瘞扳除瘛瑕?0.5-2%瘞??暻駁??soflurane蝬剜?暻駁?嚗?釣??撖惇?駁獄?楛摨艾n5.銵???瘜典???蝝efazolin 15 mg/kg?迫??meloxicam 0.4 mg/kg\n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                              }
                              // 憒????批捆隤芣??????蝛綽?閮剔蔭?身?批捆
                              if (formData.working_content.surgery.surgery_description === '?? || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', '隢底餈唳?銵?蝔????雿蔭??銵瘜??萄之撠?蝮怠???')
                              }
                              // 憒?銵葉???箇征嚗身蝵桅?閮剖摰?                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', '???脰?銝凋?閰阡?鞊祇暻駁?瘛勗漲??賊?????閬?隤踵瘞扳除??瘞????暻駁?瘞??瞈漲嚗??釣??皞恬??乩???????剁???閮?敹歲??詨?擃澈?n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                              }
                              // 銵??扯風憿??梁?園??銝?身蝵?                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value === 'non_survival_surgery') {
                              // 憒??舫?摮暑??嚗?身蝵格?銵車憿"??瘣餅?銵?
                              updateWorkingContent('surgery', 'surgery_type', 'non_survival')
                              // 憒?銵?皞??????蝛綽?閮剔蔭?身?批捆
                              if (formData.working_content.surgery.preop_preparation === '?? || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', '1.撖阡??銵?蝳??喳?12撠?嚗?蝳偌?n2.閰阡?鞊祇蝬?瘣銋曉?嚗誑???? Azeperonum 40 mg/mL)3-5 mg/kg??.03-0.05 mg/kg?踵?撟?Atropine簧 1 mg/mL)??瘜典??桅???蝝啗?撖惇?餃?賊?n3.蝬?0-30??敺?隞?.4 mg/kg?陸-50(Zoletil簧-50)??瘜典?隤?暻駁?\n4.蝬?-10??敺?撠惇?餌宏?單?銵狗銝?隞亥韌憪輸脰?瘞?恣?恣嚗銝除擃獄??嚗誑2-3 L/min瘚?瘞扳除瘛瑕?0.5-2%瘞??暻駁??soflurane蝬剜?暻駁?嚗?釣??撖惇?駁獄?楛摨艾n5.銵???瘜典???蝝efazolin 15 mg/kg?迫??meloxicam 0.4 mg/kg\n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                              }
                              // 憒????批捆隤芣??????蝛綽?閮剔蔭?身?批捆
                              if (formData.working_content.surgery.surgery_description === '?? || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', '隢底餈唳?銵?蝔????雿蔭??銵瘜??萄之撠?蝮怠???')
                              }
                              // 憒?銵葉???箇征嚗身蝵桅?閮剖摰?                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', '???脰?銝凋?閰阡?鞊祇暻駁?瘛勗漲??賊?????閬?隤踵瘞扳除??瘞????暻駁?瘞??瞈漲嚗??釣??皞恬??乩???????剁???閮?敹歲??詨?擃澈?n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                              }
                              // 銵??扯風憿??梁?園??銝?身蝵?                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value && value !== '') {
                              // 憒?銝摮暑????摮暑??嚗?‵撖???
                              updateWorkingContent('surgery', 'surgery_type', '??)
                              updateWorkingContent('surgery', 'preop_preparation', '??)
                              updateWorkingContent('surgery', 'surgery_description', '??)
                              updateWorkingContent('surgery', 'monitoring', '??)
                              updateWorkingContent('surgery', 'postop_expected_impact', '??)
                              updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                              updateWorkingContent('surgery', 'postop_care', '??)
                              updateWorkingContent('surgery', 'postop_care_type', undefined)
                              updateWorkingContent('surgery', 'expected_end_point', '??)
                              updateWorkingContent('surgery', 'drugs', [])
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="隢?? />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="survival_surgery">1. 摮暑??嚗?憛怠神6. ??閮??賂?</SelectItem>
                            <SelectItem value="non_survival_surgery">2. ??瘣餅?銵?隢‵撖?. ??閮??賂?</SelectItem>
                            <SelectItem value="gas_only">3. ?噩?亙?閰阡?嚗?雿輻瘞??暻駁?(Isoflurane 1-2%)隤?敺??脰?撖阡?( Isoflurane inhalation before experiment)</SelectItem>
                            <SelectItem value="azeperonum_atropine">4. 雿輻???? Azeperonum 40 mg/mL)3-5 mg/kg??.03-0.05 mg/kg?踵?撟?Atropine簧 1 mg/mL)??瘜典??桅?敺?瘞??暻駁?(Isoflurane 1-2%)?脰?撖阡? (Using Azeperonum and Atropine IM with Isoflurane inhalation before experiment)</SelectItem>
                            <SelectItem value="other">5. ?嗡?</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.anesthesia.anesthesia_type === 'other' && (
                        <div className="space-y-2">
                          <Label>隢牧??*</Label>
                          <Textarea
                            value={formData.working_content.design.anesthesia.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'anesthesia.other_description', e.target.value)}
                            placeholder="隢牧?隞獄?撘?
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.2 閰喟敦?膩?閰阡??批捆??蝔?*/}
                <div className="space-y-2">
                  <Label>4.1.2 隢底蝝唳?餈啣??抵岫撽摰孵?瘚? (Animal experiment procedures)?岫撽?鈭鞈?(experimental injections or inoculations) ??鈭???府??銋??晞銵 (blood withdrawals)?蔣??撖?(CT, MRI, X-ray)??摰?(methods of restraint)???(frequency) *</Label>
                  <p className="text-sm text-muted-foreground mb-2">???賊??批捆隢?活6. ??閮?訾葉隤芣? (surgical procedures fill in surgical plan)</p>
                  <Textarea
                    value={formData.working_content.design.procedures}
                    onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
                    placeholder="隢底蝝唳?餈啣??抵岫撽摰孵?瘚?"
                    rows={8}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.3 撖阡??蝑?閰摯 */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.3 撖阡??蝑?閰摯 *</Label>
                    <Select
                      value={formData.working_content.design.pain.category}
                      onValueChange={(val) => updateWorkingContent('design', 'pain.category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder="隢??蝝? /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="B">Category B 蝜???撖?/SelectItem>
                        <SelectItem value="C">Category C ??脰?銝??????餈怎??????拚脰??芷??剜??敺桃??血?蝺翰??雿???銝?雿輻?唳迫???/SelectItem>
                        <SelectItem value="D">Category D ??脰??航?Ｙ??潛???餈怎???嚗??策鈭?嗡?甇Ｙ??獄???桀??乓?/SelectItem>
                        <SelectItem value="E">Category E ??脰??航?Ｙ??潛???餈怎???嚗?銝?蝯虫?甇Ｙ??獄???桀??乓?/SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.4 ?臬?撖阡??憌脤??ㄡ瘞?*/}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.1.4 ?臬?撖阡??憌脤??ㄡ瘞?/Label>
                    <Select
                      value={formData.working_content.design.restrictions.is_restricted === null ? '' : (formData.working_content.design.restrictions.is_restricted === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'restrictions.is_restricted', isYes as boolean | null)
                        // 憒??豢?"??嚗?蝛箇??雿?                        if (!isYes) {
                          updateWorkingContent('design', 'restrictions.restriction_type', undefined)
                          updateWorkingContent('design', 'restrictions.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="隢?? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">??/SelectItem>
                        <SelectItem value="yes">??/SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.restrictions.is_restricted === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>隢???園???*</Label>
                        <Select
                          value={formData.working_content.design.restrictions.restriction_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'restrictions.restriction_type', value)
                            // 憒??豢??????嗡?"嚗?蝛箏隞牧??                            if (value !== 'other') {
                              updateWorkingContent('design', 'restrictions.other_description', undefined)
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="隢?? />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="fasting_before_anesthesia">暻駁???憌?/SelectItem>
                            <SelectItem value="other">?嗡?</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.restrictions.restriction_type === 'other' && (
                        <div className="space-y-2">
                          <Label>隢牧??*</Label>
                          <Textarea
                            value={formData.working_content.design.restrictions.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'restrictions.other_description', e.target.value)}
                            placeholder="隢牧?隞??嗆撘?
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.5 撖阡???蝯?銋?璈?*/}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.1.5 撖阡???蝯?銋?璈?隞亙???箇雿車?啣虜???衣????????甇Ｚ岫撽?/Label>
                  </div>
                  <div className="space-y-2">
                    <Label>撖阡?蝯?嚗?/Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.experimental_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.experimental_endpoint', e.target.value)}
                      placeholder="隢牧?祕撽???????"
                      rows={3}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>鈭粹?蝯?嚗?/Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.humane_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.humane_endpoint', e.target.value)}
                      placeholder="隢牧?犖??暺?
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.6 ?摰?甇餅??蝯?蝵格撘?*/}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.1.6 ?摰?甇餅??蝯?蝵格撘?/Label>
                    <Select
                      value={formData.working_content.design.final_handling.method || ''}
                      onValueChange={(value) => {
                        updateWorkingContent('design', 'final_handling.method', value)
                        // 皜征?嗡??賊??摰?                        if (value !== 'euthanasia') {
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
                        <SelectValue placeholder="隢??蝵格撘? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="euthanasia">摰?甇?/SelectItem>
                        <SelectItem value="transfer">頧?</SelectItem>
                        <SelectItem value="other">?嗡?</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {/* 1. 摰?甇?*/}
                  {formData.working_content.design.final_handling.method === 'euthanasia' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">摰?甇鳴?</Label>
                      <Select
                        value={formData.working_content.design.final_handling.euthanasia_type || ''}
                        onValueChange={(value) => {
                          updateWorkingContent('design', 'final_handling.euthanasia_type', value)
                          // 憒??豢??????嗡?"嚗?蝛箏隞牧??                          if (value !== 'other') {
                            updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
                          }
                        }}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="隢?? />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="kcl">暻駁?銝?Zoletil簧-50 4.4 mg/kg)嚗誑KCl 摰?甇餃??曇????扼D-04-03-00閰阡?鞊祇摰?甇餉?蝭?皞?璆剔?摨?銵?/SelectItem>
                          <SelectItem value="electrocution">暻駁?銝?Zoletil簧-50 4.4 mg/kg)嚗誑220V?餅?敺銵???扼D-04-03-00閰阡?鞊祇摰?甇餉?蝭?皞?璆剔?摨?銵?/SelectItem>
                          <SelectItem value="other">?嗡?嚗?/SelectItem>
                        </SelectContent>
                      </Select>
                      {formData.working_content.design.final_handling.euthanasia_type === 'other' && (
                        <div className="space-y-2 mt-2">
                          <Textarea
                            value={formData.working_content.design.final_handling.euthanasia_other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.euthanasia_other_description', e.target.value)}
                            placeholder="隢牧?隞?璅香?孵?"
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}

                  {/* 2. 頧? */}
                  {formData.working_content.design.final_handling.method === 'transfer' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">頧?</Label>
                      <div className="space-y-3">
                        <div className="space-y-2">
                          <Label className="text-sm">?亙?????</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_name', e.target.value)}
                            placeholder="隢‵撖急????
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">?亙??雿?</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_org}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_org', e.target.value)}
                            placeholder="隢‵撖急?雿?
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">閮?迂嚗?/Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.project_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.project_name', e.target.value)}
                            placeholder="隢‵撖怨??怠?蝔?
                          />
                        </div>
                      </div>
                    </div>
                  )}

                  {/* 3. ?嗡? */}
                  {formData.working_content.design.final_handling.method === 'other' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">?嗡?嚗?/Label>
                      <Textarea
                        value={formData.working_content.design.final_handling.other_description || ''}
                        onChange={(e) => updateWorkingContent('design', 'final_handling.other_description', e.target.value)}
                        placeholder="隢牧?隞?蝵格撘?
                        rows={3}
                      />
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.2 ?撅????寞? */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label className="text-base font-semibold">4.2 ?撅????寞?</Label>
                    <Textarea
                      value={formData.working_content.design.carcass_disposal.method}
                      onChange={(e) => updateWorkingContent('design', 'carcass_disposal.method', e.target.value)}
                      placeholder="隢牧???拙?擃??瘜?
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.3 ?臬雿輻??亦??飛?亙??隞鞈?*/}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.3 ?臬雿輻??亦??飛?亙??隞鞈?/Label>
                    <Select
                      value={formData.working_content.design.non_pharma_grade.used === null ? '' : (formData.working_content.design.non_pharma_grade.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'non_pharma_grade.used', isYes as boolean | null)
                        // 憒??豢?"??嚗?蝛箄牧??雿?                        if (!isYes) {
                          updateWorkingContent('design', 'non_pharma_grade.description', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="隢?? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">??/SelectItem>
                        <SelectItem value="yes">??/SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.non_pharma_grade.used === true && (
                      <div className="space-y-2 mt-2">
                        <Label>隢牧?鞈芣扯釭???冽批?雿輻銋?摮貊???*</Label>
                        <Textarea
                          value={formData.working_content.design.non_pharma_grade.description}
                          onChange={(e) => updateWorkingContent('design', 'non_pharma_grade.description', e.target.value)}
                          placeholder="隢牧?鞈芣扯釭???冽批?雿輻銋?摮貊???
                          rows={4}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.4 ?臬雿輻?勗拿?抒鞈芣???*/}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>4.4 ?臬雿輻?勗拿?抒鞈芣???/Label>
                    <Select
                      value={formData.working_content.design.hazards.used === null ? '' : (formData.working_content.design.hazards.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'hazards.used', isYes as boolean | null)
                        // 憒??豢?"??嚗?蝛箇??雿?                        if (!isYes) {
                          updateWorkingContent('design', 'hazards.materials', [])
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="隢?? />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">??/SelectItem>
                        <SelectItem value="yes">??/SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.hazards.used === true && (
                      <div className="space-y-4 mt-2 pl-6 border-l-2 border-slate-200">
                        <div className="space-y-3">
                          <div className="space-y-2">
                            <Label className="text-sm font-medium">隢?摰單抒鞈芷???</Label>
                            <Select
                              value={formData.working_content.design.hazards.selected_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('design', 'hazards.selected_type', value)
                                // 皜征?嗡?憿??????芯????????
                                const currentMaterials = formData.working_content.design.hazards.materials.filter(m => m.type === value)
                                updateWorkingContent('design', 'hazards.materials', currentMaterials)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder="隢???? />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="biological">1. ??扳???/SelectItem>
                                <SelectItem value="radioactive">2. ?曉???/SelectItem>
                                <SelectItem value="chemical">3. ?梢?批?摮貉??/SelectItem>
                              </SelectContent>
                            </Select>
                          </div>

                          {/* 憿舐內?訾葉憿?????銵?*/}
                          {formData.working_content.design.hazards.selected_type && (
                            <div className="space-y-3">
                              <div className="flex justify-between items-center">
                                <Label className="text-sm font-medium">
                                  {formData.working_content.design.hazards.selected_type === 'biological' && '1. ??扳???}
                                  {formData.working_content.design.hazards.selected_type === 'radioactive' && '2. ?曉???}
                                  {formData.working_content.design.hazards.selected_type === 'chemical' && '3. ?梢?批?摮貉??}
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
                                  ?啣?
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
                                          placeholder="?迂"
                                          value={material.agent_name}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].agent_name = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                        <Input
                                          placeholder="???券?"
                                          value={material.amount}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].amount = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                      </div>
                                      {/* ?抒?銝 */}
                                      <div className="space-y-2">
                                        <Label className="text-sm">?抒?</Label>
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
                                          placeholder="??抒??唳迨????????
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

                {/* ?∩辣?曄內嚗???4.4 銝???嚗蝷?4.5 ??4.6嚗???4.4 銝???嚗蝷?4.5嚗恣?嗉?? */}
                {formData.working_content.design.hazards.used === true && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 ?勗拿?抒鞈芸??嗅誥璉???孵? */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label className="text-base font-semibold">4.5 ?勗拿?抒鞈芸??嗅誥璉???孵?嚗???銝餉?璈?隤銋???隞塚?</Label>
                      </div>

                      {/* 4.5.1 ?賜?寞????蝙?典? */}
                      <div className="space-y-2">
                        <Label>4.5.1 ?賜?寞????蝙?典?</Label>
                        <Textarea
                          value={formData.working_content.design.hazards.operation_location_method}
                          onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                          placeholder="隢牧??冽瘜??蝙?典?"
                          rows={4}
                        />
                      </div>

                      {/* 4.5.2 靽風?芣 */}
                      <div className="space-y-2">
                        <Label>4.5.2 靽風?芣</Label>
                        <p className="text-sm text-muted-foreground mb-2">??閰阡?鈭箏?祕撽??拐誑?ˉ擗憓??∟?銋?霅瑟??</p>
                        <Textarea
                          value={formData.working_content.design.hazards.protection_measures}
                          onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                          placeholder="隢牧??霅瑟??
                          rows={4}
                        />
                      </div>

                      {/* 4.5.3 撖阡?撱Ｘ??抵?撅?銋??撘?*/}
                      <div className="space-y-2">
                        <Label>4.5.3 撖阡?撱Ｘ??抵?撅?銋??撘?/Label>
                        <Textarea
                          value={formData.working_content.design.hazards.waste_and_carcass_disposal}
                          onChange={(e) => updateWorkingContent('design', 'hazards.waste_and_carcass_disposal', e.target.value)}
                          placeholder="隢牧?祕撽誥璉??擃????孵?"
                          rows={4}
                        />
                      </div>
                    </div>

                    <div className="h-px bg-border my-4" />

                    {/* 4.6 ?臬雿輻蝞∪?亙? */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label>4.6 ?臬雿輻蝞∪?亙?</Label>
                        <Select
                          value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                          onValueChange={(value) => {
                            const isYes = value === 'yes'
                            updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                            // 憒??豢?"??嚗?蝛箇??雿?                            if (!isYes) {
                              updateWorkingContent('design', 'controlled_substances.items', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="隢?? />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">??/SelectItem>
                            <SelectItem value="yes">??/SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">蝞∪?亙??”</Label>
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
                              ?啣?
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
                                  <Label className="text-sm">?亙??迂 *</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖怨??蝔?
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">?詨?蝺刻? *</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖急?楊??
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">???券? *</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖急???券?"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">蝞∪?亙?蝞∠?鈭?*</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖怎恣?嗉?恣?犖"
                                  />
                                </div>
                              </div>
                              {/* ?抒?銝 */}
                              <div className="space-y-2">
                                <Label className="text-sm">?抒?</Label>
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
                                  placeholder="??抒??唳迨????????
                                />
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </>
                )}

                {/* 憒? 4.4 銝???嚗蝷?4.5嚗恣?嗉?? */}
                {formData.working_content.design.hazards.used === false && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 ?臬雿輻蝞∪?亙? */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label>4.5 ?臬雿輻蝞∪?亙?</Label>
                        <Select
                          value={formData.working_content.design.controlled_substances.used === null ? '' : (formData.working_content.design.controlled_substances.used === true ? 'yes' : 'no')}
                          onValueChange={(value) => {
                            const isYes = value === 'yes'
                            updateWorkingContent('design', 'controlled_substances.used', isYes as boolean | null)
                            // 憒??豢?"??嚗?蝛箇??雿?                            if (!isYes) {
                              updateWorkingContent('design', 'controlled_substances.items', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="隢?? />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">??/SelectItem>
                            <SelectItem value="yes">??/SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">蝞∪?亙??”</Label>
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
                              ?啣?
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
                                  <Label className="text-sm">?亙??迂 *</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖怨??蝔?
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">?詨?蝺刻? *</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖急?楊??
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">???券? *</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖急???券?"
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">蝞∪?亙?蝞∠?鈭?*</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder="隢‵撖怎恣?嗉?恣?犖"
                                  />
                                </div>
                              </div>
                              {/* ?抒?銝 */}
                              <div className="space-y-2">
                                <Label className="text-sm">?抒?</Label>
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
                                  placeholder="??抒??唳迨????????
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
                <CardTitle>5. ?賊?閬???????br />(Guidelines and References)</CardTitle>
                <CardDescription>憛怠神?祈??怠???瘜???撘??</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>?賊?閬?隤芣?</Label>
                  <Textarea
                    value={formData.working_content.guidelines.content}
                    onChange={(e) => updateWorkingContent('guidelines', 'content', e.target.value)}
                    placeholder="靘?嚗閮?萄儐?靽風瘜?撖阡???扯風?蝙?冽?撘?
                    rows={5}
                  />
                </div>
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">???餃?銵?/h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const newRefs = [...formData.working_content.guidelines.references, { citation: '', url: '' }]
                        updateWorkingContent('guidelines', 'references', newRefs)
                      }}
                    >
                      ?啣??
                    </Button>
                  </div>
                  {formData.working_content.guidelines.references.map((ref, index) => (
                    <div key={index} className="grid w-full gap-2 relative">
                      <div className="flex gap-2 items-start">
                        <div className="grid gap-2 flex-1">
                          <Input
                            placeholder="?撘 (Citation)"
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
                <CardTitle>6. ??閮??br />(Animal Surgical Plan)</CardTitle>
                <CardDescription>憛怠神??蝔桅????????質?銵??扯風</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {(() => {
                  const needsSurgeryPlan = formData.working_content.design.anesthesia.is_under_anesthesia === true &&
                    (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
                     formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery')
                  
                  if (!needsSurgeryPlan) {
                    // 憒?銝?閬‵撖急?銵??嚗＊蝷???
                    return (
                      <div className="space-y-4">
                        <div className="space-y-2">
                          <Label>6.1 ??蝔桅?</Label>
                          <Input value="?? disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.2 銵?皞?</Label>
                          <Textarea value="?? disabled rows={3} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.3 ?∟??芣 (Aseptic Techniques)</Label>
                          <Input value="?? disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.4 ???批捆隤芣?</Label>
                          <Textarea value="?? disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.5 銵葉??</Label>
                          <Textarea value="?? disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.6 摮暑??嚗?隤芣???銵??航撠祕撽??拚?銋蔣??</Label>
                          <Textarea value="?? disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.7 ??臬???甈∩誑銝???嚗???神?箸????:</Label>
                          <Input value="?? disabled />
                        </div>
                        <div className="space-y-2">
                          <Label>6.8 隢牧???抵?敺霅瑕?甇Ｙ?蝯西?寞?:</Label>
                          <div className="space-y-4">
                            <div className="space-y-2">
                              <Label>??憿?</Label>
                              <Input value="?? disabled />
                            </div>
                            <div className="space-y-2">
                              <Label>閰喟敦?批捆</Label>
                              <Textarea value="?? disabled rows={5} />
                            </div>
                          </div>
                        </div>
                        <div className="space-y-2">
                          <Label>6.9 隢牧?祕撽???????(Please specify expected experimental end point):</Label>
                          <Textarea value="?? disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <Label>6.10 ???刻鞈?嚗??獄??撠?迫???嗡??亦鞈?:</Label>
                          <Input value="?? disabled />
                        </div>
                      </div>
                    )
                  }
                  
                  // 憒??閬‵撖急?銵??嚗＊蝷箸迤撣貉”??                  return (
                    <>
                      <div className="space-y-2">
                        <Label>6.1 ??蝔桅? *</Label>
                        <Input
                          value={formData.working_content.surgery.surgery_type === 'survival' ? '摮暑??' : 
                                 formData.working_content.surgery.surgery_type === 'non_survival' ? '??瘣餅?銵? : 
                                 formData.working_content.surgery.surgery_type || ''}
                          disabled
                          className="bg-slate-50"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.2 銵?皞? *</Label>
                        <Textarea
                          value={formData.working_content.surgery.preop_preparation}
                          onChange={(e) => updateWorkingContent('surgery', 'preop_preparation', e.target.value)}
                          placeholder="隤芣?蝳?蝳偌?????柴?瘥?"
                          rows={8}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.3 ?∟??芣 (Aseptic Techniques) *</Label>
                        <div className="space-y-2">
                          {[
                            { value: 'surgical_site_disinfection', label: '?銵瘨?(Surgical site disinfection)' },
                            { value: 'instrument_disinfection', label: '?冽１瘨?(Instrument disinfection)' },
                            { value: 'sterilized_gowns_gloves', label: '?∟???銵????(Sterilizd surgical gowns and gloves)' },
                            { value: 'sterilized_drapes', label: '?∟???閬?Sterilized surgical drapes' },
                            { value: 'surgical_hand_disinfection', label: '銵?urgical hand disinfection' }
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
                        <Label>6.4 ???批捆隤芣? *</Label>
                        <Textarea
                          value={formData.working_content.surgery.surgery_description}
                          onChange={(e) => updateWorkingContent('surgery', 'surgery_description', e.target.value)}
                          placeholder="隢底餈唳?銵?蝔????雿蔭??銵瘜??萄之撠?蝮怠???"
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.5 銵葉?? *</Label>
                        <Textarea
                          value={formData.working_content.surgery.monitoring}
                          onChange={(e) => updateWorkingContent('surgery', 'monitoring', e.target.value)}
                          placeholder="???脰?銝凋?閰阡?鞊祇暻駁?瘛勗漲??賊?????閬?隤踵瘞扳除??瘞????暻駁?瘞??瞈漲嚗??釣??皞恬??乩???????剁???閮?敹歲??詨?擃澈???U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?"
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.6 摮暑??嚗?隤芣???銵??航撠祕撽??拚?銋蔣??</Label>
                        <Textarea
                          value={formData.working_content.surgery.postop_expected_impact}
                          onChange={(e) => updateWorkingContent('surgery', 'postop_expected_impact', e.target.value)}
                          placeholder="隢牧????敺?賢?撖阡????銋蔣??
                          rows={4}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>6.7 ??臬???甈∩誑銝???嚗???神?箸????:</Label>
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
                            <Label htmlFor="multiple_surgeries" className="font-normal cursor-pointer">??/Label>
                          </div>
                          {formData.working_content.surgery.multiple_surgeries.used && (
                            <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                              <div className="space-y-2">
                                <Label>?賊? *</Label>
                                <Input
                                  type="number"
                                  value={formData.working_content.surgery.multiple_surgeries.number || ''}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.number', parseInt(e.target.value) || 0)}
                                  placeholder="隢撓?交?銵活??
                                />
                              </div>
                              <div className="space-y-2">
                                <Label>?? *</Label>
                                <Textarea
                                  value={formData.working_content.surgery.multiple_surgeries.reason}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.reason', e.target.value)}
                                  placeholder="隢牧????
                                  rows={3}
                                />
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>6.8 隢牧???抵?敺霅瑕?甇Ｙ?蝯西?寞?: *</Label>
                        <div className="space-y-4">
                          <div className="space-y-2">
                            <Label>??憿? *</Label>
                            <Select
                              value={formData.working_content.surgery.postop_care_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('surgery', 'postop_care_type', value as 'orthopedic' | 'non_orthopedic')
                                // ?寞??豢??芸?閮剔蔭撠???閮剖摰?                                if (value === 'orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', '1.銵?瘥閰摯鞊祇?亙熒???靘?敺?瘜脰??瑕霅瑞??n\n2.銵?7?亙瘥?脰??潛?閰摯靘惇?餌?瘜蒂靘銝???蝯虫?甇Ｙ??亙???蝝n撉函???\n甇Ｙ??功nketoprofen 1-3 mg/kg IM SID (3憭?\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14憭拇??瑟?蝯虫?)\n??蝝ncefazolin 15 mg/kg IM BID (1-7憭?\ncephalexin 30 mg/kg PO BID (8-14憭拇??瑟?蝯虫?)\n\n3.?亙??拍?撣豢?敶ｇ????賊撣急?蝷箄??n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                                } else if (value === 'non_orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', '1.銵?瘥閰摯鞊祇?亙熒???靘?敺?瘜脰??瑕霅瑞??n\n2.銵?7?亙瘥?脰??潛?閰摯靘惇?餌?瘜蒂靘銝???蝯虫?甇Ｙ??亙???蝝n?爸蝘?銵n甇Ｙ??功nmeloxicam 0.1-0.4 mg/kg IM SID (3憭?\nmeloxicam 0.1-0.4 mg/kg PO SID (4-14憭拇??瑟?蝯虫?)\n??蝝npencillin 10000 u/kg IM SID (1-7憭?\namoxicillin 20 mg/kg PO BID (8-14憭拇??瑟?蝯虫?)\n\n3.?亙??拍?撣豢?敶ｇ????賊撣急?蝷箄??n靘U-03-09-00閰阡?鞊祇憭???璅?雿平蝔??詻脰?')
                                }
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder="隢??銵??? /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="orthopedic">撉函???</SelectItem>
                                <SelectItem value="non_orthopedic">?爸蝘?銵?/SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                          <div className="space-y-2">
                            <Label>閰喟敦?批捆 *</Label>
                            <Textarea
                              value={formData.working_content.surgery.postop_care}
                              onChange={(e) => updateWorkingContent('surgery', 'postop_care', e.target.value)}
                              placeholder="隢牧???抵?敺霅瑕?甇Ｙ?蝯西?寞?"
                              rows={15}
                            />
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>6.9 隢牧?祕撽???????(Please specify expected experimental end point): *</Label>
                        <Textarea
                          value={formData.working_content.surgery.expected_end_point}
                          onChange={(e) => updateWorkingContent('surgery', 'expected_end_point', e.target.value)}
                          placeholder="隢牧?祕撽???????"
                          rows={4}
                        />
                      </div>
                      <div className="space-y-4">
                        <div className="flex justify-between items-center">
                          <Label>6.10 ???刻鞈?嚗??獄??撠?迫???嗡??亦鞈? *</Label>
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
                            + ?啣?
                          </Button>
                        </div>
                        <div className="border rounded-md overflow-hidden">
                          <div className="overflow-x-auto">
                            <table className="w-full border-collapse">
                              <thead>
                                <tr className="bg-slate-100">
                                  <th className="border p-2 text-left text-sm font-semibold">?亙??迂</th>
                                  <th className="border p-2 text-left text-sm font-semibold">??</th>
                                  <th className="border p-2 text-left text-sm font-semibold">????</th>
                                  <th className="border p-2 text-left text-sm font-semibold">?餌?</th>
                                  <th className="border p-2 text-left text-sm font-semibold">蝯西?桃?</th>
                                  <th className="border p-2 text-center text-sm font-semibold w-16">??</th>
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
                                        placeholder="?亙??迂"
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
                                        placeholder="??"
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
                                        placeholder="????"
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
                                        placeholder="?餌?"
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
                                        placeholder="蝯西?桃?"
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
                                      ?怎?刻鞈?嚗?暺??? ?啣??溶??                                    </td>
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
                <CardTitle>7. 撖阡??鞈?<br />(Animal Information)</CardTitle>
                <CardDescription>憛怠神撖阡???拍車??皞?憌潮?鞈?</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">?皜嚗?摨溶?瘥??批銝?嚗?/h3>
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
                          housing_location: '鞊砍?憯怎??批'
                        }]
                        updateWorkingContent('animals', 'animals', newAnimals) // Special case for direct array update if passing null key or just replace 'animals'
                      }}
                    >
                      嚗憓???                    </Button>
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
                          <Label>?拍車 *</Label>
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
                            <SelectTrigger><SelectValue placeholder="隢?蝔? /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="pig">鞊?/SelectItem>
                              <SelectItem value="other">?嗡?</SelectItem>
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
                              placeholder="隢‵撖怎蝔?
                            />
                          )}
                        </div>
                        <div className="space-y-2">
                          <Label>?頂</Label>
                          {animal.species === 'pig' ? (
                            <Select
                              value={animal.strain || ''}
                              onValueChange={(value) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].strain = value as 'white_pig' | 'mini_pig'
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder="隢??蝟? /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="white_pig">?質惇</SelectItem>
                                <SelectItem value="mini_pig">餈瑚?鞊?/SelectItem>
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
                              placeholder="隢‵撖怠?蝟?
                            />
                          ) : (
                            <Input disabled placeholder="隢??豢??拍車" />
                          )}
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>?批 *</Label>
                          <Select
                            value={animal.sex || ''}
                            onValueChange={(value) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              newAnimals[index].sex = value
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                          >
                            <SelectTrigger><SelectValue placeholder="隢?批" /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="male">??/SelectItem>
                              <SelectItem value="female">瘥?/SelectItem>
                              <SelectItem value="unlimited">銝?</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="space-y-2">
                          <Label>?賊? *</Label>
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
                            placeholder="隢撓?交??
                          />
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>撟湧翩嚗?/Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.age_unlimited && (
                              <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                  <Label>?撠僑朣?(?翩)</Label>
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
                                        // 憒??憭扳?朣∪??潛??潭??撠?朣∴??芸?隤踵?憭扳?朣?                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= value) {
                                          newAnimals[index].age_max = value + 1
                                        }
                                      } else if (value < 3 && value > 0) {
                                        newAnimals[index].age_min = 3
                                        // 憒??憭扳?朣∪??潛???嚗?矽?湔?憭扳?朣?                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= 3) {
                                          newAnimals[index].age_max = 4
                                        }
                                      } else {
                                        newAnimals[index].age_min = undefined
                                      }
                                      updateWorkingContent('animals', 'animals', newAnimals)
                                    }}
                                    placeholder="?撠?朣∴??喳?3嚗?
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label>?憭批僑朣?(?翩)</Label>
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
                                    placeholder="?憭扳?朣?
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
                            <Label htmlFor={`age_unlimited_${index}`} className="font-normal cursor-pointer">銝?</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>擃?蝭?嚗?/Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.weight_unlimited && (
                              <div className="space-y-4">
                                <div className="grid grid-cols-2 gap-4">
                                  <div className="space-y-2">
                                    <Label>?撠???(kg)</Label>
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
                                          // 憒??憭折????潛??潭??撠????芸?隤踵?憭折???                                          if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= roundedValue) {
                                            newAnimals[index].weight_max = roundedValue + 5
                                          }
                                        } else if (value < 20 && value > 0) {
                                          newAnimals[index].weight_min = 20
                                          // 憒??憭折????潛???0嚗?矽?湔?憭折???                                          if (newAnimals[index].weight_max !== undefined && newAnimals[index].weight_max <= 20) {
                                            newAnimals[index].weight_max = 25
                                          }
                                        } else {
                                          newAnimals[index].weight_min = undefined
                                        }
                                        updateWorkingContent('animals', 'animals', newAnimals)
                                      }}
                                      placeholder="?撠????喳?20kg嚗?
                                    />
                                    <p className="text-xs text-muted-foreground">瘥??祆銝????/p>
                                  </div>
                                  <div className="space-y-2">
                                    <Label>?憭折???(kg)</Label>
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
                                          // 蝣箔??憭折??之?潭?撠???                                          if (newAnimals[index].weight_max <= minWeight) {
                                            newAnimals[index].weight_max = minWeight + 5
                                          }
                                        } else if (value <= minWeight && value > 0) {
                                          newAnimals[index].weight_max = minWeight + 5
                                        } else {
                                          newAnimals[index].weight_max = undefined
                                        }
                                        updateWorkingContent('animals', 'animals', newAnimals)
                                      }}
                                      placeholder="?憭折???敹?憭扳?撠???"
                                    />
                                  </div>
                                </div>
                                {animal.weight_min !== undefined && animal.weight_max !== undefined && (
                                  <div className="space-y-4">
                                    <div className="space-y-2">
                                      <Label>隤踵???- ?撠???/Label>
                                      <Slider
                                        min={20}
                                        max={Math.max(200, (animal.weight_max || 0) + 50)}
                                        step={5}
                                        value={animal.weight_min ?? 20}
                                        onChange={(value: number) => {
                                          const newAnimals = [...formData.working_content.animals.animals]
                                          const minValue = Math.max(20, Math.round(value / 5) * 5)
                                          newAnimals[index].weight_min = minValue
                                          // 蝣箔??撠潔?憭扳?憭批潘?銝?憭折????之?潭?撠???                                          if (minValue >= (animal.weight_max || 0)) {
                                            newAnimals[index].weight_max = minValue + 5
                                          }
                                          updateWorkingContent('animals', 'animals', newAnimals)
                                        }}
                                        className="w-full"
                                      />
                                      <p className="text-xs text-muted-foreground text-center">{animal.weight_min || 20} kg</p>
                                    </div>
                                    <div className="space-y-2">
                                      <Label>隤踵???- ?憭折???/Label>
                                      <Slider
                                        min={Math.max(25, (animal.weight_min || 20) + 5)}
                                        max={200}
                                        step={5}
                                        value={animal.weight_max ?? 25}
                                        onChange={(value: number) => {
                                          const newAnimals = [...formData.working_content.animals.animals]
                                          const maxValue = Math.round(value / 5) * 5
                                          const minWeight = newAnimals[index].weight_min || 20
                                          // 蝣箔??憭批澆之?潭?撠?                                          if (maxValue > minWeight) {
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
                            <Label htmlFor={`weight_unlimited_${index}`} className="font-normal cursor-pointer">銝?</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>憌潮??啣?嚗惇?ㄚ???/Label>
                      </div>
                    </div>
                  ))}
                  <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md">
                    <h4 className="font-semibold text-sm mb-2">?酉嚗?/h4>
                    <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
                      <li>?餉惇/?質惇嚗?皞??蝜???鞊砍?憯怎??批)嚗惇/餈瑚?鞊穿?靘?嚗??抒?畾(鞊砍?憯怎??批)</li>
                      <li>?餅批嚗??????/li>
                      <li>?餃僑朣∴?憭抒??翩蝭?(隞乩?)????/li>
                      <li>?駁???憭抒?擃?蝭?????/li>
                      <li>?餌鞊祆??瑕翰??7??朣∪??00?祆嚗?撖?頞?3??撱箄降雿輻餈瑚?鞊祇脰?閰阡???/li>
                    </ul>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'personnel' && (
            <Card>
              <CardHeader>
                <CardTitle>8. 閰阡?鈭箏鞈?<br />(Personnel Working on Animal Study)</CardTitle>
                <CardDescription>???祈??思?閰阡?鈭箏皜????/CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">8.1 鞎痊?脰??閰阡?銋?犖?∟???/h3>
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
                      + ?啣?
                    </Button>
                  </div>
                  <div className="border rounded-md overflow-hidden">
                    <div className="overflow-x-auto">
                      <table className="w-full border-collapse">
                        <thead>
                          <tr className="bg-slate-100">
                            <th className="border p-2 text-left text-sm font-semibold">蝺刻?</th>
                            <th className="border p-2 text-left text-sm font-semibold">憪?</th>
                            <th className="border p-2 text-left text-sm font-semibold">?瑞迂</th>
                            <th className="border p-2 text-left text-sm font-semibold">撌乩??批捆</th>
                            <th className="border p-2 text-left text-sm font-semibold">???閰阡?撟湔</th>
                            <th className="border p-2 text-left text-sm font-semibold">閮毀/鞈/????</th>
                            <th className="border p-2 text-center text-sm font-semibold w-16">??</th>
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
                                  placeholder="憪?"
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
                                  placeholder="?瑞迂"
                                  className="border-0 focus-visible:ring-0"
                                />
                              </td>
                              <td className="border p-2">
                                <div className="space-y-2 min-w-[300px]">
                                  <div className="flex flex-wrap gap-2">
                                    {[
                                      { value: 'a', label: 'a.閮???(Supervision)' },
                                      { value: 'b', label: 'b.憌潮??折“(Animal care)' },
                                      { value: 'c', label: 'c.靽?(Restraint)' },
                                      { value: 'd', label: 'd.暻駁?甇Ｙ?(Anesthesia and analgesia)' },
                                      { value: 'e', label: 'e.??(Surgery)' },
                                      { value: 'f', label: 'f.???舀(Surgery assistance)' },
                                      { value: 'g', label: 'g.閫撖皜?Monitoring)' },
                                      { value: 'h', label: 'h.摰?甇?Euthanasia)' },
                                      { value: 'i', label: 'i.?嗡?(Other)' }
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
                                      placeholder="隢牧?隞極雿摰?*"
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
                                  placeholder="撟湔"
                                  className="border-0 focus-visible:ring-0 w-20"
                                />
                              </td>
                              <td className="border p-2">
                                <div className="space-y-2 min-w-[400px]">
                                  <div className="flex flex-wrap gap-2 mb-2">
                                    {[
                                      { value: 'A', label: 'A. 撖阡???扯風?蝙?典??⊥???蝯??∟?蝺渡' },
                                      { value: 'B', label: 'B. IACUC?閮毀???? },
                                      { value: 'C', label: 'C. 頛餃?摰閮毀?? },
                                      { value: 'D', label: 'D. ??Ｘ平?函?蝳賣??函?蝧???銵?蝧?' },
                                      { value: 'E', label: 'E. 撖阡??瘜??霅瑞恣?' }
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
                                              // 蝘駁閰脰?蝺渡??????                                              newPersonnel[index].training_certificates = (newPersonnel[index].training_certificates || []).filter(
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
                                  {/* 憿舐內瘥歇?貉?蝺渡?霅蝺刻??” */}
                                  {(person.trainings || []).map((trainingCode: string) => {
                                    const certificates = (person.training_certificates || []).filter((cert: any) => cert.training_code === trainingCode)
                                    return (
                                      <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                                        <Label className="text-xs font-semibold">{trainingCode}:</Label>
                                        {certificates.map((cert: any, certIndex: number) => {
                                          // ?曉閰脰??詨摰 training_certificates ?貊?銝剔?蝝Ｗ?
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
                                                placeholder="霅蝺刻?"
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
                                          + ?啣?霅
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
                                ?怎鈭箏鞈?嚗?暺??? ?啣??溶??                              </td>
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
                <CardTitle>9. ?辣<br />(Attachments)</CardTitle>
                <CardDescription>銝?賊??辣??隞?/CardDescription>
              </CardHeader>
              <CardContent>
                <div className="text-center py-8 text-muted-foreground">
                  <Paperclip className="h-12 w-12 mx-auto mb-2" />
                  <p>?辣銝?撠?</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
