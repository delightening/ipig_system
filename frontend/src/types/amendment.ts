/**
 * 修正案型別
 */

import { createLabelMap, createLabelOptions } from '@/lib/i18nLabels'

import type { PendingOwner } from './pendingOwner'

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

/** 修正案狀態名稱（getter 版：每次讀取才依當下語言翻譯，見 `@/lib/i18nLabels`；沿用 `amendments.status.*`） */
export const amendmentStatusNames: Record<AmendmentStatus, string> = createLabelMap(
    'amendments.status',
    [
        'DRAFT',
        'SUBMITTED',
        'CLASSIFIED',
        'UNDER_REVIEW',
        'REVISION_REQUIRED',
        'RESUBMITTED',
        'APPROVED',
        'REJECTED',
        'ADMIN_APPROVED',
        'EFFECTIVE',
    ],
)

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

/** 修正案類型名稱（getter 版；沿用 `amendments.types.*`） */
export const amendmentTypeNames: Record<AmendmentType, string> = createLabelMap(
    'amendments.types',
    ['MAJOR', 'MINOR', 'PENDING'],
)

// 變更項目選項（多選）。`value` 是送後端的代碼；`label` 為 getter（沿用 `amendments.changeItemLabels.*`）。
export const AMENDMENT_CHANGE_ITEM_OPTIONS = createLabelOptions('amendments.changeItemLabels', [
    'ANIMAL_COUNT',
    'PROCEDURE',
    'PERSONNEL',
    'DURATION',
    'FUNDING',
    'FACILITY',
    'SPECIES',
    'ANESTHESIA',
    'EUTHANASIA',
    'OTHER',
] as const)

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
    /** 這件現在卡在誰手上；僅待分類 / 已分類待送審有值 */
    pending_owner?: PendingOwner
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
