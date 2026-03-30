import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useMutation } from '@tanstack/react-query'
import { Copy, Loader2, CheckCircle2 } from 'lucide-react'

import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { toast } from '@/components/ui/use-toast'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import type { CreateInvitationResponse } from '@/types/invitation'

const createInvitationSchema = z.object({
    email: z.string().email('請輸入有效的電子郵件'),
    organization: z.string().optional(),
})
type CreateInvitationForm = z.infer<typeof createInvitationSchema>

interface InvitationCreateDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    onSuccess: () => void
}

export function InvitationCreateDialog({ open, onOpenChange, onSuccess }: InvitationCreateDialogProps) {
    const [result, setResult] = useState<CreateInvitationResponse | null>(null)

    const form = useForm<CreateInvitationForm>({
        resolver: zodResolver(createInvitationSchema),
        defaultValues: { email: '', organization: '' },
    })

    const createMutation = useMutation({
        mutationFn: (data: CreateInvitationForm) =>
            invitationApi.create({ email: data.email, organization: data.organization || undefined }),
        onSuccess: (res) => {
            setResult(res.data)
            onSuccess()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error)
            toast({ variant: 'destructive', title: '建立邀請失敗', description: msg })
        },
    })

    const handleClose = () => {
        setResult(null)
        form.reset()
        onOpenChange(false)
    }

    const handleCopyLink = async (link: string) => {
        try {
            await navigator.clipboard.writeText(link)
            toast({ title: '已複製連結' })
        } catch {
            toast({ variant: 'destructive', title: '複製失敗，請手動複製' })
        }
    }

    if (result) {
        return (
            <Dialog open={open} onOpenChange={handleClose}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <CheckCircle2 className="h-5 w-5 text-status-success-text" />
                            邀請已送出
                        </DialogTitle>
                        <DialogDescription>
                            已發送邀請 Email 至 {result.invitation.email}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4">
                        <div>
                            <p className="text-sm text-muted-foreground mb-2">
                                或複製以下連結，透過其他管道傳給客戶：
                            </p>
                            <div className="flex items-center gap-2">
                                <Input value={result.invite_link} readOnly className="font-mono text-xs" />
                                <Button
                                    variant="outline"
                                    size="icon"
                                    onClick={() => handleCopyLink(result.invite_link)}
                                    aria-label="複製連結"
                                >
                                    <Copy className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                        <p className="text-sm text-muted-foreground">
                            此連結將於 {new Date(result.invitation.expires_at).toLocaleDateString('zh-TW')} 過期
                        </p>
                    </div>
                    <DialogFooter>
                        <Button onClick={handleClose}>完成</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        )
    }

    return (
        <Dialog open={open} onOpenChange={handleClose}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>邀請客戶</DialogTitle>
                    <DialogDescription>輸入客戶 Email 即可發送邀請連結</DialogDescription>
                </DialogHeader>
                <form onSubmit={form.handleSubmit((data) => createMutation.mutate(data))} className="space-y-4">
                    <FormField label="客戶 Email" htmlFor="invite-email" error={form.formState.errors.email?.message} required>
                        <Input
                            id="invite-email"
                            type="email"
                            placeholder="wang.daming@hospital.org"
                            {...form.register('email')}
                        />
                    </FormField>
                    <FormField label="組織名稱（選填）" htmlFor="invite-org" error={form.formState.errors.organization?.message}>
                        <Input
                            id="invite-org"
                            placeholder="台大醫院"
                            {...form.register('organization')}
                        />
                    </FormField>
                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={handleClose}>取消</Button>
                        <Button type="submit" disabled={createMutation.isPending}>
                            {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            送出邀請
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
