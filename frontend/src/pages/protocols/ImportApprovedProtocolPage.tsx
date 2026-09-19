import { useState, useEffect } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Loader2 } from 'lucide-react'

import api from '@/lib/api'
import { importApprovedProtocol, type ImportApprovedProtocolRequest } from '@/lib/api/protocol'
import {
  PROTOCOL_FORM_VERSIONS,
  PROTOCOL_FORM_VERSION_LABELS,
  LATEST_PROTOCOL_FORM_VERSION,
  normalizeFormVersion,
} from '@/lib/constants/protocolVersionManifests'
import { PageHeader } from '@/components/ui/page-header'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

import { PiSelector } from './components/import-review/PiSelector'
import { MilestonesPicker } from './components/import-review/MilestonesPicker'
import { DateRangePicker } from './components/import-review/DateRangePicker'
import {
  EMPTY_EXTERNAL_PI,
  type ExternalPiData,
} from './components/import-review/externalPi'
import { isStaff, userLabel, PI_OTHER, type UserOption } from './components/import-review/users'
import { ResearchBasicFields } from './protocol-edit/ResearchBasicFields'
import { defaultFormData } from './protocol-edit/constants'
import { useAuthUser, useAuthHasRole } from '@/stores/auth'
import type { ProtocolWorkingContent } from '@/types/protocol'
import {
  MILESTONES,
  EMPTY_MILESTONES,
  milestonesOutOfOrder,
  milestonePayload,
  type MilestoneState,
} from './components/import-review/milestones'

/**
 * 匯入已核准計劃：場內既有、已通過審查的計劃補登進系統成 APPROVED（跳過審查流程）。
 * PI 可選系統內使用者（排除試驗工作人員）或「其他」填外部 PI / 委託單位資訊。
 * 匯入後導向補登作業頁繼續補登內容與審查文件。
 */
