/**
 * 修正案型別
 */

export type AmendmentType = 'MAJOR' | 'MINOR' | 'PENDING'
export type AmendmentStatus =
    | 'DRAFT'
    | 'SUBMITTED'
    | 'CLASSIFIED'
    | 'UNDER_REVIEW'
    | 'REVISION_REQUIRED'
    | 'RESUBMITTED'
    | 'APPROVED'
    | 'REJECTED'
    | 'ADMIN_APPROVED'

export const amendmentStatusNames: Record<AmendmentStatus, string> = {
    DRAFT: '草稿',
    SUBMITTED: '已提交',
    CLASSIFIED: '已分類',
    UNDER_REVIEW: '審查中',
    REVISION_REQUIRED: '需修訂',
    RESUBMITTED: '已重送',
    APPROVED: '已核准',
    REJECTED: '已否決',
    ADMIN_APPROVED: '行政核准',
}

// Status colors
export const amendmentStatusColors: Record<AmendmentStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
    DRAFT: 'secondary',
    SUBMITTED: 'default',
    CLASSIFIED: 'warning',
    UNDER_REVIEW: 'outline',
    REVISION_REQUIRED: 'destructive',
    RESUBMITTED: 'default',
    APPROVED: 'success',
    REJECTED: 'destructive',
    ADMIN_APPROVED: 'success',
}

export const amendmentTypeNames: Record<AmendmentType, string> = {
    MAJOR: '重大變更',
    MINOR: '小變更',
    PENDING: '待分類',
}

// 變更項目選項（多選）
export const AMENDMENT_CHANGE_ITEM_OPTIONS = [
    { value: 'ANIMAL_COUNT', label: '動物數量' },
    { value: 'PROCEDURE', label: '實驗程序' },
    { value: 'PERSONNEL', label: '試驗工作人員' },
    { value: 'DURATION', label: '執行期間' },
    { value: 'FUNDING', label: '經費來源' },
    { value: 'FACILITY', label: '設施/場地' },
    { value: 'SPECIES', label: '動物種類/品系' },
    { value: 'ANESTHESIA', label: '麻醉方式' },
    { value: 'EUTHANASIA', label: '安樂死方法' },
    { value: 'OTHER', label: '其他' },
] as const

export interface Amendment {
    id: string
    protocol_id: string
    amendment_no: string
    revision_number: number
    amendment_type: AmendmentType
    status: AmendmentStatus
    title: string
    description?: string
    change_items?: string[]
    changes_content?: Record<string, unknown>
    submitted_by?: string
    submitted_at?: string
    classified_by?: string
    classified_at?: string
    classification_remark?: string
    created_by: string
    created_at: string
    updated_at: string
    /** R30-B: optimistic lock 版本號（forward-compat：後端 amendments 表尚未加 version 欄，
     * 待 R30 後續 PR 補上 migration 與 service 邏輯後啟用） */
    version?: number
}

export interface AmendmentListItem extends Amendment {
    protocol_iacuc_no?: string
    protocol_title?: string
    submitted_by_name?: string
    classified_by_name?: string
}

export interface CreateAmendmentRequest {
    protocol_id: string
    title: string
    description?: string
    change_items?: string[]
    changes_content?: Record<string, unknown>
}

export interface UpdateAmendmentRequest {
    title?: string
    description?: string
    change_items?: string[]
    changes_content?: Record<string, unknown>
    /** R30-B: optimistic lock 版本號（forward-compat — 待後端 amendments 表加 version
     * 欄位後啟用 lost-update 防護；目前送出會被後端忽略，無副作用） */
    version?: number
}

export interface ClassifyAmendmentRequest {
    amendment_type: AmendmentType
    remark?: string
}

export interface ChangeAmendmentStatusRequest {
    to_status: AmendmentStatus
    remark?: string
}

export interface RecordAmendmentDecisionRequest {
    decision: 'APPROVE' | 'REJECT' | 'REVISION'
    comment?: string
}

export interface AmendmentVersion {
    id: string
    amendment_id: string
    version_no: number
    content_snapshot: Record<string, unknown>
    submitted_at: string
    submitted_by: string
}

export interface AmendmentStatusHistory {
    id: string
    amendment_id: string
    from_status?: AmendmentStatus
    to_status: AmendmentStatus
    changed_by: string
    remark?: string
    created_at: string
}

export interface AmendmentReviewAssignment {
    id: string
    amendment_id: string
    reviewer_id: string
    assigned_by: string
    assigned_at: string
    decision?: string
    decided_at?: string
    comment?: string
    reviewer_name?: string
    reviewer_email?: string
}
