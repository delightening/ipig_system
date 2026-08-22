import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { AlertCircle, CheckCircle2, Loader2, Syringe, Trash2, X, XCircle } from 'lucide-react'
import api from '@/lib/api'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { useDebounce } from '@/hooks/useDebounce'
import { lookupAliveAnimalByEarTag } from '@/lib/api'
import { animalStatusNames, type AnimalStatus } from '@/types/animal'
import { animalSpeciesLabel } from '@/lib/animalSpecies'
import type { AnimalListItem, AnimalWeight } from '@/types'
import { DrugCombobox } from '@/components/animal/DrugCombobox'
import { TREATMENT_CATEGORY_OPTIONS, TREATMENT_ROUTE_OPTIONS } from './treatmentConstants'

/** 量體重順便登錄的施打資料（結構化，對齊 observation treatments） */
export interface WeightMedication {
  category: string
  drug: string
  drug_option_id?: string
  dosage: string
  dosage_unit: string
  route: string
}

const emptyMedication = (): WeightMedication => ({
  category: '', drug: '', drug_option_id: undefined, dosage: '', dosage_unit: '', route: '',
})

/** 單列回報給上層的解析結果 */
export interface ResolvedWeightRow {
  id: string
  animalId: string | null
  /** 相符動物的耳號（供送出失敗時顯示），查無時為輸入原字串 */
  earTag: string
  weight: string
  valid: boolean
  /** 完整施打資料（藥名+劑量皆有）才回報，否則 null */
  medication: WeightMedication | null
  /** 有開始填施打但不完整（缺藥名或劑量）→ 上層擋送出 */
  medPartial: boolean
}

interface Props {
  id: string
  canRemove: boolean
  /** 此列耳號與其他列重複（由上層判定） */
  duplicate: boolean
  /** 此列體重已成功登錄（施打失敗保留重送中）：鎖定耳號/體重，僅可修改施打，
   *  避免使用者改了體重卻因重送跳過體重而被默默忽略 */
  weightSaved?: boolean
  onChange: (row: ResolvedWeightRow) => void
  onRemove: () => void
}

const MED_SELECT_CLASS =
  'h-9 w-full rounded-md border border-border bg-white px-2 text-sm focus:border-status-info-solid focus:outline-hidden'

/** 施打欄位面板（藥品類別 / 藥名+劑量 / 途徑）；標題與「清除施打」併入上層的動物狀態行 */
function MedicationFields({
  value, onChange,
}: { value: WeightMedication; onChange: (v: WeightMedication) => void }) {
  return (
    <div className="mt-3 space-y-2 border-t border-dashed pt-3">
      <div className="grid grid-cols-1 gap-2 @[520px]:grid-cols-[130px_1fr_120px]">
        <select aria-label="藥品類別" value={value.category} className={MED_SELECT_CLASS}
          onChange={(e) => onChange({
            ...value, category: e.target.value,
            // 換類別時清掉已選藥物，避免殘留上一個類別選的藥物與新類別不一致
            drug: '', drug_option_id: undefined, dosage: '', dosage_unit: '',
          })}>
          <option value="" disabled>藥品類別</option>
          {TREATMENT_CATEGORY_OPTIONS.map((o) => (<option key={o.value} value={o.value}>{o.label}</option>))}
        </select>
        <DrugCombobox
          categoryFilter={value.category || undefined}
          value={{ drug_option_id: value.drug_option_id, drug_name: value.drug, dosage_value: value.dosage, dosage_unit: value.dosage_unit }}
          onChange={(sel) => onChange({ ...value, drug: sel.drug_name, drug_option_id: sel.drug_option_id, dosage: sel.dosage_value, dosage_unit: sel.dosage_unit })}
        />
        <select aria-label="施打途徑" value={value.route} className={MED_SELECT_CLASS}
          onChange={(e) => onChange({ ...value, route: e.target.value })}>
          <option value="">施打途徑</option>
          {TREATMENT_ROUTE_OPTIONS.map((o) => (<option key={o.value} value={o.value}>{o.label}</option>))}
        </select>
      </div>
      {(() => {
        const anyFilled = !!(value.drug.trim() || value.dosage.trim() || value.category || value.route)
        const complete = !!(value.drug.trim() && value.dosage.trim())
        return anyFilled && !complete ? (
          <p className="text-xs text-status-error-text">施打需完整填寫藥品名稱與劑量（或按「清除施打」取消）</p>
        ) : null
      })()}
    </div>
  )
}

