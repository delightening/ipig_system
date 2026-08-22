import { useState } from 'react'
import { Ban } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { useAuthIsAdmin, useAuthUser } from '@/stores/auth'
import type { OvertimeWithUser } from '@/types/hr'

import { useOvertimeMutations } from '../hooks/useOvertimeMutations'

/**
 * R86-2：作廢已核准的加班單。
 *
 * 補登腳本重跑造成的重複單一旦核准就沒有撤銷路徑（delete 只受理草稿、reject 只受理待審），
 * 而補休餘額已經授出。作廢通道由負責人單簽 + 理由必填，後端會擋下「作廢自己的單」與
 * 「補休已被使用」兩種情形——這裡只負責不顯示注定失敗的按鈕。
 */
export function OvertimeVoidButton({ overtime }: { overtime: OvertimeWithUser }) {
    const [open, setOpen] = useState(false)
    const isAdmin = useAuthIsAdmin()
    const currentUser = useAuthUser()
    const { voidOvertime } = useOvertimeMutations()

    const canVoid =
        isAdmin && overtime.status === 'approved' && overtime.user_id !== currentUser?.id
    if (!canVoid) return null

    return (
        <>
            <Button
                variant="outline"
                size="sm"
                onClick={() => setOpen(true)}
                disabled={voidOvertime.isPending}
            >
                <Ban className="h-4 w-4 mr-1" />
                作廢
            </Button>
            <DeleteReasonDialog
                open={open}
                onOpenChange={setOpen}
                copy={{
                    title: '作廢加班單',
                    description:
                        '作廢後此加班單不再生效，已授出的補休餘額會一併收回。原始紀錄保留，不會被刪除。',
                    actionNoun: '作廢',
                }}
                isPending={voidOvertime.isPending}
                onConfirm={(reason) =>
                    voidOvertime.mutate(
                        { id: overtime.id, reason },
                        { onSuccess: () => setOpen(false) }
                    )
                }
            />
        </>
    )
}
