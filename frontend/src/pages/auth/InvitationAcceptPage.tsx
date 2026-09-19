import { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { useMutation } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'
import { Loader2, Eye, EyeOff, AlertTriangle } from 'lucide-react'

import { useToggle } from '@/hooks/useToggle'
import { invitationApi } from '@/lib/api/invitation'
import { getApiErrorMessage } from '@/lib/apiError'
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

interface AcceptForm {
    display_name: string
    phone: string
    organization: string
    position?: string
    password: string
    confirm_password: string
    agree_terms: boolean
}

const PHONE_PATTERN = /^\d{9,10}$/
const PASSWORD_CHAR_PATTERN = /(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/

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
                verify={verifyState.data}
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
    const { t } = useTranslation()
    return (
        <Card className="w-full max-w-md animate-fade-in">
            <CardContent className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                <span className="ml-3 text-muted-foreground">{t('auth.invitation.verifying')}</span>
            </CardContent>
        </Card>
    )
}

function InvalidState({ reason }: { reason: string }) {
    const { t } = useTranslation()
    const messages: Record<string, { title: string; desc: React.ReactNode }> = {
        already_accepted: {
            title: t('auth.invitation.alreadyAcceptedTitle'),
            desc: (
                <Trans
                    i18nKey="auth.invitation.alreadyAcceptedDescription"
                    components={{ resetLink: <Link to="/forgot-password" className="text-primary hover:underline ml-1" /> }}
                />
            ),
        },
        expired: {
            title: t('auth.invitation.expiredTitle'),
            desc: t('auth.invitation.expiredDescription'),
        },
        revoked: {
            title: t('auth.invitation.revokedTitle'),
            desc: t('auth.invitation.revokedDescription'),
        },
        not_found: {
            title: t('auth.invitation.notFoundTitle'),
            desc: t('auth.invitation.notFoundDescription'),
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
    verify: InvitationVerifyResponse
    showPassword: boolean
    togglePassword: () => void
    navigate: ReturnType<typeof useNavigate>
}

function RegistrationForm({ token, verify, showPassword, togglePassword, navigate }: RegistrationFormProps) {
    const { t } = useTranslation()
    const email = verify.email!
    const form = useForm<AcceptForm>({
        defaultValues: {
            display_name: verify.display_name || '',
            phone: verify.phone || '',
            organization: verify.organization || '',
            position: verify.position || '',
            password: '',
            confirm_password: '',
            agree_terms: false,
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
            // 2026-05-18: 移除 sessionExpiresAt（前端不再做 time-based 倒數）。
            // accessTokenExpiresAt 會由 useProactiveRefresh bootstrap 階段呼叫
            // /auth/refresh 後建立，這裡不需要手動寫死。
            useAuthStore.setState({
                user,
                isAuthenticated: true,
                isInitialized: true,
            })
            toast({ title: t('auth.invitation.welcomeToastTitle'), description: t('auth.invitation.welcomeToastDescription') })
            navigate('/my-projects')
        },
        onError: (err) => {
            toast({ variant: 'destructive', title: t('auth.invitation.registerFailedTitle'), description: getApiErrorMessage(err) })
        },
    })

    return (
        <Card className="w-full max-w-lg animate-fade-in">
            <CardHeader className="text-center">
                <div className="mx-auto mb-4">
                    <img src="/pigmodel-logo.png" alt="Logo" className="h-20 w-auto" />
                </div>
                <CardTitle className="text-2xl font-bold">{t('auth.invitation.completeRegistration')}</CardTitle>
                <CardDescription>{t('auth.invitation.description')}</CardDescription>
            </CardHeader>
            <CardContent>
                <form onSubmit={form.handleSubmit((d) => acceptMutation.mutate(d))} className="space-y-4">
                    <FormField label={t('common.email')} htmlFor="reg-email">
                        <Input id="reg-email" value={email} readOnly className="bg-muted" />
                    </FormField>

                    {verify.roles.length > 0 && (
                        <FormField label={t('auth.invitation.assignedRolesLabel')} htmlFor="reg-roles">
                            <div
                                id="reg-roles"
                                className="flex flex-wrap gap-2 px-3 py-2 rounded-md bg-muted text-sm"
                            >
                                {verify.roles.map((r) => (
                                    <span
                                        key={r.id}
                                        className="inline-flex items-center rounded-full border border-border bg-background px-2 py-0.5 text-xs"
                                    >
                                        {r.name}
                                    </span>
                                ))}
                            </div>
                        </FormField>
                    )}

                    <FormField label={t('auth.invitation.nameLabel')} htmlFor="reg-name" error={form.formState.errors.display_name?.message} required>
                        <Input id="reg-name" placeholder={t('auth.invitation.namePlaceholder')} {...form.register('display_name', {
                            required: t('auth.invitation.nameRequired'),
                            maxLength: { value: 100, message: t('auth.invitation.nameTooLong') },
                        })} />
                    </FormField>

                    <FormField label={t('auth.invitation.phoneLabel')} htmlFor="reg-phone" error={form.formState.errors.phone?.message} required>
                        <Input id="reg-phone" placeholder="0912345678" {...form.register('phone', {
                            required: t('auth.invitation.phoneInvalid'),
                            pattern: { value: PHONE_PATTERN, message: t('auth.invitation.phoneInvalid') },
                        })} />
                    </FormField>

                    <FormField label={t('auth.invitation.organizationLabel')} htmlFor="reg-org" error={form.formState.errors.organization?.message} required>
                        <Input id="reg-org" placeholder={t('auth.invitation.organizationPlaceholder')} {...form.register('organization', { required: t('auth.invitation.organizationRequired') })} />
                    </FormField>

                    <FormField label={t('auth.invitation.positionLabel')} htmlFor="reg-position">
                        <Input id="reg-position" placeholder={t('auth.invitation.positionPlaceholder')} {...form.register('position')} />
                    </FormField>

                    <FormField label={t('auth.fields.password')} htmlFor="reg-password" error={form.formState.errors.password?.message} required>
                        <div className="relative">
                            <Input
                                id="reg-password"
                                type={showPassword ? 'text' : 'password'}
                                placeholder={t('auth.invitation.passwordPlaceholder')}
                                {...form.register('password', {
                                    required: t('auth.passwordRules.minLengthShort', { min: 10 }),
                                    minLength: { value: 10, message: t('auth.passwordRules.minLengthShort', { min: 10 }) },
                                    pattern: { value: PASSWORD_CHAR_PATTERN, message: t('auth.invitation.passwordCharsRequired') },
                                })}
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

                    <FormField label={t('auth.invitation.confirmPasswordLabel')} htmlFor="reg-confirm" error={form.formState.errors.confirm_password?.message} required>
                        <Input id="reg-confirm" type="password" placeholder={t('auth.invitation.confirmPasswordPlaceholder')} {...form.register('confirm_password', {
                            validate: (value, formValues) => value === formValues.password || t('auth.invitation.passwordMismatch'),
                        })} />
                    </FormField>

                    <div className="flex items-start gap-2">
                        <input type="hidden" {...form.register('agree_terms', {
                            validate: v => v === true || t('auth.invitation.termsRequired'),
                        })} />
                        <Checkbox
                            id="reg-terms"
                            checked={form.watch('agree_terms') === true}
                            onCheckedChange={(checked) => form.setValue('agree_terms', checked === true, { shouldValidate: true })}
                        />
                        <label htmlFor="reg-terms" className="text-sm leading-tight cursor-pointer">
                            <Trans
                                i18nKey="auth.invitation.agreeTerms"
                                components={{
                                    termsLink: <a href="/terms" target="_blank" rel="noopener noreferrer" className="text-primary hover:underline ml-1" />,
                                }}
                            />
                        </label>
                    </div>
                    {form.formState.errors.agree_terms && (
                        <p className="text-sm text-destructive">{form.formState.errors.agree_terms.message}</p>
                    )}

                    <Button type="submit" className="w-full" disabled={acceptMutation.isPending}>
                        {acceptMutation.isPending ? (
                            <><Loader2 className="mr-2 h-4 w-4 animate-spin" />{t('auth.invitation.creatingAccount')}</>
                        ) : t('auth.invitation.completeRegistration')}
                    </Button>
                </form>
            </CardContent>
        </Card>
    )
}
