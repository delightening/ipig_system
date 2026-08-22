// 訓練/資格子區塊：可重複的清單——每筆選類別（A–F）+ 證書編號 + 取得日期，
// 支援同一類別多筆（多個 A、多個 C…）。「+ 新增」加一筆，每筆可移除。
// 儲存端已是 Vec<UserTraining> JSON（無唯一限制），故此為純前端多筆化。
import { useTranslation } from 'react-i18next'
import { Award, AlertCircle, Plus, Trash2 } from 'lucide-react'
import type { UserTraining } from '@/types/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const TRAINING_CODES = ['A', 'B', 'C', 'D', 'E', 'F']

interface Props {
    trainings: UserTraining[]
    addTraining: () => void
    removeTraining: (index: number) => void
    updateTraining: (index: number, field: 'code' | 'certificate_no' | 'received_date', value: string) => void
}

export function TrainingSection({ trainings, addTraining, removeTraining, updateTraining }: Props) {
    const { t } = useTranslation()

    return (
        <div className="space-y-4 pt-4 border-t border-border">
            <div className="flex items-center justify-between">
                <Label className="text-base font-semibold flex items-center gap-2">
                    <Award className="h-5 w-5 text-status-warning-text" />
                    {t('profile.trainings')}
                </Label>
                {trainings.length === 0 ? (
                    <Badge variant="destructive">{t('profile.notFilled')}</Badge>
                ) : (
                    <Badge variant="success">{t('profile.completedCount', { count: trainings.length })}</Badge>
                )}
            </div>

            {trainings.length === 0 ? (
                <p className="text-sm text-muted-foreground italic">{t('profile.notFilled')}</p>
            ) : (
                <div className="space-y-3">
                    {trainings.map((training, index) => (
                        <div key={index} className="rounded-xl border border-border bg-muted/30 p-3 space-y-2">
                            <div className="flex items-start gap-2">
                                <Select
                                    value={training.code || undefined}
                                    onValueChange={value => updateTraining(index, 'code', value)}
                                >
                                    <SelectTrigger className="flex-1">
                                        <SelectValue placeholder={t('profile.selectCategory')} />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {TRAINING_CODES.map(code => (
                                            <SelectItem key={code} value={code}>
                                                {t(`aup.personnel.trainings.${code}`)}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="icon"
                                    className="shrink-0 text-muted-foreground hover:text-destructive"
                                    aria-label={t('profile.removeTraining')}
                                    onClick={() => removeTraining(index)}
                                >
                                    <Trash2 className="h-4 w-4" />
                                </Button>
                            </div>
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                                <Input
                                    placeholder={t('profile.certNoPlaceholder')}
                                    value={training.certificate_no || ''}
                                    onChange={e => updateTraining(index, 'certificate_no', e.target.value)}
                                />
                                <Input
                                    type="date"
                                    value={training.received_date || ''}
                                    onChange={e => updateTraining(index, 'received_date', e.target.value)}
                                />
                            </div>
                        </div>
                    ))}
                </div>
            )}

            <Button type="button" variant="outline" size="sm" onClick={addTraining} className="w-full sm:w-auto">
                <Plus className="mr-1.5 h-4 w-4" />
                {t('profile.addTraining')}
            </Button>

            <div className="rounded-lg bg-status-warning-bg p-4 border border-status-warning-text/20 flex gap-3">
                <AlertCircle className="h-5 w-5 text-status-warning-text shrink-0 mt-0.5" />
                <div className="text-xs text-status-warning-text space-y-1">
                    <p className="font-bold">{t('profile.trainingNotice')}</p>
                    <p>{t('aup.personnel.roles.list')}</p>
                    <p className="opacity-80">{t('profile.trainingNoticeDetail')}</p>
                </div>
            </div>
        </div>
    )
}
