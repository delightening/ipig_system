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
    ? getApiErrorMessage(usersErrorValue, '可指派使用者名單載入失敗，請重新整理或聯絡管理員')
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
      toast({ title: '成功', description: '已匯入核准計劃，請接續補登審查文件' })
      navigate(created?.id ? `/protocols/${created.id}/import-review` : '/protocols')
    },
    onError: (err: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(err, '匯入失敗'), variant: 'destructive' }),
  })

  const validate = (): string | null => {
    if (!title.trim()) return '計畫名稱為必填'
    if (!piUserId) return '請選擇計畫主持人（PI）'
    if (isExternalPi) {
      if (!externalPi.piName.trim()) return '請填寫外部 PI 姓名'
      if (!externalPi.piEmail.trim()) return '請填寫外部 PI 的 Email'
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(externalPi.piEmail.trim())) return '外部 PI 的 Email 格式不正確'
      if (!externalPi.piPhone.trim()) return '請填寫外部 PI 的電話'
    }
    if (!sdUserId) return '請選擇計劃負責人（Study Director）'
    if (!iacucNo.trim()) return 'IACUC 編號為必填'
    if (!applicationNo.trim()) return '申請編號為必填'
    if (!endDate) return '計畫結束日為必填'
    if (startDate && endDate < startDate) return '計畫結束日不可早於計畫核准通過日（起始日）'
    const missing = MILESTONES.find((m) => m.required && !milestones[m.key])
    if (missing) return `審查里程碑「${missing.label}」為必填`
    if (milestonesOutOfOrder(milestones)) {
      return '審查里程碑日期必須依時序遞增（申請→預審→獸醫→委員一審→補件→委員二審→核准）'
    }
    return null
  }

  const handleSubmit = () => {
    const error = validate()
    if (error) {
      toast({ title: '錯誤', description: error, variant: 'destructive' })
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
          <ArrowLeft className="h-4 w-4 mr-2" />回到計畫列表
        </Link>
      </div>

      <PageHeader
        title="匯入已核准計劃"
        description="補登場內既有、已通過審查的計劃（直接核准、不再經審查流程）。匯入後即可關聯 PI / 豬隻並進行會計與管理。"
      />

      <div className="max-w-3xl space-y-4 rounded-lg border bg-card p-6">
        {userLoadError && (
          <p className="text-sm text-destructive">{userLoadError}</p>
        )}
        <div className="grid gap-2">
          <Label>計畫名稱 *</Label>
          <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="例：異種移植豬腎臟灌流研究" />
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
          <Label>計劃負責人（Study Director）*</Label>
          {canSelectAllUsers ? (
            <Select value={sdUserId} onValueChange={setSdUserId} disabled={usersLoading}>
              <SelectTrigger>
                <SelectValue placeholder={usersLoading ? '載入中…' : '選擇計劃負責人（限試驗工作人員）'} />
              </SelectTrigger>
              <SelectContent>
                {sdOptions.map((u) => (
                  <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            // staff 自行補登：自動帶入本人、唯讀（不可指派他人）
            <Input value={selfSdUser ? userLabel(selfSdUser) : (usersLoading ? '載入中…' : '—')} readOnly disabled />
          )}
          <p className="text-sm text-muted-foreground">
            {canSelectAllUsers
              ? '本公司負責執行的試驗工作人員（EXPERIMENT_STAFF）；亦為補登期間可編輯本計劃者。'
              : '已自動帶入您本人為計劃負責人（僅執行秘書／管理員可指派他人）。'}
          </p>
        </div>

        <div className="grid gap-2">
          <Label>IACUC 編號 *（既有核准編號）</Label>
          <Input value={iacucNo} onChange={(e) => setIacucNo(e.target.value)} placeholder="例：PIG-115001" />
        </div>

        <div className="grid gap-2">
          <Label>申請編號 *</Label>
          <Input value={applicationNo} onChange={(e) => setApplicationNo(e.target.value)} placeholder="例：APIG-103001" />
        </div>

        <div className="grid gap-2">
          <Label>計畫書版本 *</Label>
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
            此計劃原始送件的計畫書表單版本；系統依該版顯示對應欄位（舊版沒有的欄位不顯示、舊版特有欄位才出現）。
          </p>
        </div>

        {/* C2：研究資料整段於匯入時一次填入（匯入後於編輯頁鎖定，改錯需刪除重匯入） */}
        <div className="space-y-3 border-t pt-4">
          <div>
            <h3 className="text-base font-semibold">研究資料</h3>
            <p className="text-sm text-muted-foreground">於匯入時填寫；匯入後此段將鎖定，如需更正請刪除整筆重新匯入。</p>
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
          <Label>備註</Label>
          <Textarea value={remark} onChange={(e) => setRemark(e.target.value)} rows={2} placeholder="匯入說明（選填）" />
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="outline" asChild>
            <Link to="/protocols">取消</Link>
          </Button>
          <Button onClick={handleSubmit} disabled={importMutation.isPending}>
            {importMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            匯入計劃
          </Button>
        </div>
      </div>
    </div>
  )
}
