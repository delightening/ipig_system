import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { UserCheck, UserCog, Loader2, X } from 'lucide-react'

import api from '@/lib/api'
import { authorizePiDelegate, revokePiDelegate } from '@/lib/api/protocol'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Select, SelectTrigger, SelectValue, SelectContent, SelectItem,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDate } from '@/lib/utils'
import { queryKeys } from '@/lib/queryKeys'
import { useAuthUser } from '@/stores/auth'
import { userLabel, type UserOption } from './import-review/users'
import type { ProtocolResponse } from '@/types'

interface Props {
  protocolId: string
}

/**
 * PI 代理授權卡片：外部 PI（尚未開通系統帳號）計畫，由計劃負責人（SD）核准
 * 一位代理人，代替 PI 簽署結案 / 安樂死核准 / 修正案等動作，並留有核可證據
 * （見 backend/src/services/protocol/pi_delegate.rs）。
 *
 * 只有 pi_is_external=true 時才有意義（PI 已有真帳號的計畫不顯示，父層已控管）。
 * 核准/撤銷的實際授權判斷在後端；這裡的顯示 gating 只是方便使用者，不是安全邊界。
 */
export function PiDelegateCard({ protocolId }: Props) {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const user = useAuthUser()
  const [selectedDelegate, setSelectedDelegate] = useState('')

  const { data: protocolResponse } = useQuery({
    queryKey: queryKeys.protocols.detail(protocolId),
    queryFn: async () => (await api.get<ProtocolResponse>(`/protocols/${protocolId}`)).data,
  })

  const isStudyDirector = !!user?.id && protocolResponse?.protocol.study_director_user_id === user.id
  const hasEscalationPrivilege = user?.roles?.some(
    (r) => ['IACUC_STAFF', 'SYSTEM_ADMIN', 'admin'].includes(r)
  ) ?? false
  // 顯示表單的門檻取寬鬆的一邊（SD 或執秘/admin 皆可能被後端放行，實際規則
  // 依「核准他人 vs 核准 SD 自己」而異，見 pi_delegate.rs）；真正把關在後端。
  const canManage = isStudyDirector || hasEscalationPrivilege

  const { data: candidates } = useQuery({
    queryKey: ['protocols', 'assignable-users'],
    queryFn: async () => (await api.get<UserOption[]>('/protocols/assignable-users')).data,
    enabled: canManage,
  })

  const authorizeMutation = useMutation({
    mutationFn: (delegateUserId: string) => authorizePiDelegate(protocolId, delegateUserId),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.piDelegate.authorizeSuccess') })
      qc.invalidateQueries({ queryKey: queryKeys.protocols.detail(protocolId) })
      setSelectedDelegate('')
    },
    onError: (e) => toast({
      title: t('common.error'),
      description: getApiErrorMessage(e, t('protocols.piDelegate.authorizeFailed')),
      variant: 'destructive',
    }),
  })

  const revokeMutation = useMutation({
    mutationFn: () => revokePiDelegate(protocolId),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.piDelegate.revokeSuccess') })
      qc.invalidateQueries({ queryKey: queryKeys.protocols.detail(protocolId) })
    },
    onError: (e) => toast({
      title: t('common.error'),
      description: getApiErrorMessage(e, t('protocols.piDelegate.revokeFailed')),
      variant: 'destructive',
    }),
  })

  if (!protocolResponse?.protocol.pi_is_external) return null

  const delegate = protocolResponse.pi_delegate

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base flex items-center gap-2">
          <UserCog className="h-4 w-4" />
          {t('protocols.piDelegate.title')}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground">
          {t('protocols.piDelegate.description')}
        </p>

        {delegate ? (
          <div className="flex items-center justify-between gap-3 rounded border bg-muted/30 px-3 py-2">
            <div className="flex items-center gap-2 text-sm">
              <UserCheck className="h-4 w-4 text-status-success-text" />
              <span>
                {t('protocols.piDelegate.currentLabel')}
                <span className="font-medium">{delegate.delegate_name}</span>
                {protocolResponse.is_pi_delegate && (
                  <Badge variant="outline" className="ml-2">
                    {t('protocols.piDelegate.actingBadge')}
                  </Badge>
                )}
              </span>
              <span className="text-muted-foreground">
                {t('protocols.piDelegate.authorizedBy', {
                  name: delegate.authorized_by_name,
                  date: formatDate(delegate.authorized_at),
                })}
              </span>
            </div>
            {canManage && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => revokeMutation.mutate()}
                disabled={revokeMutation.isPending}
              >
                {revokeMutation.isPending
                  ? <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  : <X className="h-4 w-4 mr-2" />}
                {t('protocols.piDelegate.revoke')}
              </Button>
            )}
          </div>
        ) : (
          <div className="text-sm text-status-warning-text">
            {t('protocols.piDelegate.none')}
          </div>
        )}

        {canManage && !delegate && (
          <div className="flex items-center gap-2">
            <Select value={selectedDelegate} onValueChange={setSelectedDelegate}>
              <SelectTrigger className="max-w-xs">
                <SelectValue placeholder={t('protocols.piDelegate.selectPlaceholder')} />
              </SelectTrigger>
              <SelectContent>
                {candidates?.map((c) => (
                  <SelectItem key={c.id} value={c.id}>{userLabel(c)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              disabled={!selectedDelegate || authorizeMutation.isPending}
              onClick={() => authorizeMutation.mutate(selectedDelegate)}
            >
              {authorizeMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('protocols.piDelegate.authorize')}
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