/** 施打切換鈕：桌面（@600px+）直式純圖示、貼耳號輸入欄左側；手機維持橫式圖示+文字。
 *  兩者共用同一份 disabled / active 邏輯，只是形狀跟掛載位置不同，故各自渲染、用斷點互斥顯示。 */
function MedToggleButton({
  active, disabled, onClick, variant,
}: { active: boolean; disabled: boolean; onClick: () => void; variant: 'mobile' | 'desktop' }) {
  const title = disabled ? '請先輸入有效耳號才能登記施打' : active ? '施打中，點擊清除' : '施打（量體重順便，選填）'
  const activeClass = active
    ? 'border-primary bg-primary text-primary-foreground'
    : 'border-dashed border-primary bg-status-info-bg text-primary hover:bg-status-info-bg/70'

  if (variant === 'desktop') {
    return (
      <button
        type="button"
        onClick={onClick}
        disabled={disabled}
        title={title}
        aria-label="施打"
        className={`hidden h-10 w-9 flex-none items-center justify-center rounded-lg border transition-colors disabled:cursor-not-allowed disabled:opacity-40 @[600px]:flex ${activeClass}`}
      >
        {active ? <X className="h-4 w-4" /> : <Syringe className="h-4 w-4" />}
      </button>
    )
  }
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`mt-2 inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-40 @[600px]:hidden ${activeClass}`}
    >
      {active ? <X className="h-3 w-3" /> : <Syringe className="h-3 w-3" />} 施打
    </button>
  )
}

/** 動物客戶/委託資訊（後端 GET /animals/:id/client-info） */
interface ClientInfo {
  status: string
  iacuc_no: string | null
  sponsor_name: string | null
  pi_name: string | null
}

/** 狀態相依的客戶/委託摘要：實驗中/已完成帶案號+委託機構+PI；已轉讓帶接收方；未分配顯「未分配」 */
function clientSummary(info: ClientInfo): string {
  const client = [info.iacuc_no, info.sponsor_name, info.pi_name].filter(Boolean).join(' · ')
  switch (info.status) {
    case 'in_experiment':
      return client ? `實驗中｜${client}` : '實驗中'
    case 'completed':
      return client ? `已完成｜${client}` : '實驗完成'
    case 'transferred':
      return client ? `已轉讓→接收 ${client}` : '已轉讓'
    default:
      return animalStatusNames[info.status as AnimalStatus] ?? info.status
  }
}

/** 組合動物資訊摘要：編號 · 品系 · 客戶（狀態相依）· 欄位 */
function animalDisplay(animal: AnimalListItem, info: ClientInfo | undefined): string {
  const breed = animalSpeciesLabel(animal)
  const clientPart = info ? clientSummary(info) : animalStatusNames[animal.status]
  const parts = [animal.animal_no || '—', breed, clientPart]
  if (animal.pen_location) parts.push(animal.pen_location)
  return parts.join(' · ')
}

/** YYYY-MM-DD → M/D（直接拆字串，避開時區位移） */
function shortDate(measureDate: string): string {
  const parts = measureDate.split('-')
  if (parts.length !== 3) return measureDate
  return `${Number(parts[1])}/${Number(parts[2])}`
}

