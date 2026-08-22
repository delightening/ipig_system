/** 補登審查者選擇值：系統內用 id、院外用姓名擇一。 */
export interface ReviewerValue {
  reviewer_id: string | null
  reviewer_name: string
}

export const EMPTY_REVIEWER: ReviewerValue = { reviewer_id: null, reviewer_name: '' }
