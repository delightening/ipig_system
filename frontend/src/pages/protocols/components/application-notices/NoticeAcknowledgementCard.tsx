import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { CheckCircle2, FileSignature, Loader2, AlertTriangle } from 'lucide-react'

import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { HandwrittenSignaturePad, type SignatureData } from '@/components/ui/handwritten-signature-pad'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDate } from '@/lib/utils'
import { queryKeys } from '@/lib/queryKeys'

interface ActiveNotice {
  id: string
  version_label: string
  title: string
  content: string
  effective_from: string
}
interface NoticeStatus {
  active_notice: ActiveNotice | null
  acknowledged: boolean
  acknowledged_at?: string | null
}

interface Props {
  protocolId: string
}

/** 申請須知簽署卡片：DRAFT 計畫填表/送審前，顯示生效須知 + 手寫簽署狀態。 */
export function NoticeAcknowledgementCard({ protocolId }: Props) {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const [signOpen, setSignOpen] = useState(false)
  const [sig, setSig] = useState<SignatureData | null>(null)

  const { data: status, isLoading } = useQuery({
    queryKey: ['protocol-notice-status', protocolId],
    queryFn: async () => (await api.get<NoticeStatus>(`/protocols/${protocolId}/notice-acknowledgement`)).data,
  })

  const ackMutation = useMutation({
    mutationFn: async (s: SignatureData) =>
      api.post(`/protocols/${protocolId}/acknowledge-notice`, {
        handwriting_svg: s.svg,
        stroke_data: s.strokeData,
      }),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocolPages.applicationNotices.acknowledgement.signed') })
      qc.invalidateQueries({ queryKey: ['protocol-notice-status', protocolId] })
      qc.invalidateQueries({ queryKey: queryKeys.protocols.detail(protocolId) })
      setSignOpen(false)
      setSig(null)
    },
    onError: (e) => toast({ title: t('common.error'), description: getApiErrorMessage(e, t('protocolPages.applicationNotices.acknowledgement.signFailed')), variant: 'destructive' }),
  })

  // 載入中或無生效須知 → 不顯示卡片（無須知門檻）
  if (isLoading || !status?.active_notice) return null

  const notice = status.active_notice

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <FileSignature className="h-4 w-4" />
          {t('protocolPages.applicationNotices.acknowledgement.cardTitle', { version: notice.version_label })}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {status.acknowledged ? (
          <div className="flex items-center gap-2 text-sm text-status-success-text">
            <CheckCircle2 className="h-4 w-4" />
            {status.acknowledged_at
              ? t('protocolPages.applicationNotices.acknowledgement.signedOn', { date: formatDate(status.acknowledged_at) })
              : t('protocolPages.applicationNotices.acknowledgement.signedStatus')}
          </div>
        ) : (
          <div className="flex items-center gap-2 text-sm text-status-warning-text">
            <AlertTriangle className="h-4 w-4" />
            {t('protocolPages.applicationNotices.acknowledgement.unsignedWarning')}
          </div>
        )}

        <div className="max-h-48 overflow-y-auto rounded border bg-muted/30 p-3 text-sm whitespace-pre-wrap break-words">
          {notice.content}
        </div>

        {!status.acknowledged && (
          <Button size="sm" onClick={() => setSignOpen(true)}>
            <FileSignature className="h-4 w-4 mr-2" />{t('protocolPages.applicationNotices.acknowledgement.signButton')}
          </Button>
        )}
      </CardContent>

      {/* 簽署 dialog */}
      <Dialog open={signOpen} onOpenChange={(o) => { if (!o) { setSignOpen(false); setSig(null) } }}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('protocolPages.applicationNotices.acknowledgement.dialogTitle', { title: notice.title, version: notice.version_label })}</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <p className="text-sm text-muted-foreground">{t('protocolPages.applicationNotices.acknowledgement.dialogHint')}</p>
            <div className="max-h-40 overflow-y-auto rounded border bg-muted/30 p-3 text-sm whitespace-pre-wrap break-words">
              {notice.content}
            </div>
            <HandwrittenSignaturePad onSignatureChange={setSig} height={180} />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => { setSignOpen(false); setSig(null) }}>{t('common.cancel')}</Button>
            <Button onClick={() => sig && ackMutation.mutate(sig)} disabled={!sig || ackMutation.isPending}>
              {ackMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}{t('protocolPages.applicationNotices.acknowledgement.confirmSign')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  )
}
