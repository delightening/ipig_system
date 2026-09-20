import { forwardRef, useCallback, useEffect, useImperativeHandle, useRef, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { WeightEntryRow, type ResolvedWeightRow, type WeightMedication } from './WeightEntryRow'
import { treatmentCategoryLabel, treatmentRouteLabel } from './treatmentConstants'

// 施打 observation 的病歷內容（content 為 observation 必填欄）
function medicationContent(m: WeightMedication): string {
  const cat = treatmentCategoryLabel(m.category)
  const dose = [m.dosage, m.dosage_unit].filter(Boolean).join(' ')
  const head = [cat ? `[${cat}]` : '', m.drug, dose].filter(Boolean).join(' ')
  const route = treatmentRouteLabel(m.route)
  return `施打紀錄（量體重順便）：${head}${route ? `，途徑 ${route}` : ''}`
}

// 驅蟲施打 → animal_vaccinations.deworming_dose 組字：「藥名 劑量 單位（途徑）」。
// 2026-07-21 裁定：驅蟲類施打單寫疫苗/驅蟲表（疫苗分頁、病歷 PDF、匯出可見），
// 不再寫入觀察紀錄 treatments，避免同一事件兩套紀錄源；抗生素/其他屬治療，維持觀察紀錄。
function dewormingDoseText(m: WeightMedication): string {
  const dose = [m.dosage, m.dosage_unit].filter(Boolean).join(' ')
  const route = treatmentRouteLabel(m.route)
  return [m.drug, dose].filter(Boolean).join(' ') + (route ? `（${route}）` : '')
}

// 施打 → observation treatments 一筆（結構化欄位，空值省略）
function buildTreatment(m: WeightMedication) {
  return {
    drug: m.drug,
    dosage: m.dosage,
    dosage_unit: m.dosage_unit || undefined,
    drug_option_id: m.drug_option_id,
    category: m.category || undefined,
    route: m.route || undefined,
  }
}

// 本地日曆日期（避免 toISOString() 的 UTC 位移在 UTC+8 凌晨少一天）
const today = () => {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

interface SubmitError {
  id: string
  earTag: string
  message: string
}

/** 供上層（ImportDialog 底部單一按鈕）觸發送出 */
export interface ManualWeightEntryHandle {
  submit: () => void
}

interface Props {
  /** 回報目前狀態給上層，用以驅動底部統一按鈕的 enable / 衝突判定 / loading */
  onStatusChange: (status: { ready: boolean; hasInput: boolean; pending: boolean }) => void
}

export const ManualWeightEntry = forwardRef<ManualWeightEntryHandle, Props>(
  function ManualWeightEntry({ onStatusChange }, ref) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const idCounter = useRef(0)
    const newId = () => `row-${idCounter.current++}`

    const [rowIds, setRowIds] = useState<string[]>(() => [newId()])
    const [resolved, setResolved] = useState<Record<string, ResolvedWeightRow>>({})
    const [measureDate, setMeasureDate] = useState(today)
    // 列 id → 已成功存體重的 animalId：施打失敗保留該列重送時，跳過體重（不重複登錄）。
    // 用 state 而非 ref：weightSaved 會決定 WeightEntryRow 的 disabled 渲染，render 期間
    // 讀 ref 違反 React 純函式規則（React 19 Compiler 可能 memoize 而漏掉更新）。
    // 記 animalId 而非布林屬防禦：值不匹配時鎖不生效、體重照送，不會誤跳過。
    const [savedWeights, setSavedWeights] = useState<Record<string, string>>({})

    const handleRowChange = useCallback((row: ResolvedWeightRow) => {
      setResolved((prev) => ({ ...prev, [row.id]: row }))
    }, [])

    const addRow = () => setRowIds((prev) => [...prev, newId()])

    const removeRow = (id: string) => {
      setRowIds((prev) => prev.filter((rid) => rid !== id))
      setResolved((prev) => {
        const next = { ...prev }
        delete next[id]
        return next
      })
      setSavedWeights((prev) => {
        const next = { ...prev }
        delete next[id]
        return next
      })
    }

    // 同批次重複耳號偵測：同一 animalId 出現於多列時全數標記重複並擋下送出
    const animalIdCounts: Record<string, number> = {}
    for (const id of rowIds) {
      const aid = resolved[id]?.animalId
      if (aid) animalIdCounts[aid] = (animalIdCounts[aid] ?? 0) + 1
    }
    const isDuplicate = (id: string) => {
      const aid = resolved[id]?.animalId
      return aid != null && (animalIdCounts[aid] ?? 0) > 1
    }

    const validRows = rowIds
      .filter((id) => resolved[id]?.valid && resolved[id]?.animalId != null && !isDuplicate(id))
      .map((id) => resolved[id])

    // 有列填了一半施打（缺藥名或劑量）→ 擋下整批送出
    const hasMedError = rowIds.some((id) => resolved[id]?.medPartial)
    const canSubmit =
      validRows.length > 0 && validRows.length === rowIds.length && !!measureDate && !hasMedError
    const hasInput = rowIds.some((id) => {
      const r = resolved[id]
      return !!r && (r.earTag.trim() !== '' || r.weight.trim() !== '')
    })

    const submitMutation = useMutation({
      // 每列先送體重；體重成功且有填施打，再送施打（施打依附體重）。
      // 施打路由：驅蟲 → animal_vaccinations（疫苗/驅蟲表，單寫）；抗生素/其他 → observation。
      // 分兩類錯誤：weightErrors（體重未存，可重送）、medErrors（體重已存但施打失敗，
      // 該列保留供修正重送——weightSavedRef 記住已存體重，重送只補施打不重複登錄體重）。
      mutationFn: async () => {
        const weightErrors: SubmitError[] = []
        const medErrors: SubmitError[] = []
        let successCount = 0
        // 本次送出的已存體重快照（mutation 內不可 setState，結束後於 onSuccess 套用）
        const nextSavedWeights: Record<string, string> = { ...savedWeights }
        await Promise.all(
          validRows.map(async (row) => {
            const animalId = row.animalId as string
            if (nextSavedWeights[row.id] !== animalId) {
              try {
                await api.post(`/animals/${animalId}/weights`, {
                  measure_date: measureDate,
                  weight: parseFloat(row.weight),
                  // 匯入對話框強制只允許場內存活動物（後端存活防呆）
                  enforce_active: true,
                })
                nextSavedWeights[row.id] = animalId
              } catch (e) {
                weightErrors.push({ id: row.id, earTag: row.earTag, message: getApiErrorMessage(e, t('animalRecords.weights.manual.weightFailed')) })
                return
              }
            }
            if (row.medication) {
              try {
                if (row.medication.category === 'dewormer') {
                  await api.post(`/animals/${animalId}/vaccinations`, {
                    administered_date: measureDate,
                    deworming_dose: dewormingDoseText(row.medication),
                  })
                } else {
                  await api.post(`/animals/${animalId}/observations`, {
                    event_date: measureDate,
                    record_type: 'observation',
                    content: medicationContent(row.medication),
                    treatments: [buildTreatment(row.medication)],
                  })
                }
              } catch (e) {
                medErrors.push({ id: row.id, earTag: row.earTag, message: getApiErrorMessage(e, t('animalRecords.weights.manual.medFailed')) })
                return
              }
            }
            successCount += 1
          }),
        )
        return { successCount, weightErrors, medErrors, nextSavedWeights }
      },
      onSuccess: ({ successCount, weightErrors, medErrors, nextSavedWeights }) => {
        queryClient.invalidateQueries({ queryKey: ['animals'] })
        queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
        queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
        queryClient.invalidateQueries({ queryKey: ['animal-vaccinations'] })
        if (weightErrors.length === 0 && medErrors.length === 0) {
          toast({ title: t('animalRecords.weights.manual.successTitle'), description: t('animalRecords.weights.manual.successDescription', { count: successCount }) })
          idCounter.current = 0
          setSavedWeights({})
          setResolved({})
          setRowIds([newId()])
          return
        }
        setSavedWeights(nextSavedWeights)
        toast({
          title: t('animalRecords.weights.manual.partialTitle'),
          description: t('animalRecords.weights.manual.partialDescription', { success: successCount, weight: weightErrors.length, med: medErrors.length }),
          variant: 'destructive',
        })
        // 保留兩類失敗列供修正後重送：體重未存者整列重送；體重已存但施打失敗者
        // 由 weightSavedRef 擋掉體重重複登錄，重送只補施打。
        const retryIds = new Set([...weightErrors, ...medErrors].map((e) => e.id))
        setRowIds((prev) => {
          const kept = prev.filter((id) => retryIds.has(id))
          return kept.length > 0 ? kept : [newId()]
        })
        setResolved((prev) => {
          const next: Record<string, ResolvedWeightRow> = {}
          for (const id of Object.keys(prev)) {
            if (retryIds.has(id)) next[id] = prev[id]
          }
          return next
        })
      },
      onError: (error: unknown) => {
        toast({ title: t('animalRecords.weights.manual.failedTitle'), description: getApiErrorMessage(error, t('animalRecords.weights.manual.unknownError')), variant: 'destructive' })
      },
    })

    const { mutate: submitMutate, isPending } = submitMutation
    useImperativeHandle(ref, () => ({ submit: () => submitMutate() }), [submitMutate])

    // 回報狀態給上層（統一按鈕用）
    useEffect(() => {
      onStatusChange({ ready: canSubmit, hasInput, pending: isPending })
    }, [canSubmit, hasInput, isPending, onStatusChange])

    const weightErrors = submitMutation.data?.weightErrors ?? []
    const medErrors = submitMutation.data?.medErrors ?? []

    return (
      <div className="flex h-full flex-col gap-3">
        <div className="flex flex-none items-center justify-end gap-2">
          <Label htmlFor="manual-weight-date" className="text-xs text-muted-foreground">{t('animalRecords.weights.measureDate')}</Label>
          <Input
            id="manual-weight-date"
            type="date"
            value={measureDate}
            onChange={(e) => setMeasureDate(e.target.value)}
            className="h-8 w-40"
          />
        </div>

        <div className="@container min-h-0 flex-1 space-y-2 overflow-y-auto">
          {rowIds.map((id) => (
            <WeightEntryRow
              key={id}
              id={id}
              canRemove={rowIds.length > 1}
              duplicate={isDuplicate(id)}
              weightSaved={
                savedWeights[id] != null && savedWeights[id] === resolved[id]?.animalId
              }
              onChange={handleRowChange}
              onRemove={() => removeRow(id)}
            />
          ))}
        </div>

        <Button type="button" variant="outline" size="sm" onClick={addRow} className="flex-none self-start">
          <Plus className="mr-1 h-4 w-4" /> {t('animalRecords.weights.manual.addRow')}
        </Button>

        {weightErrors.length > 0 && (
          <ul className="flex-none space-y-0.5 text-xs text-status-error-text">
            {weightErrors.map((err, i) => (
              <li key={`weight-err-${i}`}>{t('animalRecords.weights.manual.weightError', { earTag: err.earTag, message: err.message })}</li>
            ))}
          </ul>
        )}
        {medErrors.length > 0 && (
          <ul className="flex-none space-y-0.5 text-xs text-status-warning-text">
            {medErrors.map((err, i) => (
              <li key={`med-err-${i}`}>{t('animalRecords.weights.manual.medError', { earTag: err.earTag, message: err.message })}</li>
            ))}
          </ul>
        )}
      </div>
    )
  },
)
