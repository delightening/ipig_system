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
    | 'EFFECTIVE'

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
    EFFECTIVE: '已生效',
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
    EFFECTIVE: 'success',
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

/**
 * R71-12：結構化變更明細（存入既有 changes_content jsonb，無需 migration）。
 * 一個整體「變更目的」+ 多列「項次 / 改動前 / 改動後」前後對照。
 * 舊資料無此結構時前端優雅退回顯示 title/description + change_items。
 */
export interface AmendmentChangeDetailItem {
    section: string // 項次，例 "4.1.2"
    before: string // 改動前
    after: string // 改動後
}
export interface AmendmentChangeContent {
    purpose?: string // 變更目的
    items?: AmendmentChangeDetailItem[]
}

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
    /** R30-25 GLP §58：amendment 正式生效時點。null/undefined = 尚未生效（含 APPROVED 但未啟用）。 */
    effective_from?: string | null
    /** R30-B: optimistic lock 版本號（forward-compat：後端 amendments 表尚未加 version 欄，
     * 待 R30 後續 PR 補上 migration 與 service 邏輯後啟用） */
    version?: number
    /** P6：補登歷史變更標記（紙本核准回溯，跳過 live 審查與簽章） */
    is_historical?: boolean
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
    /** 院外委員（補登歷史變更）為 null，姓名走 reviewer_name */
    reviewer_id?: string | null
    assigned_by: string
    assigned_at: string
    decision?: string
    decided_at?: string
    comment?: string
    reviewer_name?: string
    reviewer_email?: string
}

// ── P6：補登歷史變更 ──

export interface CreateHistoricalAmendmentRequest {
    protocol_id: string
    title: string
    description?: string
    change_items?: string[]
    changes_content?: Record<string, unknown>
    /** 歷史分類：MAJOR（委員審）/ MINOR（執秘行政核准） */
    amendment_type: 'MAJOR' | 'MINOR'
    /** 原始送件日期（ISO，回填） */
    submitted_at?: string
    /** 原始分類日期（ISO，回填） */
    classified_at?: string
    classification_remark?: string
}

export interface FinalizeHistoricalAmendmentRequest {
    /** 原始生效日期（ISO）；留空取後端 NOW() */
    effective_from?: string
    remark?: string
}

export interface HistoricalAmendmentReviewer {
    /** 系統內委員 user id；院外委員留空，改填 reviewer_name */
    reviewer_id?: string
    reviewer_name?: string
    decision?: 'APPROVE' | 'REJECT' | 'REVISION'
    comment?: string
    decided_at?: string
}

export interface RecordHistoricalReviewsRequest {
    reviewers: HistoricalAmendmentReviewer[]
}
