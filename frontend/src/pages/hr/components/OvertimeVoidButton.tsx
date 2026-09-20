import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
    const { t } = useTranslation()
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
                {t('hrPages.overtime.void.button')}
            </Button>
            <DeleteReasonDialog
                open={open}
                onOpenChange={setOpen}
                copy={{
                    title: t('hrPages.overtime.void.title'),
                    description: t('hrPages.overtime.void.description'),
                    // 小寫名詞：DeleteReasonDialog 會把它內插進 "Confirm {{noun}}"、
                    // "Reason for {{noun}}"、"The {{noun}} will be recorded…" 等句子
                    actionNoun: t('hrPages.overtime.void.actionNoun'),
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
