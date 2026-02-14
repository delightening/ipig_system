/**
 * 認證與使用者型別
 */

// 使用者訓練紀錄
export interface UserTraining {
    code: string
    certificate_no?: string
    received_date?: string
}

// 使用者
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
    // AUP 第 8 節人員資料
    entry_date?: string | null
    position?: string | null
    aup_roles?: string[]
    years_experience?: number
    trainings?: UserTraining[]
}

// 簡易使用者資訊
export interface UserSimple {
    id: string
    email: string
    display_name?: string
}

// 登入回應
export interface LoginResponse {
    access_token: string
    refresh_token: string
    token_type: string
    expires_in: number
    user: User
}

// 角色
export interface Role {
    id: string
    code: string
    name: string
    description?: string
    is_internal: boolean
    is_system: boolean
    is_active: boolean
    permissions: Permission[]
    created_at: string
    updated_at: string
}

// 權限
export interface Permission {
    id: string
    code: string
    name: string
    module?: string
    description?: string
    created_at: string
}

// 請求型別
export interface CreateUserRequest {
    email: string
    password: string
    display_name: string
    role_ids: string[]
}

export interface UpdateUserRequest {
    email?: string
    display_name?: string
    phone?: string
    organization?: string
    is_active?: boolean
    role_ids?: string[]
    // AUP 第 8 節人員資料
    entry_date?: string | null
    position?: string | null
    aup_roles?: string[]
    years_experience?: number
    trainings?: UserTraining[]
}

export interface CreateRoleRequest {
    code: string
    name: string
    permission_ids: string[]
}

export interface UpdateRoleRequest {
    name?: string
    permission_ids?: string[]
}

// 密碼變更
export interface ChangeOwnPasswordRequest {
    current_password: string
    new_password: string
}

export interface ResetPasswordRequest {
    new_password: string
}

// 密碼重設
export interface ForgotPasswordRequest {
    email: string
}

export interface ResetPasswordWithTokenRequest {
    token: string
    new_password: string
}
