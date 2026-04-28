/**
 * AUP 計畫書型別（Protocol、審查、活動）
 *
 * 注意：ProtocolWorkingContent 已定義在 ./protocol.ts
 */

import { ProtocolWorkingContent } from './protocol'

// 計畫狀態
export type ProtocolStatus =
    | 'DRAFT'
    | 'SUBMITTED'
    | 'PRE_REVIEW'
    | 'PRE_REVIEW_REVISION_REQUIRED'
    | 'VET_REVIEW'
    | 'VET_REVISION_REQUIRED'
    | 'UNDER_REVIEW'
    | 'REVISION_REQUIRED'
    | 'RESUBMITTED'
    | 'APPROVED'
    | 'APPROVED_WITH_CONDITIONS'
    | 'DEFERRED'
    | 'REJECTED'
    | 'SUSPENDED'
    | 'CLOSED'
    | 'DELETED'

export const protocolStatusNames: Record<ProtocolStatus, string> = {
    DRAFT: '草稿',
    SUBMITTED: '已提交',
    PRE_REVIEW: '行政預審',
    PRE_REVIEW_REVISION_REQUIRED: '行政預審補件',
    VET_REVIEW: '獸醫審查',
    VET_REVISION_REQUIRED: '獸醫要求修訂',
    UNDER_REVIEW: '審查中',
    REVISION_REQUIRED: '需修訂',
    RESUBMITTED: '已重送',
    APPROVED: '已核准',
    APPROVED_WITH_CONDITIONS: '附條件核准',
    DEFERRED: '延後審議',
    REJECTED: '已否決',
    SUSPENDED: '已暫停',
    CLOSED: '已結案',
    DELETED: '已刪除',
}

// 計畫書
export interface Protocol {
    id: string
    protocol_no: string
    iacuc_no?: string
    title: string
    status: ProtocolStatus
    pi_user_id: string
    working_content?: ProtocolWorkingContent
    start_date?: string
    end_date?: string
    created_by: string
    created_at: string
    updated_at: string
    /** R30-B: optimistic lock 版本號，前端送 PUT 時帶回防 lost update */
    version: number
}

export interface ProtocolListItem {
    id: string
    protocol_no: string
    iacuc_no?: string
    title: string
    status: ProtocolStatus
    pi_user_id: string
    pi_name: string
    pi_organization?: string
    start_date?: string
    end_date?: string
    created_at: string
    apply_study_number?: string
}

export interface VetReviewItem {
    item_name: string
    compliance: string
    comment?: string
    pi_reply?: string
}

export interface VetReviewFormData {
    items: VetReviewItem[]
    vet_signature?: string
    signed_at?: string
}

export interface VetReviewAssignment {
    vet_id: string
    review_form?: VetReviewFormData
}

export interface ProtocolResponse {
    protocol: Protocol
    pi_name?: string
    pi_email?: string
    pi_organization?: string
    status_display: string
    vet_review?: VetReviewAssignment
}

export interface ProtocolVersion {
    id: string
    protocol_id: string
    version_no: number
    content_snapshot: ProtocolWorkingContent
    submitted_at: string
    submitted_by: string
}

export interface ProtocolStatusHistory {
    id: string
    protocol_id: string
    from_status?: ProtocolStatus
    to_status: ProtocolStatus
    changed_by: string
    remark?: string
    created_at: string
}

// 審查
export interface ReviewAssignment {
    id: string
    protocol_id: string
    reviewer_id: string
    assigned_by: string
    assigned_at: string
    completed_at?: string
    /** 是否為正式審查委員（可撰寫意見） */
    is_primary_reviewer?: boolean
    /** 審查階段 */
    review_stage?: 'PRE_REVIEW' | 'VET_REVIEW' | 'UNDER_REVIEW'
}

