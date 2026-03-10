import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/auth'
import api, { User, UpdateUserRequest } from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { TwoFactorSetup } from '@/components/auth/TwoFactorSetup'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { useToast } from '@/components/ui/use-toast'
import {
    User as UserIcon,
    GraduationCap,
    Award,
    Save,
    Loader2,
    Mail,
    Phone,
    Building2,
    Briefcase,
    History,
    CheckCircle2,
    AlertCircle
} from 'lucide-react'
import { cn } from '@/lib/utils'

const TRAINING_OPTIONS = ['A', 'B', 'C', 'D', 'E', 'F']

const AUP_ROLE_OPTIONS = [
    { value: 'PI', key: 'PI' },
    { value: 'Co-PI', key: 'Co-PI' },
    { value: 'Experimenter', key: 'Experimenter' },
    { value: 'Experiment Staff', key: 'ExperimentStaff' },
    { value: 'Veterinarian', key: 'Veterinarian' },
    { value: 'Animal Care Staff', key: 'AnimalCareStaff' }
]

export function ProfileSettingsPage() {
    const { t } = useTranslation()
    const { user: currentUser, checkAuth, hasRole } = useAuthStore()
    const { toast } = useToast()
    const queryClient = useQueryClient()

    const [formData, setFormData] = useState<UpdateUserRequest>({
        display_name: '',
        phone: '',
        phone_ext: '',
        organization: '',
        entry_date: '',
        position: '',
        aup_roles: [],
        years_experience: 0,
        trainings: [],
    })

    // Sync with currentUser when loaded
    useEffect(() => {
        if (currentUser) {
            setFormData({
                display_name: currentUser.display_name || '',
                phone: currentUser.phone || '',
                phone_ext: currentUser.phone_ext || '',
                organization: currentUser.organization || '',
                entry_date: currentUser.entry_date || '',
                position: currentUser.position || '',
                aup_roles: currentUser.aup_roles || [],
                years_experience: currentUser.years_experience || 0,
                trainings: currentUser.trainings || [],
            })
        }
    }, [currentUser])

    const updateMutation = useMutation({
        mutationFn: async (data: UpdateUserRequest) => {
            const response = await api.put<User>('/me', data)
            return response.data
        },
        onSuccess: () => {
            toast({
                title: t('common.success'),
                description: t('profile.updateSuccess'),
            })
            queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
            checkAuth() // Refresh local store
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getErrorMessage(error) || t('common.error'),
                variant: 'destructive',
            })
        },
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        // 將空字串轉為 null，避免後端 NaiveDate 解析錯誤
        const payload: UpdateUserRequest = {
            ...formData,
            entry_date: formData.entry_date || null,
            position: formData.position || null,
            trainings: formData.trainings?.map(t => ({
                ...t,
                received_date: t.received_date || undefined,
                certificate_no: t.certificate_no || undefined,
            })),
        }
        updateMutation.mutate(payload)
    }

    const toggleAupRole = (roleValue: string) => {
        const roles = formData.aup_roles || []
        if (roles.includes(roleValue)) {
            setFormData({ ...formData, aup_roles: roles.filter(r => r !== roleValue) })
        } else {
            setFormData({ ...formData, aup_roles: [...roles, roleValue] })
        }
    }

    const toggleTraining = (code: string) => {
        const trainings = formData.trainings || []
        const exists = trainings.find(t => t.code === code)
        if (exists) {
            setFormData({ ...formData, trainings: trainings.filter(t => t.code !== code) })
        } else {
            setFormData({
                ...formData,
                trainings: [...trainings, { code, certificate_no: '', received_date: '' }]
            })
        }
    }

    const updateTrainingDetail = (code: string, field: 'certificate_no' | 'received_date', value: string) => {
        const trainings = (formData.trainings || []).map(t => {
            if (t.code === code) {
                return { ...t, [field]: value }
            }
            return t
        })
        setFormData({ ...formData, trainings })
    }

    // Staff roles that should see AUP Section 8 Data
    // PI-only users should NOT see/edit this section
    const staffRoles = ['IACUC_STAFF', 'EXPERIMENT_STAFF', 'VET', 'ANIMAL_CARE_STAFF', 'admin', 'SYSTEM_ADMIN']
    const isStaff = currentUser?.roles?.some(r => staffRoles.includes(r)) ?? false

    if (!currentUser) {
        return (
            <div className="flex items-center justify-center h-[60vh]">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
        )
    }

    return (
        <div className="max-w-5xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-4xl font-extrabold tracking-tight bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
                        {t('profile.settings')}
                    </h1>
                    <p className="text-muted-foreground mt-1">
                        {t('profile.description')}
                    </p>
                </div>
                <Button
                    size="lg"
                    className="shadow-lg shadow-blue-500/20 bg-blue-600 hover:bg-blue-700 transition-all"
                    onClick={handleSubmit}
                    disabled={updateMutation.isPending}
                >
                    {updateMutation.isPending ? (
                        <Loader2 className="mr-2 h-5 w-5 animate-spin" />
                    ) : (
                        <Save className="mr-2 h-5 w-5" />
                    )}
                    {t('common.saveChanges')}
                </Button>
            </div>

            <div className={`grid grid-cols-1 ${isStaff ? 'lg:grid-cols-3' : ''} gap-8`}>
                {/* Left Column: Basic Info */}
                <div className={`space-y-8 ${isStaff ? 'lg:col-span-1' : ''}`}>
                    <Card className="border-none shadow-xl bg-white/80 backdrop-blur-sm">
                        <CardHeader className="border-b bg-slate-50/50">
                            <CardTitle className="flex items-center gap-2 text-slate-800">
                                <UserIcon className="h-5 w-5 text-blue-500" />
                                {t('profile.basicInfo')}
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="pt-6 space-y-4">
                            <div className="flex flex-col items-center pb-6 border-b border-slate-100">
                                <div className="h-24 w-24 rounded-full bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center text-white text-3xl font-bold shadow-inner">
                                    {currentUser.display_name?.[0] || currentUser.email?.[0]}
                                </div>
                                <h3 className="mt-4 font-bold text-lg text-slate-900">{currentUser.display_name}</h3>
                                <Badge variant="secondary" className="mt-1">{currentUser.roles.join(', ')}</Badge>
                            </div>

                            <div className="space-y-4 pt-4">
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2 text-slate-600">
                                        <Mail className="h-4 w-4" /> {t('profile.email')} {t('profile.readOnly')}
                                    </Label>
                                    <Input value={currentUser.email} disabled className="bg-slate-50" />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="display_name" className="flex items-center gap-2">
                                        {t('profile.displayName')}
                                    </Label>
                                    <Input
                                        id="display_name"
                                        value={formData.display_name}
                                        onChange={e => setFormData({ ...formData, display_name: e.target.value })}
                                        placeholder={t('profile.displayNamePlaceholder')}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="phone" className="flex items-center gap-2">
                                        <Phone className="h-4 w-4" /> {t('profile.phone')}
                                    </Label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="phone"
                                            className="flex-1"
                                            value={formData.phone || ''}
                                            onChange={e => setFormData({ ...formData, phone: e.target.value })}
                                            placeholder={t('profile.phonePlaceholder')}
                                        />
                                        <div className="flex items-center gap-2 shrink-0">
                                            <span className="text-muted-foreground">#</span>
                                            <Input
                                                className="w-24"
                                                placeholder={t('aup.basic.piExtension')}
                                                value={formData.phone_ext || ''}
                                                onChange={e => setFormData({ ...formData, phone_ext: e.target.value })}
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="organization" className="flex items-center gap-2">
                                        <Building2 className="h-4 w-4" /> {t('profile.organization')}
                                    </Label>
                                    <Input
                                        id="organization"
                                        value={formData.organization || ''}
                                        onChange={e => setFormData({ ...formData, organization: e.target.value })}
                                        placeholder={t('profile.organizationPlaceholder')}
                                    />
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* Right Column: AUP Section 8 Data - Only visible to staff */}
                {isStaff && (
                    <div className="lg:col-span-2 space-y-8">
                        <Card className="border-none shadow-xl bg-white/80 backdrop-blur-sm">
                            <CardHeader className="border-b bg-slate-50/50">
                                <div className="flex items-center justify-between">
                                    <CardTitle className="flex items-center gap-2 text-slate-800">
                                        <GraduationCap className="h-5 w-5 text-indigo-500" />
                                        {t('profile.aupSection8')}
                                    </CardTitle>
                                    <Badge variant="outline" className="border-indigo-200 text-indigo-600">符合規範</Badge>
                                </div>
                                <CardDescription>
                                    {t('profile.aupSection8Description')}
                                </CardDescription>
                            </CardHeader>
                            <CardContent className="pt-6 space-y-6">
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    <div className="space-y-2">
                                        <Label htmlFor="entry_date" className="flex items-center gap-2">
                                            <History className="h-4 w-4 text-slate-400" /> {t('profile.entryDate')}
                                        </Label>
                                        <Input
                                            id="entry_date"
                                            type="date"
                                            value={formData.entry_date || ''}
                                            onChange={e => setFormData({ ...formData, entry_date: e.target.value })}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="position" className="flex items-center gap-2">
                                            <Briefcase className="h-4 w-4 text-slate-400" /> {t('profile.position') || '職稱'}
                                        </Label>
                                        <Input
                                            id="position"
                                            value={formData.position || ''}
                                            onChange={e => setFormData({ ...formData, position: e.target.value })}
                                            placeholder={t('profile.positionPlaceholder')}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="years_experience" className="flex items-center gap-2">
                                            {t('profile.yearsExperience')}
                                        </Label>
                                        <Input
                                            id="years_experience"
                                            type="number"
                                            min={0}
                                            value={formData.years_experience}
                                            onChange={e => setFormData({ ...formData, years_experience: parseInt(e.target.value) || 0 })}
                                        />
                                    </div>
                                </div>

                                <div className="space-y-4 pt-4 border-t border-slate-100">
                                    <Label className="text-base font-semibold">{t('profile.aupRoles')}</Label>
                                    <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                                        {AUP_ROLE_OPTIONS.map(role => (
                                            <div
                                                key={role.value}
                                                className={cn(
                                                    "flex items-center space-x-2 p-3 rounded-lg border transition-all cursor-pointer",
                                                    formData.aup_roles?.includes(role.value)
                                                        ? "bg-indigo-50 border-indigo-200 ring-1 ring-indigo-200"
                                                        : "hover:bg-slate-50 border-slate-200"
                                                )}
                                                onClick={() => toggleAupRole(role.value)}
                                            >
                                                <Checkbox
                                                    id={`role-${role.value}`}
                                                    checked={formData.aup_roles?.includes(role.value)}
                                                    onCheckedChange={() => toggleAupRole(role.value)}
                                                />
                                                <label
                                                    htmlFor={`role-${role.value}`}
                                                    className="text-sm font-medium leading-none cursor-pointer"
                                                >
                                                    {t(`profile.roles.${role.key}`)}
                                                </label>
                                            </div>
                                        ))}
                                    </div>
                                </div>



                                <div className="space-y-4 pt-4 border-t border-slate-100">
                                    <div className="flex items-center justify-between">
                                        <Label className="text-base font-semibold flex items-center gap-2">
                                            <Award className="h-5 w-5 text-amber-500" />
                                            {t('profile.trainings')}
                                        </Label>
                                        <div className="flex gap-1">
                                            {formData.trainings?.length === 0 ? (
                                                <Badge variant="destructive" className="animate-pulse">{t('profile.notFilled')}</Badge>
                                            ) : (
                                                <Badge variant="success" className="bg-green-500">{t('profile.completedCount', { count: formData.trainings?.length })}</Badge>
                                            )}
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        {TRAINING_OPTIONS.map(code => {
                                            const training = formData.trainings?.find(t => t.code === code)
                                            return (
                                                <div
                                                    key={code}
                                                    className={cn(
                                                        "p-4 rounded-xl border transition-all space-y-3",
                                                        training
                                                            ? "bg-amber-50/50 border-amber-200 shadow-sm"
                                                            : "border-slate-100 hover:border-slate-200"
                                                    )}
                                                >
                                                    <div className="flex items-center space-x-3">
                                                        <Checkbox
                                                            id={`training-${code}`}
                                                            checked={!!training}
                                                            onCheckedChange={() => toggleTraining(code)}
                                                        />
                                                        <label
                                                            htmlFor={`training-${code}`}
                                                            className="text-sm font-bold leading-none cursor-pointer text-slate-700"
                                                        >
                                                            {t(`aup.personnel.trainings.${code}`)}
                                                        </label>
                                                    </div>

                                                    {training && (
                                                        <div className="pl-7 space-y-3 animate-in fade-in slide-in-from-top-2 duration-300">
                                                            <div className="space-y-1.5">
                                                                <Label className="text-[10px] uppercase tracking-wider text-slate-400 font-bold">
                                                                    {t('profile.certNo')}
                                                                </Label>
                                                                <Input
                                                                    size={30}
                                                                    className="h-8 text-xs bg-white"
                                                                    placeholder={t('profile.certNoPlaceholder')}
                                                                    value={training.certificate_no || ''}
                                                                    onChange={e => updateTrainingDetail(code, 'certificate_no', e.target.value)}
                                                                />
                                                            </div>
                                                        </div>
                                                    )}
                                                </div>
                                            )
                                        })}
                                    </div>

                                    {formData.trainings && formData.trainings.length > 0 && (
                                        <div className="mt-6 space-y-4 bg-slate-50 p-4 rounded-xl border border-slate-200 border-dashed">
                                            <h4 className="text-sm font-bold text-slate-700">{t('profile.trainingDetailTitle')}</h4>
                                            {formData.trainings.map((training) => (
                                                <div key={training.code} className="grid grid-cols-1 md:grid-cols-12 gap-4 items-center animate-in zoom-in-95 duration-200">
                                                    <div className="md:col-span-1">
                                                        <div className="h-8 w-8 rounded-full bg-blue-600 flex items-center justify-center text-white font-bold text-sm">
                                                            {training.code}
                                                        </div>
                                                    </div>
                                                    <div className="md:col-span-6">
                                                        <Input
                                                            placeholder={t('profile.certNo')}
                                                            value={training.certificate_no || ''}
                                                            onChange={(e) => updateTrainingDetail(training.code, 'certificate_no', e.target.value)}
                                                            onClick={(e) => e.stopPropagation()}
                                                            className="bg-white"
                                                        />
                                                    </div>
                                                    <div className="md:col-span-5">
                                                        <Input
                                                            type="date"
                                                            value={training.received_date || ''}
                                                            onChange={(e) => updateTrainingDetail(training.code, 'received_date', e.target.value)}
                                                        />
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    )}

                                    <div className="rounded-lg bg-amber-50 p-4 border border-amber-200 flex gap-3">
                                        <AlertCircle className="h-5 w-5 text-amber-600 shrink-0 mt-0.5" />
                                        <div className="text-xs text-amber-800 space-y-1">
                                            <p className="font-bold">{t('profile.trainingNotice')}</p>
                                            <p>{t('aup.personnel.roles.list')}</p>
                                            <p className="opacity-80">{t('profile.trainingNoticeDetail')}</p>
                                        </div>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>

                        {/* 兩步驟驗證（僅管理員） */}
                        {hasRole('admin') && (
                            <TwoFactorSetup
                                totpEnabled={currentUser?.totp_enabled ?? false}
                                onStatusChange={() => checkAuth()}
                            />
                        )}

                        <Card className="border-none shadow-xl bg-gradient-to-br from-slate-900 to-slate-800 text-white overflow-hidden relative">
                            <div className="absolute top-0 right-0 p-8 opacity-10">
                                <CheckCircle2 className="h-32 w-32" />
                            </div>
                            <CardContent className="p-8 relative z-10">
                                <div className="flex flex-col md:flex-row items-center gap-6">
                                    <div className="bg-white/10 p-4 rounded-full backdrop-blur-md">
                                        <Save className="h-10 w-10 text-blue-400" />
                                    </div>
                                    <div>
                                        <h3 className="text-xl font-bold">{t('profile.confirmTitle')}</h3>
                                        <p className="text-slate-300 mt-2 max-w-md">
                                            {t('profile.confirmDescription')}
                                        </p>
                                    </div>
                                    <Button
                                        size="lg"
                                        variant="secondary"
                                        className="md:ml-auto bg-white text-slate-900 hover:bg-slate-100 font-bold"
                                        onClick={handleSubmit}
                                        disabled={updateMutation.isPending}
                                    >
                                        {updateMutation.isPending ? t('profile.saving') : t('profile.confirmAndSave')}
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    </div>
                )}
            </div>
        </div>
    )
}
