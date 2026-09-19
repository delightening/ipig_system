import type { ImportApprovedProtocolRequest } from '@/lib/api/protocol'

/** 審查里程碑（依官方審查流程時序）。required=匯入必填；其餘為可選補登。 */
export type MilestoneKey = Extract<
  keyof ImportApprovedProtocolRequest,
  | 'submitted_at' | 'pre_review_at' | 'vet_review_at'
  | 'committee_first_review_at' | 'revision_required_at'
  | 'committee_second_review_at' | 'approved_at'
>
export type MilestoneState = Record<MilestoneKey, string>

// labelKey 為 i18n 鍵；渲染時才 t(labelKey)（模組頂層不可存翻譯後字串）
export const MILESTONES: ReadonlyArray<{ key: MilestoneKey; labelKey: string; required: boolean }> = [
  { key: 'submitted_at', labelKey: 'protocolPages.importReview.milestones.submitted', required: true },
  { key: 'pre_review_at', labelKey: 'protocolPages.importReview.milestones.preReview', required: true },
  { key: 'vet_review_at', labelKey: 'protocolPages.importReview.milestones.vetReview', required: true },
  { key: 'committee_first_review_at', labelKey: 'protocolPages.importReview.milestones.committeeFirst', required: true },
  { key: 'revision_required_at', labelKey: 'protocolPages.importReview.milestones.revisionRequired', required: false },
  { key: 'committee_second_review_at', labelKey: 'protocolPages.importReview.milestones.committeeSecond', required: false },
  { key: 'approved_at', labelKey: 'protocolPages.importReview.milestones.approved', required: true },
]

export const EMPTY_MILESTONES: MilestoneState = MILESTONES.reduce(
  (acc, m) => ({ ...acc, [m.key]: '' }),
  {} as MilestoneState,
)

/** 已填的里程碑是否未依時序遞增（YYYY-MM-DD 字串可直接比較） */
export function milestonesOutOfOrder(values: MilestoneState): boolean {
  const filled = MILESTONES.map((m) => values[m.key]).filter(Boolean)
  return filled.some((d, i) => i > 0 && d < filled[i - 1])
}

/** 把里程碑 state 轉成 API payload（空字串 → null） */
export function milestonePayload(values: MilestoneState): Pick<ImportApprovedProtocolRequest, MilestoneKey> {
  return MILESTONES.reduce<Pick<ImportApprovedProtocolRequest, MilestoneKey>>(
    (acc, m) => ({ ...acc, [m.key]: values[m.key] || null }),
    {} as Pick<ImportApprovedProtocolRequest, MilestoneKey>,
  )
}
