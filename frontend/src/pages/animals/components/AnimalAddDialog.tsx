import { useTranslation } from 'react-i18next'
import type { AnimalSource } from '@/lib/api'
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
  /** 物種主檔 id。動物種類的真相源，breed enum 由後端推導，前端不再自己映射。 */
  species_id: string
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
  /** 選定欄位的完整 code（例如 A01、S01），直接作為 pen_location 送出 */
  penCode: string
  onPenCodeChange: (value: string) => void
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
  penCode,
  onPenCodeChange,
  sourcesData,
  onSubmit,
  isPending,
}: AnimalAddDialogProps) {
  const { t } = useTranslation()
  const { buildings, zonesByBuilding, pensByZone } = useFacilityLayout()
  const { breedSpecies } = useBreedSpecies()

  // 「其他」是物種主檔裡的一列，選它才需要補填自由文字的品種名稱
  const selectedSpecies = breedSpecies.find(sp => sp.id === newAnimal.species_id)
  const requiresBreedOther = selectedSpecies?.code === 'other'

  const selectedBuilding = buildings.find(b => b.code === penBuilding)
  const zonesForBuilding = selectedBuilding ? (zonesByBuilding[selectedBuilding.id] ?? []) : []
  const selectedZone = zonesForBuilding.find(z => z.code === penZone)
  const pensForZone = selectedZone ? (pensByZone[selectedZone.id] ?? []) : []

  const isDisabled =
    isPending ||
    !newAnimal.ear_tag ||
    !penBuilding ||
    !penZone ||
    !penCode ||
    !newAnimal.species_id ||
    !newAnimal.birth_date ||
    !newAnimal.entry_weight ||
    !newAnimal.pre_experiment_code ||
    !newAnimal.entry_date ||
    (requiresBreedOther && !newAnimal.breed_other)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
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
                onPenCodeChange('')
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
                onPenCodeChange('')
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
              value={penCode}
              onValueChange={onPenCodeChange}
              disabled={!penZone}
            >
              <SelectTrigger>
                <SelectValue placeholder={penZone ? "選擇編號" : "請先選欄位區"} />
              </SelectTrigger>
              <SelectContent>
                {/* value 用完整 pen code：先前取 code.slice(1) 假設「區碼一字母 + 編號」，
                    遇到單字元 code（檢疫舍的 Q、羊舍的 羊）會裁成空字串。Radix Select 把空字串
                    視為「未選取」，該選項點了不會觸發 onValueChange，使用者永遠選不起來、
                    送出鈕一直 disabled。欄位詳情頁的欄號選單（AnimalHeaderCard）本來就用完整
                    code，這裡對齊它。 */}
                {pensForZone.map((pen) => (
                  <SelectItem key={pen.id} value={pen.code}>{pen.code}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>品種 *</Label>
            <Select
              value={newAnimal.species_id}
              onValueChange={(v) => onNewAnimalChange({ ...newAnimal, species_id: v, breed_other: '' })}
            >
              <SelectTrigger><SelectValue placeholder="選擇品種" /></SelectTrigger>
              <SelectContent>
                {breedSpecies.length > 0 ? (
                  breedSpecies.map((sp) => (
                    <SelectItem key={sp.id} value={sp.id}>{sp.name}</SelectItem>
                  ))
                ) : (
                  <div className="px-2 py-3 text-xs text-muted-foreground">
                    尚無可選物種，請先至「設施管理 → 物種」新增
                  </div>
                )}
              </SelectContent>
            </Select>
          </div>
          {requiresBreedOther && (
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
