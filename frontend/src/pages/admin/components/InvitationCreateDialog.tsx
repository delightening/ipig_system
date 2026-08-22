import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Copy, Loader2, CheckCircle2 } from 'lucide-react'

import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'
import { onActivateKey } from '@/lib/a11y'
import { Badge } from '@/components/ui/badge'
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
import { StaffAffiliationField } from '@/components/admin/StaffAffiliationField'
import type { CreateInvitationResponse } from '@/types/invitation'

interface CreateInvitationForm {
    email: string
    display_name: string
    organization: string
    phone?: string
    position?: string
    role_ids: string[]
    /** null = 尚未選擇；送出前必填（後端亦為必填，無 serde default） */
    is_internal: boolean | null
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

interface InvitationCreateDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    onSuccess: () => void
}

export function InvitationCreateDialog({ open, onOpenChange, onSuccess }: InvitationCreateDialogProps) {
    const { t } = useTranslation()
    const [result, setResult] = useState<CreateInvitationResponse | null>(null)

    const form = useForm<CreateInvitationForm>({
        defaultValues: {
            email: '',
            display_name: '',
            organization: '',
            phone: '',
            position: '',
            role_ids: [],
            is_internal: null,
        },
    })

    const rolesQuery = useQuery({
        queryKey: ['invitation-available-roles'],
        queryFn: () => invitationApi.availableRoles().then((r) => r.data),
        enabled: open,
        staleTime: 5 * 60 * 1000,
    })

    const roleIds = form.watch('role_ids')
    const isInternal = form.watch('is_internal')
    const toggleRole = (id: string) => {
        const next = roleIds.includes(id) ? roleIds.filter((x) => x !== id) : [...roleIds, id]
        form.setValue('role_ids', next, { shouldValidate: true })
    }

    // 已選角色的 code，供身分欄位做預選判斷。
    // ⚠️ `role.is_internal` 是**角色層級**的旗標，不能直接拿來當人的分類——
    // REVIEWER / VET / IACUC_CHAIR 在角色表標為內部，實際擔任者卻多為外聘。
    // 這裡只把 code 傳下去，由 deriveAffiliation 決定哪些能推導、哪些要人選。
    const selectedRoleCodes = (rolesQuery.data ?? [])
        .filter((r) => roleIds.includes(r.id))
        .map((r) => r.code)

    const createMutation = useMutation({
        mutationFn: (data: CreateInvitationForm) =>
            invitationApi.create({
                email: data.email,
                display_name: data.display_name,
                organization: data.organization,
                phone: data.phone || undefined,
                position: data.position || undefined,
                role_ids: data.role_ids,
                // 這個值會原封成為受邀者的 users.is_internal——先前這裡沒有
                // 對應欄位，接受邀請的 SQL 一律硬編 false。
                //
                // 斷言而非 `?? false`：**刻意不補預設值**——補了等於把「靜默猜測」
                // 留一條後路，而且與 UserCreateDialog 原本的 `?? true` 方向相反，
                // 正是本 PR 要修的「兩條建立路徑做出相反假設」。
                // 不變式由兩道防線保證：① 上方 validate 要求 typeof === 'boolean'；
                // ② 後端 `CreateInvitationRequest.is_internal` 無 serde default，
                // 真的漏帶會回 400。加執行期 if 只會多一條走不到、也量不到的分支。
                is_internal: data.is_internal as boolean,
            }),
        onSuccess: (res) => {
            setResult(res.data)
            onSuccess()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error)
            toast({ variant: 'destructive', title: t('admin.invitationCreateDialog.createFailed'), description: msg })
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
            toast({ title: t('admin.invitationCreateDialog.linkCopied') })
        } catch {
            toast({ variant: 'destructive', title: t('admin.invitationCreateDialog.copyFailed') })
        }
    }

    if (result) {
        return (
            <Dialog open={open} onOpenChange={handleClose}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <CheckCircle2 className="h-5 w-5 text-status-success-text" />
                            {t('admin.invitationCreateDialog.sentTitle')}
                        </DialogTitle>
                        <DialogDescription>
                            {t('admin.invitationCreateDialog.sentDescription', { email: result.invitation.email })}
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4">
                        <div>
                            <p className="text-sm text-muted-foreground mb-2">
                                {t('admin.invitationCreateDialog.copyLinkHint')}
                            </p>
                            <div className="flex items-center gap-2">
                                <Input value={result.invite_link} readOnly className="font-mono text-xs" />
                                <Button
                                    variant="outline"
                                    size="icon"
                                    onClick={() => handleCopyLink(result.invite_link)}
                                    aria-label={t('admin.invitationCreateDialog.copyLinkAria')}
                                >
                                    <Copy className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                        <p className="text-sm text-muted-foreground">
                            {t('admin.invitationCreateDialog.expiresAt', { date: new Date(result.invitation.expires_at).toLocaleDateString(uiLocale()) })}
                        </p>
                    </div>
                    <DialogFooter>
                        <Button onClick={handleClose}>{t('admin.invitationCreateDialog.done')}</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        )
    }

    return (
        <Dialog open={open} onOpenChange={handleClose}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t('admin.invitationCreateDialog.title')}</DialogTitle>
                    <DialogDescription>
                        {t('admin.invitationCreateDialog.description')}
                    </DialogDescription>
                </DialogHeader>
                <form
                    onSubmit={form.handleSubmit((data) => createMutation.mutate(data))}
                    className="space-y-4 max-h-[70vh] overflow-y-auto pr-1"
                >
                    <FormField label={t('common.email')} htmlFor="invite-email" error={form.formState.errors.email?.message} required>
                        <Input
                            id="invite-email"
                            type="email"
                            placeholder="wang.daming@hospital.org"
                            {...form.register('email', {
                                required: t('admin.invitationCreateDialog.emailRequired'),
                                pattern: { value: EMAIL_PATTERN, message: t('admin.invitationCreateDialog.emailRequired') },
                            })}
                        />
                    </FormField>

                    <FormField label={t('admin.invitationCreateDialog.nameLabel')} htmlFor="invite-name" error={form.formState.errors.display_name?.message} required>
                        <Input id="invite-name" placeholder={t('admin.invitationCreateDialog.namePlaceholder')} {...form.register('display_name', {
                            required: t('admin.invitationCreateDialog.nameRequired'),
                            maxLength: { value: 100, message: t('admin.invitationCreateDialog.nameMaxLength') },
                        })} />
                    </FormField>

                    <FormField label={t('admin.invitationCreateDialog.orgLabel')} htmlFor="invite-org" error={form.formState.errors.organization?.message} required>
                        <Input id="invite-org" placeholder={t('admin.invitationCreateDialog.orgPlaceholder')} {...form.register('organization', {
                            required: t('admin.invitationCreateDialog.orgRequired'),
                            maxLength: { value: 255, message: t('admin.invitationCreateDialog.orgMaxLength') },
                        })} />
                    </FormField>

                    <div className="grid grid-cols-2 gap-3">
                        <FormField label={t('admin.invitationCreateDialog.phoneLabel')} htmlFor="invite-phone" error={form.formState.errors.phone?.message}>
                            <Input id="invite-phone" placeholder="0912345678" {...form.register('phone', {
                                maxLength: { value: 20, message: t('admin.invitationCreateDialog.phoneMaxLength') },
                            })} />
                        </FormField>
                        <FormField label={t('admin.invitationCreateDialog.positionLabel')} htmlFor="invite-position" error={form.formState.errors.position?.message}>
                            <Input id="invite-position" placeholder={t('admin.invitationCreateDialog.positionPlaceholder')} {...form.register('position', {
                                maxLength: { value: 100, message: t('admin.invitationCreateDialog.positionMaxLength') },
                            })} />
                        </FormField>
                    </div>

                    <FormField label={t('admin.invitationCreateDialog.rolesLabel')} error={form.formState.errors.role_ids?.message} required>
                        <input
                            type="hidden"
                            {...form.register('role_ids', {
                                validate: v => (v && v.length > 0) || t('admin.invitationCreateDialog.rolesRequired'),
                            })}
                        />
                        <div className="flex flex-wrap gap-2 p-3 border rounded-md min-h-[3rem]">
                            {rolesQuery.isLoading && (
                                <span className="text-sm text-muted-foreground">{t('admin.invitationCreateDialog.rolesLoading')}</span>
                            )}
                            {/* 角色清單載不出來時這一區只會是空白框，而角色又是必填——
                                使用者會卡在「送不出去也看不出為什麼」。 */}
                            {rolesQuery.isError && !rolesQuery.data?.length && (
                                <span className="text-sm text-status-error-text">{t('common.loadFailed')}</span>
                            )}
                            {rolesQuery.data?.map((role) => {
                                const selected = roleIds.includes(role.id)
                                return (
                                    <Badge
                                        key={role.id}
                                        variant={selected ? 'default' : 'outline'}
                                        className="cursor-pointer"
                                        role="checkbox"
                                        tabIndex={0}
                                        aria-checked={selected}
                                        onClick={() => toggleRole(role.id)}
                                        onKeyDown={onActivateKey(() => toggleRole(role.id))}
                                    >
                                        {role.name}
                                        {role.is_internal && (
                                            <span className="ml-1 text-[10px] opacity-60">{t('admin.invitationCreateDialog.internalRole')}</span>
                                        )}
                                    </Badge>
                                )
                            })}
                        </div>
                    </FormField>

                    {/* 身分二擇一。這個值決定受邀者接受邀請後的 users.is_internal——
                        先前這裡沒有欄位，接受端一律硬編 false，導致所有走邀請進來的
                        人（含具內部角色者）都被記成外部人員。 */}
                    <input
                        type="hidden"
                        {...form.register('is_internal', {
                            // 用 typeof 而非 `v !== null`：後者放行 undefined，
                            // 而下方送出時原本的 `?? false` 會把它靜默補成「外部人員」
                            // ——正是本 PR 要消滅的那個靜默猜測。
                            validate: v =>
                                typeof v === 'boolean' ||
                                '請選擇受邀者是本場受僱人員或外部人員',
                        })}
                    />
                    <StaffAffiliationField
                        value={isInternal ?? null}
                        onChange={next => form.setValue('is_internal', next, { shouldValidate: true })}
                        selectedRoleCodes={selectedRoleCodes}
                        error={form.formState.errors.is_internal?.message}
                    />

                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={handleClose}>{t('common.cancel')}</Button>
                        <Button type="submit" disabled={createMutation.isPending}>
                            {createMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            {t('admin.invitationCreateDialog.submit')}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
