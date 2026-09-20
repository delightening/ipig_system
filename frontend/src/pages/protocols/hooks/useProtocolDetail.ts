import { useState, useMemo, useCallback, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { signatureApi } from '@/lib/api'
import type {
  ProtocolResponse,
  ProtocolStatus,
  User,
} from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { toast } from '@/components/ui/use-toast'
import { useAuthUser, useAuthHasPermission } from '@/stores/auth'
import { useSidebarStore } from '@/stores/sidebar'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { allowedTransitions, REVIEWABLE_STATUSES } from '../constants'
import type { ProtocolVersion } from '@/types/aup'
import { useProtocolMutations } from './useProtocolMutations'

interface ReviewerOption {
  id: string
  email: string
  display_name: string
}

export function useProtocolDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { t } = useTranslation()
  const user = useAuthUser()
  const hasPermission = useAuthHasPermission()
  const { dialogState, confirm } = useConfirmDialog()

  const [showStatusDialog, setShowStatusDialog] = useState(false)
  const [newStatus, setNewStatus] = useState<ProtocolStatus | ''>('')
  const [statusRemark, setStatusRemark] = useState('')
  const [selectedReviewerIds, setSelectedReviewerIds] = useState<string[]>([])
  const [showCommentPanel, setShowCommentPanel] = useState(false)
  const { sidebarOpen, setSidebarOpen } = useSidebarStore()
  const sidebarWasOpenRef = useRef(true)

  // --- Queries ---

  const { data: protocolResponse, isLoading } = useQuery({
    queryKey: queryKeys.protocols.detail(id!),
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const {
    protocol, pi_name, pi_email, pi_organization, sd_name, created_by_name, vet_review,
    is_pi_delegate,
  } = protocolResponse || {}

  const { data: allUsers } = useQuery({
    queryKey: queryKeys.users.all,
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
    },
    enabled: showStatusDialog,
  })

  // R71-7：「已核准必有簽章」不變式 —— 取得計畫電子簽章狀態，供核准前置門檻使用。
  // 後端 change_status 亦守衛；前端據此停用核准按鈕並提示「先簽再核准」。
  const { data: signatureStatus } = useQuery({
    queryKey: queryKeys.signatures.protocol(id),
    queryFn: () => signatureApi.getProtocolStatus(id!),
    enabled: !!id && showStatusDialog,
    select: (res) => res.data,
    // 簽章於獨立「簽署」頁完成（不會 invalidate 此 key），故每次開啟對話框即時重取，
    // 避免使用者剛簽完、30 秒內重開仍誤顯示「未簽章」而擋下合法核准。
    retry: false,
  })
  // 三態：true=已簽、false=確認未簽、undefined=載入中／查詢失敗（狀態未知）。
  // 僅「確認未簽（false）」才前端擋核准並顯示警示；未知狀態交由後端守衛裁定，
  // 避免查詢失敗時以誤導訊息永久擋住合法核准。
  const hasProtocolSignature = signatureStatus?.is_signed

  // --- Computed values ---

  const isVetReviewer = useMemo(() => {
    if (!user || !vet_review) return false
    return vet_review.vet_id === user.id
  }, [user, vet_review])

  const availableReviewers = useMemo<ReviewerOption[] | undefined>(() =>
    allUsers?.filter(u => u.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
      .map(u => ({ id: u.id, email: u.email, display_name: u.display_name || u.email })),
    [allUsers]
  )

  // 系統管理員（嚴格 admin，非含執秘）：預審駁回通道 + 軟刪除按鈕的角色閘門
  const isAdmin = user?.roles?.some(r => ['SYSTEM_ADMIN', 'admin'].includes(r)) ?? false

  const availableTransitions = useMemo(() => {
    if (!protocol) return []
    const base = allowedTransitions[protocol.status] || []
    // Admin 預審駁回通道：SUBMITTED / PRE_REVIEW 階段可駁回（後端亦守衛 admin + 理由必填）
    if (
      isAdmin
      && (protocol.status === 'SUBMITTED' || protocol.status === 'PRE_REVIEW')
      && !base.includes('REJECTED')
    ) {
      return [...base, 'REJECTED'] as typeof base
    }
    return base
  }, [protocol, isAdmin])

  // 軟刪除按鈕：限系統管理員、且計畫為「已否決」終態
  const canSoftDelete = isAdmin && protocol?.status === 'REJECTED'

  const cleanedWorkingContent = useMemo(() => {
    if (!protocol?.working_content) return null
    const cleanedContent = JSON.parse(JSON.stringify(protocol.working_content))
    if (cleanedContent.basic && cleanedContent.basic.apply_study_number !== undefined) {
      delete cleanedContent.basic.apply_study_number
    }
    return cleanedContent
  }, [protocol?.working_content])

  const isVet = user?.roles?.includes('VET')
  const isIACUCOrAdmin = user?.roles?.some(r =>
    ['IACUC_CHAIR', 'IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
  )
  // 與後端 create_review_comment 的 require_permission!("aup.review.comment") 對齊。
  // 保留「審查中的狀態」這一層：後端另有計畫關聯檢查，而 REVIEWABLE_STATUSES 是
  // 流程階段限制、不是授權——兩者是不同的閘，都要留。
  const canAddComment = hasPermission('aup.review.comment')
    && (isIACUCOrAdmin || (!!protocol?.status && REVIEWABLE_STATUSES.includes(protocol.status)))
  // 權威 can_edit 來自後端（= can_edit_protocol：admin / PI 含成員 PI / SD / 補登管理者），
  // 避免前端自行重算漏掉 backend 放行情境（成員 PI、import-pending）。後端未提供時（載入中 /
  // 舊後端）退回關係制估算（admin / PI(pi_user_id) / SD）。
  const canEditProtocol = protocolResponse?.can_edit
    ?? (isAdmin
      || (!!user?.id && protocol?.pi_user_id === user.id)
      || (!!user?.id && protocol?.study_director_user_id === user.id))
  // 權威 can_write_amendment 來自後端（= access::can_write_amendment：admin / PI，不含
  // SD／補登管理者，比 can_edit 更緊）。後端未提供時退回關係制估算（admin / PI(pi_user_id)，
  // 同樣不含 SD，避免讓 SD 看到必定 403 的修正案按鈕）。
  const canWriteAmendment = protocolResponse?.can_write_amendment
    ?? (isAdmin || (!!user?.id && protocol?.pi_user_id === user.id))
  // 與後端 reply_review_comment 對齊：`has_permission("aup.review.reply") || 計畫擁有者`。
  // ⚠️ 不可只看 permission —— 後端有 owner fallback，只看 permission 會把「非該角色
  // 但是計畫擁有者」的人擋在外面（他們按下去其實會成功）。canEditProtocol 取自後端
  // 權威 can_edit，正是擁有者判準。
  const canReply = hasPermission('aup.review.reply') || canEditProtocol
  // 與後端 assign_reviewer 的 require_permission!("aup.review.assign") 對齊
  // （原本比對 ['IACUC_STAFF','IACUC_CHAIR','SYSTEM_ADMIN','admin'] 四個 role 字串：
  //  剛好等於目前持有該權限的角色集合，所以這是等價替換、不改變任何人看得到什麼；
  //  差別在日後把 aup.review.assign 授予新角色時，前端會自動跟上，不必再改這裡）。
  const canAssignReviewer = hasPermission('aup.review.assign')
  // R71-8：核准/狀態變更按鈕 gate 統一為 permission token（與後端
  // change_protocol_status 的 require_permission!("aup.protocol.change_status") 對齊），
  // 取代原 role 字串比對。
  const canChangeStatus = hasPermission('aup.protocol.change_status')
  const isRevisionStatus = protocol?.status === 'REVISION_REQUIRED'
    || protocol?.status === 'PRE_REVIEW_REVISION_REQUIRED'
    || protocol?.status === 'VET_REVISION_REQUIRED'
  // 附件即計畫內容：唯讀檢視者（canEditProtocol=false）不得管理附件，與內容編輯同權。
  const canManageAttachments = canEditProtocol && (protocol?.status === 'DRAFT' || isRevisionStatus)
  // 與後端 list_review_comments 的 aup.review.identity_view 權限判斷對齊
  // （原本比對 ['IACUC_STAFF','IACUC_CHAIR','REVIEWER','VET','SYSTEM_ADMIN','admin']
  //  六個 role 字串：等價替換，不改變任何人看得到什麼）。
  const shouldAnonymizeReviewers = !hasPermission('aup.review.identity_view')

  // --- Comment panel queries ---

  const canShowPanel = !!canAddComment && protocol?.status !== 'DRAFT'

  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', id],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${id}/versions`)
      return response.data
    },
    enabled: !!id && canShowPanel,
  })

  // --- Mutations ---

  const {
    submitMutation,
    changeStatusMutation,
    softDeleteMutation,
    addCommentMutation,
  } = useProtocolMutations({
    id,
    versions,
    onStatusChangeSuccess: () => {
      setShowStatusDialog(false)
      setNewStatus('')
      setStatusRemark('')
      setSelectedReviewerIds([])
    },
    // 軟刪除後計畫已從列表隱藏，導回計畫書管理列表
    onSoftDeleteSuccess: () => navigate('/protocols'),
  })

  // --- Handlers ---

  const handleSubmit = useCallback(async () => {
    const ok = await confirm({
      title: t('protocolPages.shared.submitConfirmTitle'),
      description: t('protocols.detail.submitConfirm'),
      confirmLabel: t('protocolPages.shared.submitConfirmLabel'),
    })
    if (ok) submitMutation.mutate()
  }, [confirm, t, submitMutation])

  const handleSoftDelete = useCallback(async () => {
    const ok = await confirm({
      title: t('protocols.detail.softDeleteConfirmTitle'),
      description: t('protocols.detail.softDeleteConfirmDesc'),
      confirmLabel: t('protocols.detail.softDeleteConfirmLabel'),
      variant: 'destructive',
    })
    if (ok) softDeleteMutation.mutate()
  }, [confirm, t, softDeleteMutation])

  const handleChangeStatus = useCallback(async () => {
    if (!newStatus) return

    if (newStatus === 'UNDER_REVIEW') {
      if (selectedReviewerIds.length < 2 || selectedReviewerIds.length > 3) {
        toast({
          title: t('common.error'),
          description: t('protocols.detail.dialogs.status.selected', { count: selectedReviewerIds.length }),
          variant: 'destructive',
        })
        return
      }
    }

    // R71-7：核准前須已完成計畫電子簽章（與後端守衛一致；先簽再核准）。
    // 僅在「確認未簽（=== false）」時前端攔截；狀態未知（undefined）時放行交後端裁定。
    if (
      (newStatus === 'APPROVED' || newStatus === 'APPROVED_WITH_CONDITIONS')
      && hasProtocolSignature === false
    ) {
      toast({
        title: t('common.error'),
        description: t('protocols.detail.dialogs.status.signatureRequired'),
        variant: 'destructive',
      })
      return
    }

    // R71-10：核准 / 附條件核准為高風險不可逆動作（生成 IACUC、自動建客戶、進入已核准叢集），
    // 送出前加二次確認。
    if (newStatus === 'APPROVED' || newStatus === 'APPROVED_WITH_CONDITIONS') {
      const ok = await confirm({
        title: t('protocolPages.detail.approveConfirmTitle'),
        description: t('protocolPages.detail.approveConfirmDescription'),
        confirmLabel: t('protocolPages.detail.approveConfirmLabel'),
      })
      if (!ok) return
    }

    try {
      await changeStatusMutation.mutateAsync({
        to_status: newStatus,
        remark: statusRemark || undefined,
        reviewer_ids: newStatus === 'UNDER_REVIEW' ? selectedReviewerIds : undefined,
      })
    } catch {
      // Errors handled in mutation callbacks
    }
  }, [newStatus, selectedReviewerIds, hasProtocolSignature, changeStatusMutation, statusRemark, confirm, t])

  const handleReviewerToggle = useCallback((reviewerId: string, checked: boolean) => {
    setSelectedReviewerIds(prev =>
      checked ? [...prev, reviewerId] : prev.filter(r => r !== reviewerId)
    )
  }, [])

  const handleToggleCommentPanel = useCallback(() => {
    setShowCommentPanel(prev => {
      const willOpen = !prev
      if (willOpen) {
        sidebarWasOpenRef.current = sidebarOpen
        setSidebarOpen(false)
      } else {
        if (sidebarWasOpenRef.current) {
          setSidebarOpen(true)
        }
      }
      return willOpen
    })
  }, [sidebarOpen, setSidebarOpen])

  const sectionOptions = useMemo(() => [
    t('protocols.content.sections.researchInfo'),
    t('protocols.content.sections.purpose'),
    t('protocols.content.sections.items'),
    t('protocols.content.sections.design'),
    t('protocols.content.sections.guidelines'),
    t('protocols.content.sections.surgery'),
    t('protocols.content.sections.animals'),
    t('protocols.content.sections.personnel'),
    t('protocols.content.sections.attachments'),
    t('protocols.content.sections.signatures'),
  ], [t])

  return {
    id,
    navigate,
    t,
    protocol,
    pi_name,
    pi_email,
    pi_organization,
    sd_name,
    created_by_name,
    vet_review,
    isLoading,
    showStatusDialog,
    setShowStatusDialog,
    newStatus,
    setNewStatus,
    statusRemark,
    setStatusRemark,
    selectedReviewerIds,
    showCommentPanel,
    cleanedWorkingContent,
    availableTransitions,
    availableReviewers,
    hasProtocolSignature,
    isVetReviewer,
    isVet,
    canAddComment: !!canAddComment,
    canReply: !!canReply,
    canEditProtocol: !!canEditProtocol,
    canWriteAmendment: !!canWriteAmendment,
    isPiDelegate: !!is_pi_delegate,
    canAssignReviewer: !!canAssignReviewer,
    canChangeStatus,
    isRevisionStatus,
    canManageAttachments: !!canManageAttachments,
    shouldAnonymizeReviewers,
    canShowPanel,
    sectionOptions,
    submitMutation,
    changeStatusMutation,
    softDeleteMutation,
    addCommentMutation,
    canSoftDelete,
    handleSubmit,
    handleSoftDelete,
    handleChangeStatus,
    handleReviewerToggle,
    handleToggleCommentPanel,
    dialogState,
  }
}