export function ImportApprovedProtocolPage() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [title, setTitle] = useState('')
  const [piUserId, setPiUserId] = useState('')
  const [externalPi, setExternalPi] = useState<ExternalPiData>(EMPTY_EXTERNAL_PI)
  const [sdUserId, setSdUserId] = useState('')
  const [iacucNo, setIacucNo] = useState('')
  const [applicationNo, setApplicationNo] = useState('')
  // 版本名冊：先選版本 → 決定 source_form_version（驅動補登依版本顯示欄位）
  const [sourceFormVersion, setSourceFormVersion] = useState<string>(LATEST_PROTOCOL_FORM_VERSION)
  const [endDate, setEndDate] = useState('')
  const [milestones, setMilestones] = useState<MilestoneState>(EMPTY_MILESTONES)
  const [remark, setRemark] = useState('')
  // 計畫起始日 = 計畫核准通過日（唯讀，自里程碑帶入，不另填）
  const startDate = milestones.approved_at

  // C2：研究資料整段於匯入頁一次填入 working_content.basic（PI 由上方 PiSelector 提供，不重複）
  // 試驗機構 / 飼養位置預填：取自 i18n 預設（aup.defaults.*），與編輯頁 ProtocolEditPage 同一來源
  const [basic, setBasic] = useState<ProtocolWorkingContent['basic']>(() => ({
    ...defaultFormData.working_content.basic,
    facility: {
      ...defaultFormData.working_content.basic.facility,
      title: t('aup.defaults.facilityName'),
    },
    housing_location: t('aup.defaults.housingLocation'),
  }))
  const updateBasic = (path: string, value: unknown) =>
    setBasic((prev) => {
      const next: Record<string, unknown> = { ...(prev as unknown as Record<string, unknown>) }
      if (path.includes('.')) {
        const parts = path.split('.')
        let cur = next
        for (let i = 0; i < parts.length - 1; i++) {
          cur[parts[i]] = { ...(cur[parts[i]] as Record<string, unknown>) }
          cur = cur[parts[i]] as Record<string, unknown>
        }
        cur[parts[parts.length - 1]] = value
      } else {
        next[path] = value
      }
      return next as unknown as ProtocolWorkingContent['basic']
    })

  // PI / SD 下拉名單取自 /protocols/assignable-users（門檻同匯入權限，非 admin.user.view），
  // 故所有匯入者（含 EXPERIMENT_STAFF）皆可下拉選系統內 PI，或選「其他」填外部 PI 等 admin 邀請。
  // SD（計劃負責人）授權較嚴：僅執行秘書(IACUC_STAFF)/管理員可指派任意工作人員；
  // 一般匯入者只能設自己為 SD（後端 import_approved 亦強制此授權）。
  const currentUser = useAuthUser()
  const hasRole = useAuthHasRole()
  const canSelectAllUsers = hasRole('admin') || hasRole('SYSTEM_ADMIN') || hasRole('IACUC_STAFF')

  const {
    data: users,
    isLoading: usersLoading,
    isError: usersError,
    error: usersErrorValue,
  } = useQuery({
    queryKey: ['protocols', 'assignable-users'],
    queryFn: async () => (await api.get<UserOption[]>('/protocols/assignable-users')).data,
  })
  // 名單載入失敗時明確提示（避免 PI/SD 下拉靜默變空、表單卡死又無原因）
  const userLoadError = usersError
    ? getApiErrorMessage(usersErrorValue, t('protocolPages.importApproved.usersLoadFailed'))
    : null

  // PI 候選 = 系統內非試驗工作人員（對所有匯入者開放）
  const piOptions = (users ?? []).filter((u) => !isStaff(u))
  // SD 候選 = 試驗工作人員；非執秘/admin 限本人
  const sdOptions: UserOption[] = canSelectAllUsers
    ? (users ?? []).filter(isStaff)
    : (users ?? []).filter((u) => u.id === currentUser?.id && isStaff(u))
  const isExternalPi = piUserId === PI_OTHER

  // staff 自行補登：非執秘/admin 時自動帶入本人為 SD（唯讀，不可改他人）
  const selfSdUser = (users ?? []).find((u) => u.id === currentUser?.id && isStaff(u))
  useEffect(() => {
    if (!canSelectAllUsers && selfSdUser && !sdUserId) {
      setSdUserId(selfSdUser.id)
    }
  }, [canSelectAllUsers, selfSdUser, sdUserId])

  // 「同計畫主持人」帶入來源：匯入時 PI 尚未寫入 basic.pi，提供即時 PI 值給聯絡人繼承
  // （外部 PI 取填寫值；系統內 PI 取使用者顯示名/email，電話下拉無資料故留空）
  const selectedPiUser = (users ?? []).find((u) => u.id === piUserId)
  const piOverride = isExternalPi
    ? { name: externalPi.piName.trim(), phone: externalPi.piPhone.trim(), email: externalPi.piEmail.trim() }
    : { name: selectedPiUser ? userLabel(selectedPiUser) : '', phone: '', email: selectedPiUser?.email ?? '' }

  const importMutation = useMutation({
    mutationFn: (data: ImportApprovedProtocolRequest) => importApprovedProtocol(data),
    onSuccess: (created: { id?: string }) => {
      toast({ title: t('common.success'), description: t('protocolPages.importApproved.importSuccess') })
      navigate(created?.id ? `/protocols/${created.id}/import-review` : '/protocols')
    },
    onError: (err: unknown) =>
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('protocolPages.importApproved.importFailed')), variant: 'destructive' }),
  })

  const validate = (): string | null => {
    if (!title.trim()) return t('protocolPages.importApproved.validation.titleRequired')
    if (!piUserId) return t('protocolPages.importApproved.validation.piRequired')
    if (isExternalPi) {
      if (!externalPi.piName.trim()) return t('protocolPages.importApproved.validation.externalPiNameRequired')
      if (!externalPi.piEmail.trim()) return t('protocolPages.importApproved.validation.externalPiEmailRequired')
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(externalPi.piEmail.trim())) return t('protocolPages.importApproved.validation.externalPiEmailInvalid')
      if (!externalPi.piPhone.trim()) return t('protocolPages.importApproved.validation.externalPiPhoneRequired')
    }
    if (!sdUserId) return t('protocolPages.importApproved.validation.sdRequired')
    if (!iacucNo.trim()) return t('protocolPages.importApproved.validation.iacucNoRequired')
    if (!applicationNo.trim()) return t('protocolPages.importApproved.validation.applicationNoRequired')
    if (!endDate) return t('protocolPages.importApproved.validation.endDateRequired')
    if (startDate && endDate < startDate) return t('protocolPages.importApproved.validation.endDateBeforeStart')
    const missing = MILESTONES.find((m) => m.required && !milestones[m.key])
    if (missing) return t('protocolPages.importApproved.validation.milestoneRequired', { label: t(missing.labelKey) })
    if (milestonesOutOfOrder(milestones)) {
      return t('protocolPages.importApproved.validation.milestonesOutOfOrder')
    }
    return null
  }

  const handleSubmit = () => {
    const error = validate()
    if (error) {
      toast({ title: t('common.error'), description: error, variant: 'destructive' })
      return
    }
    // PI 資料由 PiSelector 來源衍生（同上方 piOverride，DRY），併入研究資料 basic.pi，避免重複輸入（D5）
    importMutation.mutate({
      title: title.trim(),
      pi_user_id: isExternalPi ? null : piUserId,
      working_content: { basic: { ...basic, pi: { ...basic.pi, ...piOverride } } },
      study_director_user_id: sdUserId,
      iacuc_no: iacucNo.trim(),
      application_no: applicationNo.trim() || null,
      start_date: startDate || null,
      end_date: endDate || null,
      ...milestonePayload(milestones),
      remark: remark.trim() || null,
      source_form_version: sourceFormVersion,
    })
  }

  return (
    <div className="space-y-6">
      <div>
        <Link to="/protocols" className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />{t('protocolPages.importApproved.backToList')}
        </Link>
      </div>

      <PageHeader
        title={t('protocolPages.importApproved.title')}
        description={t('protocolPages.importApproved.description')}
      />

      <div className="max-w-3xl space-y-4 rounded-lg border bg-card p-6">
        {userLoadError && (
          <p className="text-sm text-destructive">{userLoadError}</p>
        )}
        <div className="grid gap-2">
          <Label>{t('protocolPages.importApproved.fields.title')}</Label>
          <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder={t('protocolPages.importApproved.fields.titlePlaceholder')} />
        </div>

        <PiSelector
          piUserId={piUserId}
          onPiChange={setPiUserId}
          usersLoading={usersLoading}
          piOptions={piOptions}
          isExternalPi={isExternalPi}
          externalPi={externalPi}
          onExternalPiChange={setExternalPi}
        />

        <div className="grid gap-2">
          <Label>{t('protocolPages.importApproved.fields.sd')}</Label>
          {canSelectAllUsers ? (
            <Select value={sdUserId} onValueChange={setSdUserId} disabled={usersLoading}>
              <SelectTrigger>
                <SelectValue placeholder={usersLoading ? t('protocolPages.shared.loadingEllipsis') : t('protocolPages.importApproved.fields.sdPlaceholder')} />
              </SelectTrigger>
              <SelectContent>
                {sdOptions.map((u) => (
                  <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            // staff 自行補登：自動帶入本人、唯讀（不可指派他人）
            <Input value={selfSdUser ? userLabel(selfSdUser) : (usersLoading ? t('protocolPages.shared.loadingEllipsis') : '—')} readOnly disabled />
          )}
          <p className="text-sm text-muted-foreground">
            {canSelectAllUsers
              ? t('protocolPages.importApproved.fields.sdHintAll')
              : t('protocolPages.importApproved.fields.sdHintSelf')}
          </p>
        </div>

        <div className="grid gap-2">
          <Label>{t('protocolPages.importApproved.fields.iacucNo')}</Label>
          <Input value={iacucNo} onChange={(e) => setIacucNo(e.target.value)} placeholder={t('protocolPages.importApproved.fields.iacucNoPlaceholder')} />
        </div>

        <div className="grid gap-2">
          <Label>{t('protocolPages.importApproved.fields.applicationNo')}</Label>
          <Input value={applicationNo} onChange={(e) => setApplicationNo(e.target.value)} placeholder={t('protocolPages.importApproved.fields.applicationNoPlaceholder')} />
        </div>

        <div className="grid gap-2">
          <Label>{t('protocolPages.importApproved.fields.formVersion')}</Label>
          <Select value={sourceFormVersion} onValueChange={setSourceFormVersion}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PROTOCOL_FORM_VERSIONS.map((v) => (
                <SelectItem key={v} value={v}>{PROTOCOL_FORM_VERSION_LABELS[v]}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-sm text-muted-foreground">
            {t('protocolPages.importApproved.fields.formVersionHint')}
          </p>
        </div>

        {/* C2：研究資料整段於匯入時一次填入（匯入後於編輯頁鎖定，改錯需刪除重匯入） */}
        <div className="space-y-3 border-t pt-4">
          <div>
            <h3 className="text-base font-semibold">{t('protocolPages.importApproved.researchInfo.title')}</h3>
            <p className="text-sm text-muted-foreground">{t('protocolPages.importApproved.researchInfo.hint')}</p>
          </div>
          <ResearchBasicFields basic={basic} onUpdate={updateBasic} piOverride={piOverride} formVersion={normalizeFormVersion(sourceFormVersion)} />
        </div>

        <MilestonesPicker value={milestones} onChange={setMilestones} />

        <DateRangePicker
          startDate={startDate}
          endDate={endDate}
          onEndChange={setEndDate}
        />

        <div className="grid gap-2">
          <Label>{t('protocolPages.shared.remark')}</Label>
          <Textarea value={remark} onChange={(e) => setRemark(e.target.value)} rows={2} placeholder={t('protocolPages.importApproved.fields.remarkPlaceholder')} />
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="outline" asChild>
            <Link to="/protocols">{t('common.cancel')}</Link>
          </Button>
          <Button onClick={handleSubmit} disabled={importMutation.isPending}>
            {importMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('protocolPages.importApproved.submit')}
          </Button>
        </div>
      </div>
    </div>
  )
}
