import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
  ProtocolResponse,
  CreateProtocolRequest,
  UpdateProtocolRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
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
import { FileInfo } from '@/components/ui/file-upload'
import {
  ArrowLeft,
  Save,
  Send,
  Loader2,
} from 'lucide-react'

// 從拆分的模組匯入
import { ProtocolFormData } from '@/types/protocol'
import { defaultFormData, sectionKeys } from './protocol-edit/constants'
import { validateRequiredFields } from './protocol-edit/validation'
import { deepMerge } from './protocol-edit/utils'

// 匯入各 Section 元件
import { SectionBasic } from './protocol-edit/SectionBasic'
import { SectionPurpose } from './protocol-edit/SectionPurpose'
import { SectionItems } from './protocol-edit/SectionItems'
import { SectionDesign } from './protocol-edit/SectionDesign'
import { SectionGuidelines } from './protocol-edit/SectionGuidelines'
import { SectionSurgery } from './protocol-edit/SectionSurgery'
import { SectionAnimals } from './protocol-edit/SectionAnimals'
import { SectionPersonnel } from './protocol-edit/SectionPersonnel'
import { SectionAttachments } from './protocol-edit/SectionAttachments'
import { SectionSignature } from './protocol-edit/SectionSignature'

type FormData = ProtocolFormData

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
    trainings_other_text: '',
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
      setFormData(() => {
        // 使用遞歸合併確保新欄位被保留
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
            if (anesthesiaType === 'survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'survival'
            } else if (anesthesiaType === 'non_survival_surgery') {
              mergedWorkingContent.surgery.surgery_type = 'non_survival'
            }
            if (!mergedWorkingContent.surgery.preop_preparation || mergedWorkingContent.surgery.preop_preparation.trim() === '') {
              mergedWorkingContent.surgery.preop_preparation = t('aup.defaults.preop_Preparation')
            }
            if (!mergedWorkingContent.surgery.surgery_description || mergedWorkingContent.surgery.surgery_description.trim() === '') {
              mergedWorkingContent.surgery.surgery_description = t('aup.defaults.surgeryDescription')
            }
            if (!mergedWorkingContent.surgery.monitoring || mergedWorkingContent.surgery.monitoring.trim() === '') {
              mergedWorkingContent.surgery.monitoring = t('aup.defaults.monitoring')
            }
          } else {
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
            if (!mergedWorkingContent.surgery.drugs || mergedWorkingContent.surgery.drugs.length === 0) {
              mergedWorkingContent.surgery.drugs = []
            }
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

  const handleSave = async (isSubmit = false) => {
    const validationError = isSubmit
      ? validateRequiredFields(formData, t)
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
      const basicContent = {
        ...formData.working_content.basic,
        study_title: formData.title,
        start_date: formData.start_date,
        end_date: formData.end_date,
      }

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

  // Section 元件的共用 props
  const sectionProps = {
    formData,
    updateWorkingContent,
    setFormData,
    t,
    isIACUCStaff: isIACUCStaff ?? false,
    isNew,
  }

  // Section 元件對應表
  const sectionComponents: Record<string, React.ReactNode> = {
    basic: <SectionBasic {...sectionProps} />,
    purpose: <SectionPurpose {...sectionProps} />,
    items: <SectionItems {...sectionProps} />,
    design: <SectionDesign {...sectionProps} />,
    guidelines: <SectionGuidelines {...sectionProps} />,
    surgery: <SectionSurgery {...sectionProps} />,
    animals: <SectionAnimals {...sectionProps} />,
    personnel: <SectionPersonnel {...sectionProps} onAddPersonnel={() => {
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
    }} />,
    attachments: <SectionAttachments {...sectionProps} />,
    signature: <SectionSignature {...sectionProps} />,
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
          {sectionComponents[activeSection]}
        </div>
      </div>

      {/* 新增人員對話框 */}
      <Dialog open={isAddPersonnelDialogOpen} onOpenChange={setIsAddPersonnelDialogOpen}>
        <DialogContent className="max-w-4xl">
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
                          let calculatedYears = selectedStaff.years_experience || 0
                          if (selectedStaff.entry_date) {
                            const entryYear = new Date(selectedStaff.entry_date).getFullYear()
                            const currentYear = new Date().getFullYear()
                            calculatedYears = currentYear - entryYear
                          }
                          setNewPersonnel({
                            ...newPersonnel,
                            name: selectedStaff.display_name,
                            position: t('aup.personnel.defaults.researcher'),
                            years_experience: calculatedYears,
                            roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                            trainings: (selectedStaff.trainings || []).map((tr: any) => tr.code),
                            training_certificates: (selectedStaff.trainings || []).map((tr: any) => ({
                              training_code: tr.code,
                              certificate_no: tr.certificate_no || ''
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
                      placeholder={t('aup.personnel.placeholders.name')}
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
                          const newTrainings = newPersonnel.trainings.filter(tr => tr !== training.value)
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
              {newPersonnel.trainings.filter(tr => tr !== 'F').map((trainingCode: string) => {
                const certificates = newPersonnel.training_certificates.filter(cert => cert.training_code === trainingCode)
                return (
                  <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                    <Label className="text-xs font-semibold">{trainingCode}:</Label>
                    {certificates.map((cert, certIndex) => {
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
  )
}
