// 基本資料卡片（可編輯）：頭像、email（唯讀）、顯示名稱、聯絡電話 + 分機、所屬單位。
// 由 ProfileSettingsPage 於編輯模式渲染；欄位值與變更皆由父層透過 props 控制。
import { useTranslation } from 'react-i18next'
import { User as UserIcon, Mail, Phone, Building2 } from 'lucide-react'
import type { User } from '@/types/auth'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'

export interface BasicInfoValues {
    display_name: string
    phone: string
    phone_ext: string
    organization: string
}

interface Props {
    user: User
    values: BasicInfoValues
    onChange: (field: keyof BasicInfoValues, value: string) => void
}

export function ProfileBasicInfoCard({ user, values, onChange }: Props) {
    const { t } = useTranslation()

    return (
        <Card>
            <CardHeader className="border-b bg-muted/50">
                <CardTitle className="flex items-center gap-2">
                    <UserIcon className="h-5 w-5 text-primary" />
                    {t('profile.basicInfo')}
                </CardTitle>
            </CardHeader>
            <CardContent className="pt-6 space-y-4">
                <div className="flex flex-col items-center pb-6 border-b border-border">
                    <div className="h-24 w-24 rounded-full bg-primary flex items-center justify-center text-primary-foreground text-3xl font-bold">
                        {user.display_name?.[0] || user.email?.[0]}
                    </div>
                    <h3 className="mt-4 font-bold text-lg text-foreground">{user.display_name}</h3>
                    <Badge variant="secondary" className="mt-1">{user.roles.join(', ')}</Badge>
                </div>

                <div className="space-y-4 pt-4">
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2 text-muted-foreground">
                            <Mail className="h-4 w-4" /> {t('profile.email')} {t('profile.readOnly')}
                        </Label>
                        <Input value={user.email} disabled className="bg-muted" aria-label={t('profile.email')} />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="display_name">{t('profile.displayName')}</Label>
                        <Input
                            id="display_name"
                            value={values.display_name}
                            onChange={e => onChange('display_name', e.target.value)}
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
                                value={values.phone}
                                onChange={e => onChange('phone', e.target.value)}
                                placeholder={t('profile.phonePlaceholder')}
                            />
                            <div className="flex items-center gap-2 shrink-0">
                                <span className="text-muted-foreground">#</span>
                                <Input
                                    className="w-24"
                                    placeholder={t('aup.basic.piExtension')}
                                    value={values.phone_ext}
                                    onChange={e => onChange('phone_ext', e.target.value)}
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
                            value={values.organization}
                            onChange={e => onChange('organization', e.target.value)}
                            placeholder={t('profile.organizationPlaceholder')}
                        />
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}
