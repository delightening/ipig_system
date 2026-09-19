import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { Animal, UpdateAnimalRequest, UpdateAnimalRequestValue } from '@/lib/api'
import { animalSpeciesLabel } from '@/lib/animalSpecies'
import { useAssignableProtocols } from '@/hooks/useAssignableProtocols'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { uiLocale } from '@/lib/utils'
import { Loader2 } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
}

export function QuickEditAnimalDialog({ open, onOpenChange, animalId }: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [formData, setFormData] = useState<UpdateAnimalRequest>({})
  const [entryWeightInput, setEntryWeightInput] = useState<string>('')

  // Query animal data
  const { data: animal, isLoading: animalLoading } = useQuery({
    queryKey: ['animal', animalId],
    queryFn: async () => {
      const res = await api.get<Animal>(`/animals/${animalId}`)
      return res.data
    },
    enabled: open && !!animalId,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
    refetchOnMount: true,
  })

  // Initialize form data when animal loads
  useEffect(() => {
    if (animal) {
      setFormData({
        entry_weight: animal.entry_weight ? Number(animal.entry_weight) : undefined,
        pen_location: animal.pen_location || undefined,
        iacuc_no: animal.iacuc_no || undefined,
      })
      setEntryWeightInput(animal.entry_weight !== undefined && animal.entry_weight !== null ? String(animal.entry_weight) : '')
    }
  }, [animal])

  // 可指派計畫（IACUC No. 下拉）：後端 ?assignable=true 已過濾，dialog 開啟才抓
  const { data: approvedProtocols } = useAssignableProtocols({ enabled: open })

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async (data: UpdateAnimalRequest) => {
      return api.put<Animal>(`/animals/${animalId}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
      toast({ title: t('common.success'), description: t('animalActions.quickEdit.updated') })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalActions.common.updateFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // R30-B: 帶當前 version 防 lost update（從 query 結果取，避免 form state stale）
    updateMutation.mutate({ ...formData, version: animal?.version })
  }

  const handleChange = (field: keyof UpdateAnimalRequest, value: UpdateAnimalRequestValue) => {
    setFormData((prev) => ({ ...prev, [field]: value || undefined }))
  }

  if (!open) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{t('animalActions.quickEdit.title')}</DialogTitle>
          <DialogDescription>{t('animalActions.quickEdit.description')}</DialogDescription>
        </DialogHeader>

        {animalLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : animal ? (
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* 唯讀資訊區域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-foreground border-b pb-2">{t('animalActions.quickEdit.readonlySection')}</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.systemNo')}</Label>
                  <Input value={animal.id.slice(0, 8)} disabled className="bg-muted" title={animal.id} />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.earTag')}</Label>
                  <Input value={animal.ear_tag} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.breed')}</Label>
                  <Input value={animalSpeciesLabel(animal, (breed) => t(`animals.breedLabels.${breed}`))} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.gender')}</Label>
                  <Input value={t(`animals.genderLabels.${animal.gender}`)} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.birthDate')}</Label>
                  <Input
                    value={animal.birth_date ? new Date(animal.birth_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' }) : '-'}
                    disabled
                    className="bg-muted"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animals.entryDate')}</Label>
                  <Input
                    value={new Date(animal.entry_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                    disabled
                    className="bg-muted"
                  />
                </div>
              </div>
            </div>

            {/* 可编辑字段区域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-status-purple-text border-b border-status-purple-border pb-2">
                {t('animalActions.quickEdit.editableSection')}
              </h3>
              <div className="grid grid-cols-2 gap-4">
                {/* 進場體重 */}
                <div className="space-y-2">
                  <Label htmlFor="entry_weight">
                    {t('animalActions.common.entryWeightKg')} <span className="text-status-error-solid">*</span>
                  </Label>
                  <Input
                    id="entry_weight"
                    type="text"
                    inputMode="decimal"
                    value={entryWeightInput}
                    onChange={(e) => {
                      const value = e.target.value
                      // 只允許數字和一個小數點
                      const numericValue = value.replace(/[^\d.]/g, '')
                      // 確保只有一個小數點
                      const parts = numericValue.split('.')
                      const filteredValue = parts.length > 2
                        ? parts[0] + '.' + parts.slice(1).join('')
                        : numericValue
                      // 更新輸入值
                      setEntryWeightInput(filteredValue)
                      // 如果為空或只有小數點，設為 undefined，否則轉換為數字
                      if (filteredValue === '' || filteredValue === '.') {
                        handleChange('entry_weight', undefined)
                      } else {
                        const numValue = parseFloat(filteredValue)
                        if (!isNaN(numValue)) {
                          handleChange('entry_weight', numValue)
                        }
                      }
                    }}
                    onBlur={() => {
                      // 當失去焦點時，清理尾部的小數點
                      if (entryWeightInput === '.') {
                        setEntryWeightInput('')
                        handleChange('entry_weight', undefined)
                      } else if (entryWeightInput && entryWeightInput.endsWith('.')) {
                        const cleaned = entryWeightInput.slice(0, -1)
                        setEntryWeightInput(cleaned)
                        const numValue = parseFloat(cleaned)
                        if (!isNaN(numValue)) {
                          handleChange('entry_weight', numValue)
                        }
                      }
                    }}
                    placeholder={t('animalActions.quickEdit.entryWeightHint')}
                  />
                </div>

                {/* 欄位 */}
                <div className="space-y-2">
                  <Label htmlFor="pen_location">{t('animals.pen')}</Label>
                  <Input
                    id="pen_location"
                    value={formData.pen_location || ''}
                    onChange={(e) => handleChange('pen_location', e.target.value)}
                    placeholder={t('animalActions.quickEdit.penHint')}
                  />
                </div>

                {/* IACUC No. */}
                <div className="space-y-2">
                  <Label htmlFor="iacuc_no">IACUC No.</Label>
                  <Select
                    value={formData.iacuc_no || ''}
                    onValueChange={(v) => handleChange('iacuc_no', v === '' ? undefined : v)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('animalActions.quickEdit.iacucHint')} />
                    </SelectTrigger>
                    <SelectContent>
                      {approvedProtocols?.map((protocol) => (
                        <SelectItem key={protocol.id} value={protocol.iacuc_no!}>
                          {protocol.iacuc_no}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            {/* 資訊提示區域（唯讀） */}
            <div className="space-y-4 pt-4 border-t">
              <h3 className="text-sm font-semibold text-foreground border-b pb-2">{t('animalActions.quickEdit.statusSection')}</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animalActions.quickEdit.onMedication')}</Label>
                  <Input
                    value={t('animalActions.quickEdit.onMedicationHint')}
                    disabled
                    className="bg-muted text-muted-foreground text-xs"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">{t('animalActions.common.vetRecommendation')}</Label>
                  <Input
                    value={t('animalActions.quickEdit.vetRecommendationHint')}
                    disabled
                    className="bg-muted text-muted-foreground text-xs"
                  />
                </div>
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={updateMutation.isPending}
              >
                {t('common.cancel')}
              </Button>
              <Button type="submit" disabled={updateMutation.isPending} className="bg-status-purple-solid hover:bg-status-purple-solid/90">
                {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                {t('animalActions.quickEdit.saveChanges')}
              </Button>
            </DialogFooter>
          </form>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
