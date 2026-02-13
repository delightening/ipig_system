/**
 * 統一表單驗證模組
 *
 * 提供各模組共用的 Zod Schema 與驗證工具，
 * 搭配 react-hook-form + @hookform/resolvers 使用。
 *
 * 使用方式：
 *   import { loginSchema, partnerSchema, getApiErrorMessage } from '@/lib/validation'
 *   const form = useForm({ resolver: zodResolver(loginSchema) })
 */

import { z } from 'zod'
import { AxiosError } from 'axios'

// ============================================
// 共用驗證規則
// ============================================

/** 必填字串（至少 1 字元） */
export const requiredString = (fieldName: string) =>
    z.string().min(1, `${fieldName} 為必填欄位`)

/** Email 欄位 */
export const emailField = z.string().email('請輸入有效的電子郵件')

/** 密碼欄位（至少 6 字元） */
export const passwordField = z.string().min(6, '密碼至少 6 個字元')

/** 可選字串（空字串視為 undefined） */
export const optionalString = z.string().optional().transform(v => v || undefined)

/** 正數 */
export const positiveNumber = (fieldName: string) =>
    z.number().positive(`${fieldName} 必須為正數`)

/** UUID 格式 */
export const uuidField = z.string().uuid('無效的 ID 格式')

// ============================================
// 模組 Schema
// ============================================

/** 登入表單 */
export const loginSchema = z.object({
    email: emailField,
    password: z.string().min(1, '請輸入密碼'),
})
export type LoginFormData = z.infer<typeof loginSchema>

/** 忘記密碼表單 */
export const forgotPasswordSchema = z.object({
    email: emailField,
})
export type ForgotPasswordFormData = z.infer<typeof forgotPasswordSchema>

/** 變更密碼表單 */
export const changePasswordSchema = z.object({
    current_password: z.string().min(1, '請輸入目前密碼'),
    new_password: passwordField,
    confirm_password: z.string().min(1, '請確認新密碼'),
}).refine(data => data.new_password === data.confirm_password, {
    message: '新密碼與確認密碼不一致',
    path: ['confirm_password'],
})
export type ChangePasswordFormData = z.infer<typeof changePasswordSchema>

/** 合作夥伴（供應商/客戶）表單 */
export const partnerSchema = z.object({
    partner_type: z.enum(['supplier', 'customer'], { required_error: '請選擇類型' }),
    name: requiredString('名稱'),
    code: optionalString,
    tax_id: optionalString,
    phone: optionalString,
    email: z.string().email('請輸入有效的電子郵件').optional().or(z.literal('')),
    address: optionalString,
    payment_terms: optionalString,
    supplier_category: optionalString,
})
export type PartnerFormData = z.infer<typeof partnerSchema>

/** 產品表單 */
export const productSchema = z.object({
    name: requiredString('產品名稱'),
    sku: requiredString('SKU'),
    category_id: optionalString,
    base_uom: requiredString('基本單位'),
    spec: optionalString,
    track_batch: z.boolean().default(false),
    track_expiry: z.boolean().default(false),
    safety_stock: optionalString,
    reorder_point: optionalString,
})
export type ProductFormData = z.infer<typeof productSchema>

/** 使用者建立表單 */
export const createUserSchema = z.object({
    email: emailField,
    password: passwordField,
    display_name: requiredString('顯示名稱'),
    phone: optionalString,
    organization: optionalString,
    role_ids: z.array(z.string()).min(1, '至少選擇一個角色'),
})
export type CreateUserFormData = z.infer<typeof createUserSchema>

/** Amendment（計畫變更）表單 */
export const amendmentSchema = z.object({
    protocol_id: uuidField,
    title: z.string().min(1, '標題為必填').max(200, '標題不得超過 200 字元'),
    description: optionalString,
    change_items: z.array(z.string()).optional(),
    changes_content: z.any().optional(),
})
export type AmendmentFormData = z.infer<typeof amendmentSchema>

// ============================================
// API 錯誤訊息擷取工具
// ============================================

/**
 * 從 Axios 錯誤中取得使用者友善的錯誤訊息
 *
 * 後端回傳格式可能為：
 * - { error: { message: "..." } }
 * - { message: "..." }
 * - 純文字字串
 */
export function getApiErrorMessage(error: unknown, fallback = '操作失敗，請稍後再試'): string {
    if (error instanceof AxiosError) {
        const data = error.response?.data
        if (typeof data === 'string') return data
        if (data?.error?.message) return data.error.message
        if (data?.message) return data.message
        if (error.response?.status === 403) return '權限不足，無法執行此操作'
        if (error.response?.status === 404) return '找不到資源'
        if (error.response?.status === 409) return '資料衝突，請重新整理後再試'
        if (error.response?.status === 422) return '提交的資料格式不正確'
    }
    if (error instanceof Error) return error.message
    return fallback
}
