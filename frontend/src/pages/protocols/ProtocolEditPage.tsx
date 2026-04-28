import { lazy, Suspense, useState, useEffect } from 'react'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
  ProtocolResponse,
  CreateProtocolRequest,
  UpdateProtocolRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { useAuthStore } from '@/stores/auth'
import {
  ArrowLeft,
  ArrowRight,
  Save,
  Send,
  Loader2,
} from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { getApiErrorMessage } from '@/lib/validation'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useUnsavedChangesGuard } from '@/hooks/useUnsavedChangesGuard'
import { UnsavedChangesDialog } from '@/components/UnsavedChangesDialog'

import { ProtocolFormData } from '@/types/protocol'
import type { ValidationResult } from '@/types/aiReview'
import { defaultFormData, sectionKeys } from './protocol-edit/constants'
import { validateRequiredFields, findNextEmptyField } from './protocol-edit/validation'
import { mergeProtocolData } from './protocol-edit/mergeProtocolData'
import { AddPersonnelDialog } from './protocol-edit/AddPersonnelDialog'
import { ValidationPanel } from '@/components/protocol/ValidationPanel'
import { AIReviewButton } from '@/components/protocol/AIReviewButton'
import { AIReviewPanel } from '@/components/protocol/AIReviewPanel'
import { aiReviewApi } from '@/lib/api'

import { Skeleton } from '@/components/ui/skeleton'

const SectionBasic = lazy(() => import('./protocol-edit/SectionBasic').then(m => ({ default: m.SectionBasic })))
const SectionPurpose = lazy(() => import('./protocol-edit/SectionPurpose').then(m => ({ default: m.SectionPurpose })))
const SectionItems = lazy(() => import('./protocol-edit/SectionItems').then(m => ({ default: m.SectionItems })))
const SectionDesign = lazy(() => import('./protocol-edit/SectionDesign').then(m => ({ default: m.SectionDesign })))
const SectionGuidelines = lazy(() => import('./protocol-edit/SectionGuidelines').then(m => ({ default: m.SectionGuidelines })))
const SectionSurgery = lazy(() => import('./protocol-edit/SectionSurgery').then(m => ({ default: m.SectionSurgery })))
const SectionAnimals = lazy(() => import('./protocol-edit/SectionAnimals').then(m => ({ default: m.SectionAnimals })))
const SectionPersonnel = lazy(() => import('./protocol-edit/SectionPersonnel').then(m => ({ default: m.SectionPersonnel })))
const SectionAttachments = lazy(() => import('./protocol-edit/SectionAttachments').then(m => ({ default: m.SectionAttachments })))
const SectionSignature = lazy(() => import('./protocol-edit/SectionSignature').then(m => ({ default: m.SectionSignature })))

const SectionFallback = () => <Skeleton variant="form" fields={4} />

type FormData = ProtocolFormData

interface StaffMember {
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
}

