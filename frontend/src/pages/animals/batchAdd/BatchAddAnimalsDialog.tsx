/**
 * 批次新增動物（批量進豬）對話框。
 *
 * 流程：填共用資料 → 貼耳號清單／範圍 → 預覽表格逐隻補體重（與選了「逐隻不同」的欄位）
 * → 送出。後端 `POST /animals/batch` 是部分成功：成功的從表格移除，耳號重複與失敗的留下並
 * 標上原因，使用者修正或確認後可再送一次。
 *
 * 規格由使用者 2026-11-12 逐項裁定：欄位容量不足時預覽即標紅、不能送出，由使用者逐隻改欄位
 * （不自動溢出到別的欄位）；耳號重複沿用單隻新增的規則。
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'

import api from '@/lib/api'
import type { AnimalSource } from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'
import { taipeiTodayIso } from '@/lib/utils'
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'

import { useBreedSpecies } from '../hooks/useBreedSpecies'
import { useFacilityLayout } from '../hooks/useFacilityLayout'
import { BatchAnimalRows, type PenOption, type SpeciesOption } from './BatchAnimalRows'
import {
  BATCH_CREATE_MAX_ANIMALS,
  addTagsToRows,
  applyBatchResults,
  buildBatchItem,
  checkRows,
  parseEarTagInput,
  type BatchCreateResult,
  type BatchRow,
  type BatchShared,
  type FieldMode,
  type FieldModes,
  type Gender,
  type ModalField,
  type PenInfo,
} from './batchAddLogic'

interface BatchCreateResponse {
  results: BatchCreateResult[]
  created_count: number
  not_created_count: number
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  sourcesData: AnimalSource[] | undefined
}

const ALL_SAME: FieldModes = { species: 'same', gender: 'same', birthDate: 'same', pen: 'same' }

function initialShared(): BatchShared {
  return {
    entryDate: taipeiTodayIso(),
    sourceId: '',
    preExperimentCode: '',
    speciesId: '',
    breedOther: '',
    gender: 'male',
    birthDate: '',
    penId: '',
  }
}

/** 「整批相同／逐隻不同」切換。 */
function ModeToggle({
  value,
  onChange,
  label,
}: {
  value: FieldMode
  onChange: (mode: FieldMode) => void
  label: string
}) {
  const { t } = useTranslation()
  const button = (mode: FieldMode, text: string) => (
    <button
      type="button"
      aria-pressed={value === mode}
      onClick={() => onChange(mode)}
      className={`px-2 py-0.5 ${value === mode ? 'bg-primary text-primary-foreground' : 'bg-card text-muted-foreground hover:bg-muted'}`}
    >
      {text}
    </button>
  )
  return (
    <div role="group" aria-label={label} className="inline-flex overflow-hidden rounded-md border text-xs">
      {button('same', t('animalPages.batchAdd.modeSame'))}
      {button('each', t('animalPages.batchAdd.modeEach'))}
    </div>
  )
}

