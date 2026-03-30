import { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useMutation } from '@tanstack/react-query'
import { Loader2, Eye, EyeOff, AlertTriangle } from 'lucide-react'

import { useToggle } from '@/hooks/useToggle'
import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/validation'
import { useAuthStore } from '@/stores/auth'
import {
    getPasswordStrength, getStrengthLabel, getStrengthColor,
} from '@/lib/passwordValidation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { FormField } from '@/components/ui/form-field'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import type { InvitationVerifyResponse } from '@/types/invitation'

const acceptInvitationSchema = z.object({
    display_name: z.string().min(1, '姓名為必填').max(100, '姓名不得超過 100 字'),
    phone: z.string().regex(/^\d{9,10}$/, '電話必須為 9-10 位數字'),
    organization: z.string().min(1, '組織為必填'),
    position: z.string().optional(),
    password: z.string().min(10, '密碼至少 10 個字元').regex(
        /(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/,
        '密碼必須包含大小寫字母及數字',
    ),
    confirm_password: z.string(),
    agree_terms: z.literal(true, { error: '必須同意服務條款' }),
}).refine(data => data.password === data.confirm_password, {
    message: '密碼不一致', path: ['confirm_password'],
})

type AcceptForm = z.infer<typeof acceptInvitationSchema>

export function InvitationAcceptPage() {
    const { token } = useParams<{ token: string }>()
    const navigate = useNavigate()
    const [showPassword, togglePassword] = useToggle()
    const [verifyState, setVerifyState] = useState<
        | { status: 'loading' }
        | { status: 'valid'; data: InvitationVerifyResponse }
        | { status: 'invalid'; reason: string }
    >({ status: 'loading' })

    useEffect(() => {
        if (!token) {
            setVerifyState({ status: 'invalid', reason: 'not_found' })
            return
        }
        invitationApi.verify(token)
            .then(res => {
                if (res.data.valid) {
                    setVerifyState({ status: 'valid', data: res.data })
                } else {
                    setVerifyState({ status: 'invalid', reason: res.data.reason || 'unknown' })
                }
            })
            .catch(() => {
                setVerifyState({ status: 'invalid', reason: 'not_found' })
            })
    }, [token])

    if (verifyState.status === 'loading') {
        return <AcceptPageShell><LoadingState /></AcceptPageShell>
    }

    if (verifyState.status === 'invalid') {
        return <AcceptPageShell><InvalidState reason={verifyState.reason} /></AcceptPageShell>
    }

    return (
        <AcceptPageShell>
            <RegistrationForm
                token={token!}
                email={verifyState.data.email!}
                organization={verifyState.data.organization}
                showPassword={showPassword}
                togglePassword={togglePassword}
                navigate={navigate}
            />
        </AcceptPageShell>
    )
}

function AcceptPageShell({ children }: { children: React.ReactNode }) {
    return (
        <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900">
            <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiMyMDIwMjAiIGZpbGwtb3BhY2l0eT0iMC4wNSI+PHBhdGggZD0iTTM2IDM0djJoLTJ2LTJoMnptMC00aDJ2MmgtMnYtMnptLTQgMHYyaC0ydi0yaDJ6bTIgMGgydjJoLTJ2LTJ6bS0yIDRoMnYyaC0ydi0yem0yIDBoMnYyaC0ydi0yeiIvPjwvZz48L2c+PC9zdmc+')] opacity-40" />
            <div className="relative flex min-h-screen items-center justify-center p-4">
                {children}
            </div>
        </div>
    )
}

function LoadingState() {
    return (
        <Card className="w-full max-w-md animate-fade-in">
            <CardContent className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                <span className="ml-3 text-muted-foreground">驗證邀請連結中...</span>
            </CardContent>
        </Card>
    )
}

function InvalidState({ reason }: { reason: string }) {
    const messages: Record<string, { title: string; desc: React.ReactNode }> = {
        already_accepted: {
            title: '此邀請已使用',
            desc: <>如忘記密碼，請前往<Link to="/forgot-password" className="text-primary hover:underline ml-1">重設密碼</Link></>,
        },
        expired: {
            title: '此邀請已過期',
            desc: '請聯繫管理員重新發送邀請',
        },
        revoked: {
            title: '此邀請已被撤銷',
            desc: '請聯繫管理員取得新的邀請連結',
        },
        not_found: {
            title: '此邀請連結無效',
            desc: '請確認連結是否正確，或聯繫管理員',
        },
    }
    const msg = messages[reason] || messages.not_found

    return (
        <Card className="w-full max-w-md animate-fade-in">
            <CardContent className="text-center py-12">
                <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-status-warning-text" />
                <h2 className="text-xl font-semibold mb-2">{msg.title}</h2>
                <p className="text-muted-foreground">{msg.desc}</p>
            </CardContent>
        </Card>
    )
}

interface RegistrationFormProps {
    token: string
    email: string
    organization?: string
    showPassword: boolean
    togglePassword: () => void
    navigate: ReturnType<typeof useNavigate>
}

function RegistrationForm({ token, email, organization, showPassword, togglePassword, navigate }: RegistrationFormProps) {
    const form = useForm<AcceptForm>({
        resolver: zodResolver(acceptInvitationSchema),
        defaultValues: {
            display_name: '',
            phone: '',
            organization: organization || '',
            position: '',
            password: '',
            confirm_password: '',
            agree_terms: false as never,
        },
    })

    const passwordValue = form.watch('password')
    const strength = getPasswordStrength(passwordValue || '')

    const acceptMutation = useMutation({
        mutationFn: (data: AcceptForm) =>
            invitationApi.accept({
                invitation_token: token,
                display_name: data.display_name,
                phone: data.phone,
                organization: data.organization,
                password: data.password,
                position: data.position || undefined,
                agree_terms: data.agree_terms,
            }),
        onSuccess: (res) => {
            const { user } = res.data
            useAuthStore.setState({
                user,
                isAuthenticated: true,
                isInitialized: true,
                sessionExpiresAt: Date.now() + 6 * 60 * 60 * 1000,
            })
            toast({ title: '歡迎加入！', description: '帳號已建立，即將前往我的計劃書' })
            navigate('/my-projects')
        },
        onError: (err) => {
            toast({ variant: 'destructive', title: '註冊失敗', description: getApiErrorMessage(err) })
        },
    })

    return (
        <Card className="w-full max-w-lg animate-fade-in">
            <CardHeader className="text-center">
                <div className="mx-auto mb-4">
                    <img src="/pigmodel-logo.png" alt="Logo" className="h-20 w-auto" />
                </div>
                <CardTitle className="text-2xl font-bold">完成註冊</CardTitle>
                <CardDescription>請填寫以下資料完成帳號設定</CardDescription>
            </CardHeader>
            <CardContent>
                <form onSubmit={form.handleSubmit((d) => acceptMutation.mutate(d))} className="space-y-4">
                    <FormField label="Email" htmlFor="reg-email">
                        <Input id="reg-email" value={email} readOnly className="bg-muted" />
                    </FormField>

                    <FormField label="姓名" htmlFor="reg-name" error={form.formState.errors.display_name?.message} required>
                        <Input id="reg-name" placeholder="王大明" {...form.register('display_name')} />
                    </FormField>

                    <FormField label="電話" htmlFor="reg-phone" error={form.formState.errors.phone?.message} required>
                        <Input id="reg-phone" placeholder="0912345678" {...form.register('phone')} />
                    </FormField>

                    <FormField label="組織" htmlFor="reg-org" error={form.formState.errors.organization?.message} required>
                        <Input id="reg-org" placeholder="台大醫院" {...form.register('organization')} />
                    </FormField>

                    <FormField label="職稱（選填）" htmlFor="reg-position">
                        <Input id="reg-position" placeholder="主治醫師" {...form.register('position')} />
                    </FormField>

                    <FormField label="密碼" htmlFor="reg-password" error={form.formState.errors.password?.message} required>
                        <div className="relative">
                            <Input
                                id="reg-password"
                                type={showPassword ? 'text' : 'password'}
                                placeholder="至少 10 字元，含大小寫及數字"
                                {...form.register('password')}
                            />
                            <Button type="button" variant="ghost" size="icon" className="absolute right-0 top-0 h-full px-3 hover:bg-transparent" onClick={togglePassword}>
                                {showPassword ? <EyeOff className="h-4 w-4 text-muted-foreground" /> : <Eye className="h-4 w-4 text-muted-foreground" />}
                            </Button>
                        </div>
                        {passwordValue && (
                            <div className="mt-2 space-y-1">
                                <div className="flex gap-1">
                                    {Array.from({ length: 5 }).map((_, i) => (
                                        <div key={i} className={`h-1.5 flex-1 rounded-full ${i < strength ? getStrengthColor(strength) : 'bg-muted'}`} />
                                    ))}
                                </div>
                                <p className="text-xs text-muted-foreground">{getStrengthLabel(strength)}</p>
                            </div>
                        )}
                    </FormField>

                    <FormField label="確認密碼" htmlFor="reg-confirm" error={form.formState.errors.confirm_password?.message} required>
                        <Input id="reg-confirm" type="password" placeholder="再次輸入密碼" {...form.register('confirm_password')} />
                    </FormField>

                    <div className="flex items-start gap-2">
                        <Checkbox
                            id="reg-terms"
                            checked={form.watch('agree_terms') === true}
                            onCheckedChange={(checked) => form.setValue('agree_terms', checked === true ? true : false as never)}
                        />
                        <label htmlFor="reg-terms" className="text-sm leading-tight cursor-pointer">
                            我同意
                            <a href="/terms" target="_blank" rel="noopener noreferrer" className="text-primary hover:underline ml-1">
                                服務條款
                            </a>
                        </label>
                    </div>
                    {form.formState.errors.agree_terms && (
                        <p className="text-sm text-destructive">{form.formState.errors.agree_terms.message}</p>
                    )}

                    <Button type="submit" className="w-full" disabled={acceptMutation.isPending}>
                        {acceptMutation.isPending ? (
                            <><Loader2 className="mr-2 h-4 w-4 animate-spin" />建立帳號中...</>
                        ) : '完成註冊'}
                    </Button>
                </form>
            </CardContent>
        </Card>
    )
}
