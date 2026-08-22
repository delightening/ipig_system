import { useState, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/auth'
import api, { User, UpdateUserRequest, UserTraining } from '@/lib/api'
import { TRAINING_ROLES, AUP_PERSONNEL_ROLES, hasAnyRole } from '@/lib/constants'
import { getErrorMessage } from '@/types/error'
import { TwoFactorSetup } from '@/components/auth/TwoFactorSetup'
import { McpKeysSection } from '@/components/settings/McpKeysSection'
import { ProfileBasicInfoCard } from '@/components/settings/ProfileBasicInfoCard'
import { AupPersonnelCard } from '@/components/settings/AupPersonnelCard'
import { DisplayPreferencesCard } from '@/components/settings/DisplayPreferencesCard'
import { ProfileSaveBar } from '@/components/settings/ProfileSaveBar'
import { ProfileSummaryView } from '@/components/settings/ProfileSummaryView'
import { PageHeader } from '@/components/ui/page-header'
import { useToast } from '@/components/ui/use-toast'
import { Loader2 } from 'lucide-react'

type ProfileFormData = {
    display_name: string
    phone: string
    phone_ext: string
    organization: string
}

type AupData = {
    entry_date: string
    position: string
    aup_roles: string[]
    years_experience: number
    trainings: UserTraining[]
}

/** 依角色計算必填欄位是否全數填妥（衍生，不寫入 DB）。 */
function computeCompletion(
    profile: ProfileFormData,
    aup: AupData,
    showPersonnel: boolean,
    showEntryDate: boolean,
) {
    const required = [profile.display_name, profile.phone, profile.organization]
    if (showPersonnel) required.push(aup.position)
    if (showEntryDate) required.push(aup.entry_date)
    const done = required.filter(v => (v || '').trim() !== '').length
    const total = required.length
    return { done, total, percent: Math.round((done / total) * 100), isComplete: done === total }
}

export function ProfileSettingsPage() {
    const { t } = useTranslation()
    const { user: currentUser, checkAuth, hasRole } = useAuthStore()
    const { toast } = useToast()
    const queryClient = useQueryClient()

    const { reset, watch, setValue, setError } = useForm<ProfileFormData>({
        defaultValues: { display_name: '', phone: '', phone_ext: '', organization: '' },
    })
    const [aupData, setAupData] = useState<AupData>({
        entry_date: '', position: '', aup_roles: [], years_experience: 0, trainings: [],
    })
    const [editing, setEditing] = useState(false)
    const initializedRef = useRef(false)

    const profile = watch()

    // 角色衍生旗標
    const isInternal = currentUser?.is_internal === true
    const showPersonnel = hasAnyRole(currentUser?.roles, AUP_PERSONNEL_ROLES)
    const showTraining = hasAnyRole(currentUser?.roles, TRAINING_ROLES)
    const showEntryDate = isInternal
    const showTwoFactor = hasRole('admin')
    const showMcp = isInternal

    // 首次載入 currentUser 時同步表單，並僅初始化一次 editing（完整→唯讀，未完整→編輯）
    useEffect(() => {
        if (!currentUser) return
        const nextProfile: ProfileFormData = {
            display_name: currentUser.display_name || '',
            phone: currentUser.phone || '',
            phone_ext: currentUser.phone_ext || '',
            organization: currentUser.organization || '',
        }
        const nextAup: AupData = {
            entry_date: currentUser.entry_date || '',
            position: currentUser.position || '',
            aup_roles: currentUser.aup_roles || [],
            years_experience: currentUser.years_experience || 0,
            trainings: currentUser.trainings || [],
        }
        reset(nextProfile)
        setAupData(nextAup)
        if (!initializedRef.current) {
            initializedRef.current = true
            const personnel = hasAnyRole(currentUser.roles, AUP_PERSONNEL_ROLES)
            const entryDate = currentUser.is_internal === true
            setEditing(!computeCompletion(nextProfile, nextAup, personnel, entryDate).isComplete)
        }
    }, [currentUser, reset])

    const updateMutation = useMutation({
        mutationFn: async (data: UpdateUserRequest) => {
            const response = await api.put<User>('/me', data)
            return response.data
        },
        onSuccess: () => {
            toast({ title: t('common.success'), description: t('profile.updateSuccess') })
            queryClient.invalidateQueries({ queryKey: ['users', 'me'] })
            checkAuth()
            setEditing(false)
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getErrorMessage(error) || t('common.error'),
                variant: 'destructive',
            })
        },
    })

    const setProfileField = (field: keyof ProfileFormData, value: string) => setValue(field, value)
    const setAupField = (field: 'entry_date' | 'position' | 'years_experience', value: string | number) =>
        setAupData(prev => ({ ...prev, [field]: value }))

    // 訓練/資格改為可重複清單（多個 A、多個 C…），故以 index 定位而非以 code。
    const addTraining = () =>
        setAupData(prev => ({
            ...prev,
            trainings: [...prev.trainings, { code: '', certificate_no: '', received_date: '' }],
        }))

    const removeTraining = (index: number) =>
        setAupData(prev => ({
            ...prev,
            trainings: prev.trainings.filter((_, i) => i !== index),
        }))

    const updateTraining = (
        index: number,
        field: 'code' | 'certificate_no' | 'received_date',
        value: string,
    ) =>
        setAupData(prev => ({
            ...prev,
            trainings: prev.trainings.map((tr, i) => (i === index ? { ...tr, [field]: value } : tr)),
        }))

    const handleSave = () => {
        if (!profile.display_name.trim()) {
            setError('display_name', { message: '顯示名稱為必填欄位' })
            return
        }
        // 合併 profile fields + AUP fields。
        // aup_roles 不送：屬權責宣告，由 admin/IACUC 指派（後端 update_me 亦會忽略）。
        const payload: UpdateUserRequest = {
            ...profile,
            entry_date: aupData.entry_date || null,
            position: aupData.position || null,
            years_experience: aupData.years_experience,
            // 過濾未選類別的空列（使用者按了「新增」但未選 A–F），避免存入無效紀錄。
            trainings: aupData.trainings
                .filter(tr => tr.code)
                .map(tr => ({
                    ...tr,
                    received_date: tr.received_date || undefined,
                    certificate_no: tr.certificate_no || undefined,
                })),
        }
        updateMutation.mutate(payload)
    }

    if (!currentUser) {
        return (
            <div className="flex items-center justify-center h-[60vh]">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
        )
    }

    const completion = computeCompletion(profile, aupData, showPersonnel, showEntryDate)

    return (
        <div className="max-w-5xl mx-auto space-y-8 pb-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <PageHeader title={t('profile.settings')} description={t('profile.description')} />

            {editing ? (
                <>
                    <ProfileBasicInfoCard
                        user={currentUser}
                        values={profile}
                        onChange={setProfileField}
                    />
                    {/* 進度/儲存列：置於基本資料下方，捲動時 sticky 吸頂（見 ProfileSaveBar） */}
                    <ProfileSaveBar
                        doneCount={completion.done}
                        totalCount={completion.total}
                        percent={completion.percent}
                        isComplete={completion.isComplete}
                        isSaving={updateMutation.isPending}
                        onSave={handleSave}
                    />
                    {showPersonnel && (
                        <AupPersonnelCard
                            values={aupData}
                            showEntryDate={showEntryDate}
                            showTraining={showTraining}
                            onFieldChange={setAupField}
                            addTraining={addTraining}
                            removeTraining={removeTraining}
                            updateTraining={updateTraining}
                        />
                    )}
                </>
            ) : (
                <ProfileSummaryView
                    values={{
                        display_name: profile.display_name,
                        phone: profile.phone,
                        organization: profile.organization,
                        position: aupData.position,
                        entry_date: aupData.entry_date,
                        trainings: aupData.trainings,
                    }}
                    showPersonnel={showPersonnel}
                    showEntryDate={showEntryDate}
                    showTraining={showTraining}
                    onEdit={() => setEditing(true)}
                />
            )}

            <DisplayPreferencesCard />

            {showTwoFactor && (
                <TwoFactorSetup
                    totpEnabled={currentUser.totp_enabled ?? false}
                    onStatusChange={() => checkAuth()}
                />
            )}

            {showMcp && <McpKeysSection />}
        </div>
    )
}
