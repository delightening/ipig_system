import { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
  ProtocolResponse,
  CreateProtocolRequest,
  UpdateProtocolRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { DatePicker } from '@/components/ui/date-picker'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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

const sectionKeys = [
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

import { ProtocolFormData } from '@/types/protocol'

type FormData = ProtocolFormData

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
    guidelines: { content: '', references: [] },
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

export function ProtocolEditPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user } = useAuthStore()
  const isNew = !id

  const [activeSection, setActiveSection] = useState('basic')
  const [formData, setFormData] = useState<FormData>(defaultFormData)
  const [isAddPersonnelDialogOpen, setIsAddPersonnelDialogOpen] = useState(false)
  const [newPersonnel, setNewPersonnel] = useState({
    name: '',
    position: '',
    roles: [] as string[],
    roles_other_text: '',
    years_experience: 0,
    trainings: [] as string[],
    trainings_other_text: '', // 如果選擇 F.其他，需要填寫說明
    training_certificates: [] as Array<{ training_code: string; certificate_no: string }>,
  })
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

  const { data: staffMembers = [] } = useQuery({
    queryKey: ['staff'],
    queryFn: async () => {
      const response = await api.get<{
        id: string
        display_name: string
        email: string
        phone?: string
        organization?: string
        entry_date?: string
        position?: string
        aup_roles?: string[]
        years_experience?: number
        trainings?: { code: string; certificate_no?: string; received_date?: string }[]
      }[]>('/hr/staff')
      return response.data
    },
  })

  // 新建計畫書時設置預設值
  useEffect(() => {
    if (isNew) {
      setFormData((prev) => {
        const updated = { ...prev }
        // 設置機構名稱預設值
        if (!updated.working_content.basic.facility?.title || !updated.working_content.basic.facility.title.trim()) {
          updated.working_content.basic.facility = {
            ...updated.working_content.basic.facility,
            title: t('aup.defaults.facilityName')
          }
        }
        // 設置飼養地點預設值
        if (!updated.working_content.basic.housing_location || !updated.working_content.basic.housing_location.trim()) {
          updated.working_content.basic.housing_location = t('aup.defaults.housingLocation')
        }
        // 設置人道終點預設值
        if (!updated.working_content.design.endpoints.humane_endpoint || !updated.working_content.design.endpoints.humane_endpoint.trim()) {
          updated.working_content.design.endpoints.humane_endpoint = t('aup.defaults.humaneEndpoint')
        }
        // 設置動物屍體處理方法預設值
        if (!updated.working_content.design.carcass_disposal.method || !updated.working_content.design.carcass_disposal.method.trim()) {
          updated.working_content.design.carcass_disposal.method = t('aup.defaults.carcassDisposal')
        }
        return updated
      })
    }
  }, [isNew, t])

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
              title: t('aup.defaults.facilityName')
            }
          }
          if (!mergedWorkingContent.basic.housing_location || !mergedWorkingContent.basic.housing_location.trim()) {
            mergedWorkingContent.basic.housing_location = t('aup.defaults.housingLocation')
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
            mergedWorkingContent.design.endpoints.humane_endpoint = t('aup.defaults.humaneEndpoint')
          }
        }

        // 確保動物屍體處理方法有預設內容
        if (mergedWorkingContent.design && mergedWorkingContent.design.carcass_disposal) {
          if (!mergedWorkingContent.design.carcass_disposal.method || !mergedWorkingContent.design.carcass_disposal.method.trim()) {
            mergedWorkingContent.design.carcass_disposal.method = t('aup.defaults.carcassDisposal')
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

        // 確保 attachments 格式正確，轉換為 FileInfo 格式
        if (mergedWorkingContent.attachments) {
          mergedWorkingContent.attachments = (mergedWorkingContent.attachments as any[]).map((att: any) => {
            // 如果已經是 FileInfo 格式（有 id, file_name, file_path 等），直接返回
            if (att.id && att.file_name && att.file_path !== undefined) {
              return {
                id: att.id,
                file_name: att.file_name,
                file_path: att.file_path,
                file_size: att.file_size || 0,
                file_type: att.file_type || att.mime_type || 'application/pdf',
                created_at: att.created_at
              } as FileInfo
            }
            // 如果是舊格式（name, type），轉換為 FileInfo 格式
            if (att.name && att.type) {
              return {
                id: `legacy-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
                file_name: att.name,
                file_path: '',
                file_size: 0,
                file_type: att.type
              } as FileInfo
            }
            return att
          }).filter(Boolean)
        } else {
          mergedWorkingContent.attachments = []
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
              mergedWorkingContent.surgery.preop_preparation = t('aup.defaults.preopPreparation')
            }
            // 如果手術內容說明為空，設置預設內容
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = t('aup.defaults.surgeryDescription')
            }
            // 如果術中監控為空，設置預設內容
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = t('aup.defaults.monitoring')
            }
            // 如果術後照護類型未選擇，不自動設置（讓用戶選擇）
          } else {
            // 如果不需要填寫手術計劃書，自動填寫"略"
            if (!mergedWorkingContent.surgery.surgery_type || mergedWorkingContent.surgery.surgery_type.trim() === '') {
              mergedWorkingContent.surgery.surgery_type = t('aup.defaults.omitted')
            }
            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = t('aup.defaults.omitted')
            }
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = t('aup.defaults.omitted')
            }
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = t('aup.defaults.omitted')
            }
            if (!mergedWorkingContent.surgery.postop_expected_impact || mergedWorkingContent.surgery.postop_expected_impact.trim() === '') {
              mergedWorkingContent.surgery.postop_expected_impact = t('aup.defaults.omitted')
            }
            if (!mergedWorkingContent.surgery.postop_care || mergedWorkingContent.surgery.postop_care.trim() === '') {
              mergedWorkingContent.surgery.postop_care = t('aup.defaults.postopCareTemplate')
            }
            if (!mergedWorkingContent.surgery.expected_end_point || mergedWorkingContent.surgery.expected_end_point.trim() === '') {
              mergedWorkingContent.surgery.expected_end_point = t('aup.defaults.omitted')
            }
            // 如果不需要填寫手術計劃書，清空用藥資訊
            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = []
            }
            // 如果手術用藥資訊為空，設置預設內容
            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              const drugDefaults = t('aup.defaults.drugDefaults', { returnObjects: true }) as any[]
              mergedWorkingContent.surgery.drugs = drugDefaults.map(d => ({
                drug_name: d.name,
                dose: d.dose,
                route: d.route,
                frequency: d.frequency,
                purpose: d.purpose
              }))
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
        title: t('common.success'),
        description: t('aup.messages.created'),
      })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
      navigate(`/protocols/${response.data.id}`)
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('aup.messages.createFailed'),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async (data: UpdateProtocolRequest) => api.put(`/protocols/${id}`, data),
    onSuccess: () => {
      toast({
        title: t('common.success'),
        description: t('aup.messages.saved'),
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('aup.messages.saveFailed'),
        variant: 'destructive',
      })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/submit`),
    onSuccess: () => {
      toast({
        title: t('common.success'),
        description: t('aup.messages.submitted'),
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      navigate(`/protocols/${id}`)
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('aup.messages.submitFailed'),
        variant: 'destructive',
      })
    },
  })

  // 驗證必填字段（Section 1 - 研究資料）
  const validateRequiredFields = (): string | null => {
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
        return t('aup.surgery.validation.preopPreparationRequired')
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

  const handleSave = async (isSubmit = false) => {
    // Validate required fields
    // When saving as draft, only study title is required; when submitting, all fields must be validated
    const validationError = isSubmit
      ? validateRequiredFields()
      : (!formData.title.trim() ? t('aup.basic.validation.titleRequired') : null)

    if (validationError) {
      toast({
        title: t('common.error'),
        description: validationError,
        variant: 'destructive',
      })
      return false
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
      return true
    } catch (error) {
      // 錯誤已在 mutation 的 onError 中處理
      return false
    } finally {
      setIsSaving(false)
    }
  }

  const handleSubmit = async () => {
    if (!id) return
    const isSaved = await handleSave(true)
    if (!isSaved) return

    if (confirm(t('aup.messages.confirmSubmit'))) {
      submitMutation.mutate()
    }
  }

  const updateWorkingContent = (section: keyof FormData['working_content'], path: string, value: any) => {
    setFormData((prev) => {
      const newContent = { ...prev.working_content }
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
              {isNew ? t('aup.newProtocol') : t('aup.editProtocol')}
            </h1>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => handleSave()} disabled={isSaving}>
            {isSaving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            {t('aup.saveDraft')}
          </Button>
          {!isNew && (
            <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              {t('aup.submitForReview')}
            </Button>
          )}
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[280px_1fr]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-lg">{t('aup.sections')}</CardTitle>
          </CardHeader>
          <CardContent className="p-2">
            <nav className="space-y-1">
              {sectionKeys.map((section) => (
                <button
                  key={section.key}
                  onClick={() => setActiveSection(section.key)}
                  className={`w-full flex items-start gap-3 px-3 py-2 rounded-lg text-left transition-colors ${activeSection === section.key
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-slate-600 hover:bg-slate-100'
                    }`}
                >
                  <section.icon className="h-4 w-4 mt-0.5 flex-shrink-0" />
                  <span className="text-sm font-medium">{t(section.labelKey)}</span>
                </button>
              ))}
            </nav>
          </CardContent>
        </Card>

        <div className="space-y-6">
          {activeSection === 'basic' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('aup.section1')}</CardTitle>
                <CardDescription>{t('aup.basic.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 1. GLP & Title */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label htmlFor="title">{t('aup.basic.studyTitle')} *</Label>
                    <Input
                      id="title"
                      value={formData.title}
                      onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
                      placeholder={t('aup.basic.studyTitlePlaceholder')}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.glpAttribute')} *</Label>
                    <div className="flex items-center space-x-2 pt-2">
                      <Checkbox
                        id="is_glp"
                        checked={formData.working_content.basic.is_glp}
                        onCheckedChange={(checked) => updateWorkingContent('basic', 'is_glp', checked)}
                      />
                      <Label htmlFor="is_glp">{t('aup.basic.glpCompliant')}</Label>
                    </div>
                  </div>
                </div>

                {/* 2. IDs and Dates */}
                <div className={`grid gap-4 ${isNew || !isIACUCStaff ? 'md:grid-cols-1' : 'md:grid-cols-2'}`}>
                  {/* Study No: hidden on new page, only editable by IACUC staff on edit page */}
                  {(!isNew && isIACUCStaff) && (
                    <div className="space-y-2">
                      <Label htmlFor="apply_study_number">{t('aup.basic.studyNo')}</Label>
                      <Input
                        id="apply_study_number"
                        value={formData.working_content.basic.apply_study_number || ''}
                        onChange={(e) => updateWorkingContent('basic', 'apply_study_number', e.target.value)}
                        placeholder={t('aup.basic.studyNoPlaceholder')}
                      />
                    </div>
                  )}
                  <div className="space-y-2">
                    <Label>{t('aup.basic.expectedPeriod')} *</Label>
                    <div className="flex gap-2 items-center">
                      <DatePicker
                        value={formData.start_date}
                        onChange={(value) => setFormData(prev => ({ ...prev, start_date: value }))}
                        required
                      />
                      <span className="self-center">{t('aup.basic.to')}</span>
                      <DatePicker
                        value={formData.end_date}
                        onChange={(value) => setFormData(prev => ({ ...prev, end_date: value }))}
                        required
                      />
                    </div>
                  </div>
                </div>

                {/* 3. Types */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>{t('aup.basic.projectType')} *</Label>
                    <Select
                      value={formData.working_content.basic.project_type}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_type', val)}
                    >
                      <SelectTrigger><SelectValue placeholder={t('aup.basic.selectProjectType')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="1_basic_research">{t('aup.projectTypes.1_basic_research')}</SelectItem>
                        <SelectItem value="2_applied_research">{t('aup.projectTypes.2_applied_research')}</SelectItem>
                        <SelectItem value="3_pre_market_testing">{t('aup.projectTypes.3_pre_market_testing')}</SelectItem>
                        <SelectItem value="4_educational">{t('aup.projectTypes.4_educational')}</SelectItem>
                        <SelectItem value="5_biologics_manufacturing">{t('aup.projectTypes.5_biologics_manufacturing')}</SelectItem>
                        <SelectItem value="6_other">{t('aup.projectTypes.6_other')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.basic.project_type === '6_other' && (
                      <div className="pt-2">
                        <Input
                          placeholder={t('aup.basic.specifyOther')}
                          value={formData.working_content.basic.project_type_other || ''}
                          onChange={(e) => updateWorkingContent('basic', 'project_type_other', e.target.value)}
                        />
                      </div>
                    )}
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.basic.projectCategory')} *</Label>
                    <Select
                      value={formData.working_content.basic.project_category}
                      onValueChange={(val) => updateWorkingContent('basic', 'project_category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder={t('aup.basic.selectProjectCategory')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="1_medical">{t('aup.projectCategories.1_medical')}</SelectItem>
                        <SelectItem value="2_agricultural">{t('aup.projectCategories.2_agricultural')}</SelectItem>
                        <SelectItem value="3_drugs_vaccines">{t('aup.projectCategories.3_drugs_vaccines')}</SelectItem>
                        <SelectItem value="4_supplements">{t('aup.projectCategories.4_supplements')}</SelectItem>
                        <SelectItem value="5_food">{t('aup.projectCategories.5_food')}</SelectItem>
                        <SelectItem value="6_toxics_chemicals">{t('aup.projectCategories.6_toxics_chemicals')}</SelectItem>
                        <SelectItem value="7_medical_materials">{t('aup.projectCategories.7_medical_materials')}</SelectItem>
                        <SelectItem value="8_pesticide">{t('aup.projectCategories.8_pesticide')}</SelectItem>
                        <SelectItem value="9_animal_drugs_vaccines">{t('aup.projectCategories.9_animal_drugs_vaccines')}</SelectItem>
                        <SelectItem value="10_animal_supplements_feed">{t('aup.projectCategories.10_animal_supplements_feed')}</SelectItem>
                        <SelectItem value="11_cosmetics">{t('aup.projectCategories.11_cosmetics')}</SelectItem>
                        <SelectItem value="12_other">{t('aup.projectCategories.12_other')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.basic.project_category === '12_other' && (
                      <div className="pt-2">
                        <Input
                          placeholder={t('aup.basic.specifyOther')}
                          value={formData.working_content.basic.project_category_other || ''}
                          onChange={(e) => updateWorkingContent('basic', 'project_category_other', e.target.value)}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 資金來源 */}
                <div className="space-y-4">
                  <Label className="text-base font-semibold">{t('aup.basic.fundingSources')} (複選)</Label>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 pl-4 text-sm">
                    {['moa', 'mohw', 'nstc', 'moe', 'env', 'other'].map((option) => (
                      <div key={option} className="flex items-center space-x-3">
                        <Checkbox
                          id={`funding_${option}`}
                          checked={formData.working_content.basic.funding_sources.includes(option)}
                          onCheckedChange={(checked) => {
                            const current = formData.working_content.basic.funding_sources || []
                            let updated: string[]
                            if (checked) {
                              updated = [...current, option]
                            } else {
                              updated = current.filter(s => s !== option)
                            }
                            updateWorkingContent('basic', 'funding_sources', updated)
                          }}
                        />
                        <Label htmlFor={`funding_${option}`} className="font-normal cursor-pointer">
                          {t(`aup.basic.fundingSourceOptions.${option}`)}
                        </Label>
                      </div>
                    ))}
                  </div>
                  {formData.working_content.basic.funding_sources.includes('other') && (
                    <div className="pl-4 pt-2 lg:w-1/2">
                      <Input
                        placeholder={t('aup.basic.specifyOther')}
                        value={formData.working_content.basic.funding_other || ''}
                        onChange={(e) => updateWorkingContent('basic', 'funding_other', e.target.value)}
                      />
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4. PI Info */}
                <div className="space-y-4">
                  <h3 className="font-semibold">{t('aup.basic.pi')}</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>{t('aup.basic.name')} *</Label>
                      <Input
                        value={formData.working_content.basic.pi.name}
                        onChange={(e) => updateWorkingContent('basic', 'pi.name', e.target.value)}
                        placeholder={t('aup.basic.piNamePlaceholder')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.email')} *</Label>
                      <Input
                        value={formData.working_content.basic.pi.email}
                        onChange={(e) => updateWorkingContent('basic', 'pi.email', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.phone')} *</Label>
                      <Input
                        value={formData.working_content.basic.pi.phone}
                        onChange={(e) => updateWorkingContent('basic', 'pi.phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.address')} *</Label>
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
                  <h3 className="font-semibold">{t('aup.basic.sponsor')}</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>{t('aup.basic.organizationName')} *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.name}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.name', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.contactPerson')} *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_person}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_person', e.target.value)}
                        placeholder={t('aup.basic.contactPersonPlaceholder')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.contactPhone')} *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_phone}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_phone', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.contactEmail')} *</Label>
                      <Input
                        value={formData.working_content.basic.sponsor.contact_email}
                        onChange={(e) => updateWorkingContent('basic', 'sponsor.contact_email', e.target.value)}
                      />
                    </div>
                  </div>
                </div>

                {/* 6. Facility */}
                <div className="space-y-4">
                  <h3 className="font-semibold">{t('aup.basic.facilityAndLocation')}</h3>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div className="space-y-2">
                      <Label>{t('aup.basic.facilityName')} *</Label>
                      <Input
                        value={formData.working_content.basic.facility.title}
                        onChange={(e) => updateWorkingContent('basic', 'facility.title', e.target.value)}
                        disabled={!useAuthStore.getState().hasRole('admin') && !useAuthStore.getState().hasRole('IACUC_STAFF')}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>{t('aup.basic.location')} *</Label>
                      <Input
                        value={formData.working_content.basic.housing_location}
                        onChange={(e) => updateWorkingContent('basic', 'housing_location', e.target.value)}
                        disabled={!useAuthStore.getState().hasRole('admin') && !useAuthStore.getState().hasRole('IACUC_STAFF')}
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
                <CardTitle>{t('aup.section2')}</CardTitle>
                <CardDescription>{t('aup.purpose.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 2.1 Purpose and Significance */}
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.purpose.significance')}</h3>
                  <Textarea
                    value={formData.working_content.purpose.significance}
                    onChange={(e) => updateWorkingContent('purpose', 'significance', e.target.value)}
                    placeholder={t('aup.purpose.significancePlaceholder')}
                    rows={5}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.2 Replacement Principle */}
                <div className="space-y-4">
                  <h3 className="font-semibold">{t('aup.purpose.replacementPrinciple')}</h3>

                  {/* 2.2.1 Live Animal Necessity */}
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.liveAnimalNecessity')} *</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.rationale}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.rationale', e.target.value)}
                      placeholder={t('aup.purpose.liveAnimalNecessityPlaceholder')}
                      rows={4}
                    />
                  </div>

                  {/* 2.2.2 Alternative Methods Search */}
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.altSearchLabel')} *</Label>
                    <div className="space-y-4 pl-4">
                      <div className="flex items-start space-x-3 py-2">
                        <Checkbox
                          id="search_altbib"
                          checked={formData.working_content.purpose.replacement.alt_search.platforms.includes('altbib')}
                          onCheckedChange={(checked) => {
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 'altbib']
                              : current.filter(p => p !== 'altbib')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_altbib" className="font-normal leading-relaxed flex-1">
                          {t('aup.purpose.altbibLabel')}<br />
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
                          onCheckedChange={(checked) => {
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 'db_alm']
                              : current.filter(p => p !== 'db_alm')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_db_alm" className="font-normal leading-relaxed flex-1">
                          {t('aup.purpose.dbAlmLabel')}<br />
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
                          onCheckedChange={(checked) => {
                            const current = formData.working_content.purpose.replacement.alt_search.platforms
                            const updated = checked
                              ? [...current, 're_place']
                              : current.filter(p => p !== 're_place')
                            updateWorkingContent('purpose', 'replacement.alt_search.platforms', updated)
                          }}
                          className="mt-1"
                        />
                        <Label htmlFor="search_re_place" className="font-normal leading-relaxed flex-1">
                          {t('aup.purpose.rePlaceLabel')}<br />
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
                        placeholder={t('aup.purpose.otherDbPlaceholder')}
                        value={formData.working_content.purpose.replacement.alt_search.other_name || ''}
                        onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.other_name', e.target.value)}
                        className="mt-2"
                      />
                    )}
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.searchKeywords')} *</Label>
                    <Input
                      value={formData.working_content.purpose.replacement.alt_search.keywords}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.keywords', e.target.value)}
                      placeholder={t('aup.purpose.searchKeywordsPlaceholder')}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.searchConclusion')} *</Label>
                    <Textarea
                      value={formData.working_content.purpose.replacement.alt_search.conclusion}
                      onChange={(e) => updateWorkingContent('purpose', 'replacement.alt_search.conclusion', e.target.value)}
                      placeholder={t('aup.purpose.searchConclusionPlaceholder')}
                      rows={3}
                    />
                  </div>

                  {/* 2.2.3 Duplicate Experiment */}
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.duplicateExperiment')}</Label>
                    <Select
                      value={formData.working_content.purpose.duplicate.experiment ? 'yes' : 'no'}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('purpose', 'duplicate.experiment', isYes)
                        if (!isYes) {
                          updateWorkingContent('purpose', 'duplicate.justification', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.purpose.duplicate.experiment && (
                      <div className="space-y-2 mt-2">
                        <Label>{t('aup.purpose.duplicateJustification')} *</Label>
                        <Textarea
                          value={formData.working_content.purpose.duplicate.justification}
                          onChange={(e) => updateWorkingContent('purpose', 'duplicate.justification', e.target.value)}
                          placeholder={t('aup.purpose.duplicateJustificationPlaceholder')}
                          rows={3}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 2.3 Reduction Principle */}
                <div className="space-y-4">
                  <h3 className="font-semibold">{t('aup.purpose.reductionPrinciple')}</h3>
                  <div className="space-y-2">
                    <Label>{t('aup.purpose.reductionDesign')} *</Label>
                    <Textarea
                      value={formData.working_content.purpose.reduction.design}
                      onChange={(e) => updateWorkingContent('purpose', 'reduction.design', e.target.value)}
                      placeholder={t('aup.purpose.reductionDesignPlaceholder')}
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
                <CardTitle>{t('aup.section3')}</CardTitle>
                <CardDescription>{t('aup.items.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>{t('aup.items.useTestItemLabel')} *</Label>
                  <Select
                    value={formData.working_content.items.use_test_item === null ? '' : (formData.working_content.items.use_test_item ? 'yes' : 'no')}
                    onValueChange={(value) => {
                      const isYes = value === 'yes'
                      updateWorkingContent('items', 'use_test_item', isYes)
                      // If "No" is selected, clear the substance list
                      if (!isYes) {
                        updateWorkingContent('items', 'test_items', [])
                        updateWorkingContent('items', 'control_items', [])
                      }
                    }}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('common.pleaseSelect')} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="no">{t('common.no')}</SelectItem>
                      <SelectItem value="yes">{t('common.yes')}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {formData.working_content.items.use_test_item === false && (
                  <p className="text-muted-foreground italic">{t('aup.items.skipped')}</p>
                )}

                {formData.working_content.items.use_test_item === true && (
                  <>
                    {/* Test Items */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">{t('aup.items.testItems')}</h3>
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
                          {t('aup.items.add')}
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
                              <Label>{t('aup.items.itemName')} *</Label>
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
                              <Label>{t('aup.items.dosageForm')} *</Label>
                              <Input
                                value={item.form || ''}
                                onChange={(e) => {
                                  const newItems = [...formData.working_content.items.test_items]
                                  newItems[index].form = e.target.value
                                  updateWorkingContent('items', 'test_items', newItems)
                                }}
                                placeholder={t('aup.items.dosageFormPlaceholder')}
                              />
                            </div>
                          </div>
                          <div className="space-y-2">
                            <Label>{t('aup.items.purpose')} *</Label>
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
                            <Label>{t('aup.items.storageConditions')} *</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newItems = [...formData.working_content.items.test_items]
                                newItems[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                              placeholder={t('aup.items.storageConditionsPlaceholder')}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>{t('aup.items.isSterile')} *</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newItems = [...formData.working_content.items.test_items]
                                const isYes = value === 'yes'
                                newItems[index].is_sterile = isYes
                                // If "Yes" is selected, clear the explanation field
                                if (isYes) {
                                  newItems[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'test_items', newItems)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder={t('common.pleaseSelect')} />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">{t('common.no')}</SelectItem>
                                <SelectItem value="yes">{t('common.yes')}</SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>{t('aup.items.nonSterileJustification')} *</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newItems = [...formData.working_content.items.test_items]
                                    newItems[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'test_items', newItems)
                                  }}
                                  placeholder={t('aup.items.nonSterilePlaceholder')}
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* Photos Upload */}
                          <div className="space-y-2">
                            <Label>{t('aup.items.photos')}</Label>
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
                              placeholder={t('aup.items.photosPlaceholder')}
                            />
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* Control Items */}
                    <div className="space-y-4 border p-4 rounded-md">
                      <div className="flex justify-between items-center">
                        <h3 className="font-semibold">{t('aup.items.controlItems')}</h3>
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
                          {t('aup.items.add')}
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
                              <Label>{t('aup.items.controlName')} *</Label>
                              <Input
                                value={item.name}
                                onChange={(e) => {
                                  const newControls = [...formData.working_content.items.control_items]
                                  newControls[index].name = e.target.value
                                  updateWorkingContent('items', 'control_items', newControls)
                                }}
                                placeholder={t('aup.items.controlNamePlaceholder')}
                              />
                            </div>
                            <div className="space-y-2">
                              <Label>{t('aup.items.purpose')} *</Label>
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
                            <Label>{t('aup.items.storageConditions')} *</Label>
                            <Input
                              value={item.storage_conditions || ''}
                              onChange={(e) => {
                                const newControls = [...formData.working_content.items.control_items]
                                newControls[index].storage_conditions = e.target.value
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                              placeholder={t('aup.items.storageConditionsPlaceholder')}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label>{t('aup.items.isSterile')} *</Label>
                            <Select
                              value={item.is_sterile ? 'yes' : 'no'}
                              onValueChange={(value) => {
                                const newControls = [...formData.working_content.items.control_items]
                                const isYes = value === 'yes'
                                newControls[index].is_sterile = isYes
                                // If "Yes" is selected, clear the explanation field
                                if (isYes) {
                                  newControls[index].non_sterile_justification = ''
                                }
                                updateWorkingContent('items', 'control_items', newControls)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder={t('common.pleaseSelect')} />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="no">{t('common.no')}</SelectItem>
                                <SelectItem value="yes">{t('common.yes')}</SelectItem>
                              </SelectContent>
                            </Select>
                            {!item.is_sterile && (
                              <div className="space-y-2 mt-2">
                                <Label>{t('aup.items.nonSterileJustification')} *</Label>
                                <Textarea
                                  value={item.non_sterile_justification || ''}
                                  onChange={(e) => {
                                    const newControls = [...formData.working_content.items.control_items]
                                    newControls[index].non_sterile_justification = e.target.value
                                    updateWorkingContent('items', 'control_items', newControls)
                                  }}
                                  placeholder={t('aup.items.nonSterilePlaceholder')}
                                  rows={3}
                                />
                              </div>
                            )}
                          </div>
                          {/* Photos Upload */}
                          <div className="space-y-2">
                            <Label>{t('aup.items.photos')}</Label>
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
                              placeholder={t('aup.items.photosPlaceholder')}
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
                <CardTitle>{t('aup.section4')}</CardTitle>
                <CardDescription>{t('aup.design.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 4.1 Title */}
                <div className="space-y-2">
                  <h3 className="font-semibold">{t('aup.design.title4_1')}</h3>
                </div>

                {/* 4.1.1 Is experiment conducted under anesthesia */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>{t('aup.design.anesthesiaLabel')}</Label>
                    <Select
                      value={formData.working_content.design.anesthesia.is_under_anesthesia === null ? '' : (formData.working_content.design.anesthesia.is_under_anesthesia === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'anesthesia.is_under_anesthesia', isYes as boolean | null)
                        // If "No" is selected, clear related fields and auto-fill surgery plan with "N/A"
                        if (!isYes) {
                          updateWorkingContent('design', 'anesthesia.anesthesia_type', undefined)
                          updateWorkingContent('design', 'anesthesia.other_description', undefined)
                          // Auto-fill surgery plan with "N/A" when not under anesthesia
                          const naText = 'N/A'
                          updateWorkingContent('surgery', 'surgery_type', naText)
                          updateWorkingContent('surgery', 'preop_preparation', naText)
                          updateWorkingContent('surgery', 'surgery_description', naText)
                          updateWorkingContent('surgery', 'monitoring', naText)
                          updateWorkingContent('surgery', 'postop_expected_impact', naText)
                          updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                          updateWorkingContent('surgery', 'postop_care', naText)
                          updateWorkingContent('surgery', 'postop_care_type', undefined)
                          updateWorkingContent('surgery', 'expected_end_point', naText)
                          updateWorkingContent('surgery', 'drugs', [])
                          updateWorkingContent('surgery', 'aseptic_techniques', [])
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.anesthesia.is_under_anesthesia === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>{t('aup.design.selectAnesthesiaType')}</Label>
                        <Select
                          value={formData.working_content.design.anesthesia.anesthesia_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'anesthesia.anesthesia_type', value)
                            // If not "Other", clear other description
                            if (value !== 'other') {
                              updateWorkingContent('design', 'anesthesia.anesthesia_other_description', undefined)
                            }
                            // Auto-set surgery plan based on selection
                            if (value === 'survival_surgery') {
                              // If survival surgery, auto-set surgery type to "Survival"
                              updateWorkingContent('surgery', 'surgery_type', 'survival')
                              // If pre-op preparation is "N/A" or empty, set default content
                              if (formData.working_content.surgery.preop_preparation === '略' || formData.working_content.surgery.preop_preparation === t('common.na') || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', t('aup.design.templates.preop_preparation'))
                              }
                              // If surgery description is "N/A" or empty, set default content
                              if (formData.working_content.surgery.surgery_description === '略' || formData.working_content.surgery.surgery_description === t('common.na') || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', t('aup.design.templates.surgery_description'))
                              }
                              // If monitoring is empty, set default content
                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', t('aup.design.templates.monitoring'))
                              }
                              // Post-op care type is user-selected, not auto-set
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value === 'non_survival_surgery') {
                              // If non-survival surgery, auto-set surgery type to "Non-survival"
                              updateWorkingContent('surgery', 'surgery_type', 'non_survival')
                              // If pre-op preparation is "N/A" or empty, set default content
                              if (formData.working_content.surgery.preop_preparation === '略' || formData.working_content.surgery.preop_preparation === t('common.na') || !formData.working_content.surgery.preop_preparation) {
                                updateWorkingContent('surgery', 'preop_preparation', t('aup.design.templates.preop_preparation'))
                              }
                              // If surgery description is "N/A" or empty, set default content
                              if (formData.working_content.surgery.surgery_description === '略' || formData.working_content.surgery.surgery_description === t('common.na') || !formData.working_content.surgery.surgery_description) {
                                updateWorkingContent('surgery', 'surgery_description', t('aup.design.templates.surgery_description'))
                              }
                              // If monitoring is empty, set default content
                              if (!formData.working_content.surgery.monitoring) {
                                updateWorkingContent('surgery', 'monitoring', t('aup.design.templates.monitoring'))
                              }
                              // Post-op care type is user-selected, not auto-set
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            } else if (value && value !== '') {
                              // If not survival or non-survival surgery, auto-fill "N/A"
                              const naText = 'N/A'
                              updateWorkingContent('surgery', 'surgery_type', naText)
                              updateWorkingContent('surgery', 'preop_preparation', naText)
                              updateWorkingContent('surgery', 'surgery_description', naText)
                              updateWorkingContent('surgery', 'monitoring', naText)
                              updateWorkingContent('surgery', 'postop_expected_impact', naText)
                              updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                              updateWorkingContent('surgery', 'postop_care', naText)
                              updateWorkingContent('surgery', 'postop_care_type', undefined)
                              updateWorkingContent('surgery', 'expected_end_point', naText)
                              updateWorkingContent('surgery', 'drugs', [])
                              updateWorkingContent('surgery', 'aseptic_techniques', [])
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder={t('common.pleaseSelect')} />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="survival_surgery">{t('aup.design.anesthesiaTypes.survival')}</SelectItem>
                            <SelectItem value="non_survival_surgery">{t('aup.design.anesthesiaTypes.non_survival')}</SelectItem>
                            <SelectItem value="gas_only">{t('aup.design.anesthesiaTypes.gas_only')}</SelectItem>
                            <SelectItem value="azeperonum_atropine">{t('aup.design.anesthesiaTypes.azeperonum_atropine')}</SelectItem>
                            <SelectItem value="other">{t('aup.design.anesthesiaTypes.other')}</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.anesthesia.anesthesia_type === 'other' && (
                        <div className="space-y-2">
                          <Label>{t('aup.design.explainOther')}</Label>
                          <Textarea
                            value={formData.working_content.design.anesthesia.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'anesthesia.other_description', e.target.value)}
                            placeholder={t('aup.design.explainOtherPlaceholder')}
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.2 Detailed narrative of animal experiment content and procedures */}
                <div className="space-y-2">
                  <Label>{t('aup.design.proceduresLabel')}</Label>
                  <p className="text-sm text-muted-foreground mb-2">{t('aup.design.proceduresNote')}</p>
                  <Textarea
                    value={formData.working_content.design.procedures}
                    onChange={(e) => updateWorkingContent('design', 'procedures', e.target.value)}
                    placeholder={t('aup.design.proceduresPlaceholder')}
                    rows={8}
                  />
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.3 Assessment of experimental animal levels */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>{t('aup.design.painCategoryLabel')}</Label>
                    <Select
                      value={formData.working_content.design.pain.category}
                      onValueChange={(val) => updateWorkingContent('design', 'pain.category', val)}
                    >
                      <SelectTrigger><SelectValue placeholder={t('common.pleaseSelect')} /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="B">{t('aup.design.painCategories.B')}</SelectItem>
                        <SelectItem value="C">{t('aup.design.painCategories.C')}</SelectItem>
                        <SelectItem value="D">{t('aup.design.painCategories.D')}</SelectItem>
                        <SelectItem value="E">{t('aup.design.painCategories.E')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.4 Whether to restrict diet or water for experimental animals */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>{t('aup.design.restrictionsLabel')}</Label>
                    <Select
                      value={formData.working_content.design.restrictions.is_restricted === null ? '' : (formData.working_content.design.restrictions.is_restricted === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'restrictions.is_restricted', isYes as boolean | null)
                        // If "No" is selected, clear related fields
                        if (!isYes) {
                          updateWorkingContent('design', 'restrictions.restriction_type', undefined)
                          updateWorkingContent('design', 'restrictions.other_description', undefined)
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {formData.working_content.design.restrictions.is_restricted === true && (
                    <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                      <div className="space-y-2">
                        <Label>{t('aup.design.selectRestrictionType')}</Label>
                        <Select
                          value={formData.working_content.design.restrictions.restriction_type || ''}
                          onValueChange={(value) => {
                            updateWorkingContent('design', 'restrictions.restriction_type', value)
                            // If not "Other", clear other description
                            if (value !== 'other') {
                              updateWorkingContent('design', 'restrictions.other_description', undefined)
                            }
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder={t('common.pleaseSelect')} />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="fasting_before_anesthesia">{t('aup.design.restrictionTypes.fasting')}</SelectItem>
                            <SelectItem value="other">{t('aup.design.restrictionTypes.other')}</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.restrictions.restriction_type === 'other' && (
                        <div className="space-y-2">
                          <Label>{t('aup.design.explainRestrictionOther')}</Label>
                          <Textarea
                            value={formData.working_content.design.restrictions.other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'restrictions.other_description', e.target.value)}
                            placeholder={t('aup.design.explainRestrictionOtherPlaceholder')}
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.5 Expected timing of experiment completion */}
                <div className="space-y-4">
                  <Label>{t('aup.design.endpointsTitle')}</Label>
                  <div className="space-y-2">
                    <Label>{t('aup.design.experimentalEndpoint')}</Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.experimental_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.experimental_endpoint', e.target.value)}
                      placeholder={t('aup.design.experimentalEndpointPlaceholder')}
                      rows={3}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.design.humaneEndpoint')}</Label>
                    <Textarea
                      value={formData.working_content.design.endpoints.humane_endpoint}
                      onChange={(e) => updateWorkingContent('design', 'endpoints.humane_endpoint', e.target.value)}
                      placeholder={t('aup.design.humaneEndpointPlaceholder')}
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.1.6 Animal euthanasia or final disposal method */}
                <div className="space-y-4">
                  <Label>{t('aup.design.finalHandlingTitle')}</Label>
                  <div className="space-y-2">
                    <Select
                      value={formData.working_content.design.final_handling.method || ''}
                      onValueChange={(value) => {
                        updateWorkingContent('design', 'final_handling.method', value)
                        // Clear other options content
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
                        <SelectValue placeholder={t('aup.design.selectHandlingMethod')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="euthanasia">{t('aup.design.handlingMethods.euthanasia')}</SelectItem>
                        <SelectItem value="transfer">{t('aup.design.handlingMethods.transfer')}</SelectItem>
                        <SelectItem value="other">{t('aup.design.handlingMethods.other')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {/* 1. Euthanasia */}
                  {formData.working_content.design.final_handling.method === 'euthanasia' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">{t('aup.design.euthanasiaLabel')}</Label>
                      <Select
                        value={formData.working_content.design.final_handling.euthanasia_type || ''}
                        onValueChange={(value) => {
                          updateWorkingContent('design', 'final_handling.euthanasia_type', value)
                          // If not "Other", clear other description
                          if (value !== 'other') {
                            updateWorkingContent('design', 'final_handling.euthanasia_other_description', undefined)
                          }
                        }}
                      >
                        <SelectTrigger>
                          <SelectValue placeholder={t('common.pleaseSelect')} />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="kcl">{t('aup.design.euthanasiaTypes.kcl')}</SelectItem>
                          <SelectItem value="electrocution">{t('aup.design.euthanasiaTypes.electrocution')}</SelectItem>
                          <SelectItem value="other">{t('aup.design.euthanasiaTypes.other')}</SelectItem>
                        </SelectContent>
                      </Select>
                      {formData.working_content.design.final_handling.euthanasia_type === 'other' && (
                        <div className="space-y-2 mt-2">
                          <Textarea
                            value={formData.working_content.design.final_handling.euthanasia_other_description || ''}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.euthanasia_other_description', e.target.value)}
                            placeholder={t('aup.design.euthanasiaOtherPlaceholder')}
                            rows={3}
                          />
                        </div>
                      )}
                    </div>
                  )}

                  {/* 2. Transfer */}
                  {formData.working_content.design.final_handling.method === 'transfer' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">{t('aup.design.transferLabel')}</Label>
                      <div className="space-y-3">
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.recipientName')}</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_name', e.target.value)}
                            placeholder={t('aup.design.recipientNamePlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.recipientOrg')}</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.recipient_org}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.recipient_org', e.target.value)}
                            placeholder={t('aup.design.recipientOrgPlaceholder')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-sm">{t('aup.design.projectName')}</Label>
                          <Input
                            value={formData.working_content.design.final_handling.transfer.project_name}
                            onChange={(e) => updateWorkingContent('design', 'final_handling.transfer.project_name', e.target.value)}
                            placeholder={t('aup.design.projectNamePlaceholder')}
                          />
                        </div>
                      </div>
                    </div>
                  )}

                  {/* 3. Other */}
                  {formData.working_content.design.final_handling.method === 'other' && (
                    <div className="space-y-3 border-l-2 border-slate-200 pl-6">
                      <Label className="text-sm font-medium">{t('aup.design.handlingMethods.other')}: </Label>
                      <Textarea
                        value={formData.working_content.design.final_handling.other_description || ''}
                        onChange={(e) => updateWorkingContent('design', 'final_handling.other_description', e.target.value)}
                        placeholder={t('aup.design.otherHandlingPlaceholder')}
                        rows={3}
                      />
                    </div>
                  )}
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.2 Animal carcass disposal method */}
                <div className="space-y-4">
                  <h3 className="font-semibold">{t('aup.design.carcassDisposalLabel')}</h3>
                  <div className="space-y-2">
                    <Textarea
                      value={formData.working_content.design.carcass_disposal.method}
                      onChange={(e) => updateWorkingContent('design', 'carcass_disposal.method', e.target.value)}
                      placeholder={t('aup.design.carcassDisposalPlaceholder')}
                      rows={4}
                    />
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.3 Use of non-pharmaceutical chemical drugs or other substances */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <h3 className="font-semibold">{t('aup.design.nonPharmaLabel')}</h3>
                    <Select
                      value={formData.working_content.design.non_pharma_grade.used === null ? '' : (formData.working_content.design.non_pharma_grade.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'non_pharma_grade.used', isYes as boolean | null)
                        // If "No" is selected, clear description field
                        if (!isYes) {
                          updateWorkingContent('design', 'non_pharma_grade.description', '')
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.non_pharma_grade.used === true && (
                      <div className="space-y-2 mt-2">
                        <Label>{t('aup.design.nonPharmaExplain')}</Label>
                        <Textarea
                          value={formData.working_content.design.non_pharma_grade.description}
                          onChange={(e) => updateWorkingContent('design', 'non_pharma_grade.description', e.target.value)}
                          placeholder={t('aup.design.nonPharmaExplainPlaceholder')}
                          rows={4}
                        />
                      </div>
                    )}
                  </div>
                </div>

                <div className="h-px bg-border my-4" />

                {/* 4.4 Use of hazardous materials */}
                <div className="space-y-4">
                  <div className="space-y-2">
                    <h3 className="font-semibold">{t('aup.design.hazardsLabel')}</h3>
                    <Select
                      value={formData.working_content.design.hazards.used === null ? '' : (formData.working_content.design.hazards.used === true ? 'yes' : 'no')}
                      onValueChange={(value) => {
                        const isYes = value === 'yes'
                        updateWorkingContent('design', 'hazards.used', isYes as boolean | null)
                        // If "No" is selected, clear related fields
                        if (!isYes) {
                          updateWorkingContent('design', 'hazards.materials', [])
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('common.pleaseSelect')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="no">{t('common.no')}</SelectItem>
                        <SelectItem value="yes">{t('common.yes')}</SelectItem>
                      </SelectContent>
                    </Select>
                    {formData.working_content.design.hazards.used === true && (
                      <div className="space-y-4 mt-2 pl-6 border-l-2 border-slate-200">
                        <div className="space-y-3">
                          <div className="space-y-2">
                            <Label className="text-sm font-medium">{t('aup.design.selectHazardType')}</Label>
                            <Select
                              value={formData.working_content.design.hazards.selected_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('design', 'hazards.selected_type', value)
                                // Clear materials of other types, keep only materials of current type
                                const currentMaterials = formData.working_content.design.hazards.materials.filter(m => m.type === value)
                                updateWorkingContent('design', 'hazards.materials', currentMaterials)
                              }}
                            >
                              <SelectTrigger>
                                <SelectValue placeholder={t('aup.design.selectHazardTypePlaceholder')} />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="biological">{t('aup.design.hazardTypes.biological')}</SelectItem>
                                <SelectItem value="radioactive">{t('aup.design.hazardTypes.radioactive')}</SelectItem>
                                <SelectItem value="chemical">{t('aup.design.hazardTypes.chemical')}</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>

                          {/* Show material list for selected type */}
                          {formData.working_content.design.hazards.selected_type && (
                            <div className="space-y-3">
                              <div className="flex justify-between items-center">
                                <Label className="text-sm font-medium">
                                  {formData.working_content.design.hazards.selected_type === 'biological' && t('aup.design.hazardTypes.biological')}
                                  {formData.working_content.design.hazards.selected_type === 'radioactive' && t('aup.design.hazardTypes.radioactive')}
                                  {formData.working_content.design.hazards.selected_type === 'chemical' && t('aup.design.hazardTypes.chemical')}
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
                                  {t('aup.items.add')}
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
                                          placeholder={t('aup.design.agentNamePlaceholder')}
                                          value={material.agent_name}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].agent_name = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                        <Input
                                          placeholder={t('aup.design.amountPlaceholder')}
                                          value={material.amount}
                                          onChange={(e) => {
                                            const materials = [...formData.working_content.design.hazards.materials]
                                            materials[materialIndex].amount = e.target.value
                                            updateWorkingContent('design', 'hazards.materials', materials)
                                          }}
                                        />
                                      </div>
                                      {/* Photo Upload */}
                                      <div className="space-y-2">
                                        <Label className="text-sm">{t('aup.items.photos')}</Label>
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
                                          placeholder={t('aup.items.photosPlaceholder')}
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

                {/* Conditional Visibility: if 4.4 is "Yes", show 4.5 and 4.6; if 4.4 is "No", show 4.5 (Controlled Substances) */}
                {formData.working_content.design.hazards.used === true && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 Hazardous substances and waste disposal methods */}
                    <div className="space-y-4">
                      <h3 className="font-semibold">{t('aup.design.hazardsWasteLabel')}</h3>

                      {/* 4.5.1 Administration method, route and place of use */}
                      <div className="space-y-2">
                        <Label>{t('aup.design.operationLocationLabel')}</Label>
                        <Textarea
                          value={formData.working_content.design.hazards.operation_location_method}
                          onChange={(e) => updateWorkingContent('design', 'hazards.operation_location_method', e.target.value)}
                          placeholder={t('aup.design.operationLocationPlaceholder')}
                          rows={4}
                        />
                      </div>

                      {/* 4.5.2 Protection measures */}
                      <div className="space-y-2">
                        <Label>{t('aup.design.protectionMeasuresLabel')}</Label>
                        <p className="text-sm text-muted-foreground mb-2">{t('aup.design.protectionMeasuresSubtitle')}</p>
                        <Textarea
                          value={formData.working_content.design.hazards.protection_measures}
                          onChange={(e) => updateWorkingContent('design', 'hazards.protection_measures', e.target.value)}
                          placeholder={t('aup.design.protectionMeasuresPlaceholder')}
                          rows={4}
                        />
                      </div>

                      {/* 4.5.3 Disposal of experimental waste and carcasses */}
                      <div className="space-y-2">
                        <Label>{t('aup.design.wasteDisposalLabel')}</Label>
                        <Textarea
                          value={formData.working_content.design.hazards.waste_and_carcass_disposal}
                          onChange={(e) => updateWorkingContent('design', 'hazards.waste_and_carcass_disposal', e.target.value)}
                          placeholder={t('aup.design.wasteDisposalPlaceholder')}
                          rows={4}
                        />
                      </div>
                    </div>

                    <div className="h-px bg-border my-4" />

                    {/* 4.6 Use of controlled substances */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.design.controlledSubstancesLabel.section4_6')}</h3>
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
                            <SelectValue placeholder={t('common.pleaseSelect')} />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">{t('common.no')}</SelectItem>
                            <SelectItem value="yes">{t('common.yes')}</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">{t('aup.design.controlledSubstancesList')}</Label>
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
                              {t('aup.items.add')}
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
                                  <Label className="text-sm">{t('aup.design.drugNameLabel')}</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.drugNamePlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.approvalNoLabel')}</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.approvalNoPlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.drugAmountLabel')}</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.drugAmountPlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.authorizedPersonLabel')}</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.authorizedPersonPlaceholder')}
                                  />
                                </div>
                              </div>
                              {/* Photo Upload */}
                              <div className="space-y-2">
                                <Label className="text-sm">{t('aup.items.photos')}</Label>
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
                                  placeholder={t('aup.items.photosPlaceholder')}
                                />
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </>
                )}

                {/* If 4.4 is "No", show 4.5 (Controlled Substances) */}
                {formData.working_content.design.hazards.used === false && (
                  <>
                    <div className="h-px bg-border my-4" />

                    {/* 4.5 Use of controlled substances */}
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label>{t('aup.design.controlledSubstancesLabel.section4_5')}</Label>
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
                            <SelectValue placeholder={t('common.pleaseSelect')} />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="no">{t('common.no')}</SelectItem>
                            <SelectItem value="yes">{t('common.yes')}</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>

                      {formData.working_content.design.controlled_substances.used === true && (
                        <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                          <div className="flex justify-between items-center">
                            <Label className="text-sm font-medium">{t('aup.design.controlledSubstancesList')}</Label>
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
                              {t('aup.items.add')}
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
                                  <Label className="text-sm">{t('aup.design.drugNameLabel')}</Label>
                                  <Input
                                    value={item.drug_name}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].drug_name = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.drugNamePlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.approvalNoLabel')}</Label>
                                  <Input
                                    value={item.approval_no}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].approval_no = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.approvalNoPlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.drugAmountLabel')}</Label>
                                  <Input
                                    value={item.amount}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].amount = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.drugAmountPlaceholder')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label className="text-sm">{t('aup.design.authorizedPersonLabel')}</Label>
                                  <Input
                                    value={item.authorized_person}
                                    onChange={(e) => {
                                      const items = [...formData.working_content.design.controlled_substances.items]
                                      items[index].authorized_person = e.target.value
                                      updateWorkingContent('design', 'controlled_substances.items', items)
                                    }}
                                    placeholder={t('aup.design.authorizedPersonPlaceholder')}
                                  />
                                </div>
                              </div>
                              {/* Photo Upload */}
                              <div className="space-y-2">
                                <Label className="text-sm">{t('aup.items.photos')}</Label>
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
                                  placeholder={t('aup.items.photosPlaceholder')}
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
                <CardTitle>{t('aup.section5')}</CardTitle>
                <CardDescription>{t('aup.guidelines.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-2">
                  <Label>{t('aup.guidelines.contentLabel')}</Label>
                  <Textarea
                    value={formData.working_content.guidelines.content}
                    onChange={(e) => updateWorkingContent('guidelines', 'content', e.target.value)}
                    placeholder={t('aup.guidelines.contentPlaceholder')}
                    rows={5}
                  />
                </div>
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">{t('aup.guidelines.referencesTitle')}</h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const newRefs = [...formData.working_content.guidelines.references, { citation: '', url: '' }]
                        updateWorkingContent('guidelines', 'references', newRefs)
                      }}
                    >
                      {t('aup.guidelines.addReference')}
                    </Button>
                  </div>
                  {formData.working_content.guidelines.references.map((ref, index) => (
                    <div key={index} className="grid w-full gap-2 relative">
                      <div className="flex gap-2 items-start">
                        <div className="grid gap-2 flex-1">
                          <Input
                            placeholder={t('aup.guidelines.citationPlaceholder')}
                            value={ref.citation}
                            onChange={(e) => {
                              const newRefs = [...formData.working_content.guidelines.references]
                              newRefs[index].citation = e.target.value
                              updateWorkingContent('guidelines', 'references', newRefs)
                            }}
                          />
                          <Input
                            placeholder={t('aup.guidelines.urlPlaceholder')}
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
                <div className="flex justify-between items-center">
                  <div>
                    <CardTitle>{t('aup.section6')}</CardTitle>
                    <CardDescription>{t('aup.surgery.subtitle')}</CardDescription>
                  </div>
                  {formData.working_content.design.anesthesia.is_under_anesthesia === true &&
                    (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
                      formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery') && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          // Load default values for surgery section
                          // 6.2 Pre-operative
                          updateWorkingContent('surgery', 'preop_preparation', t('aup.defaults.preopPreparation'))
                          // 6.3 Aseptic (unbox all)
                          updateWorkingContent('surgery', 'aseptic_techniques', [])
                          // 6.4 Surgery description (clear)
                          updateWorkingContent('surgery', 'surgery_description', '')
                          // 6.5 Monitoring
                          updateWorkingContent('surgery', 'monitoring', t('aup.defaults.monitoring'))
                          // 6.6 Post-operative impact (clear)
                          updateWorkingContent('surgery', 'postop_expected_impact', '')
                          // 6.7 Multiple surgeries (set to "No")
                          updateWorkingContent('surgery', 'multiple_surgeries', { used: false, number: 0, reason: '' })
                          // 6.8 Orthopedic/Non-orthopedic (deselect)
                          updateWorkingContent('surgery', 'postop_care_type', undefined)
                          updateWorkingContent('surgery', 'postop_care', '')
                          // 6.9 Expected end point (clear)
                          updateWorkingContent('surgery', 'expected_end_point', '')

                          // 6.10 Load default drugs
                          const defaultDrugs = t('aup.defaults.drugDefaults', { returnObjects: true }) as any[]
                          if (Array.isArray(defaultDrugs)) {
                            const formattedDrugs = defaultDrugs.map(drug => ({
                              drug_name: drug.name,
                              dose: drug.dose,
                              route: drug.route,
                              frequency: drug.frequency,
                              purpose: drug.purpose
                            }))
                            updateWorkingContent('surgery', 'drugs', formattedDrugs)
                          }
                        }}

                      >
                        {t('aup.surgery.loadDefaults')}
                      </Button>
                    )}
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                {(() => {
                  const needsSurgeryPlan = formData.working_content.design.anesthesia.is_under_anesthesia === true &&
                    (formData.working_content.design.anesthesia.anesthesia_type === 'survival_surgery' ||
                      formData.working_content.design.anesthesia.anesthesia_type === 'non_survival_surgery')

                  if (!needsSurgeryPlan) {
                    // If surgery plan is not required, show "N/A"
                    return (
                      <div className="space-y-4">
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.surgeryType')}</h3>
                          <Input value={t('common.na')} disabled />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.preopPreparation')}</h3>
                          <Textarea value={t('common.na')} disabled rows={3} />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.asepticTechniques')}</h3>
                          <Input value={t('common.na')} disabled />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.surgeryDescription')}</h3>
                          <Textarea value={t('common.na')} disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.monitoring')}</h3>
                          <Textarea value={t('common.na')} disabled rows={5} />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.expectedImpact')}</h3>
                          <Textarea value={t('common.na')} disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.multipleSurgeries')}</h3>
                          <Input value={t('common.na')} disabled />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.postopCare')}</h3>
                          <div className="space-y-4">
                            <div className="space-y-2">
                              <Label>{t('aup.surgery.labels.postopCareType')}</Label>
                              <Input value={t('common.na')} disabled />
                            </div>
                            <div className="space-y-2">
                              <Label>{t('aup.surgery.labels.postopCareDetail')}</Label>
                              <Textarea value={t('common.na')} disabled rows={5} />
                            </div>
                          </div>
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.expectedEndPoint')}</h3>
                          <Textarea value={t('common.na')} disabled rows={4} />
                        </div>
                        <div className="space-y-2">
                          <h3 className="font-semibold">{t('aup.surgery.labels.drugInfo')}</h3>
                          <Input value={t('common.na')} disabled />
                        </div>
                      </div>
                    )
                  }

                  // If surgery plan is required, show normal form
                  return (
                    <>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.surgeryType')}</h3>
                        <Input
                          value={formData.working_content.surgery.surgery_type === 'survival' ? t('aup.surgery.types.survival') :
                            formData.working_content.surgery.surgery_type === 'non_survival' ? t('aup.surgery.types.non_survival') :
                              formData.working_content.surgery.surgery_type || ''}
                          disabled
                          className="bg-slate-50"
                        />
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.preopPreparation')}</h3>
                        <Textarea
                          value={formData.working_content.surgery.preop_preparation}
                          onChange={(e) => updateWorkingContent('surgery', 'preop_preparation', e.target.value)}
                          placeholder={t('aup.surgery.placeholders.preopPreparation')}
                          rows={8}
                        />
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.asepticTechniques')}</h3>
                        <div className="space-y-2">
                          {[
                            { value: 'surgical_site_disinfection', label: t('aup.surgery.asepticTechniques.surgical_site_disinfection') },
                            { value: 'instrument_disinfection', label: t('aup.surgery.asepticTechniques.instrument_disinfection') },
                            { value: 'sterilized_gowns_gloves', label: t('aup.surgery.asepticTechniques.sterilized_gowns_gloves') },
                            { value: 'sterilized_drapes', label: t('aup.surgery.asepticTechniques.sterilized_drapes') },
                            { value: 'surgical_hand_disinfection', label: t('aup.surgery.asepticTechniques.surgical_hand_disinfection') }
                          ].map(item => (
                            <div key={item.value} className="flex items-center space-x-2">
                              <Checkbox
                                id={`aseptic_${item.value}`}
                                checked={formData.working_content.surgery.aseptic_techniques.includes(item.value)}
                                onCheckedChange={(checked) => {
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
                        <h3 className="font-semibold">{t('aup.surgery.labels.surgeryDescription')}</h3>
                        <Textarea
                          value={formData.working_content.surgery.surgery_description}
                          onChange={(e) => updateWorkingContent('surgery', 'surgery_description', e.target.value)}
                          placeholder={t('aup.surgery.placeholders.surgeryDescription')}
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.monitoring')}</h3>
                        <Textarea
                          value={formData.working_content.surgery.monitoring}
                          onChange={(e) => updateWorkingContent('surgery', 'monitoring', e.target.value)}
                          placeholder={t('aup.surgery.placeholders.monitoring')}
                          rows={5}
                        />
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.expectedImpact')}</h3>
                        <Textarea
                          value={formData.working_content.surgery.postop_expected_impact}
                          onChange={(e) => updateWorkingContent('surgery', 'postop_expected_impact', e.target.value)}
                          placeholder={t('aup.surgery.labels.expectedImpact')}
                          rows={4}
                        />
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.multipleSurgeries')}</h3>
                        <div className="space-y-4">
                          <Select
                            value={formData.working_content.surgery.multiple_surgeries.used === true ? 'yes' : formData.working_content.surgery.multiple_surgeries.used === false ? 'no' : ''}
                            onValueChange={(value) => {
                              const isYes = value === 'yes'
                              updateWorkingContent('surgery', 'multiple_surgeries.used', isYes)
                              if (!isYes) {
                                updateWorkingContent('surgery', 'multiple_surgeries.number', 0)
                                updateWorkingContent('surgery', 'multiple_surgeries.reason', '')
                              }
                            }}
                          >
                            <SelectTrigger>
                              <SelectValue placeholder={t('common.pleaseSelect')} />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="no">{t('common.no')}</SelectItem>
                              <SelectItem value="yes">{t('common.yes')}</SelectItem>
                            </SelectContent>
                          </Select>
                          {formData.working_content.surgery.multiple_surgeries.used && (
                            <div className="space-y-4 pl-6 border-l-2 border-slate-200">
                              <div className="space-y-2">
                                <Label>{t('aup.items.amount')} *</Label>
                                <Input
                                  type="number"
                                  value={formData.working_content.surgery.multiple_surgeries.number || ''}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.number', parseInt(e.target.value) || 0)}
                                  placeholder={t('aup.items.amount')}
                                />
                              </div>
                              <div className="space-y-2">
                                <Label>{t('aup.items.reason')} *</Label>
                                <Textarea
                                  value={formData.working_content.surgery.multiple_surgeries.reason}
                                  onChange={(e) => updateWorkingContent('surgery', 'multiple_surgeries.reason', e.target.value)}
                                  placeholder={t('aup.items.reason')}
                                  rows={3}
                                />
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.postopCare')}</h3>
                        <div className="space-y-4">
                          <div className="space-y-2">
                            <Label>{t('aup.surgery.labels.postopCareType')} *</Label>
                            <Select
                              value={formData.working_content.surgery.postop_care_type || ''}
                              onValueChange={(value) => {
                                updateWorkingContent('surgery', 'postop_care_type', value as 'orthopedic' | 'non_orthopedic')
                                // Auto-set corresponding default content based on selection
                                if (value === 'orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', t('aup.surgery.postOpTemplates.orthopedic'))
                                } else if (value === 'non_orthopedic') {
                                  updateWorkingContent('surgery', 'postop_care', t('aup.surgery.postOpTemplates.non_orthopedic'))
                                }
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder={t('aup.surgery.labels.postopCareType')} /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="orthopedic">{t('aup.surgery.postOpTypes.orthopedic')}</SelectItem>
                                <SelectItem value="non_orthopedic">{t('aup.surgery.postOpTypes.non_orthopedic')}</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                          <div className="space-y-2">
                            <Label>{t('aup.surgery.labels.postopCareDetail')} *</Label>
                            <Textarea
                              value={formData.working_content.surgery.postop_care}
                              onChange={(e) => updateWorkingContent('surgery', 'postop_care', e.target.value)}
                              placeholder={t('aup.surgery.placeholders.postopCare')}
                              rows={15}
                            />
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <h3 className="font-semibold">{t('aup.surgery.labels.expectedEndPoint')}</h3>
                        <Textarea
                          value={formData.working_content.surgery.expected_end_point}
                          onChange={(e) => updateWorkingContent('surgery', 'expected_end_point', e.target.value)}
                          placeholder={t('aup.surgery.placeholders.expectedEndPoint')}
                          rows={4}
                        />
                      </div>
                      <div className="space-y-4">
                        <div className="flex justify-between items-center">
                          <h3 className="font-semibold">{t('aup.surgery.labels.drugInfo')}</h3>
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
                            + {t('aup.items.add')}
                          </Button>
                        </div>
                        <div className="border rounded-md overflow-hidden">
                          <div className="overflow-x-auto">
                            <table className="w-full border-collapse">
                              <thead>
                                <tr className="bg-slate-100">
                                  <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.name')}</th>
                                  <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.dose')}</th>
                                  <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.route')}</th>
                                  <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.frequency')}</th>
                                  <th className="border p-2 text-left text-sm font-semibold">{t('aup.surgery.drugs.headers.purpose')}</th>
                                  <th className="border p-2 text-center text-sm font-semibold w-16">{t('common.actions')}</th>
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
                                        placeholder={t('aup.surgery.drugs.headers.name')}
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
                                        placeholder={t('aup.surgery.drugs.headers.dose')}
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
                                        placeholder={t('aup.surgery.drugs.headers.route')}
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
                                        placeholder={t('aup.surgery.drugs.headers.frequency')}
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
                                        placeholder={t('aup.surgery.drugs.headers.purpose')}
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
                                      {t('aup.surgery.noDrugs')}
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
                <CardTitle>{t('aup.section7')}</CardTitle>
                <CardDescription>{t('aup.animals.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4 border p-4 rounded-md">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">{t('aup.animals.listHeader')}</h3>
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
                          housing_location: ''
                        }]
                        updateWorkingContent('animals', 'animals', newAnimals)
                      }}
                    >
                      + {t('aup.animals.addAnimal')}
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
                          <Label>{t('aup.animals.labels.species')} *</Label>
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
                            <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.species')} /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="pig">{t('aup.animals.species.pig')}</SelectItem>
                              <SelectItem value="other">{t('aup.animals.species.other')}</SelectItem>
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
                              placeholder={t('aup.animals.placeholders.speciesOther')}
                            />
                          )}
                        </div>
                        <div className="space-y-2">
                          <Label>{t('aup.animals.labels.strain')}</Label>
                          {animal.species === 'pig' ? (
                            <Select
                              value={animal.strain || ''}
                              onValueChange={(value) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].strain = value as 'white_pig' | 'mini_pig'
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            >
                              <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.strain')} /></SelectTrigger>
                              <SelectContent>
                                <SelectItem value="white_pig">{t('aup.animals.strains.white_pig')}</SelectItem>
                                <SelectItem value="mini_pig">{t('aup.animals.strains.mini_pig')}</SelectItem>
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
                              placeholder={t('aup.animals.placeholders.strainOther')}
                            />
                          ) : (
                            <Input disabled placeholder={t('aup.animals.placeholders.selectSpeciesFirst')} />
                          )}
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>{t('aup.animals.labels.sex')} *</Label>
                          <Select
                            value={animal.sex || ''}
                            onValueChange={(value) => {
                              const newAnimals = [...formData.working_content.animals.animals]
                              newAnimals[index].sex = value
                              updateWorkingContent('animals', 'animals', newAnimals)
                            }}
                          >
                            <SelectTrigger><SelectValue placeholder={t('aup.animals.placeholders.sex')} /></SelectTrigger>
                            <SelectContent>
                              <SelectItem value="male">{t('aup.animals.sexTypes.male')}</SelectItem>
                              <SelectItem value="female">{t('aup.animals.sexTypes.female')}</SelectItem>
                              <SelectItem value="unlimited">{t('aup.animals.sexTypes.unlimited')}</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="space-y-2">
                          <Label>{t('aup.animals.labels.number')} *</Label>
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
                            placeholder={t('aup.animals.placeholders.number')}
                          />
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>{t('aup.animals.labels.age')} </Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.age_unlimited && (
                              <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                  <Label>{t('aup.animals.labels.minAge')}</Label>
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
                                        // If max age is less than or equal to new min age, auto-adjust max age
                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= value) {
                                          newAnimals[index].age_max = value + 1
                                        }
                                      } else if (value < 3 && value > 0) {
                                        newAnimals[index].age_min = 3
                                        // If max age is less than or equal to 3, auto-adjust max age
                                        if (newAnimals[index].age_max !== undefined && newAnimals[index].age_max <= 3) {
                                          newAnimals[index].age_max = 4
                                        }
                                      } else {
                                        newAnimals[index].age_min = undefined
                                      }
                                      updateWorkingContent('animals', 'animals', newAnimals)
                                    }}
                                    placeholder={t('aup.animals.placeholders.minAge')}
                                  />
                                </div>
                                <div className="space-y-2">
                                  <Label>{t('aup.animals.labels.maxAge')}</Label>
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
                                    placeholder={t('aup.animals.placeholders.maxAge')}
                                  />
                                </div>
                              </div>
                            )}
                          </div>
                          <div className="flex items-center gap-2">
                            <Checkbox
                              id={`age_unlimited_${index}`}
                              checked={animal.age_unlimited || false}
                              onCheckedChange={(checked) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].age_unlimited = checked
                                if (checked) {
                                  newAnimals[index].age_min = undefined
                                  newAnimals[index].age_max = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            />
                            <Label htmlFor={`age_unlimited_${index}`} className="font-normal cursor-pointer">{t('aup.animals.labels.unlimited')}</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>{t('aup.animals.labels.weight')} </Label>
                        <div className="flex gap-4 items-start">
                          <div className="flex-1">
                            {!animal.weight_unlimited && (
                              <div className="space-y-4">
                                <div className="grid grid-cols-2 gap-4">
                                  <div className="space-y-2">
                                    <Label>{t('aup.animals.labels.minWeight')}</Label>
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
                                          // If max weight is less than or equal to new min weight, auto-adjust max weight
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
                                      placeholder={t('aup.animals.placeholders.minWeight')}
                                    />
                                    <p className="text-xs text-muted-foreground">{t('aup.animals.labels.weightInterval')}</p>
                                  </div>
                                  <div className="space-y-2">
                                    <Label>{t('aup.animals.labels.maxWeight')}</Label>
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
                                          // Ensure max weight is greater than min weight
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
                                      placeholder={t('aup.animals.placeholders.maxWeight')}
                                    />
                                  </div>
                                </div>
                              </div>
                            )}
                          </div>
                          <div className="flex items-center gap-2">
                            <Checkbox
                              id={`weight_unlimited_${index}`}
                              checked={animal.weight_unlimited || false}
                              onCheckedChange={(checked) => {
                                const newAnimals = [...formData.working_content.animals.animals]
                                newAnimals[index].weight_unlimited = checked
                                if (checked) {
                                  newAnimals[index].weight_min = undefined
                                  newAnimals[index].weight_max = undefined
                                }
                                updateWorkingContent('animals', 'animals', newAnimals)
                              }}
                            />
                            <Label htmlFor={`weight_unlimited_${index}`} className="font-normal cursor-pointer">{t('aup.animals.labels.unlimited')}</Label>
                          </div>
                        </div>
                      </div>
                      <div className="space-y-2">
                        <Label>{t('aup.animals.labels.housing')}</Label>
                      </div>
                    </div>
                  ))}
                  <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-md">
                    <h4 className="font-semibold text-sm mb-2">{t('aup.animals.notes.title')}</h4>
                    <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
                      <li>{t('aup.animals.notes.item1')}</li>
                      <li>{t('aup.animals.notes.item2')}</li>
                      <li>{t('aup.animals.notes.item3')}</li>
                      <li>{t('aup.animals.notes.item4')}</li>
                      <li>{t('aup.animals.notes.item5')}</li>
                    </ul>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'personnel' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('aup.section8')}</CardTitle>
                <CardDescription>{t('aup.personnel.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="font-semibold">{t('aup.personnel.listHeader')}</h3>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setNewPersonnel({
                          name: '',
                          position: '',
                          roles: [],
                          roles_other_text: '',
                          years_experience: 0,
                          trainings: [],
                          trainings_other_text: '',
                          training_certificates: [],
                        })
                        setIsAddPersonnelDialogOpen(true)
                      }}
                    >
                      + {t('aup.personnel.addPersonnel')}
                    </Button>
                  </div>
                  <div className="border rounded-md overflow-hidden">
                    <div className="overflow-x-auto">
                      <table className="w-full border-collapse">
                        <thead>
                          <tr className="bg-slate-100">
                            <th className="border p-2 text-center text-sm font-semibold w-16">{t('aup.personnel.table.num')}</th>
                            <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.name')}</th>
                            <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.position')}</th>
                            <th className="border p-2 text-center text-sm font-semibold w-32">{t('aup.personnel.table.roles')}</th>
                            <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.experience')}</th>
                            <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.trainings')}</th>
                            <th className="border p-2 text-center text-sm font-semibold w-16">{t('aup.personnel.table.actions')}</th>
                          </tr>
                        </thead>
                        <tbody>
                          {(formData.working_content.personnel || []).map((person: any, index: number) => (
                            <tr key={index} className="hover:bg-slate-50">
                              <td className="border p-2 w-8">
                                <div className="px-2 py-1 text-center font-medium">
                                  {index + 1}
                                </div>
                              </td>
                              <td className="border p-2 w-24">
                                <div className="px-2 py-1 text-center truncate">
                                  {person.name || '-'}
                                </div>
                              </td>
                              <td className="border p-2 w-24">
                                <div className="px-2 py-1 truncate">
                                  {t('aup.personnel.defaults.researcher')}
                                </div>
                              </td>
                              <td className="border p-2 w-32"> {/* Work Content */}
                                <div className="space-y-1 overflow-hidden">
                                  <div className="flex flex-wrap gap-1">
                                    {(person.roles || []).map((role: string) => (
                                      <Badge key={role} variant="outline" className="text-xs">
                                        {role}
                                      </Badge>
                                    ))}
                                    {(!person.roles || person.roles.length === 0) && (
                                      <span className="text-muted-foreground text-sm">-</span>
                                    )}
                                  </div>
                                  {(person.roles || []).includes('i') && person.roles_other_text && (
                                    <div className="text-xs text-muted-foreground mt-1 truncate">
                                      {t('aup.personnel.roles.otherLabel')}{person.roles_other_text}
                                    </div>
                                  )}
                                </div>
                              </td>
                              <td className="border p-2 w-24">
                                <div className="px-2 py-1 text-center">
                                  {person.years_experience ? `${person.years_experience} ${t('aup.personnel.experienceUnit')}` : '-'}
                                </div>
                              </td>
                              <td className="border p-2">
                                <div className="space-y-2 overflow-hidden">
                                  <div className="flex flex-wrap gap-1 mb-2">
                                    {(person.trainings || []).map((trainingCode: string) => (
                                      <Badge key={trainingCode} variant="outline" className="text-xs">
                                        {trainingCode}
                                      </Badge>
                                    ))}
                                    {(!person.trainings || person.trainings.length === 0) && (
                                      <span className="text-muted-foreground text-sm">-</span>
                                    )}
                                  </div>
                                  {/* Show explanation for F. Other */}
                                  {(person.trainings || []).includes('F') && person.trainings_other_text && (
                                    <div className="space-y-1 pl-4 border-l-2 border-slate-200">
                                      <div className="text-xs font-semibold truncate">F:</div>
                                      <div className="text-xs text-muted-foreground truncate">
                                        {person.trainings_other_text}
                                      </div>
                                    </div>
                                  )}
                                  {/* Show certificate number list for each selected training */}
                                  {(person.trainings || []).filter((t: string) => t !== 'F').map((trainingCode: string) => {
                                    const certificates = (person.training_certificates || []).filter((cert: any) => cert.training_code === trainingCode)
                                    if (certificates.length === 0) return null
                                    return (
                                      <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                                        <div className="text-xs font-semibold whitespace-nowrap truncate">{trainingCode}:</div>
                                        {certificates.map((cert: any, certIndex: number) => (
                                          <div key={certIndex} className="text-xs text-muted-foreground whitespace-nowrap truncate">
                                            {cert.certificate_no || '-'}
                                          </div>
                                        ))}
                                      </div>
                                    )
                                  })}
                                </div>
                              </td>
                              <td className="border p-2 text-center w-16">
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
                              <td colSpan={8} className="border p-4 text-center text-muted-foreground">
                                {t('aup.personnel.table.noPersonnel')}
                              </td>
                            </tr>
                          )}
                        </tbody>
                      </table>
                    </div>
                  </div>
                  <div className="mt-4 p-4 bg-slate-50 rounded-md">
                    <p className="text-sm font-semibold mb-2">{t('aup.personnel.roles.title')}</p>
                    <p className="text-xs text-muted-foreground">
                      {t('aup.personnel.roles.list')}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'attachments' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('aup.section9')}</CardTitle>
                <CardDescription>{t('aup.attachments.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label>{t('aup.attachments.label')}</Label>
                  <FileUpload
                    value={formData.working_content.attachments || []}
                    onChange={(attachments) => {
                      setFormData((prev) => ({
                        ...prev,
                        working_content: {
                          ...prev.working_content,
                          attachments
                        }
                      }))
                    }}
                    accept="application/pdf,.pdf"
                    placeholder={t('aup.attachments.placeholder')}
                    maxSize={20}
                    maxFiles={10}
                    showPreview={false}
                    hint={t('aup.attachments.hint')}
                  />
                </div>
              </CardContent>
            </Card>
          )}

          {activeSection === 'signature' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('aup.section10')}</CardTitle>
                <CardDescription>{t('aup.signature.subtitle')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label>{t('aup.signature.label')}</Label>
                  <FileUpload
                    value={formData.working_content.signature || []}
                    onChange={(signature) => {
                      setFormData((prev) => ({
                        ...prev,
                        working_content: {
                          ...prev.working_content,
                          signature
                        }
                      }))
                    }}
                    accept="image/*,.png,.jpg,.jpeg,.gif,.bmp"
                    placeholder={t('aup.signature.placeholder')}
                    maxSize={5}
                    maxFiles={5}
                    showPreview={true}
                    hint={t('aup.signature.hint')}
                  />
                </div>
              </CardContent>
            </Card>
          )}

          {/* 新增人員對話框 */}
          <Dialog open={isAddPersonnelDialogOpen} onOpenChange={setIsAddPersonnelDialogOpen}>
            <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
              <DialogHeader>
                <DialogTitle>{t('aup.personnel.addDialog.title')}</DialogTitle>
                <DialogDescription>{t('aup.personnel.addDialog.description')}</DialogDescription>
              </DialogHeader>
              <div className="space-y-4 py-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>{t('aup.personnel.addDialog.labels.name')} *</Label>
                    {isIACUCStaff ? (
                      <div className="flex gap-2">
                        <Select
                          onValueChange={(value) => {
                            const selectedStaff = staffMembers.find((s: any) => s.id === value)
                            if (selectedStaff) {
                              // 從入職日期計算年資，若無則使用 years_experience
                              let calculatedYears = selectedStaff.years_experience || 0
                              if (selectedStaff.entry_date) {
                                const entryYear = new Date(selectedStaff.entry_date).getFullYear()
                                const currentYear = new Date().getFullYear()
                                calculatedYears = currentYear - entryYear
                              }

                              setNewPersonnel({
                                ...newPersonnel,
                                name: selectedStaff.display_name,
                                position: t('aup.personnel.defaults.researcher'),  // 固定值
                                years_experience: calculatedYears,
                                roles: ['b', 'c', 'd', 'f', 'g', 'h'],  // 研究人員預設工作內容
                                trainings: (selectedStaff.trainings || []).map((t: any) => t.code),
                                training_certificates: (selectedStaff.trainings || []).map((t: any) => ({
                                  training_code: t.code,
                                  certificate_no: t.certificate_no || ''
                                }))
                              })
                            }
                          }}
                        >
                          <SelectTrigger className="w-48">
                            <SelectValue placeholder={t('aup.personnel.addDialog.placeholders.name')} />
                          </SelectTrigger>
                          <SelectContent>
                            {staffMembers.map((staff: any) => (
                              <SelectItem key={staff.id} value={staff.id}>
                                {staff.display_name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                        <Input
                          value={newPersonnel.name}
                          onChange={(e) => setNewPersonnel({ ...newPersonnel, name: e.target.value })}
                          placeholder={t('aup.personnel.addDialog.placeholders.name')}
                          className="flex-1"
                        />
                      </div>
                    ) : (
                      <Input
                        value={newPersonnel.name}
                        onChange={(e) => setNewPersonnel({ ...newPersonnel, name: e.target.value })}
                        placeholder={t('aup.personnel.addDialog.placeholders.name')}
                      />
                    )}
                  </div>
                  <div className="space-y-2">
                    <Label>{t('aup.personnel.addDialog.labels.position')}</Label>
                    <Input
                      value={newPersonnel.position}
                      onChange={(e) => setNewPersonnel({ ...newPersonnel, position: e.target.value })}
                      placeholder={t('aup.personnel.addDialog.placeholders.position')}
                    />
                  </div>
                </div>
                <div className="space-y-2">
                  <Label>{t('aup.personnel.addDialog.labels.roles')} *</Label>
                  <div className="flex flex-wrap gap-2">
                    {['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'].map(role => (
                      <div key={role} className="flex items-center space-x-1">
                        <Checkbox
                          id={`new_role_${role}`}
                          checked={newPersonnel.roles.includes(role)}
                          onCheckedChange={(checked) => {
                            if (checked) {
                              setNewPersonnel({ ...newPersonnel, roles: [...newPersonnel.roles, role] })
                            } else {
                              const newRoles = newPersonnel.roles.filter(r => r !== role)
                              setNewPersonnel({
                                ...newPersonnel,
                                roles: newRoles,
                                roles_other_text: role === 'i' ? '' : newPersonnel.roles_other_text
                              })
                            }
                          }}
                        />
                        <Label htmlFor={`new_role_${role}`} className="text-sm font-normal cursor-pointer">{role}</Label>
                      </div>
                    ))}
                  </div>
                  <div className="mt-2 p-3 bg-slate-50 rounded-md">
                    <p className="text-xs text-muted-foreground">
                      {t('aup.personnel.roles.title')}<br />{t('aup.personnel.roles.list')}
                    </p>
                  </div>
                  {newPersonnel.roles.includes('i') && (
                    <Input
                      value={newPersonnel.roles_other_text}
                      onChange={(e) => setNewPersonnel({ ...newPersonnel, roles_other_text: e.target.value })}
                      placeholder={t('aup.personnel.addDialog.placeholders.rolesOther')}
                      className="mt-2"
                    />
                  )}
                </div>
                <div className="space-y-2">
                  <Label>{t('aup.personnel.addDialog.labels.experience')} *</Label>
                  <Input
                    type="number"
                    min="0"
                    step="1"
                    value={newPersonnel.years_experience || ''}
                    onChange={(e) => setNewPersonnel({ ...newPersonnel, years_experience: parseInt(e.target.value) || 0 })}
                    placeholder={t('aup.personnel.addDialog.placeholders.experience')}
                  />
                </div>
                <div className="space-y-2">
                  <Label>{t('aup.personnel.addDialog.labels.trainings')} *</Label>
                  <div className="flex flex-wrap gap-2 mb-2">
                    {[
                      { value: 'A', label: t('aup.personnel.trainings.A') },
                      { value: 'B', label: t('aup.personnel.trainings.B') },
                      { value: 'C', label: t('aup.personnel.trainings.C') },
                      { value: 'D', label: t('aup.personnel.trainings.D') },
                      { value: 'E', label: t('aup.personnel.trainings.E') },
                      { value: 'F', label: t('aup.personnel.trainings.F') }
                    ].map(training => (
                      <div key={training.value} className="flex items-center space-x-1">
                        <Checkbox
                          id={`new_training_${training.value}`}
                          checked={newPersonnel.trainings.includes(training.value)}
                          onCheckedChange={(checked) => {
                            if (checked) {
                              setNewPersonnel({ ...newPersonnel, trainings: [...newPersonnel.trainings, training.value] })
                            } else {
                              const newTrainings = newPersonnel.trainings.filter(t => t !== training.value)
                              setNewPersonnel({
                                ...newPersonnel,
                                trainings: newTrainings,
                                trainings_other_text: training.value === 'F' ? '' : newPersonnel.trainings_other_text,
                                training_certificates: newPersonnel.training_certificates.filter(
                                  cert => cert.training_code !== training.value
                                )
                              })
                            }
                          }}
                        />
                        <Label htmlFor={`new_training_${training.value}`} className="text-xs font-normal cursor-pointer">{training.label}</Label>
                      </div>
                    ))}
                  </div>
                  {newPersonnel.trainings.includes('F') && (
                    <Input
                      value={newPersonnel.trainings_other_text}
                      onChange={(e) => setNewPersonnel({ ...newPersonnel, trainings_other_text: e.target.value })}
                      placeholder={t('aup.personnel.addDialog.placeholders.trainingsOther')}
                      className="mt-2"
                    />
                  )}
                  {/* Show certificate number list for each selected training */}
                  {newPersonnel.trainings.filter(t => t !== 'F').map((trainingCode: string) => {
                    const certificates = newPersonnel.training_certificates.filter(cert => cert.training_code === trainingCode)
                    return (
                      <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                        <Label className="text-xs font-semibold">{trainingCode}:</Label>
                        {certificates.map((cert, certIndex) => {
                          // 找到該證書在完整 training_certificates 數組中的索引
                          let globalCertIndex = -1
                          let count = 0
                          for (let i = 0; i < newPersonnel.training_certificates.length; i++) {
                            if (newPersonnel.training_certificates[i].training_code === trainingCode) {
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
                                value={cert.certificate_no}
                                onChange={(e) => {
                                  const newCerts = [...newPersonnel.training_certificates]
                                  if (globalCertIndex >= 0 && globalCertIndex < newCerts.length) {
                                    newCerts[globalCertIndex].certificate_no = e.target.value
                                    setNewPersonnel({ ...newPersonnel, training_certificates: newCerts })
                                  }
                                }}
                                placeholder={t('aup.personnel.addDialog.placeholders.certNo')}
                                className="text-xs h-7"
                              />
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6 text-red-500"
                                onClick={() => {
                                  const newCerts = [...newPersonnel.training_certificates]
                                  if (globalCertIndex >= 0 && globalCertIndex < newCerts.length) {
                                    newCerts.splice(globalCertIndex, 1)
                                    setNewPersonnel({ ...newPersonnel, training_certificates: newCerts })
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
                            setNewPersonnel({
                              ...newPersonnel,
                              training_certificates: [
                                ...newPersonnel.training_certificates,
                                { training_code: trainingCode, certificate_no: '' }
                              ]
                            })
                          }}
                        >
                          + {t('aup.personnel.addDialog.buttons.addCert')}
                        </Button>
                      </div>
                    )
                  })}
                </div>
              </div>
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => setIsAddPersonnelDialogOpen(false)}
                >
                  {t('aup.personnel.addDialog.buttons.cancel')}
                </Button>
                <Button
                  type="button"
                  onClick={() => {
                    // 驗證必填欄位
                    if (!newPersonnel.name.trim()) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.nameRequired'),
                        variant: 'destructive',
                      })
                      return
                    }
                    if (newPersonnel.roles.length === 0) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.rolesRequired'),
                        variant: 'destructive',
                      })
                      return
                    }
                    if (newPersonnel.roles.includes('i') && !newPersonnel.roles_other_text.trim()) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.rolesOtherRequired'),
                        variant: 'destructive',
                      })
                      return
                    }
                    if (newPersonnel.years_experience <= 0) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.experienceRequired'),
                        variant: 'destructive',
                      })
                      return
                    }
                    if (newPersonnel.trainings.length === 0) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.trainingsRequired'),
                        variant: 'destructive',
                      })
                      return
                    }
                    if (newPersonnel.trainings.includes('F') && !newPersonnel.trainings_other_text.trim()) {
                      toast({
                        title: t('common.error'),
                        description: t('aup.personnel.addDialog.validation.trainingsOtherRequired'),
                        variant: 'destructive',
                      })
                      return
                    }

                    const newPersonnelList = [...(formData.working_content.personnel || []), newPersonnel]
                    setFormData((prev) => ({
                      ...prev,
                      working_content: {
                        ...prev.working_content,
                        personnel: newPersonnelList
                      }
                    }))
                    setIsAddPersonnelDialogOpen(false)
                    toast({
                      title: t('common.success'),
                      description: t('aup.personnel.addDialog.messages.added'),
                    })
                  }}
                >
                  {t('aup.personnel.addDialog.buttons.add')}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>
    </div >
  )
}
