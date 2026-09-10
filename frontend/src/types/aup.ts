/**
 * AUP 計畫書型別（Protocol、審查、活動）
 *
 * 注意：ProtocolWorkingContent 已定義在 ./protocol.ts
 */

import type { PendingOwner } from './pendingOwner'
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
    /** 原始申請時間（歷史匯入用） */
    submitted_at?: string
    /** 原始通過時間（歷史匯入用） */
    approved_at?: string
    created_by: string
    created_at: string
    updated_at: string
    /** R30-B: optimistic lock 版本號，前端送 PUT 時帶回防 lost update */
    version: number
    /** 申請編號（例 APIG-103001），與 iacuc_no（核准編號）區分（匯入補登） */
    application_no?: string
    /** 匯入補登中：APPROVED 但允許編輯 working_content（import P1） */
    import_pending?: boolean
    /** 原計劃書版本號文字（補登，例 v2.1） */
    original_version_label?: string
    /** 計劃負責人 / Study Director（本公司員工 EXPERIMENT_STAFF） */
    study_director_user_id?: string
    /** 匯入計劃建立時間；非 null = 補登匯入計劃（永久標記，P6 補登歷史變更 gate） */
    imported_at?: string | null
    /** 計畫書表單版本鍵（C/D/E/F…）；驅動版本名冊 manifest 渲染；null=最新版 */
    source_form_version?: string | null
    /** PI 是否為外部人員（無系統帳號）；true 時 pi_user_id 只是建立者/匯入者的
     *  佔位值，不代表任何人是 PI。驅動 PI 代理授權卡片是否顯示。 */
    pi_is_external?: boolean
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
    /** 匯入計劃標記（非 null = 補登匯入計劃）；admin 可刪除此類計劃 */
    imported_at?: string | null
    /** 當前使用者是否可編輯此計畫（PI / SD / admin）。後端列表計算，供按鈕 gating。 */
    can_edit?: boolean
    /** 這件現在卡在誰手上；目前僅行政受理 / 預審那幾關有值 */
    pending_owner?: PendingOwner
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
    /** 系統內獸醫師 FK；院外獸醫師（補登）為 null */
    vet_id: string | null
    /** 院外獸醫師姓名（補登，vet_id 為 null 時） */
    vet_name?: string
    decision?: string | null
    decision_remark?: string | null
    review_form?: VetReviewFormData
}

export interface ProtocolResponse {
    protocol: Protocol
    pi_name?: string
    pi_email?: string
    pi_organization?: string
    /** 計劃負責人（Study Director，公司內部）顯示名 */
    sd_name?: string
    /** 建立者 / 匯入者顯示名（與 PI 客人區分） */
    created_by_name?: string
    status_display: string
    vet_review?: VetReviewAssignment
    /** 後端權威：當前使用者是否可編輯此計畫（admin / PI / SD / 補登管理者） */
    can_edit?: boolean
    /** 後端權威：當前使用者是否可建立/更新/提交此計畫的變更申請（admin / PI，不含 SD） */
    can_write_amendment?: boolean
    /** 目前生效中的 PI 代理授權（外部 PI 尚未開通帳號前，由 SD 核准的代簽人）；
     *  `pi_is_external=false` 或尚無生效授權時為 null。 */
    pi_delegate?: PiDelegateInfo | null
    /** 當前使用者是否就是 `pi_delegate` 本人；供顯示「以代理人身分操作」徽章。 */
    is_pi_delegate?: boolean
}

/** PI 代理授權（`protocol_pi_delegates`）唯讀顯示用投影。 */
export interface PiDelegateInfo {
    id: string
    delegate_user_id: string
    delegate_name: string
    authorized_by: string
    authorized_by_name: string
    authorized_at: string
    /** 授權自動失效時點；null = 不設期限。過期後代理權自動收回，不需手動撤銷。 */
    expires_at?: string | null
    reason?: string | null
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
    /** 系統內審查者 FK；院外審查者（補登）為 null */
    reviewer_id: string | null
    /** 院外審查者姓名（補登，reviewer_id 為 null 時） */
    reviewer_name?: string
    content: string
    /**
     * 意見類型。`COMMENT` = 一般意見，申請人需要回覆；
     * `NO_OBJECTION` = 無意見／無異議，申請人不需要回覆。
     *
     * ⚠️ 兩者都算「已發表意見」——審查委員選「無意見」仍然滿足後端
     * 「每位被指派的委員都必須發表意見才能核准」的閘門條件。
     *
     * 選填是為了相容舊資料；缺漏時前端一律當作一般意見（保守側）。
     */
    comment_type?: 'COMMENT' | 'NO_OBJECTION'
    is_resolved: boolean
    resolved_by?: string
    resolved_at?: string
    /** 審查階段（補登委員二審加 FINAL_REVIEW / FINAL） */
    review_stage?: 'PRE_REVIEW' | 'VET_REVIEW' | 'UNDER_REVIEW' | 'FINAL_REVIEW' | 'FINAL'
    created_at: string
    updated_at: string
}

export interface ReviewCommentResponse extends ReviewComment {
    reviewer_name: string
    reviewer_email?: string
    parent_comment_id?: string
    replied_by?: string
    replied_by_name?: string
    replied_by_email?: string
    /** 對應計畫書項次（如 4.1.2，補登審查文件填寫） */
    section_no?: string | null
}

// 審查請求
export interface CreateProtocolRequest {
    title: string
    pi_user_id?: string
    working_content?: ProtocolWorkingContent
    start_date?: string
    end_date?: string
    /** 計劃負責人（SD，內部 EXPERIMENT_STAFF）。客戶/PI 留空，由執秘事後指派 */
    study_director_user_id?: string
}

export interface UpdateProtocolRequest {
    title?: string
    working_content?: ProtocolWorkingContent
    start_date?: string
    end_date?: string
    /** 計劃負責人（SD）變更。僅執秘/admin 可改他人；GLP 已指派後鎖定 */
    study_director_user_id?: string
    /** R30-B: optimistic lock。從 query 結果取當前 version 回送；
     * 後端命中 0 row → 409 Conflict。omit → 跳過版本檢查（向後相容） */
    version?: number
    /** 版本名冊：重選版本 / 升級最新版時更新表單版本鍵（C/D/E/F…）；omit → 不變更 */
    source_form_version?: string
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
