/**
 * 共用型別定義
 *
 * 包含跨模組共用的基礎型別，如 User、分頁、通用回應等。
 * 不應包含特定模組的業務型別（參見 protocol.ts、hr.ts、erp.ts）。
 */

/** 分頁請求參數 */
export interface PaginationQuery {
    page?: number
    per_page?: number
    sort_by?: string
    sort_order?: 'asc' | 'desc'
}

/** 分頁回應結構 */
export interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

/** 通用 API 錯誤回應 */
export interface ApiErrorResponse {
    error: {
        code?: string
        message: string
        details?: Record<string, string[]>
    }
}

/** 通用 ID + 名稱配對 */
export interface IdNamePair {
    id: string
    name: string
}

/** 帶有審計欄位的基礎型別 */
export interface Auditable {
    created_at: string
    updated_at: string
    created_by?: string
    updated_by?: string
}

/** 通用狀態選項 */
export interface StatusOption {
    value: string
    label: string
    color?: string
    icon?: string
}

/** 通知型別 */
export interface Notification {
    id: string
    user_id: string
    type: string
    title: string
    message: string
    link?: string
    is_read: boolean
    created_at: string
}

/** 檔案上傳回應 */
export interface UploadResponse {
    id: string
    filename: string
    original_name: string
    content_type: string
    size: number
    url: string
}

/** 使用者訓練資訊 */
export interface UserTraining {
    code: string
    certificate_no?: string
    received_date?: string
}

/** 使用者型別 */
export interface User {
    id: string
    email: string
    display_name: string
    phone?: string
    organization?: string
    is_active: boolean
    roles: string[]
    permissions: string[]
    must_change_password?: boolean
    entry_date?: string | null
    position?: string | null
    aup_roles?: string[]
    years_experience?: number
    trainings?: UserTraining[]
}

/** 登入回應 */
export interface LoginResponse {
    access_token: string
    refresh_token: string
    token_type: string
    expires_in: number
    user: User
}

/** 角色定義 */
export interface Role {
    id: string
    name: string
    code: string
    description?: string
    permissions: string[]
    is_active: boolean
}
