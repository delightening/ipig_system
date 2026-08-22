import api from './client'
import { formatEarTag } from '@/lib/utils'
import type { AnimalListItem, AnimalStatus, PaginatedResponse } from '@/types'

/**
 * 「存活且場內存在」排除的狀態：已死亡（安樂死／猝死）與已轉讓（離場）。
 * 對齊後端「在欄活躍」定義（services/animal/core/update.rs）。
 */
const EXCLUDED_STATUSES: readonly AnimalStatus[] = ['euthanized', 'sudden_death', 'transferred']

export interface EarTagLookupResult {
  /** 正規化後的耳號（純數字 <100 補三位數） */
  formattedEarTag: string
  /** 唯一相符且存活的動物；查無、已死亡／轉讓、或多筆相符時為 null */
  animal: AnimalListItem | null
}

/**
 * 依耳號查詢「存活中」的動物。
 *
 * 複用標準動物列表查詢（GET /animals?keyword=）再於前端精確比對耳號 + 過濾存活狀態，
 * 與「動物列表空欄打耳號移動動物」採用相同的驗證方式，不需專屬後端 endpoint。
 */
export async function lookupAliveAnimalByEarTag(earTag: string): Promise<EarTagLookupResult> {
  const formattedEarTag = formatEarTag(earTag.trim())
  const res = await api.get<PaginatedResponse<AnimalListItem>>(
    `/animals?keyword=${encodeURIComponent(formattedEarTag)}`,
  )
  const matches = (res.data.data ?? []).filter(
    (a) => a.ear_tag === formattedEarTag && !EXCLUDED_STATUSES.includes(a.status),
  )
  return { formattedEarTag, animal: matches.length === 1 ? matches[0] : null }
}