export function BatchAddAnimalsDialog({ open, onOpenChange, sourcesData }: Props) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { pensByZone } = useFacilityLayout()
  const { breedSpecies } = useBreedSpecies()

  const [shared, setShared] = useState<BatchShared>(initialShared)
  const [modes, setModes] = useState<FieldModes>(ALL_SAME)
  const [rows, setRows] = useState<BatchRow[]>([])
  const [tagText, setTagText] = useState('')
  const [tagNote, setTagNote] = useState<string | null>(null)
  const [penAutoSwitched, setPenAutoSwitched] = useState(false)
  const keySeq = useRef(0)

  // 非同步回呼讀「最新」值，避免閉包拿到送出前的舊狀態
  const rowsRef = useRef(rows)
  rowsRef.current = rows
  const sharedRef = useRef(shared)
  sharedRef.current = shared

  useEffect(() => {
    if (!open) return
    setShared(initialShared())
    setModes(ALL_SAME)
    setRows([])
    setTagText('')
    setTagNote(null)
    setPenAutoSwitched(false)
  }, [open])

  // ── 欄位與品種選項 ────────────────────────────────────────────────────
  const pens = useMemo(
    () =>
      Object.values(pensByZone)
        .flat()
        .sort((a, b) =>
          `${a.building_code}${a.zone_code}${a.code}`.localeCompare(`${b.building_code}${b.zone_code}${b.code}`),
        ),
    [pensByZone],
  )
  const pensById = useMemo(() => new Map<string, PenInfo>(pens.map((p) => [p.id, p])), [pens])
  const penOptions: PenOption[] = useMemo(
    () =>
      pens.map((p) => ({
        id: p.id,
        code: p.code,
        label: `${p.building_name} / ${p.zone_name ?? p.zone_code} / ${p.code}${p.capacity > 0 ? `（${p.current_count}/${p.capacity}）` : ''}`,
      })),
    [pens],
  )
  // 逐隻選品種不提供「其他」：它需要另填自由文字的品種名稱，只在「整批相同」時支援
  const rowSpeciesOptions: SpeciesOption[] = useMemo(
    () => breedSpecies.filter((s) => s.code !== 'other').map((s) => ({ id: s.id, name: s.name })),
    [breedSpecies],
  )
  const sharedSpecies = breedSpecies.find((s) => s.id === shared.speciesId)
  const needsBreedOther =
    modes.species === 'same' && sharedSpecies?.code === 'other' && !shared.breedOther.trim()

  // ── 逐隻檢查 ──────────────────────────────────────────────────────────
  const todayIso = taipeiTodayIso()
  const { issuesByKey, hasPenFull } = useMemo(
    () => checkRows(rows, shared, modes, pensById, todayIso),
    [rows, shared, modes, pensById, todayIso],
  )

  const makeRow = useCallback(
    (earTag: string): BatchRow => {
      const s = sharedRef.current
      const speciesIsOther = breedSpecies.find((sp) => sp.id === s.speciesId)?.code === 'other'
      keySeq.current += 1
      return {
        key: `row-${keySeq.current}`,
        earTag,
        // 預填共用值：切到「逐隻不同」時每列從共用值開始改
        speciesId: speciesIsOther ? '' : s.speciesId,
        gender: s.gender,
        birthDate: s.birthDate,
        penId: s.penId,
        weight: '',
        confirmForce: false,
      }
    },
    [breedSpecies],
  )

  const changeMode = useCallback((field: ModalField, mode: FieldMode) => {
    setModes((prev) => ({ ...prev, [field]: mode }))
    if (mode !== 'each') return
    const s = sharedRef.current
    setRows((prev) =>
      prev.map((r) => {
        switch (field) {
          case 'species':
            return { ...r, speciesId: r.speciesId || s.speciesId }
          case 'gender':
            return { ...r, gender: s.gender }
          case 'birthDate':
            return { ...r, birthDate: r.birthDate || s.birthDate }
          case 'pen':
            return { ...r, penId: r.penId || s.penId }
        }
      }),
    )
  }, [])

  // 欄位容量不足 → 自動改成「逐隻指定欄位」，讓使用者能逐隻改欄位（不自動溢出）
  useEffect(() => {
    if (hasPenFull && modes.pen === 'same') {
      changeMode('pen', 'each')
      setPenAutoSwitched(true)
    }
  }, [hasPenFull, modes.pen, changeMode])

  // ── 列表操作 ──────────────────────────────────────────────────────────
  const handleAddTags = () => {
    const parsed = parseEarTagInput(tagText)
    const added = addTagsToRows(rows, parsed.tags, makeRow)
    setRows(added.rows)

    const notes: string[] = []
    if (parsed.invalid.length > 0) notes.push(t('animalPages.batchAdd.noteInvalid', { items: parsed.invalid.join('、') }))
    if (parsed.duplicates.length > 0)
      notes.push(t('animalPages.batchAdd.noteDuplicates', { items: parsed.duplicates.join('、') }))
    if (added.skippedExisting > 0)
      notes.push(t('animalPages.batchAdd.noteSkippedExisting', { count: added.skippedExisting }))
    if (added.skippedOverLimit > 0)
      notes.push(
        t('animalPages.batchAdd.noteSkippedOverLimit', { count: added.skippedOverLimit, max: BATCH_CREATE_MAX_ANIMALS }),
      )
    setTagNote(notes.length > 0 ? notes.join(' ') : null)
    // 只有真的有東西加進去或全部都有回饋時才清空輸入；全無效時保留，方便修改
    if (parsed.tags.length > 0) setTagText('')
  }

  /** 改動資料欄位會清掉該列先前的後端結果（已不代表目前內容）。 */
  const updateRow = (key: string, patch: Partial<BatchRow>) =>
    setRows((prev) =>
      prev.map((r) =>
        r.key === key
          ? { ...r, ...patch, serverOutcome: undefined, serverMessage: undefined, confirmForce: false }
          : r,
      ),
    )
  const toggleConfirm = (key: string, confirmed: boolean) =>
    setRows((prev) => prev.map((r) => (r.key === key ? { ...r, confirmForce: confirmed } : r)))
  const removeRow = (key: string) => setRows((prev) => prev.filter((r) => r.key !== key))

  const patchShared = (patch: Partial<BatchShared>) => setShared((prev) => ({ ...prev, ...patch }))

  // ── 送出 ──────────────────────────────────────────────────────────────
  const mutation = useMutation({
    mutationFn: async (sendRows: BatchRow[]) => {
      const animals = sendRows.map((r) => buildBatchItem(r, shared, modes, pensById))
      const res = await api.post<BatchCreateResponse>('/animals/batch', { animals })
      return { sentKeys: sendRows.map((r) => r.key), data: res.data }
    },
    onSuccess: ({ sentKeys, data }) => {
      for (const key of ['animals', 'animals-by-pen', 'animals-stats', 'facility-pens']) {
        queryClient.invalidateQueries({ queryKey: [key] })
      }
      const applied = applyBatchResults(rowsRef.current, sentKeys, data.results)
      setRows(applied.rows)
      if (applied.rows.length === 0) {
        toast({
          title: t('common.success'),
          description: t('animalPages.batchAdd.createdAll', { count: applied.createdCount }),
        })
        onOpenChange(false)
        return
      }
      toast({
        title: t('animalPages.batchAdd.createdPartialTitle'),
        description: t('animalPages.batchAdd.createdPartial', {
          created: applied.createdCount,
          remaining: applied.rows.length,
        }),
        variant: 'destructive',
      })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalPages.batchAdd.createFailed')),
        variant: 'destructive',
      })
    },
  })

  const hasIssues = rows.some((r) => (issuesByKey[r.key]?.length ?? 0) > 0)
  const hasUnconfirmedDuplicate = rows.some((r) => r.serverOutcome === 'duplicate_warning' && !r.confirmForce)
  const sharedInvalid = !shared.entryDate || !shared.preExperimentCode.trim() || needsBreedOther
  const canSubmit =
    rows.length > 0 && !hasIssues && !hasUnconfirmedDuplicate && !sharedInvalid && !mutation.isPending
  const hasServerResult = rows.some((r) => r.serverOutcome)

  const speciesSelectOptions = breedSpecies.map((s) => ({ id: s.id, name: s.name }))

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="2xl">
        <DialogHeader>
          <DialogTitle>{t('animalPages.batchAdd.title')}</DialogTitle>
          <DialogDescription>{t('animalPages.batchAdd.description')}</DialogDescription>
        </DialogHeader>

        <div className="max-h-[65vh] space-y-4 overflow-y-auto pr-1">
          {/* ── 整批共用資料 ── */}
          <section className="space-y-3">
            <h3 className="text-sm font-semibold">{t('animalPages.batchAdd.sharedTitle')}</h3>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
              <div className="space-y-1">
                <Label htmlFor="batch_entry_date">{t('animals.entryDate')} *</Label>
                <Input
                  id="batch_entry_date"
                  type="date"
                  value={shared.entryDate}
                  onChange={(e) => patchShared({ entryDate: e.target.value })}
                />
              </div>
              <div className="space-y-1">
                <Label>{t('animalPages.shared.source')}</Label>
                <Select
                  value={shared.sourceId || 'none'}
                  onValueChange={(v) => patchShared({ sourceId: v === 'none' ? '' : v })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('animalPages.addDialog.sourcePlaceholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">{t('animalPages.addDialog.sourceNone')}</SelectItem>
                    {sourcesData?.map((s) => (
                      <SelectItem key={s.id} value={s.id}>
                        {s.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label htmlFor="batch_pre_code">{t('animalPages.shared.preExperimentCode')} *</Label>
                <Input
                  id="batch_pre_code"
                  value={shared.preExperimentCode}
                  placeholder={t('animalPages.addDialog.preExperimentCodePlaceholder')}
                  onChange={(e) => patchShared({ preExperimentCode: e.target.value })}
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              {/* 品種 */}
              <div className="space-y-1">
                <div className="flex items-center justify-between gap-2">
                  <Label>{t('animals.breed')} *</Label>
                  <ModeToggle value={modes.species} onChange={(m) => changeMode('species', m)} label={t('animals.breed')} />
                </div>
                {modes.species === 'same' ? (
                  <>
                    <Select
                      value={shared.speciesId}
                      onValueChange={(v) => patchShared({ speciesId: v, breedOther: '' })}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder={t('animalPages.shared.selectBreed')} />
                      </SelectTrigger>
                      <SelectContent>
                        {speciesSelectOptions.map((s) => (
                          <SelectItem key={s.id} value={s.id}>
                            {s.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {sharedSpecies?.code === 'other' && (
                      <Input
                        value={shared.breedOther}
                        placeholder={t('animalPages.shared.enterBreedName')}
                        aria-label={t('animalPages.addDialog.breedOther')}
                        onChange={(e) => patchShared({ breedOther: e.target.value })}
                      />
                    )}
                  </>
                ) : (
                  <p className="text-xs text-muted-foreground">{t('animalPages.batchAdd.eachHint')}</p>
                )}
              </div>

              {/* 性別 */}
              <div className="space-y-1">
                <div className="flex items-center justify-between gap-2">
                  <Label>{t('animals.gender')} *</Label>
                  <ModeToggle value={modes.gender} onChange={(m) => changeMode('gender', m)} label={t('animals.gender')} />
                </div>
                {modes.gender === 'same' ? (
                  <Select value={shared.gender} onValueChange={(v) => patchShared({ gender: v as Gender })}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="male">{t('animals.genderLabels.male')}</SelectItem>
                      <SelectItem value="female">{t('animals.genderLabels.female')}</SelectItem>
                    </SelectContent>
                  </Select>
                ) : (
                  <p className="text-xs text-muted-foreground">{t('animalPages.batchAdd.eachHint')}</p>
                )}
              </div>

              {/* 出生日期 */}
              <div className="space-y-1">
                <div className="flex items-center justify-between gap-2">
                  <Label>{t('animals.birthDate')} *</Label>
                  <ModeToggle
                    value={modes.birthDate}
                    onChange={(m) => changeMode('birthDate', m)}
                    label={t('animals.birthDate')}
                  />
                </div>
                {modes.birthDate === 'same' ? (
                  <Input
                    type="date"
                    value={shared.birthDate}
                    max={todayIso}
                    onChange={(e) => patchShared({ birthDate: e.target.value })}
                  />
                ) : (
                  <p className="text-xs text-muted-foreground">{t('animalPages.batchAdd.eachHint')}</p>
                )}
              </div>

              {/* 欄位 */}
              <div className="space-y-1">
                <div className="flex items-center justify-between gap-2">
                  <Label>{t('animalPages.addDialog.penNumber')} *</Label>
                  <ModeToggle
                    value={modes.pen}
                    onChange={(m) => {
                      setPenAutoSwitched(false)
                      changeMode('pen', m)
                    }}
                    label={t('animalPages.addDialog.penNumber')}
                  />
                </div>
                {modes.pen === 'same' ? (
                  <Select value={shared.penId} onValueChange={(v) => patchShared({ penId: v })}>
                    <SelectTrigger>
                      <SelectValue placeholder={t('animalPages.batchAdd.select')}>
                        {pensById.get(shared.penId)?.code}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {penOptions.map((p) => (
                        <SelectItem key={p.id} value={p.id}>
                          {p.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                ) : (
                  <p className="text-xs text-muted-foreground">
                    {penAutoSwitched ? t('animalPages.batchAdd.penAutoSwitched') : t('animalPages.batchAdd.eachHint')}
                  </p>
                )}
              </div>
            </div>
          </section>

          {/* ── 耳號輸入 ── */}
          <section className="space-y-2">
            <Label htmlFor="batch_ear_tags">{t('animals.earTag')} *</Label>
            <Textarea
              id="batch_ear_tags"
              rows={2}
              value={tagText}
              placeholder={t('animalPages.batchAdd.earTagsPlaceholder')}
              onChange={(e) => setTagText(e.target.value)}
            />
            <div className="flex flex-wrap items-center gap-2">
              <Button type="button" variant="outline" size="sm" onClick={handleAddTags} disabled={!tagText.trim()}>
                {t('animalPages.batchAdd.addToList')}
              </Button>
              <span className="text-xs text-muted-foreground">{t('animalPages.batchAdd.earTagsHint')}</span>
            </div>
            {tagNote && (
              <p className="text-xs text-status-warning-text" role="status">
                {tagNote}
              </p>
            )}
          </section>

          {/* ── 預覽 ── */}
          <section className="space-y-2">
            <h3 className="text-sm font-semibold">
              {t('animalPages.batchAdd.listTitle', { count: rows.length, max: BATCH_CREATE_MAX_ANIMALS })}
            </h3>
            {hasServerResult && (
              <p className="text-xs text-status-warning-text" role="status">
                {t('animalPages.batchAdd.partialResultHint')}
              </p>
            )}
            {rows.length === 0 ? (
              <p className="rounded-md border border-dashed p-6 text-center text-sm text-muted-foreground">
                {t('animalPages.batchAdd.emptyList')}
              </p>
            ) : (
              <BatchAnimalRows
                rows={rows}
                modes={modes}
                issuesByKey={issuesByKey}
                speciesOptions={rowSpeciesOptions}
                penOptions={penOptions}
                onChange={updateRow}
                onToggleConfirm={toggleConfirm}
                onRemove={removeRow}
              />
            )}
          </section>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={() => mutation.mutate(rows)}
            disabled={!canSubmit}
            className="bg-primary hover:bg-primary/90"
          >
            {mutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t('animalPages.batchAdd.submit', { count: rows.length })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
