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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAuthUser, useAuthIsGuest } from '@/stores/auth'
import {
  ArrowLeft,
  ArrowRight,
  Save,
  Send,
  Loader2,
} from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { getApiErrorMessage } from '@/lib/apiError'
import {
  normalizeFormVersion,
  PROTOCOL_FORM_VERSIONS,
  PROTOCOL_FORM_VERSION_LABELS,
  type ProtocolFormVersion,
} from '@/lib/constants/protocolVersionManifests'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useUnsavedChangesGuard } from '@/hooks/useUnsavedChangesGuard'
import { UnsavedChangesDialog } from '@/components/UnsavedChangesDialog'

import { ProtocolFormData } from '@/types/protocol'
import type { ValidationResult } from '@/types/aiReview'
import { defaultFormData, sectionKeys } from './protocol-edit/constants'
import { DEMO_PROTOCOL_DETAIL_P1 } from '@/lib/guest-demo/protocols'
import { validateRequiredFields, findNextEmptyField } from './protocol-edit/validation'
import { mergeProtocolData } from './protocol-edit/mergeProtocolData'
import { AddPersonnelDialog } from './protocol-edit/AddPersonnelDialog'
import type { SdFieldMode } from './protocol-edit/SectionBasic'
import { isStaff, userLabel, type UserOption } from './components/import-review/users'
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
  const user = useAuthUser()
  const isGuestUser = useAuthIsGuest()
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
  // 編輯中的人員列 index；null 表示新增模式
  const [editingPersonnelIndex, setEditingPersonnelIndex] = useState<number | null>(null)
  const [isDirty, setIsDirty] = useState(false)
  // 版本名冊：重選版本（僅補登匯入計畫可改）；null=沿用 protocol.source_form_version
  const [versionOverride, setVersionOverride] = useState<ProtocolFormVersion | null>(null)
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
  // 版本名冊：本計畫的表單版本（重選覆寫 > protocol 值 > 最新版）
  const effectiveVersion = versionOverride ?? normalizeFormVersion(protocol?.source_form_version)

  const { data: staffMembers = [] } = useQuery({
    queryKey: ['staff'],
    queryFn: async () => {
      // include_external=true：計畫書人員不是人事作業，外聘獸醫 / 外部委員本來就會
      // 掛在計畫書上。端點預設只回本場受僱人員，這裡要明確放寬。
      const response = await api.get<StaffMember[]>('/hr/staff?include_external=true')
      return response.data
    },
  })

  // ── 計劃負責人（SD）角色化指派 ──────────────────────────────
  // 規則：SD = 本公司內部 EXPERIMENT_STAFF，由執行秘書/admin 指定（客戶/PI 不填）。
  //  · 執秘/admin → 下拉指派任意 staff（select）
  //  · EXPERIMENT_STAFF 本人新增 → 自動帶入自己、唯讀（self）
  //  · 客戶/外部 PI → 隱藏 SD 區塊（hidden），待執秘事後於編輯頁指派
  // GLP 鎖：GLP 計劃一旦已指派 SD 即不可變更（後端亦守一層）。
  const roles = user?.roles ?? []
  const canAssignSd = roles.some((r) => ['IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r))
  const isExperimentStaff = roles.includes('EXPERIMENT_STAFF')
  const sdMode: SdFieldMode = canAssignSd ? 'select' : isExperimentStaff ? 'self' : 'hidden'
  const [sdUserId, setSdUserId] = useState('')

  // SD 候選名單（僅執秘/admin 需要；其餘角色不撈以免無權限 403）
  const { data: assignableUsers = [], isLoading: sdLoading } = useQuery({
    queryKey: ['protocols', 'assignable-users'],
    queryFn: async () => (await api.get<UserOption[]>('/protocols/assignable-users')).data,
    enabled: canAssignSd,
  })
  const sdOptions = assignableUsers.filter(isStaff).map((u) => ({ id: u.id, label: userLabel(u) }))

  // 初始化 SD 選擇：新增 + self → 帶入本人；編輯 → 沿用既有 SD；其餘情境清空，
  // 避免同一元件實例切換 context（新建↔編輯、或無 SD 計畫）時殘留 stale 值被送出。
  useEffect(() => {
    if (isNew) {
      setSdUserId(sdMode === 'self' && user?.id ? user.id : '')
    } else if (protocol?.study_director_user_id) {
      setSdUserId(protocol.study_director_user_id)
    } else {
      setSdUserId('')
    }
  }, [isNew, sdMode, user?.id, protocol?.study_director_user_id])

  const selfLabel = user?.display_name || user?.email || ''
  // GLP 鎖以「已持久化的計畫」GLP 狀態為準（非可變 formData），與後端鎖一致：
  // 使用者本地翻 GLP off 不應在 UI 解鎖（後端仍會拒絕）。
  const persistedIsGlp = protocol?.working_content?.basic?.is_glp === true
  const sdLocked = !isNew && persistedIsGlp && !!protocol?.study_director_user_id
  // self 模式顯示：新增帶本人；編輯顯示既有 SD 名（無則提示待指派）
  const sdSelfLabel = isNew
    ? selfLabel
    : protocolResponse?.sd_name || t('aup.basic.sdPendingAssign')

  useEffect(() => {
    if (!isNew) return
    // R51 follow-up：guest 進 /protocols/new 改為預填 demo 計畫書讓使用者體驗表單流程
    //（fieldset disabled 已擋寫入；左欄 sticky 提醒不會儲存）。
    if (isGuestUser) {
      setFormData((prev) => mergeProtocolData(DEMO_PROTOCOL_DETAIL_P1.protocol, prev, t))
      return
    }
    // 一般 user：保留原本「最小預設值」行為（避免 PI 看到完全空表覺得不友善）
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
  }, [isNew, isGuestUser, t])

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
    // SD 僅在「可設定」時帶上：執秘/admin 下拉值；staff 本人新增帶自己。
    // self 模式編輯（staff 不可改）/ 客戶隱藏 → 省略（後端 COALESCE 保留既有）。
    const sdSubmitId =
      sdMode === 'select' ? sdUserId : sdMode === 'self' && isNew ? user?.id : undefined
    return {
      title: formData.title,
      working_content: { ...formData.working_content, basic: basicContent },
      start_date: formData.start_date || undefined,
      end_date: formData.end_date || undefined,
      study_director_user_id: sdSubmitId || undefined,
      source_form_version: versionOverride ?? undefined,
    }
  }

  const handleSave = (isSubmit = false) => {
    if (isGuestUser) {
      toast({ title: t('guest.demoMode'), description: t('guest.demoSaveBlocked') })
      return
    }
    const validationError = isSubmit
      ? validateRequiredFields(formData, t, effectiveVersion)
      : (!formData.title.trim() ? t('aup.basic.validation.titleRequired') : null)

    if (validationError) {
      toast({ title: t('common.error'), description: validationError, variant: 'destructive' })
      return
    }

    const data = buildSaveData()
    if (isNew) {
      createMutation.mutate(data)
    } else {
      // R30-B: 帶當前 version 防 lost update（version 從 query 結果取，避免 form
      // state 的 stale value）
      updateMutation.mutate(
        { ...data, version: protocol?.version },
        { onSuccess: () => setIsDirty(false) },
      )
    }
  }

  const handleSubmit = async () => {
    if (isGuestUser) {
      toast({ title: t('guest.demoMode'), description: t('guest.demoSubmitBlocked') })
      return
    }
    if (!id) return
    const validationError = validateRequiredFields(formData, t, effectiveVersion)
    if (validationError) {
      toast({ title: t('common.error'), description: validationError, variant: 'destructive' })
      return
    }
    // §7 最小體重 <20kg：送出前二次確認（不阻擋，提醒為體型很小的豬隻）
    const hasSmallPig = formData.working_content.animals.animals?.some(
      a => a.species === 'pig' && !a.weight_unlimited && a.weight_min != null && a.weight_min < 20,
    )
    if (hasSmallPig) {
      const proceed = await confirm({
        title: t('aup.animals.labels.smallPigConfirmTitle'),
        description: t('aup.animals.labels.smallPigConfirmDesc'),
        confirmLabel: t('common.confirm'),
      })
      if (!proceed) return
    }
    const data = buildSaveData()
    // R30-B: 帶當前 version 防 lost update
    updateMutation.mutate({ ...data, version: protocol?.version }, {
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
      if (path === '') {
        // 頂層純量欄位（如 document_archiving）：section 本身即為值
        ;(newContent as Record<string, unknown>)[section] = value
        return { ...prev, working_content: newContent as FormData['working_content'] }
      }
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
    formVersion: effectiveVersion,
    refinementUpgradeHint:
      !!protocol?.source_form_version &&
      normalizeFormVersion(protocol.source_form_version) !== 'F' &&
      effectiveVersion === 'F',
  }

  const sectionComponents: Record<string, React.ReactNode> = {
    // 匯入計畫：補登中（import_pending）仍可改研究資料，完成補登後才鎖定（之後走 amendment）。
    // 對齊後端 update：import_pending 期間允許編輯 working_content。
    basic: (
      <SectionBasic
        {...sectionProps}
        disabled={!!protocol?.imported_at && !protocol?.import_pending}
        sdMode={sdMode}
        sdUserId={sdUserId}
        onSdChange={(v) => { setSdUserId(v); setIsDirty(true) }}
        sdOptions={sdOptions}
        sdSelfLabel={sdSelfLabel}
        sdLocked={sdLocked}
        sdLoading={sdLoading}
      />
    ),
    purpose: <SectionPurpose {...sectionProps} />,
    items: <SectionItems {...sectionProps} />,
    design: <SectionDesign {...sectionProps} />,
    guidelines: <SectionGuidelines {...sectionProps} />,
    surgery: <SectionSurgery {...sectionProps} />,
    animals: <SectionAnimals {...sectionProps} />,
    personnel: (
      <SectionPersonnel
        {...sectionProps}
        isExternal={!!protocol?.imported_at}
        onAddPersonnel={() => {
          setEditingPersonnelIndex(null)
          setIsAddPersonnelDialogOpen(true)
        }}
        onEditPersonnel={(index) => {
          setEditingPersonnelIndex(index)
          setIsAddPersonnelDialogOpen(true)
        }}
      />
    ),
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
            <div className="flex gap-2">
              {/* R32: guest 看得到按鈕但點擊 no-op + toast（demo 體驗用，不能真寫 DB）*/}
              <Button
                size="sm"
                variant="outline"
                onClick={() => handleSave()}
                disabled={!isGuestUser && (createMutation.isPending || updateMutation.isPending)}
                title={isGuestUser ? t('guest.demoSaveBlocked') : undefined}
              >
                {!isGuestUser && (createMutation.isPending || updateMutation.isPending) ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Save className="mr-2 h-4 w-4" />
                )}
                {t('aup.saveDraft')}
              </Button>
              {!isNew && id && !isGuestUser && (
                <AIReviewButton protocolId={id} />
              )}
              {!isNew && (
                <Button
                  size="sm"
                  onClick={handleSubmit}
                  disabled={!isGuestUser && (submitMutation.isPending || isValidating)}
                  title={isGuestUser ? t('guest.demoSubmitBlocked') : undefined}
                >
                  {!isGuestUser && (submitMutation.isPending || isValidating) ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Send className="mr-2 h-4 w-4" />
                  )}
                  {t('aup.submitForReview')}
                </Button>
              )}
            </div>
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
        <div className="space-y-4">
          {/* R51 follow-up：guest 進 /protocols/new 的 demo 提醒 — sticky 跟著 nav 一起捲動 */}
          {isGuestUser && isNew && (
            <Card className="border-status-warning-border bg-status-warning-bg">
              <CardContent className="p-4 space-y-2">
                <div className="font-semibold text-status-warning-text">Demo 模式</div>
                <p className="text-sm text-status-warning-text">
                  以下表單已填入範例計畫書供你體驗，<strong>輸入內容不會被儲存</strong>，所有欄位為唯讀。
                </p>
                <p className="text-xs text-status-warning-text/80">
                  正式使用請登入 PI 帳號後在「計畫書管理」建立新計畫書。
                </p>
              </CardContent>
            </Card>
          )}
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
              const next = findNextEmptyField(formData, t, effectiveVersion)
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
        </div>

        <div className="space-y-6">
          {/* R49 follow-up：guest 看完整表單但全部 input disabled（fieldset 原生禁用後代 form controls）*/}
          <fieldset disabled={isGuestUser} className="border-0 m-0 p-0 min-w-0 disabled:opacity-70">
            {protocol?.imported_at && (
              <div className="mb-4 flex flex-wrap items-center gap-2 rounded-md border bg-muted/40 p-3">
                <span className="text-sm font-medium">計畫書版本</span>
                <Select
                  value={effectiveVersion}
                  onValueChange={(v) => { setVersionOverride(v as ProtocolFormVersion); setIsDirty(true) }}
                  disabled={!protocol?.import_pending}
                >
                  <SelectTrigger className="w-auto min-w-[220px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {PROTOCOL_FORM_VERSIONS.map((v) => (
                      <SelectItem key={v} value={v}>{PROTOCOL_FORM_VERSION_LABELS[v]}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <span className="text-xs text-muted-foreground">
                  依此版本顯示對應欄位；改版後請儲存。{!protocol?.import_pending && '（完成補登後鎖定）'}
                </span>
              </div>
            )}
            <Suspense fallback={<SectionFallback />}>
              {sectionComponents[activeSection]}
            </Suspense>
          </fieldset>
        </div>
      </div>

      <AddPersonnelDialog
        open={isAddPersonnelDialogOpen}
        onOpenChange={(open) => {
          setIsAddPersonnelDialogOpen(open)
          if (!open) setEditingPersonnelIndex(null)
        }}
        staffMembers={staffMembers}
        isIACUCStaff={isIACUCStaff ?? false}
        editingIndex={editingPersonnelIndex}
        editingPerson={
          editingPersonnelIndex != null
            ? formData.working_content.personnel?.[editingPersonnelIndex]
            : null
        }
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
        onSave={(index, personnel) => {
          setIsDirty(true)
          setFormData((prev) => {
            const list = [...(prev.working_content.personnel || [])]
            // 保留既有 id 等欄位，僅以表單內容覆蓋可編輯欄位
            list[index] = { ...list[index], ...personnel }
            return {
              ...prev,
              working_content: { ...prev.working_content, personnel: list },
            }
          })
        }}
      />
      <ConfirmDialog state={dialogState} />
      <UnsavedChangesDialog isBlocked={isBlocked} onProceed={proceed} onReset={reset} />
    </div>
  )
}