export function ProtocolEditPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user, isGuest } = useAuthStore()
  const isGuestUser = isGuest()
  const isNew = !id

  const [searchParams, setSearchParams] = useSearchParams()
  const validKeys = sectionKeys.map(s => s.key)
  const sectionParam = searchParams.get('section')
  const activeSection = sectionParam && validKeys.includes(sectionParam) ? sectionParam : 'basic'
  const setActiveSection = (key: string) => {
    setSearchParams({ section: key }, { replace: false })
  }
  const [formData, setFormData] = useState<FormData>(defaultFormData)
  const [isAddPersonnelDialogOpen, setIsAddPersonnelDialogOpen] = useState(false)
  const [isDirty, setIsDirty] = useState(false)
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null)
  const [isValidating, setIsValidating] = useState(false)

  const { isBlocked, proceed, reset } = useUnsavedChangesGuard(isDirty)
  const { dialogState, confirm } = useConfirmDialog()
  const isIACUCStaff = user?.roles?.some(r => ['IACUC_STAFF', 'SYSTEM_ADMIN'].includes(r))

  const { data: protocolResponse, isLoading } = useQuery({
    queryKey: ['protocol', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !isNew,
  })

  const protocol = protocolResponse?.protocol

  const { data: staffMembers = [] } = useQuery({
    queryKey: ['staff'],
    queryFn: async () => {
      const response = await api.get<StaffMember[]>('/hr/staff')
      return response.data
    },
  })

  useEffect(() => {
    if (isNew) {
      setFormData((prev) => {
        const updated = { ...prev }
        if (!updated.working_content.basic.facility?.title?.trim()) {
          updated.working_content.basic.facility = {
            ...updated.working_content.basic.facility,
            title: t('aup.defaults.facilityName'),
          }
        }
        if (!updated.working_content.basic.housing_location?.trim()) {
          updated.working_content.basic.housing_location = t('aup.defaults.housingLocation')
        }
        if (!updated.working_content.design.endpoints.humane_endpoint?.trim()) {
          updated.working_content.design.endpoints.humane_endpoint = t('aup.defaults.humaneEndpoint')
        }
        if (!updated.working_content.design.carcass_disposal.method?.trim()) {
          updated.working_content.design.carcass_disposal.method = t('aup.defaults.carcassDisposal')
        }
        return updated
      })
    }
  }, [isNew, t])

  useEffect(() => {
    if (protocol) {
      setFormData((prev) => mergeProtocolData(protocol, prev, t))
    }
  }, [protocol, t])

  const createMutation = useMutation({
    mutationFn: async (data: CreateProtocolRequest) => api.post('/protocols', data),
    onSuccess: (response) => {
      setIsDirty(false)
      toast({ title: t('common.success'), description: t('aup.messages.created') })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
      navigate(`/protocols/${response.data.id}`)
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('aup.messages.createFailed')), variant: 'destructive' })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async (data: UpdateProtocolRequest) => api.put(`/protocols/${id}`, data),
    onSuccess: () => {
      setIsDirty(false)
      toast({ title: t('common.success'), description: t('aup.messages.saved') })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('aup.messages.saveFailed')), variant: 'destructive' })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => api.post(`/protocols/${id}/submit`),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('aup.messages.submitted') })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      navigate(`/protocols/${id}`)
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('aup.messages.submitFailed')), variant: 'destructive' })
    },
  })

  const buildSaveData = () => {
    const basicContent = {
      ...formData.working_content.basic,
      study_title: formData.title,
      start_date: formData.start_date,
      end_date: formData.end_date,
    }
    if (!isIACUCStaff) {
      basicContent.apply_study_number = ''
    }
    return {
      title: formData.title,
      working_content: { ...formData.working_content, basic: basicContent },
      start_date: formData.start_date || undefined,
      end_date: formData.end_date || undefined,
    }
  }

  const handleSave = (isSubmit = false) => {
    const validationError = isSubmit
      ? validateRequiredFields(formData, t)
      : (!formData.title.trim() ? t('aup.basic.validation.titleRequired') : null)

    if (validationError) {
      toast({ title: t('common.error'), description: validationError, variant: 'destructive' })
      return
    }

    const data = buildSaveData()
    if (isNew) {
      createMutation.mutate(data)
    } else {
      updateMutation.mutate(data, { onSuccess: () => setIsDirty(false) })
    }
  }

  const handleSubmit = async () => {
    if (!id) return
    const validationError = validateRequiredFields(formData, t)
    if (validationError) {
      toast({ title: t('common.error'), description: validationError, variant: 'destructive' })
      return
    }
    const data = buildSaveData()
    updateMutation.mutate(data, {
      onSuccess: async () => {
        setIsDirty(false)
        // R20-3: 先呼叫 validate endpoint
        setIsValidating(true)
        try {
          const result = await aiReviewApi.validate(id)
          setIsValidating(false)
          if (result.errors.length > 0) {
            setValidationResult(result)
            return
          }
          if (result.warnings.length > 0) {
            setValidationResult(result)
            return
          }
        } catch {
          setIsValidating(false)
          // 驗證 API 失敗不阻擋提交
        }
        const ok = await confirm({ title: '送出計畫書', description: t('aup.messages.confirmSubmit'), confirmLabel: '確認送出' })
        if (ok) submitMutation.mutate()
      },
    })
  }

  const handleIgnoreAndSubmit = async () => {
    setValidationResult(null)
    const ok = await confirm({ title: '送出計畫書', description: t('aup.messages.confirmSubmit'), confirmLabel: '確認送出' })
    if (ok) submitMutation.mutate()
  }

  const updateWorkingContent = (section: keyof FormData['working_content'], path: string, value: unknown) => {
    setIsDirty(true)
    setFormData((prev) => {
      const newContent = { ...prev.working_content }
      const sectionData: Record<string, unknown> = { ...(newContent[section] as Record<string, unknown>) }
      if (path.includes('.')) {
        const parts = path.split('.')
        let current = sectionData as Record<string, unknown>
        for (let i = 0; i < parts.length - 1; i++) {
          current[parts[i]] = { ...(current[parts[i]] as Record<string, unknown>) }
          current = current[parts[i]] as Record<string, unknown>
        }
        current[parts[parts.length - 1]] = value
      } else {
        sectionData[path] = value
      }
      ;(newContent as Record<string, unknown>)[section] = sectionData
      return { ...prev, working_content: newContent as FormData['working_content'] }
    })
  }

  const sectionProps = {
    formData,
    updateWorkingContent,
    setFormData,
    t,
    isIACUCStaff: isIACUCStaff ?? false,
    isNew,
  }

  const sectionComponents: Record<string, React.ReactNode> = {
    basic: <SectionBasic {...sectionProps} />,
    purpose: <SectionPurpose {...sectionProps} />,
    items: <SectionItems {...sectionProps} />,
    design: <SectionDesign {...sectionProps} />,
    guidelines: <SectionGuidelines {...sectionProps} />,
    surgery: <SectionSurgery {...sectionProps} />,
    animals: <SectionAnimals {...sectionProps} />,
    personnel: <SectionPersonnel {...sectionProps} onAddPersonnel={() => setIsAddPersonnelDialogOpen(true)} />,
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
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate(-1)} aria-label="返回">
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <PageHeader
          title={isNew ? t('aup.newProtocol') : t('aup.editProtocol')}
          className="flex-1"
          actions={
            !isGuestUser && (
              <div className="flex gap-2">
                <Button size="sm" variant="outline" onClick={() => handleSave()} disabled={createMutation.isPending || updateMutation.isPending}>
                  {createMutation.isPending || updateMutation.isPending ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Save className="mr-2 h-4 w-4" />
                  )}
                  {t('aup.saveDraft')}
                </Button>
                {!isNew && id && (
                  <AIReviewButton protocolId={id} />
                )}
                {!isNew && (
                  <Button size="sm" onClick={handleSubmit} disabled={submitMutation.isPending || isValidating}>
                    {(submitMutation.isPending || isValidating) ? (
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    ) : (
                      <Send className="mr-2 h-4 w-4" />
                    )}
                    {t('aup.submitForReview')}
                  </Button>
                )}
              </div>
            )
          }
        />
      </div>

      {/* R20-3: Validation Panel */}
      {validationResult && (
        <ValidationPanel
          result={validationResult}
          hasErrors={validationResult.errors.length > 0}
          onDismiss={() => setValidationResult(null)}
          onIgnoreAndSubmit={validationResult.errors.length === 0 ? handleIgnoreAndSubmit : undefined}
        />
      )}

      {/* R20-6: AI Review Panel */}
      {!isNew && id && (
        <AIReviewPanel protocolId={id} />
      )}

      <div className="grid gap-6 lg:grid-cols-[320px_minmax(0,1fr)]">
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
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted-foreground hover:bg-muted'
                    }`}
                >
                  <section.icon className="h-4 w-4 mt-0.5 shrink-0" />
                  <span className="text-sm font-medium">{t(section.labelKey)}</span>
                </button>
              ))}
            </nav>
            {(() => {
              const next = findNextEmptyField(formData, t)
              if (!next) return null
              return (
                <div className="mt-3 px-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="gap-1 max-w-full"
                    onClick={() => setActiveSection(next.section)}
                  >
                    <span className="text-xs">{t('aup.nextEmptyField')}</span>
                    <ArrowRight className="h-3.5 w-3.5 shrink-0" />
                  </Button>
                  <p className="text-[11px] text-muted-foreground mt-1 px-1 truncate">{next.label}</p>
                </div>
              )
            })()}
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Suspense fallback={<SectionFallback />}>
            {sectionComponents[activeSection]}
          </Suspense>
        </div>
      </div>

      <AddPersonnelDialog
        open={isAddPersonnelDialogOpen}
        onOpenChange={setIsAddPersonnelDialogOpen}
        staffMembers={staffMembers}
        isIACUCStaff={isIACUCStaff ?? false}
        onAdd={(personnel) => {
          setIsDirty(true)
          setFormData((prev) => ({
            ...prev,
            working_content: {
              ...prev.working_content,
              personnel: [...(prev.working_content.personnel || []), personnel],
            },
          }))
        }}
      />
      <ConfirmDialog state={dialogState} />
      <UnsavedChangesDialog isBlocked={isBlocked} onProceed={proceed} onReset={reset} />
    </div>
  )
}
