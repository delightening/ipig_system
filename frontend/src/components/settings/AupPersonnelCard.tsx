// AUP 第 8 節人員資料卡片（僅具人員角色者顯示）。
// 內容：職稱（恆顯）、入職日期 + 入職前參與動物試驗年數（僅內部人員）、
// 可擔任 AUP 角色（唯讀徽章，不送出）、訓練/資格子區塊（僅訓練角色）。
import { useTranslation } from 'react-i18next'
import { GraduationCap, History, Briefcase } from 'lucide-react'
import type { UserTraining } from '@/types/auth'
import { AUP_ROLE_OPTIONS } from '@/lib/constants'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { TrainingSection } from './TrainingSection'

export interface AupPersonnelValues {
    entry_date: string
    position: string
    years_experience: number
    aup_roles: string[]
    trainings: UserTraining[]
}

interface Props {
    values: AupPersonnelValues
    showEntryDate: boolean
    showTraining: boolean
    onFieldChange: (field: 'entry_date' | 'position' | 'years_experience', value: string | number) => void
    addTraining: () => void
    removeTraining: (index: number) => void
    updateTraining: (index: number, field: 'code' | 'certificate_no' | 'received_date', value: string) => void
}

export function AupPersonnelCard({
    values,
    showEntryDate,
    showTraining,
    onFieldChange,
    addTraining,
    removeTraining,
    updateTraining,
}: Props) {
    const { t } = useTranslation()
    const assignedRoles = AUP_ROLE_OPTIONS.filter(role => values.aup_roles.includes(role.value))

    return (
        <Card>
            <CardHeader className="border-b bg-muted/50">
                <CardTitle className="flex items-center gap-2 text-foreground">
                    <GraduationCap className="h-5 w-5 text-primary" />
                    {t('profile.aupSection8')}
                </CardTitle>
                <CardDescription>{t('profile.aupSection8Description')}</CardDescription>
            </CardHeader>
            <CardContent className="pt-6 space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-2">
                        <Label htmlFor="position" className="flex items-center gap-2">
                            <Briefcase className="h-4 w-4 text-muted-foreground" /> {t('profile.position')}
                        </Label>
                        <Input
                            id="position"
                            value={values.position}
                            onChange={e => onFieldChange('position', e.target.value)}
                            placeholder={t('profile.positionPlaceholder')}
                        />
                    </div>
                    {showEntryDate && (
                        <>
                            <div className="space-y-2">
                                <Label htmlFor="entry_date" className="flex items-center gap-2">
                                    <History className="h-4 w-4 text-muted-foreground" /> {t('profile.entryDate')}
                                </Label>
                                <Input
                                    id="entry_date"
                                    type="date"
                                    value={values.entry_date}
                                    onChange={e => onFieldChange('entry_date', e.target.value)}
                                />
                            </div>
                            <div className="space-y-2">
                                <Label htmlFor="years_experience">{t('profile.yearsExperience')}</Label>
                                <Input
                                    id="years_experience"
                                    type="number"
                                    min={0}
                                    value={values.years_experience}
                                    onChange={e => onFieldChange('years_experience', parseInt(e.target.value) || 0)}
                                />
                            </div>
                        </>
                    )}
                </div>

                <div className="space-y-3 pt-4 border-t border-border">
                    <div className="flex items-center justify-between">
                        <Label className="text-base font-semibold">{t('profile.aupRoles')}</Label>
                        <Badge variant="outline" className="text-muted-foreground">{t('profile.readOnly')}</Badge>
                    </div>
                    {assignedRoles.length > 0 ? (
                        <div className="flex flex-wrap gap-2">
                            {assignedRoles.map(role => (
                                <Badge key={role.value} variant="secondary" className="px-3 py-1 text-sm">
                                    {t(`profile.roles.${role.key}`)}
                                </Badge>
                            ))}
                        </div>
                    ) : (
                        <p className="text-sm text-muted-foreground italic">{t('profile.aupRolesUnassigned')}</p>
                    )}
                    <p className="text-xs text-muted-foreground">{t('profile.aupRolesManagedHint')}</p>
                </div>

                {showTraining && (
                    <TrainingSection
                        trainings={values.trainings}
                        addTraining={addTraining}
                        removeTraining={removeTraining}
                        updateTraining={updateTraining}
                    />
                )}
            </CardContent>
        </Card>
    )
}