/** 前 3 次測量（唯讀，左舊右新；不足 3 筆右側留空佔位） */
function HistoryCells({ weights }: { weights: AnimalWeight[] | undefined }) {
  const ordered = [...(weights ?? []).slice(0, 3)].reverse() // 端點為 measure_date DESC → 反轉成左舊右新
  const cells: (AnimalWeight | null)[] = [...ordered, ...Array(Math.max(0, 3 - ordered.length)).fill(null)]
  return (
    <div>
      <div className="text-[11px] text-muted-foreground @[600px]:hidden">前 3 次測量</div>
      <div className="flex gap-6 @[600px]:gap-[18px]">
        {cells.map((c, i) => (
          <div key={i} className="min-w-[46px] text-left @[600px]:min-w-[50px] @[600px]:text-center">
            <div className="text-[11px] leading-snug text-muted-foreground">{c ? shortDate(c.measure_date) : '—'}</div>
            <div className={`text-sm font-semibold leading-tight ${c ? '' : 'text-muted-foreground/40'}`}>
              {c ? `${c.weight}kg` : '—'}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export function WeightEntryRow({ id, canRemove, duplicate, weightSaved, onChange, onRemove }: Props) {
  const [earTag, setEarTag] = useState('')
  const [weight, setWeight] = useState('')
  const [showMed, setShowMed] = useState(false)
  const [med, setMed] = useState<WeightMedication>(emptyMedication)

  const debouncedEarTag = useDebounce(earTag.trim(), 400)

  // 耳號即時驗證（TanStack Query，跨列去重/快取，並區分暫時性錯誤與真正查無）
  const { data, isFetching, isError } = useQuery({
    queryKey: ['ear-tag-lookup', debouncedEarTag],
    queryFn: () => lookupAliveAnimalByEarTag(debouncedEarTag),
    enabled: debouncedEarTag.length > 0,
    staleTime: 30_000,
  })

  const animal = debouncedEarTag.length > 0 && !isError ? (data?.animal ?? null) : null
  const animalId = animal?.id ?? null

  // 前 3 次測量（耳號驗證通過才抓）
  const { data: history } = useQuery({
    queryKey: ['animal-weights-recent', animalId],
    queryFn: () => api.get<AnimalWeight[]>(`/animals/${animalId}/weights`).then((r) => r.data),
    enabled: !!animalId,
    staleTime: 30_000,
  })

  // 客戶/委託資訊（狀態相依：實驗中案號+委託+PI、已轉讓接收方…）
  const { data: clientInfo } = useQuery({
    queryKey: ['animal-client-info', animalId],
    queryFn: () => api.get<ClientInfo>(`/animals/${animalId}/client-info`).then((r) => r.data),
    enabled: !!animalId,
    staleTime: 30_000,
  })

  // 施打完整性：藥名+劑量皆有才算完整；有填一半（缺其一）則擋送出
  const medComplete = !!(med.drug.trim() && med.dosage.trim())
  const medHasAny = !!(med.drug.trim() || med.dosage.trim() || med.category || med.route)
  const medPartial = showMed && medHasAny && !medComplete

  // 回報上層：animalId + weight + 是否有效（存活 + 體重 > 0）+ 施打
  const reportedEarTag = animal?.ear_tag ?? earTag.trim()
  useEffect(() => {
    const valid = animalId != null && parseFloat(weight) > 0
    onChange({
      id, animalId, earTag: reportedEarTag, weight, valid,
      medication: showMed && medComplete ? med : null,
      medPartial,
    })
  }, [id, animalId, reportedEarTag, weight, showMed, medComplete, medPartial, med, onChange])

  const clearMedication = () => {
    setMed(emptyMedication())
    setShowMed(false)
  }

  const showChecking = debouncedEarTag.length > 0 && isFetching
  const showError = debouncedEarTag.length > 0 && !isFetching && isError
  const showNotFound = debouncedEarTag.length > 0 && !isFetching && !isError && !animal

  return (
    <div className="relative rounded-lg border p-2.5 @[600px]:rounded-none @[600px]:border-0 @[600px]:p-0">
      <div className="flex flex-col gap-2 @[600px]:flex-row @[600px]:items-start @[600px]:gap-2">
        {/* 施打切換（桌面版）：貼耳號輸入欄左側，僅 @[600px]+ 顯示。
            disabled 只擋「還沒開」時的開啟；已開啟中一律可關，避免耳號中途變
            查無/無效時面板卡死打不開也關不掉（2026-08 Qodo 審查發現） */}
        <MedToggleButton
          variant="desktop"
          active={showMed}
          disabled={!animalId && !showMed}
          onClick={() => setShowMed((v) => !v)}
        />

        {/* 耳號 + 驗證狀態 + 施打切換（手機版） */}
        <div className="min-w-0 pr-7 @[600px]:flex-1 @[600px]:pr-0">
          <Input
            value={earTag}
            onChange={(e) => setEarTag(e.target.value)}
            placeholder="耳號"
            className="font-mono"
            disabled={weightSaved}
          />
          {weightSaved && (
            <p className="mt-1 flex items-center gap-1 text-xs text-status-warning-text">
              <AlertCircle className="h-3 w-3" /> 體重已登錄，僅可修改施打後重送
            </p>
          )}
          {showChecking && (
            <p className="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
              <Loader2 className="h-3 w-3 animate-spin" /> 驗證中…
            </p>
          )}
          {showError && (
            <p className="mt-1 flex items-center gap-1 text-xs text-status-warning-text">
              <AlertCircle className="h-3 w-3" /> 驗證失敗，請稍後重試
            </p>
          )}
          {animal && duplicate && (
            <p className="mt-1 flex flex-wrap items-center gap-1 text-xs text-status-error-text">
              <XCircle className="h-3 w-3" /> 此耳號已在其他列輸入
              {showMed && (
                <>
                  <span className="ml-1 font-semibold text-primary">施打紀錄</span>
                  <button type="button" onClick={clearMedication}
                    className="font-medium text-status-error-text hover:underline">清除施打</button>
                </>
              )}
            </p>
          )}
          {animal && !duplicate && (
            <p className="mt-1 flex flex-wrap items-center gap-1 text-xs text-status-success-text">
              <CheckCircle2 className="h-3 w-3" />
              <span className="font-medium">存在</span>
              <span className="text-muted-foreground">· {animalDisplay(animal, clientInfo)}</span>
              {showMed && (
                <>
                  <span className="ml-1 font-semibold text-primary">施打紀錄</span>
                  <button type="button" onClick={clearMedication}
                    className="font-medium text-status-error-text hover:underline">清除施打</button>
                </>
              )}
            </p>
          )}
          {showNotFound && (
            <p className="mt-1 flex items-center gap-1 text-xs text-status-error-text">
              <XCircle className="h-3 w-3" /> 查無存活中的此耳號動物
            </p>
          )}
          <MedToggleButton
            variant="mobile"
            active={showMed}
            disabled={!animalId && !showMed}
            onClick={() => setShowMed((v) => !v)}
          />
        </div>

        {/* 前 3 次測量 */}
        <HistoryCells weights={history} />

        {/* 今天體重輸入（卡片獨佔整排；桌面為固定欄，標籤僅卡片顯示避免與其他欄錯位） */}
        <div className="w-full @[600px]:w-24 @[600px]:flex-none">
          <div className="text-[10px] text-muted-foreground @[600px]:hidden">今天 (kg)</div>
          <Input
            type="number"
            step="0.01"
            min="0"
            value={weight}
            onChange={(e) => setWeight(e.target.value)}
            placeholder="體重 (kg)"
            className="@[600px]:text-center"
            disabled={weightSaved}
          />
        </div>

        {/* 刪除：卡片在右上角、桌面為列內 */}
        <Button
          type="button"
          variant="ghost"
          size="icon"
          onClick={onRemove}
          disabled={!canRemove}
          aria-label="刪除此列"
          className="absolute right-2 top-2 h-7 w-7 @[600px]:static @[600px]:h-8 @[600px]:w-8"
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      {showMed && (
        <div className="@[600px]:ml-[calc(2.25rem+0.5rem)]">
          <MedicationFields value={med} onChange={setMed} />
        </div>
      )}
    </div>
  )
}
