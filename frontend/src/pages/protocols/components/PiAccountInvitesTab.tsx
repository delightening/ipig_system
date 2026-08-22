import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Send, Inbox } from 'lucide-react'

import { listPiAccountInvites, approveSendPiInvite } from '@/lib/api/protocol'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDate } from '@/lib/utils'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { useTableSort } from '@/hooks/useTableSort'
import { useAuthHasPermission } from '@/stores/auth'

import { Button } from '@/components/ui/button'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { toast } from '@/components/ui/use-toast'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

/** PI 帳號開通信核准（admin only）：補登匯入計畫的外部 PI 開通信，須 admin 核准後才寄出。 */
export function PiAccountInvitesTab() {
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  // R71-8：補前端權限 gate（與後端 require_permission!("aup.pi_invite.approve") 對齊）
  const hasPermission = useAuthHasPermission()
  const canApprove = hasPermission('aup.pi_invite.approve')

  const { data: invites, isLoading } = useQuery({
    queryKey: ['pi-account-invites', 'pending'],
    queryFn: () => listPiAccountInvites('pending'),
    staleTime: 30_000,
  })

  // 前端排序（預設不指定 → 維持後端 provisioned_at DESC 的原順序）
  const { sortedData: sortedInvites, sort, toggleSort } = useTableSort(invites)

  const approveMutation = useMutation({
    mutationFn: (inviteId: string) => approveSendPiInvite(inviteId),
    onSuccess: () => {
      toast({ title: '已核准', description: '已寄送 PI 設定密碼開通信' })
      queryClient.invalidateQueries({ queryKey: ['pi-account-invites'] })
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '寄送失敗'), variant: 'destructive' }),
  })

  const handleApprove = async (inviteId: string, email: string) => {
    const ok = await confirm({
      title: '核准並寄送開通信',
      description: `將寄送「設定密碼」連結至 ${email}，確認核准寄送？`,
      confirmLabel: '核准寄送',
    })
    if (ok) approveMutation.mutate(inviteId)
  }

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        補登匯入計畫為外部 PI 開通的帳號，其「設定密碼」開通信須由系統管理員於此核准後才寄出。
      </p>
      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                計畫編號
              </SortableTableHead>
              <SortableTableHead sortKey="protocol_title" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                計畫名稱
              </SortableTableHead>
              <SortableTableHead sortKey="pi_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                PI
              </SortableTableHead>
              <SortableTableHead sortKey="email" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                Email
              </SortableTableHead>
              <SortableTableHead sortKey="provisioned_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                開通者
              </SortableTableHead>
              <SortableTableHead sortKey="provisioned_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                開通時間
              </SortableTableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={7} className="p-0">
                  <TableSkeleton rows={5} cols={7} />
                </TableCell>
              </TableRow>
            ) : sortedInvites && sortedInvites.length > 0 ? (
              sortedInvites.map((inv) => (
                <TableRow key={inv.id}>
                  <TableCell className="font-mono">{inv.iacuc_no || '-'}</TableCell>
                  <TableCell className="max-w-[200px] whitespace-normal break-words">{inv.protocol_title}</TableCell>
                  <TableCell>{inv.pi_name}</TableCell>
                  <TableCell>{inv.email}</TableCell>
                  <TableCell>{inv.provisioned_by_name || '-'}</TableCell>
                  <TableCell>{formatDate(inv.provisioned_at)}</TableCell>
                  <TableCell className="text-right">
                    {canApprove ? (
                      <Button size="sm" onClick={() => handleApprove(inv.id, inv.email)} disabled={approveMutation.isPending}>
                        <Send className="h-4 w-4 mr-1" />核准寄送
                      </Button>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={7} icon={Inbox} title="目前無待核准的 PI 開通信" />
            )}
          </TableBody>
        </Table>
      </div>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
