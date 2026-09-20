import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { CORRECTABLE_FIELDS, type CorrectableField } from '@/lib/api'
import { Loader2 } from 'lucide-react'
import type { Animal } from '@/lib/api'

// 欄位顯示名稱走 i18n key（渲染時才 t()）；field 值本身仍是送後端的 enum。
const FIELD_LABEL_KEYS: Record<CorrectableField, string> = {
  ear_tag: 'animals.earTag',
  birth_date: 'animals.birthDate',
  gender: 'animals.gender',
  breed: 'animals.breed',
}

interface RequestCorrectionDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  animal: Animal
  onSubmit: (data: { field_name: string; new_value: string; reason: string }) => Promise<void>
}

export function RequestCorrectionDialog({
  open,
  onOpenChange,
  animal,
  onSubmit,
}: RequestCorrectionDialogProps) {
  const { t } = useTranslation()
  const [field, setField] = useState<CorrectableField>('ear_tag')
  const [newValue, setNewValue] = useState('')
  const [reason, setReason] = useState('')
  const [isPending, setIsPending] = useState(false)

  const getCurrentValue = () => {
    switch (field) {
      case 'ear_tag':
        return animal.ear_tag
      case 'birth_date':
        return animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : '-'
      case 'gender':
        return t(`animals.genderLabels.${animal.gender}`)
      case 'breed':
        return animal.breed === 'other' ? (animal.breed_other || t('animalActions.common.other')) : t(`animals.breedLabels.${animal.breed}`)
      default:
        return '-'
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newValue.trim() || !reason.trim()) return
    setIsPending(true)
    try {
      await onSubmit({
        field_name: field,
        new_value: newValue.trim(),
        reason: reason.trim(),
      })
      onOpenChange(false)
      setField('ear_tag')
      setNewValue('')
      setReason('')
    } finally {
      setIsPending(false)
    }
  }

  const handleOpenChange = (next: boolean) => {
    if (!next) {
      setField('ear_tag')
      setNewValue('')
      setReason('')
    }
    onOpenChange(next)
  }

  const renderFieldInput = () => {
    switch (field) {
      case 'ear_tag':
        return (
          <Input
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            placeholder={t('animalActions.correction.earTagHint')}
            maxLength={10}
          />
        )
      case 'birth_date':
        return (
          <Input
            type="date"
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
          />
        )
      case 'gender':
        return (
          <Select value={newValue} onValueChange={setNewValue}>
            <SelectTrigger>
              <SelectValue placeholder={t('animalActions.correction.genderHint')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="male">{t('animals.genderLabels.male')}</SelectItem>
              <SelectItem value="female">{t('animals.genderLabels.female')}</SelectItem>
            </SelectContent>
          </Select>
        )
      case 'breed':
        return (
          <Select value={newValue} onValueChange={setNewValue}>
            <SelectTrigger>
              <SelectValue placeholder={t('animalActions.correction.breedHint')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="miniature">{t('animals.breedLabels.minipig')}</SelectItem>
              <SelectItem value="white">{t('animals.breedLabels.white')}</SelectItem>
              <SelectItem value="LYD">LYD</SelectItem>
              <SelectItem value="other">{t('animals.breedLabels.other')}</SelectItem>
            </SelectContent>
          </Select>
        )
      default:
        return null
    }
  }

  // 切換欄位時重置 newValue 並設定預設
  const handleFieldChange = (v: string) => {
    setField(v as CorrectableField)
    switch (v) {
      case 'ear_tag':
        setNewValue(animal.ear_tag)
        break
      case 'birth_date':
        setNewValue(animal.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : '')
        break
      case 'gender':
        setNewValue(animal.gender)
        break
      case 'breed':
        setNewValue(animal.breed === 'other' ? 'other' : animal.breed === 'minipig' ? 'miniature' : animal.breed)
        break
      default:
        setNewValue('')
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>{t('animalActions.correction.title')}</DialogTitle>
          <DialogDescription>
            {t('animalActions.correction.description', { earTag: animal.ear_tag })}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('animalActions.correction.selectField')}</Label>
              <Select value={field} onValueChange={handleFieldChange}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CORRECTABLE_FIELDS.map((f) => (
                    <SelectItem key={f} value={f}>
                      {t(FIELD_LABEL_KEYS[f])}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>{t('animalActions.correction.currentValue')}</Label>
              <Input value={getCurrentValue()} disabled className="bg-muted" />
            </div>
            <div className="space-y-2">
              <Label>{t('animalActions.correction.newValueLabel')}</Label>
              {renderFieldInput()}
            </div>
            <div className="space-y-2">
              <Label>{t('animalActions.correction.reasonLabel')}</Label>
              <Textarea
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                placeholder={t('animalActions.correction.reasonHint')}
                rows={3}
                required
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              type="submit"
              disabled={isPending || !newValue.trim() || !reason.trim()}
              className="bg-status-purple-solid hover:bg-status-purple-solid/90"
            >
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('animalActions.correction.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
