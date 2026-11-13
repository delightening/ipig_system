/**
 * 批次新增動物（批量進豬）的純邏輯：耳號解析、逐隻檢查、組請求。
 *
 * 全部是不碰 React、不碰網路的純函式，方便單元測試。
 * 後端對應：`POST /animals/batch`（`backend/src/services/animal/core/batch_create.rs`）。
 */
import { isIsoDateAfter } from '@/lib/utils'

/** 單次批次新增的上限。與後端 `BATCH_CREATE_MAX_ANIMALS` 同值，改一邊要一起改。 */
export const BATCH_CREATE_MAX_ANIMALS = 50

/** 「整批相同」或「逐隻不同」。 */
export type FieldMode = 'same' | 'each'

/** 可切換「相同／不同」的欄位。 */
export type ModalField = 'species' | 'gender' | 'birthDate' | 'pen'
export type FieldModes = Record<ModalField, FieldMode>

export type Gender = 'male' | 'female'

/** 對話框上方「整批共用」的資料；mode 為 `same` 的欄位取這裡的值。 */
export interface BatchShared {
  entryDate: string
  sourceId: string
  preExperimentCode: string
  speciesId: string
  breedOther: string
  gender: Gender
  birthDate: string
  penId: string
}

/** 預覽表格的一列＝一隻動物。 */
export interface BatchRow {
  key: string
  /** 已正規化為三位數字 */
  earTag: string
  /** 以下四欄只有在對應欄位為「逐隻不同」時才使用 */
  speciesId: string
  gender: Gender
  birthDate: string
  penId: string
  /** 進場體重一律逐隻輸入（字串，保留使用者輸入中的狀態） */
  weight: string
  /** 送出後後端回報的結果；使用者改動該列時會被清掉 */
  serverOutcome?: 'duplicate_warning' | 'failed'
  serverMessage?: string
  /** 耳號與其他存活動物重複時，使用者是否確認仍要建立 */
  confirmForce: boolean
}

export interface PenInfo {
  id: string
  code: string
  capacity: number
  current_count: number
}

export interface EarTagParseResult {
  /** 去重後、依輸入順序的耳號（三位數字） */
  tags: string[]
  /** 無法解析的片段（原文） */
  invalid: string[]
  /** 輸入中重複出現的耳號（只列一次） */
  duplicates: string[]
}

const RANGE_SEPARATORS = '-~–—～'
const EAR_TAG_MAX_VALUE = 999

function pad3(n: number): string {
  return String(n).padStart(3, '0')
}

/**
 * 解析耳號輸入：支援清單與範圍，可混用。
 *
 * - 分隔：空白、逗號（半形／全形）、頓號、分號、換行
 * - 單一耳號：1–3 位數字，補零成三位（`5` → `005`）
 * - 範圍：`021-040`（也接受 `~`、`–`、`—`、`～`，前後可有空白），含頭尾；起點大於終點視為無效
 * - 重複的耳號只保留第一次出現，並記在 `duplicates`
 */
export function parseEarTagInput(text: string): EarTagParseResult {
  const normalized = text.replace(new RegExp(`\\s*([${RANGE_SEPARATORS}])\\s*`, 'g'), '$1')
  const tokens = normalized.split(/[\s,，、;；]+/).filter(Boolean)

  const tags: string[] = []
  const seen = new Set<string>()
  const invalid: string[] = []
  const duplicates: string[] = []
  const dupSeen = new Set<string>()

  const push = (tag: string) => {
    if (seen.has(tag)) {
      if (!dupSeen.has(tag)) {
        dupSeen.add(tag)
        duplicates.push(tag)
      }
      return
    }
    seen.add(tag)
    tags.push(tag)
  }

  const rangeRe = new RegExp(`^(\\d{1,3})[${RANGE_SEPARATORS}](\\d{1,3})$`)
  for (const token of tokens) {
    if (/^\d{1,3}$/.test(token)) {
      push(pad3(Number(token)))
      continue
    }
    const range = rangeRe.exec(token)
    if (range) {
      const lo = Number(range[1])
      const hi = Number(range[2])
      if (lo > hi || hi > EAR_TAG_MAX_VALUE) {
        invalid.push(token)
        continue
      }
      for (let n = lo; n <= hi; n += 1) push(pad3(n))
      continue
    }
    invalid.push(token)
  }
  return { tags, invalid, duplicates }
}

