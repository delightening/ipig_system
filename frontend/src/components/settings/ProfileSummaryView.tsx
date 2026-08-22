// 唯讀摘要卡片：資料完整後取代基本資料 + 人員資料兩張可編輯卡片。
// 以定義列（dl）呈現相關欄位；空值以 profile.notFilledValue 淡化顯示；提供「編輯」按鈕回到編輯模式。
import { useTranslation } from 'react-i18next'
import { CheckCircle2, Pencil } from 'lucide-react'
import type { UserTraining } from '@/types/auth'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

export interface SummaryValues {
    display_name: string
    phone: string
    organization: string
    position: string
    entry_date: string
    trainings: UserTraining[]
}

interface Props {
    values: SummaryValues
    showPersonnel: boolean
    showEntryDate: boolean
    showTraining: boolean
    onEdit: () => void
}

export function ProfileSummaryView({ values, showPersonnel, showEntryDate, showTraining, onEdit }: Props) {
    const { t } = useTranslation()

    const rows: { label: string; value: string }[] = [
        { label: t('profile.displayName'), value: values.display_name },
        { label: t('profile.phone'), value: values.phone },
        { label: t('profile.organization'), value: values.organization },
    ]
    if (showPersonnel) rows.push({ label: t('profile.position'), value: values.position })
    if (showEntryDate) rows.push({ label: t('profile.entryDate'), value: values.entry_date })

    // 訓練/資格改為逐筆列出（code．證號．日期）+ 代碼說明附註（只列實際用到的類別）。
    const usedTrainingCodes = Array.from(
        new Set(values.trainings.map(tr => tr.code).filter(Boolean)),
    ).sort()

    return (
        <Card>
            <CardHeader className="border-b bg-muted/50">
                <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">
                        <CheckCircle2 className="h-5 w-5 text-status-success-text" />
                        {t('profile.dataCompleted')}
                    </CardTitle>
                    <Button size="sm" variant="outline" onClick={onEdit}>
                        <Pencil className="mr-1.5 h-4 w-4" />
                        {t('profile.editData')}
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="pt-6">
                <dl className="divide-y divide-border">
                    {rows.map(row => (
                        <div key={row.label} className="grid grid-cols-3 gap-4 py-3">
                            <dt className="text-sm text-muted-foreground">{row.label}</dt>
                            <dd className="col-span-2 text-sm font-medium">
                                {row.value.trim() ? (
                                    <span className="text-foreground">{row.value}</span>
                                ) : (
                                    <span className="text-muted-foreground italic">{t('profile.notFilledValue')}</span>
                                )}
                            </dd>
                        </div>
                    ))}
                    {showTraining && (
                        <div className="grid grid-cols-3 gap-4 py-3">
                            <dt className="text-sm text-muted-foreground">{t('profile.trainings')}</dt>
                            <dd className="col-span-2 text-sm font-medium">
                                {values.trainings.length === 0 ? (
                                    <span className="text-muted-foreground italic">{t('profile.notFilledValue')}</span>
                                ) : (
                                    <ul className="space-y-1">
                                        {values.trainings.map((tr, index) => (
                                            <li key={index}>
                                                <span className="font-semibold text-foreground">{tr.code || '—'}.</span>{' '}
                                                <span className="text-foreground">
                                                    {tr.certificate_no || t('profile.trainingNoCert')}
                                                </span>
                                                <span className="text-muted-foreground">
                                                    ．{tr.received_date || t('profile.trainingNoDate')}
                                                </span>
                                            </li>
                                        ))}
                                    </ul>
                                )}
                            </dd>
                        </div>
                    )}
                </dl>

                {showTraining && usedTrainingCodes.length > 0 && (
                    <div className="mt-4 rounded-lg bg-muted/50 p-3 text-xs text-muted-foreground">
                        <p className="mb-1 font-semibold">{t('profile.trainingLegendTitle')}</p>
                        <ul className="space-y-0.5">
                            {usedTrainingCodes.map(code => (
                                <li key={code}>
                                    <span className="font-semibold">{code}</span> = {t(`aup.personnel.trainings.${code}`)}
                                </li>
                            ))}
                        </ul>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
