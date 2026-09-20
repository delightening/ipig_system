import { Link, useParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
        title: t('protocolPages.importReview.provision.success'),
        description: r.created_new_account
          ? t('protocolPages.importReview.provision.createdNew', { email: r.email })
          : t('protocolPages.importReview.provision.linkedExisting', { email: r.email }),
      })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
    },
    onError: (e: unknown) =>
      toast({ title: t('common.error'), description: getApiErrorMessage(e, t('protocolPages.importReview.provision.failed')), variant: 'destructive' }),
  })

  const handleProvision = async () => {
    const ok = await confirm({
      title: t('protocolPages.importReview.provision.title'),
      description: t('protocolPages.importReview.provision.confirmDescription'),
      confirmLabel: t('protocolPages.importReview.provision.confirmLabel'),
    })
    if (ok) provisionMutation.mutate()
  }

  if (isLoading) {
    return <div className="flex justify-center p-12"><Loader2 className="h-6 w-6 animate-spin" /></div>
  }
  if (!protocol) {
    return <div className="p-6 text-muted-foreground">{t('protocolPages.importReview.notFound')}</div>
  }

  const isPending = protocol.import_pending === true

  return (
    <div className="space-y-6">
      <div>
        <Link to={`/protocols/${id}`} className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />{t('protocolPages.importReview.backToDetail')}
        </Link>
      </div>

      <div>
        <h1 className="page-title break-words">{t('protocolPages.importReview.pageTitle', { title: protocol.title })}</h1>
        <p className="text-sm text-muted-foreground mt-1">
          {isPending
            ? t('protocolPages.importReview.subtitlePending')
            : t('protocolPages.importReview.subtitleDone')}
        </p>
      </div>

      {!isPending ? (
        <Card>
          <CardContent className="flex items-center gap-3 py-6 text-muted-foreground">
            <FileCheck2 className="h-5 w-5 text-status-success-solid" />
            {t('protocolPages.importReview.doneNotice')}
            <Button variant="outline" asChild className="ml-auto">
              <Link to={`/protocols/${id}`}>{t('protocolPages.importReview.viewProtocol')}</Link>
            </Button>
          </CardContent>
        </Card>
      ) : (
        <>
          <Card>
            <CardHeader><CardTitle className="text-base">{t('protocolPages.importReview.step1.title')}</CardTitle></CardHeader>
            <CardContent className="flex items-center justify-between">
              <p className="text-sm text-muted-foreground">{t('protocolPages.importReview.step1.description')}</p>
              <Button variant="outline" asChild>
                <Link to={`/protocols/${id}/edit`}><Pencil className="h-4 w-4 mr-2" />{t('protocolPages.importReview.step1.editButton')}</Link>
              </Button>
            </CardContent>
          </Card>

          {/* 外部 PI（pi_user_id 暫掛匯入者）才顯示開通卡片；缺 PI email 時仍顯示按鈕但停用 + 提示，
              避免「沒按鈕」讓使用者不知所措（補登中可至「編輯計劃內容」補填 email）。 */}
          {protocol.pi_user_id === protocol.created_by && (
            <Card>
              <CardHeader><CardTitle className="text-base">{t('protocolPages.importReview.provision.cardTitle')}</CardTitle></CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm text-muted-foreground">
                    {t('protocolPages.importReview.provision.cardDescription')}
                  </p>
                  <Button
                    variant="outline"
                    onClick={handleProvision}
                    disabled={provisionMutation.isPending || !protocol.working_content?.basic?.pi?.email?.trim()}
                    className="shrink-0"
                  >
                    <UserPlus className="h-4 w-4 mr-2" />{t('protocolPages.importReview.provision.title')}
                  </Button>
                </div>
                {!protocol.working_content?.basic?.pi?.email?.trim() && (
                  <p className="text-sm text-destructive">
                    {t('protocolPages.importReview.provision.missingEmail')}
                  </p>
                )}
              </CardContent>
            </Card>
          )}

          <div>
            <h2 className="mb-3 text-sm font-medium">{t('protocolPages.importReview.step2.title')}</h2>
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
