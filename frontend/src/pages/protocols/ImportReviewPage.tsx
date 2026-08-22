import { Link, useParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Loader2, FileCheck2, Pencil, UserPlus } from 'lucide-react'

import api from '@/lib/api'
import { provisionPiAccount } from '@/lib/api/protocol'
import type { ProtocolResponse } from '@/types'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

import { ImportReviewArtifactsForm } from './components/import-review/ImportReviewArtifactsForm'
import { ChairmanLetterUpload } from './components/import-review/ChairmanLetterUpload'
import { FinalizeImportCard } from './components/import-review/FinalizeImportCard'

/**
 * 補登作業頁：匯入已核准計劃後的歷史審查文件補登中心。
 * 1) 編輯計劃內容 2) 補登審查文件（執秘/委員/獸醫）3) 上傳主席核准同意函
 * 4) 完成補登（建 v1 版本快照 + 記原始版本號 + 解除補登中）。
 */
export function ImportReviewPage() {
  const { id = '' } = useParams()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()

  const { data: protocolResponse, isLoading } = useQuery({
    queryKey: ['protocol', id],
    queryFn: async () => (await api.get<ProtocolResponse>(`/protocols/${id}`)).data,
    enabled: !!id,
  })
  const protocol = protocolResponse?.protocol

  const provisionMutation = useMutation({
    mutationFn: () => provisionPiAccount(id),
    onSuccess: (r) => {
      toast({
        title: '已開通 PI 帳號',
        description: r.created_new_account
          ? `已建立帳號 ${r.email}；開通信待管理員核准後寄送`
          : `已關聯既有帳號 ${r.email}`,
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '開通 PI 帳號失敗'), variant: 'destructive' }),
  })

  const handleProvision = async () => {
    const ok = await confirm({
      title: '開通 PI 帳號',
      description: '為此計畫的 PI（客人）建立系統帳號並關聯；「設定密碼」開通信須由系統管理員核准後才寄出。確認開通？',
      confirmLabel: '開通',
    })
    if (ok) provisionMutation.mutate()
  }

  if (isLoading) {
    return <div className="flex justify-center p-12"><Loader2 className="h-6 w-6 animate-spin" /></div>
  }
  if (!protocol) {
    return <div className="p-6 text-muted-foreground">找不到計劃</div>
  }

  const isPending = protocol.import_pending === true

  return (
    <div className="space-y-6">
      <div>
        <Link to={`/protocols/${id}`} className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />回到計劃詳情
        </Link>
      </div>

      <div>
        <h1 className="page-title break-words">補登作業：{protocol.title}</h1>
        <p className="text-sm text-muted-foreground mt-1">
          {isPending
            ? '補登歷史審查文件，完成後建立第 1 版快照並鎖定計劃。'
            : '此計劃已完成補登（已鎖定）。'}
        </p>
      </div>

      {!isPending ? (
        <Card>
          <CardContent className="flex items-center gap-3 py-6 text-muted-foreground">
            <FileCheck2 className="h-5 w-5 text-status-success-solid" />
            此計劃已完成補登，無法再編輯補登內容。
            <Button variant="outline" asChild className="ml-auto">
              <Link to={`/protocols/${id}`}>查看計劃</Link>
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          <Card>
            <CardHeader><CardTitle className="text-base">1. 計劃內容</CardTitle></CardHeader>
            <CardContent className="flex items-center justify-between">
              <p className="text-sm text-muted-foreground">補登中可編輯計劃書內容（章節資料）。</p>
              <Button variant="outline" asChild>
                <Link to={`/protocols/${id}/edit`}><Pencil className="h-4 w-4 mr-2" />編輯計劃內容</Link>
              </Button>
            </CardContent>
          </Card>

          {/* 外部 PI（pi_user_id 暫掛匯入者）才顯示開通卡片；缺 PI email 時仍顯示按鈕但停用 + 提示，
              避免「沒按鈕」讓使用者不知所措（補登中可至「編輯計劃內容」補填 email）。 */}
          {protocol.pi_user_id === protocol.created_by && (
            <Card>
              <CardHeader><CardTitle className="text-base">PI 帳號開通</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm text-muted-foreground">
                    為外部 PI（客人）建立系統帳號並關聯，使其可登入回應審查 / 安樂死 / 簽章。「設定密碼」開通信須經系統管理員核准後寄出。
                  </p>
                  <Button
                    variant="outline"
                    onClick={handleProvision}
                    disabled={provisionMutation.isPending || !protocol.working_content?.basic?.pi?.email?.trim()}
                    className="shrink-0"
                  >
                    <UserPlus className="h-4 w-4 mr-2" />開通 PI 帳號
                  </Button>
                </div>
                {!protocol.working_content?.basic?.pi?.email?.trim() && (
                  <p className="text-sm text-destructive">
                    此計畫的 PI 尚未填寫 email，無法開通帳號。請先於「編輯計劃內容」→ 研究資料填入 PI email。
                  </p>
                )}
              </CardContent>
            </Card>
          )}

          <div>
            <h2 className="mb-3 text-sm font-medium">2. 補登審查文件</h2>
            <ImportReviewArtifactsForm protocolId={id} initialVetReview={protocolResponse?.vet_review} />
          </div>

          <ChairmanLetterUpload protocolId={id} />
          <FinalizeImportCard protocolId={id} approvalDate={protocol.approved_at ?? protocol.start_date} />
        </>
      )}
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
