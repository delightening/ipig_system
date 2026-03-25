import { useTranslation } from 'react-i18next'
import type { AnimalBreed, AnimalSource } from '@/lib/api'
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
import { useFacilityLayout } from '../hooks/useFacilityLayout'
import { useBreedSpecies } from '../hooks/useBreedSpecies'

export { BatchAssignDialog } from './BatchAssignDialog'
export { QuickAddDialog } from './QuickAddDialog'
export type { QuickAddForm } from './QuickAddDialog'
export { DuplicateWarningDialog } from './DuplicateWarningDialog'
export type { DuplicateWarningData } from './DuplicateWarningDialog'

export interface NewAnimalForm {
  ear_tag: string
  breed: AnimalBreed
  gender: 'male' | 'female'
  source_id: string
  entry_date: string
  entry_weight: string
  birth_date: string
  pre_experiment_code: string
  remark: string
  breed_other: string
}

interface AnimalAddDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  newAnimal: NewAnimalForm
  onNewAnimalChange: (form: NewAnimalForm) => void
  penBuilding: string
  onPenBuildingChange: (value: string) => void
  penZone: string
  onPenZoneChange: (value: string) => void
  penNumber: string
  onPenNumberChange: (value: string) => void
  sourcesData: AnimalSource[] | undefined
  onSubmit: () => void
  isPending: boolean
}

export function AnimalAddDialog({
  open,
  onOpenChange,
  newAnimal,
  onNewAnimalChange,
  penBuilding,
  onPenBuildingChange,
  penZone,
  onPenZoneChange,
  penNumber,
  onPenNumberChange,
  sourcesData,
  onSubmit,
  isPending,
}: AnimalAddDialogProps) {
  const { t } = useTranslation()
  const { buildings, zonesByBuilding, pensByZone } = useFacilityLayout()
  const { breedSpecies } = useBreedSpecies()

  const selectedBuilding = buildings.find(b => b.code === penBuilding)
  const zonesForBuilding = selectedBuilding ? (zonesByBuilding[selectedBuilding.id] ?? []) : []
  const selectedZone = zonesForBuilding.find(z => z.code === penZone)
  const pensForZone = selectedZone ? (pensByZone[selectedZone.id] ?? []) : []

  const isDisabled =
    isPending ||
    !newAnimal.ear_tag ||
    !penBuilding ||
    !penZone ||
    !penNumber ||
    !newAnimal.birth_date ||
    !newAnimal.entry_weight ||
    !newAnimal.pre_experiment_code ||
    !newAnimal.entry_date ||
    (newAnimal.breed === 'other' && !newAnimal.breed_other)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>新增動物</DialogTitle>
          <DialogDescription>輸入新動物的基本資料</DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="ear_tag">耳號 *</Label>
            <Input
              id="ear_tag"
              value={newAnimal.ear_tag}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, ear_tag: e.target.value })}
              placeholder="輸入耳號"
            />
            <p className="text-[10px] text-muted-foreground">若輸入數字會自動轉換為三位數（如 001）</p>
          </div>
          <div className="space-y-2">
            <Label>棟別 *</Label>
            <Select
              value={penBuilding}
              onValueChange={(v) => {
                onPenBuildingChange(v)
                onPenZoneChange('')
                onPenNumberChange('')
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder="選擇 A 棟或 B 棟" />
              </SelectTrigger>
              <SelectContent>
                {buildings.map((building) => (
                  <SelectItem key={building.id} value={building.code}>
                    {building.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>欄位區 *</Label>
            <Select
              value={penZone}
              onValueChange={(v) => {
                onPenZoneChange(v)
                onPenNumberChange('')
              }}
              disabled={!penBuilding}
            >
              <SelectTrigger>
                <SelectValue placeholder={penBuilding ? "選擇欄位區" : "請先選棟別"} />
              </SelectTrigger>
              <SelectContent>
                {zonesForBuilding.map((zone) => (
                  <SelectItem key={zone.id} value={zone.code}>{zone.name ?? zone.code}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>欄位編號 *</Label>
            <Select
              value={penNumber}
              onValueChange={onPenNumberChange}
              disabled={!penZone}
            >
              <SelectTrigger>
                <SelectValue placeholder={penZone ? "選擇編號" : "請先選欄位區"} />
              </SelectTrigger>
              <SelectContent>
                {pensForZone.map((pen) => (
                  <SelectItem key={pen.id} value={pen.code.slice(1)}>{pen.code}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>品種 *</Label>
            <Select
              value={newAnimal.breed}
              onValueChange={(v) => onNewAnimalChange({ ...newAnimal, breed: v as AnimalBreed })}
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
          </div>
          {newAnimal.breed === 'other' && (
            <div className="space-y-2">
              <Label htmlFor="breed_other">填寫品種 *</Label>
              <Input
                id="breed_other"
                value={newAnimal.breed_other}
                onChange={(e) => onNewAnimalChange({ ...newAnimal, breed_other: e.target.value })}
                placeholder="請輸入品種名稱"
              />
            </div>
          )}
          <div className="space-y-2">
            <Label>性別 *</Label>
            <Select
              value={newAnimal.gender}
              onValueChange={(v) => onNewAnimalChange({ ...newAnimal, gender: v as 'male' | 'female' })}
            >
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="male">公</SelectItem>
                <SelectItem value="female">母</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>來源</Label>
            <Select
              value={newAnimal.source_id || 'none'}
              onValueChange={(v) => onNewAnimalChange({ ...newAnimal, source_id: v === 'none' ? '' : v })}
            >
              <SelectTrigger><SelectValue placeholder="選擇來源" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="none">無</SelectItem>
                {sourcesData?.map((source) => (
                  <SelectItem key={source.id} value={source.id}>{source.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t('animals.entryDate')} *</Label>
            <Input
              type="date"
              value={newAnimal.entry_date}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, entry_date: e.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('animals.birthDate')} *</Label>
            <Input
              type="date"
              value={newAnimal.birth_date}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, birth_date: e.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="entry_weight">進場體重 (kg) *</Label>
            <Input
              id="entry_weight"
              type="text"
              inputMode="decimal"
              value={newAnimal.entry_weight}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, entry_weight: sanitizeDecimalInput(e.target.value) })}
              placeholder="輸入體重"
            />
          </div>
          <div className="space-y-2 col-span-2">
            <Label htmlFor="pre_experiment_code">實驗前代號 *</Label>
            <Input
              id="pre_experiment_code"
              value={newAnimal.pre_experiment_code}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, pre_experiment_code: e.target.value })}
              placeholder="例如 PIG-110000"
            />
          </div>
          <div className="space-y-2 col-span-2">
            <Label htmlFor="remark">備註</Label>
            <Input
              id="remark"
              value={newAnimal.remark}
              onChange={(e) => onNewAnimalChange({ ...newAnimal, remark: e.target.value })}
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
            新增
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