/** 把新耳號接在既有列後面：已在列表中的略過，並受單次上限限制。 */
export function addTagsToRows(
  rows: BatchRow[],
  tags: string[],
  makeRow: (earTag: string) => BatchRow,
  max: number = BATCH_CREATE_MAX_ANIMALS,
): { rows: BatchRow[]; skippedExisting: number; skippedOverLimit: number } {
  const existing = new Set(rows.map((r) => r.earTag))
  const next = [...rows]
  let skippedExisting = 0
  let skippedOverLimit = 0
  for (const tag of tags) {
    if (existing.has(tag)) {
      skippedExisting += 1
      continue
    }
    if (next.length >= max) {
      skippedOverLimit += 1
      continue
    }
    existing.add(tag)
    next.push(makeRow(tag))
  }
  return { rows: next, skippedExisting, skippedOverLimit }
}

/** 某一列「實際生效」的四個可切換欄位值（依各欄位的相同／不同模式取共用值或逐隻值）。 */
export function resolveRow(row: BatchRow, shared: BatchShared, modes: FieldModes) {
  return {
    speciesId: modes.species === 'same' ? shared.speciesId : row.speciesId,
    gender: modes.gender === 'same' ? shared.gender : row.gender,
    birthDate: modes.birthDate === 'same' ? shared.birthDate : row.birthDate,
    penId: modes.pen === 'same' ? shared.penId : row.penId,
  }
}

/** 逐隻檢查的問題代碼（UI 以 `animalPages.batchAdd.issue.<code>` 顯示）。 */
export type RowIssueCode =
  | 'weightInvalid'
  | 'speciesRequired'
  | 'birthRequired'
  | 'birthFuture'
  | 'penRequired'
  | 'duplicateInBatch'
  | 'penFull'

export interface RowIssue {
  code: RowIssueCode
  /** `penFull` 用：欄位代碼、容量、已有隻數 */
  params?: { pen: string; capacity: number; current: number }
}

export interface RowChecks {
  /** key → 該列的問題（空陣列＝可送出） */
  issuesByKey: Record<string, RowIssue[]>
  /** 是否有任何列因欄位容量不足被標紅 */
  hasPenFull: boolean
}

/**
 * 逐隻檢查，只做「看得出來」的事；耳號與資料庫既有動物重複只有後端查得到，不在這裡。
 *
 * 欄位容量：`capacity > 0` 才有上限（0＝無限制），可用空位 = `capacity - current_count`；
 * 同一欄位依列順序佔用，超出空位的那幾列標紅（前面的列照常通過）。
 */
