/**
 * 共用型別定義
 *
 * 包含跨模組共用的基礎型別，如分頁、通用回應等。
 * 不應包含特定模組的業務型別（參見 auth.ts、erp.ts、animal.ts 等）。
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

/** 通知型別（舊版，保留向後相容） */
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