export interface ReviewComment {
    id: string
    protocol_version_id?: string
    protocol_id?: string
    reviewer_id: string
    content: string
    is_resolved: boolean
    resolved_by?: string
    resolved_at?: string
    /** 審查階段 */
    review_stage?: 'PRE_REVIEW' | 'VET_REVIEW' | 'UNDER_REVIEW'
    created_at: string
    updated_at: string
}

export interface ReviewCommentResponse extends ReviewComment {
    reviewer_name: string
    reviewer_email: string
    parent_comment_id?: string
    replied_by?: string
    replied_by_name?: string
    replied_by_email?: string
}

// 審查請求
export interface CreateProtocolRequest {
    title: string
    pi_user_id?: string
    working_content?: ProtocolWorkingContent
    start_date?: string
    end_date?: string
}

export interface UpdateProtocolRequest {
    title?: string
    working_content?: ProtocolWorkingContent
    start_date?: string
    end_date?: string
    /** R30-B: optimistic lock。從 query 結果取當前 version 回送；
     * 後端命中 0 row → 409 Conflict。omit → 跳過版本檢查（向後相容） */
    version?: number
}

export interface ChangeStatusRequest {
    to_status: ProtocolStatus
    remark?: string
    /** 審查委員 ID 列表（當目標狀態為 UNDER_REVIEW 時必填 2-3 位） */
    reviewer_ids?: string[]
    /** 獸醫師 ID（當目標狀態為 VET_REVIEW 時可選，未設定則使用預設獸醫） */
    vet_id?: string
}

export interface CreateCommentRequest {
    protocol_version_id: string
    content: string
}

export interface ReplyCommentRequest {
    parent_comment_id: string
    content: string
}

export interface AssignReviewerRequest {
    protocol_id: string
    reviewer_id: string
}

export interface AssignCoEditorRequest {
    protocol_id: string
    user_id: string
}

export interface CoEditorAssignmentResponse {
    user_id: string
    protocol_id: string
    role_in_protocol: string
    granted_at: string
    granted_by?: string
    user_name: string
    user_email: string
    granted_by_name?: string
}

export interface ReviewAssignmentResponse extends ReviewAssignment {
    reviewer_name: string
    reviewer_email: string
    assigned_by_name: string
}

// 附件
export interface ProtocolAttachment {
    id: string
    protocol_id?: string
    protocol_version_id?: string
    file_name: string
    file_path: string
    file_size: number
    mime_type: string
    uploaded_by: string
    uploaded_by_name?: string
    created_at: string
}

// 活動紀錄
export type ProtocolActivityType =
    // 生命週期
    | 'CREATED'
    | 'UPDATED'
    | 'SUBMITTED'
    | 'RESUBMITTED'
    | 'APPROVED'
    | 'APPROVED_WITH_CONDITIONS'
    | 'CLOSED'
    | 'REJECTED'
    | 'SUSPENDED'
    | 'DELETED'
    // 審查流程
    | 'STATUS_CHANGED'
    | 'REVIEWER_ASSIGNED'
    | 'VET_ASSIGNED'
    | 'COEDITOR_ASSIGNED'
    | 'COEDITOR_REMOVED'
    // 審查意見
    | 'COMMENT_ADDED'
    | 'COMMENT_REPLIED'
    | 'COMMENT_RESOLVED'
    // 附件
    | 'ATTACHMENT_UPLOADED'
    | 'ATTACHMENT_DELETED'
    // 版本
    | 'VERSION_CREATED'
    | 'VERSION_RECOVERED'
    // 修正案
    | 'AMENDMENT_CREATED'
    | 'AMENDMENT_SUBMITTED'
    // 動物管理
    | 'ANIMAL_ASSIGNED'
    | 'ANIMAL_UNASSIGNED'

export interface ProtocolActivity {
    id: string
    protocol_id: string
    activity_type: ProtocolActivityType
    activity_type_display: string
    actor_id: string
    actor_name: string
    actor_email: string
    from_value?: string
    to_value?: string
    target_entity_type?: string
    target_entity_id?: string
    target_entity_name?: string
    remark?: string
    extra_data?: Record<string, unknown>
    created_at: string
}
