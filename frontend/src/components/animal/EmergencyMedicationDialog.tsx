import { useState, useEffect, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2, AlertTriangle } from 'lucide-react'
import { DrugCombobox } from '@/components/animal/DrugCombobox'

interface Props {
    open: boolean
    onOpenChange: (open: boolean) => void
    animalId: string
    earTag: string
}

export function EmergencyMedicationDialog({ open, onOpenChange, animalId, earTag }: Props) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    // Form state
    const [formData, setFormData] = useState({
        event_date: new Date().toISOString().split('T')[0],
        emergency_reason: '',
        drug: '',
        dosage: '',
        content: '',
        drug_option_id: undefined as string | undefined,
        dosage_unit: '',
    })

    // Countdown state for confirmation
    const [countdown, setCountdown] = useState(3)
    const [isConfirming, setIsConfirming] = useState(false)

    // Reset when dialog opens
    useEffect(() => {
        if (open) {
            setFormData({
                event_date: new Date().toISOString().split('T')[0],
                emergency_reason: '',
                drug: '',
                dosage: '',
                content: '',
                drug_option_id: undefined,
                dosage_unit: '',
            })
            setCountdown(3)
            setIsConfirming(false)
        }
    }, [open])

    // Countdown logic
    useEffect(() => {
        if (isConfirming && countdown > 0) {
            const timer = setTimeout(() => setCountdown(countdown - 1), 1000)
            return () => clearTimeout(timer)
        }
    }, [isConfirming, countdown])

    const mutation = useMutation({
        mutationFn: async () => {
            const payload = {
                event_date: formData.event_date,
                record_type: 'abnormal',
                content: formData.content,
                is_emergency_medication: true,
                emergency_reason: formData.emergency_reason,
                treatments: [
                    {
                        drug: formData.drug,
                        dosage: formData.dosage,
                    },
                ],
            }
            return api.post(`/animals/${animalId}/observations`, payload)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
            toast({
                title: t('animalRecords.emergencyMedication.recordedTitle'),
                description: t('animalRecords.emergencyMedication.recordedDescription'),
            })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('animalRecords.shared.saveFailed')),
                variant: 'destructive',
            })
        },
    })

    const handleConfirmClick = useCallback(() => {
        if (!isConfirming) {
            // Start countdown
            setIsConfirming(true)
            setCountdown(3)
        } else if (countdown === 0) {
            // Submit
            if (!formData.emergency_reason.trim() || !formData.drug.trim() || !formData.content.trim()) {
                toast({ title: t('common.error'), description: t('animalRecords.emergencyMedication.requiredFields'), variant: 'destructive' })
                return
            }
            mutation.mutate()
        }
    }, [isConfirming, countdown, formData, mutation, t])

    const handleCancel = () => {
        if (isConfirming) {
            setIsConfirming(false)
            setCountdown(3)
        } else {
            onOpenChange(false)
        }
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-error-text">
                        <AlertTriangle className="h-5 w-5" />
                        {t('animalRecords.emergencyMedication.title')}
                    </DialogTitle>
                    <DialogDescription>
                        {t('animalRecords.shared.earTagLine', { earTag })}
                        <br />
                        <span className="text-status-error-solid">
                            {t('animalRecords.emergencyMedication.description')}
                        </span>
                    </DialogDescription>
                </DialogHeader>

                {!isConfirming ? (
                    <form className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="event_date">{t('animalRecords.emergencyMedication.eventDateRequired')}</Label>
                            <Input
                                id="event_date"
                                type="date"
                                value={formData.event_date}
                                onChange={(e) => setFormData({ ...formData, event_date: e.target.value })}
                                required
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="emergency_reason">{t('animalRecords.emergencyMedication.reasonRequired')}</Label>
                            <Textarea
                                id="emergency_reason"
                                value={formData.emergency_reason}
                                onChange={(e) => setFormData({ ...formData, emergency_reason: e.target.value })}
                                placeholder={t('animalRecords.emergencyMedication.reasonPlaceholder')}
                                className="min-h-[80px]"
                                required
                            />
                        </div>

                        <div className="space-y-2">
                            <Label>{t('animalRecords.emergencyMedication.drugRequired')}</Label>
                            <DrugCombobox
                                value={{
                                    drug_option_id: formData.drug_option_id,
                                    drug_name: formData.drug,
                                    dosage_value: formData.dosage,
                                    dosage_unit: formData.dosage_unit,
                                }}
                                onChange={(sel) => setFormData({
                                    ...formData,
                                    drug: sel.drug_name,
                                    dosage: sel.dosage_value,
                                    drug_option_id: sel.drug_option_id,
                                    dosage_unit: sel.dosage_unit,
                                })}
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="content">{t('animalRecords.emergencyMedication.contentRequired')}</Label>
                            <Textarea
                                id="content"
                                value={formData.content}
                                onChange={(e) => setFormData({ ...formData, content: e.target.value })}
                                placeholder={t('animalRecords.emergencyMedication.contentPlaceholder')}
                                className="min-h-[100px]"
                                required
                            />
                        </div>

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={handleCancel}>
                                {t('common.cancel')}
                            </Button>
                            <Button
                                type="button"
                                onClick={handleConfirmClick}
                                className="bg-destructive hover:bg-destructive/90"
                                disabled={!formData.emergency_reason.trim() || !formData.drug.trim() || !formData.content.trim()}
                            >
                                <AlertTriangle className="h-4 w-4 mr-2" />
                                {t('animalRecords.emergencyMedication.confirmButton')}
                            </Button>
                        </DialogFooter>
                    </form>
                ) : (
                    <div className="space-y-6 py-4">
                        <div className="bg-status-error-bg border border-status-error-border rounded-lg p-4 text-center">
                            <AlertTriangle className="h-12 w-12 text-status-error-solid mx-auto mb-3" />
                            <h3 className="text-lg font-semibold text-status-error-text mb-2">
                                {t('animalRecords.emergencyMedication.confirmTitle')}
                            </h3>
                            <p className="text-sm text-status-error-text mb-4">
                                {t('animalRecords.emergencyMedication.confirmDescription')}
                            </p>

                            <div className="bg-status-error-bg rounded-full w-20 h-20 mx-auto flex items-center justify-center mb-4">
                                <span className="text-3xl font-bold text-status-error-text">
                                    {countdown > 0 ? countdown : '✓'}
                                </span>
                            </div>

                            <p className="text-xs text-status-error-solid">
                                {countdown > 0 ? t('animalRecords.emergencyMedication.waitToConfirm', { seconds: countdown }) : t('animalRecords.emergencyMedication.clickToConfirm')}
                            </p>
                        </div>

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={handleCancel}>
                                {t('common.cancel')}
                            </Button>
                            <Button
                                type="button"
                                onClick={handleConfirmClick}
                                className="bg-destructive hover:bg-destructive/90"
                                disabled={countdown > 0 || mutation.isPending}
                            >
                                {mutation.isPending ? (
                                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                ) : (
                                    <AlertTriangle className="h-4 w-4 mr-2" />
                                )}
                                {countdown > 0 ? t('animalRecords.emergencyMedication.waitButton', { seconds: countdown }) : t('animalRecords.emergencyMedication.executeButton')}
                            </Button>
                        </DialogFooter>
                    </div>
                )}
            </DialogContent>
        </Dialog>
    )
}
