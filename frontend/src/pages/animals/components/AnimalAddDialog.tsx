import { useTranslation } from 'react-i18next'
import type { AnimalBreed, AnimalSource, CreateAnimalRequest } from '@/lib/api'
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
import { Loader2, AlertTriangle } from 'lucide-react'
import { penBuildings, penZonesByBuilding, penNumbersByZone } from '../constants'

// ─── Main Add Animal Dialog ────────────────────────────────────────────────────

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
            <p className="text-[10px] text-slate-400">若輸入數字會自動轉換為三位數（如 001）</p>
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
                {penBuildings.map((building) => (
                  <SelectItem key={building.value} value={building.value}>
                    {building.label}
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
                {(penZonesByBuilding[penBuilding] || []).map((zone) => (
                  <SelectItem key={zone} value={zone}>{zone}</SelectItem>
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
                {(penNumbersByZone[penZone] || []).map((value) => (
                  <SelectItem key={value} value={value}>{value}</SelectItem>
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
                <SelectItem value="minipig">迷你豬</SelectItem>
                <SelectItem value="white">白豬</SelectItem>
                <SelectItem value="other">其他</SelectItem>
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
              onChange={(e) => {
                const numericValue = e.target.value.replace(/[^\d.]/g, '')
                const parts = numericValue.split('.')
                const filteredValue = parts.length > 2 ? parts[0] + '.' + parts.slice(1).join('') : numericValue
                onNewAnimalChange({ ...newAnimal, entry_weight: filteredValue })
              }}
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
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button
            onClick={onSubmit}
            disabled={isDisabled}
            className="bg-purple-600 hover:bg-purple-700"
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            新增
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

// ─── Batch Assign Dialog ───────────────────────────────────────────────────────

interface BatchAssignDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedCount: number
  iacucNo: string
  onIacucNoChange: (value: string) => void
  onSubmit: () => void
  isPending: boolean
}

export function BatchAssignDialog({
  open,
  onOpenChange,
  selectedCount,
  iacucNo,
  onIacucNoChange,
  onSubmit,
  isPending,
}: BatchAssignDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>分配動物至計畫</DialogTitle>
          <DialogDescription>
            將選中的 {selectedCount} 隻動物分配至指定的 IACUC 計畫
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="iacuc_no">IACUC No. *</Label>
            <Input
              id="iacuc_no"
              value={iacucNo}
              onChange={(e) => onIacucNoChange(e.target.value)}
              placeholder="例如 PIG-114017"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button onClick={onSubmit} disabled={isPending || !iacucNo}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認分配
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

// ─── Quick Add Dialog (from pen view) ──────────────────────────────────────────

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
            耳號 <span className="font-bold text-purple-600">{earTag}</span> 不存在，請填寫資料以新增動物至 <span className="font-bold">{penLocation}</span>
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
                <SelectItem value="minipig">迷你豬</SelectItem>
                <SelectItem value="white">白豬</SelectItem>
                <SelectItem value="other">其他</SelectItem>
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
              onChange={(e) => {
                const numericValue = e.target.value.replace(/[^\d.]/g, '')
                const parts = numericValue.split('.')
                const filteredValue = parts.length > 2 ? parts[0] + '.' + parts.slice(1).join('') : numericValue
                onFormChange({ ...form, entry_weight: filteredValue })
              }}
              placeholder="輸入體重"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button
            onClick={onSubmit}
            disabled={isDisabled}
            className="bg-purple-600 hover:bg-purple-700"
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認新增
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

// ─── Duplicate Ear Tag Warning Dialog ──────────────────────────────────────────

export interface DuplicateWarningData {
  earTag: string
  existingAnimals: Array<{ id: string; birth_date: string | null; status: string; pen_location: string | null }>
  source: 'create' | 'quickAdd'
  pendingPayload: CreateAnimalRequest & { breed_other?: string }
}

interface DuplicateWarningDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  data: DuplicateWarningData | null
  onConfirm: (payload: CreateAnimalRequest & { breed_other?: string }) => void
  isPending: boolean
}

export function DuplicateWarningDialog({
  open,
  onOpenChange,
  data,
  onConfirm,
  isPending,
}: DuplicateWarningDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-amber-600">
            <AlertTriangle className="h-5 w-5" />
            耳號重複警告
          </DialogTitle>
          <DialogDescription>
            耳號 <span className="font-semibold text-slate-900">{data?.earTag}</span> 已存在以下存活動物：
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-2 my-2">
          {data?.existingAnimals.map((animal, idx) => (
            <div key={idx} className="flex items-center gap-3 p-3 bg-amber-50 border border-amber-200 rounded-lg text-sm">
              <AlertTriangle className="h-4 w-4 text-amber-500 shrink-0" />
              <div>
                <div>出生日期: <span className="font-medium">{animal.birth_date || '未設定'}</span></div>
                <div>欄位: <span className="font-medium">{animal.pen_location || '-'}</span></div>
              </div>
            </div>
          ))}
        </div>
        <p className="text-sm text-slate-600">
          確定仍要以<span className="font-semibold">不同出生日期</span>建立新動物嗎？
        </p>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button
            onClick={() => data?.pendingPayload && onConfirm(data.pendingPayload)}
            disabled={isPending}
            className="bg-amber-600 hover:bg-amber-700"
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認建立
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
