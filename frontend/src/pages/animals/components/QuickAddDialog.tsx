import { useTranslation } from 'react-i18next'
import type { AnimalBreed } from '@/lib/api'
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
import { Loader2 } from 'lucide-react'
import { sanitizeDecimalInput } from '@/lib/utils'
import { useBreedSpecies } from '../hooks/useBreedSpecies'

export interface QuickAddForm {
  breed: AnimalBreed
  breed_other: string
  gender: 'male' | 'female'
  entry_date: string
  birth_date: string
  entry_weight: string
}

interface QuickAddDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  earTag: string
  penLocation: string
  form: QuickAddForm
  onFormChange: (form: QuickAddForm) => void
  onSubmit: () => void
  isPending: boolean
}

export function QuickAddDialog({
  open,
  onOpenChange,
  earTag,
  penLocation,
  form,
  onFormChange,
  onSubmit,
  isPending,
}: QuickAddDialogProps) {
  const { t } = useTranslation()
  const { breedSpecies } = useBreedSpecies()

  const isDisabled =
    isPending ||
    !form.entry_date ||
    !form.birth_date ||
    !form.entry_weight ||
    (form.breed === 'other' && !form.breed_other)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>新增動物</DialogTitle>
          <DialogDescription>
            耳號 <span className="font-bold text-primary">{earTag}</span> 不存在，請填寫資料以新增動物至 <span className="font-bold">{penLocation}</span>
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="space-y-2">
            <Label>品種 *</Label>
            <Select
              value={form.breed}
              onValueChange={(v) => onFormChange({ ...form, breed: v as AnimalBreed })}
            >
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                {breedSpecies.length > 0 ? (
                  breedSpecies.map((sp) => (
                    <SelectItem key={sp.id} value={sp.code === 'miniature' ? 'minipig' : sp.code}>
                      {sp.name}
                    </SelectItem>
                  ))
                ) : (
                  <>
                    <SelectItem value="minipig">迷你豬</SelectItem>
                    <SelectItem value="white">白豬</SelectItem>
                    <SelectItem value="other">其他</SelectItem>
                  </>
                )}
              </SelectContent>
            </Select>
            {form.breed === 'other' && (
              <Input
                placeholder="請輸入品種名稱"
                value={form.breed_other}
                onChange={(e) => onFormChange({ ...form, breed_other: e.target.value })}
                className="mt-2"
              />
            )}
          </div>
          <div className="space-y-2">
            <Label>性別 *</Label>
            <Select
              value={form.gender}
              onValueChange={(v) => onFormChange({ ...form, gender: v as 'male' | 'female' })}
            >
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="male">公</SelectItem>
                <SelectItem value="female">母</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t('animals.entryDate')} *</Label>
            <Input
              type="date"
              value={form.entry_date}
              onChange={(e) => onFormChange({ ...form, entry_date: e.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('animals.birthDate')} *</Label>
            <Input
              type="date"
              value={form.birth_date}
              onChange={(e) => onFormChange({ ...form, birth_date: e.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="quick_entry_weight">進場體重 (kg) *</Label>
            <Input
              id="quick_entry_weight"
              type="text"
              inputMode="decimal"
              value={form.entry_weight}
              onChange={(e) => onFormChange({ ...form, entry_weight: sanitizeDecimalInput(e.target.value) })}
              placeholder="輸入體重"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
          <Button
            onClick={onSubmit}
            disabled={isDisabled}
            className="bg-primary hover:bg-primary/90"
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認新增
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