export function checkRows(
  rows: BatchRow[],
  shared: BatchShared,
  modes: FieldModes,
  pensById: ReadonlyMap<string, PenInfo>,
  todayIso: string,
): RowChecks {
  const issuesByKey: Record<string, RowIssue[]> = {}
  const tagCount = new Map<string, number>()
  for (const r of rows) tagCount.set(r.earTag, (tagCount.get(r.earTag) ?? 0) + 1)

  const penUsed = new Map<string, number>()
  let hasPenFull = false

  for (const row of rows) {
    const issues: RowIssue[] = []
    const v = resolveRow(row, shared, modes)

    const weight = Number(row.weight)
    if (!row.weight.trim() || !Number.isFinite(weight) || weight <= 0) issues.push({ code: 'weightInvalid' })
    if (!v.speciesId) issues.push({ code: 'speciesRequired' })
    if (!v.birthDate) issues.push({ code: 'birthRequired' })
    else if (isIsoDateAfter(v.birthDate, todayIso)) issues.push({ code: 'birthFuture' })
    if ((tagCount.get(row.earTag) ?? 0) > 1) issues.push({ code: 'duplicateInBatch' })

    const pen = v.penId ? pensById.get(v.penId) : undefined
    if (!pen) {
      issues.push({ code: 'penRequired' })
    } else if (pen.capacity > 0) {
      const used = (penUsed.get(pen.id) ?? 0) + 1
      penUsed.set(pen.id, used)
      if (used > pen.capacity - pen.current_count) {
        issues.push({
          code: 'penFull',
          params: { pen: pen.code, capacity: pen.capacity, current: pen.current_count },
        })
        hasPenFull = true
      }
    }
    issuesByKey[row.key] = issues
  }
  return { issuesByKey, hasPenFull }
}

/** 送給 `POST /animals/batch` 的單隻請求。 */
export interface BatchItemPayload {
  ear_tag: string
  species_id: string
  gender: Gender
  entry_date: string
  birth_date: string
  entry_weight: number
  pen_location: string
  pen_id: string
  pre_experiment_code: string
  source_id?: string
  breed_other?: string
  force_create?: boolean
}

/**
 * 把一列展開成完整的單隻請求（共用欄位在這裡填進去）。
 *
 * 同時送 `pen_location`（欄位代碼）與 `pen_id`：`pen_id` 讓後端做容量與欄位狀態檢查，
 * 並同步欄位在欄頭數。
 */
export function buildBatchItem(
  row: BatchRow,
  shared: BatchShared,
  modes: FieldModes,
  pensById: ReadonlyMap<string, PenInfo>,
): BatchItemPayload {
  const v = resolveRow(row, shared, modes)
  const pen = pensById.get(v.penId)
  const item: BatchItemPayload = {
    ear_tag: row.earTag,
    species_id: v.speciesId,
    gender: v.gender,
    entry_date: shared.entryDate,
    birth_date: v.birthDate,
    entry_weight: Number(row.weight),
    pen_location: pen?.code ?? '',
    pen_id: v.penId,
    pre_experiment_code: shared.preExperimentCode.trim(),
  }
  if (shared.sourceId) item.source_id = shared.sourceId
  if (shared.breedOther.trim() && modes.species === 'same') item.breed_other = shared.breedOther.trim()
  if (row.confirmForce) item.force_create = true
  return item
}

/** 後端 `POST /animals/batch` 回傳中單隻動物的結果。 */
export interface BatchCreateResult {
  /** 該筆在請求 `animals` 陣列中的位置（從 0 起算） */
  index: number
  ear_tag: string
  outcome: 'created' | 'duplicate_warning' | 'failed'
  animal_id?: string | null
  message?: string | null
}

/**
 * 把後端逐隻結果套回表格：已建立的移除；耳號重複與失敗的留下並標上原因，
 * 讓使用者修正或確認後再送一次。沒送出的列原樣保留。
 */
export function applyBatchResults(
  rows: BatchRow[],
  sentKeys: readonly string[],
  results: readonly BatchCreateResult[],
): { rows: BatchRow[]; createdCount: number } {
  const byKey = new Map<string, BatchCreateResult>()
  for (const r of results) {
    const key = sentKeys[r.index]
    if (key !== undefined) byKey.set(key, r)
  }

  let createdCount = 0
  const next: BatchRow[] = []
  for (const row of rows) {
    const result = byKey.get(row.key)
    if (!result) {
      next.push(row)
      continue
    }
    if (result.outcome === 'created') {
      createdCount += 1
      continue
    }
    next.push({
      ...row,
      serverOutcome: result.outcome,
      serverMessage: result.message ?? undefined,
      confirmForce: false,
    })
  }
  return { rows: next, createdCount }
}
