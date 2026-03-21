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

/** 密碼欄位（至少 10 字元） */
export const passwordField = z.string().min(10, '密碼至少 10 個字元')

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
    partner_type: z.enum(['supplier', 'customer'], { message: '請選擇類型' }),
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
// 可選欄位驗證（來自 validations.ts 合併）
// ============================================

/** 統一編號（8 位數字或空字串） */
export const taxIdSchema = z
  .string()
  .refine(
    (v) => v === '' || /^\d{8}$/.test(v),
    { message: 'validation.taxId' },
  )

/** 電話號碼（9-10 位數字或空字串） */
export const phoneSchema = z
  .string()
  .refine(
    (v) => v === '' || /^\d{9,10}$/.test(v),
    { message: 'validation.phone' },
  )

/** 可選 Email（空字串或有效 Email） */
export const emailOptionalSchema = z
  .string()
  .refine(
    (v) => v === '' || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v),
    { message: 'validation.email' },
  )

/** 非空字串（帶自訂 message key） */
export const nonEmptyString = (messageKey = 'validation.required') =>
  z.string().min(1, { message: messageKey })

/** 合作夥伴表單 schema（舊版，含 type 欄位） */
export const partnerFormSchema = z.object({
  code: nonEmptyString('validation.required'),
  name: nonEmptyString('validation.required'),
  type: z.enum(['customer', 'vendor', 'both']),
  tax_id: taxIdSchema.optional().default(''),
  contact_person: z.string().optional().default(''),
  phone: phoneSchema.optional().default(''),
  email: emailOptionalSchema.optional().default(''),
  address: z.string().optional().default(''),
  bank_account: z.string().optional().default(''),
  notes: z.string().optional().default(''),
})

export type PartnerFormSchemaData = z.infer<typeof partnerFormSchema>

/** 倉庫表單 schema */
export const warehouseFormSchema = z.object({
  name: nonEmptyString('validation.required'),
  code: z.string().optional().default(''),
  address: z.string().optional().default(''),
  description: z.string().optional().default(''),
  is_active: z.boolean().default(true),
})

export type WarehouseFormData = z.infer<typeof warehouseFormSchema>

/** 動物表單 schema */
export const animalFormSchema = z.object({
  ear_tag: nonEmptyString('validation.required'),
  breed: z.enum(['minipig', 'miniature', 'white', 'LYD', 'other']),
  breed_other: z.string().optional().default(''),
  gender: z.enum(['male', 'female']),
  source_id: z.string().optional().default(''),
  pen_location: nonEmptyString('validation.required'),
  entry_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, { message: 'validation.dateFormat' }),
  birth_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, { message: 'validation.dateFormat' }),
  entry_weight: z
    .string()
    .optional()
    .default('')
    .refine(
      (v) => v === '' || (!isNaN(parseFloat(v)) && parseFloat(v) > 0),
      { message: 'validation.positiveNumber' },
    ),
  pre_experiment_code: nonEmptyString('validation.required'),
  remark: z.string().optional().default(''),
})

export type AnimalFormData = z.infer<typeof animalFormSchema>

/** 從 ZodError 取得第一個錯誤訊息 */
export function getFirstZodError(error: z.ZodError): string {
  const first = error.issues[0]
  return (first && 'message' in first ? String(first.message) : undefined) ?? 'validation.unknown'
}

// ============================================
// API 錯誤訊息擷取工具
// ============================================

/**
 * 從 Axios 錯誤中取得使用者友善的錯誤訊息
 *
 * 後端回傳格式：
 *   { error: { message: "...", code: 400, blocking: true } }
 *
 * 也支援：
 *   { message: "..." } 或純文字字串
 */
export function getApiErrorMessage(error: unknown, fallback = '操作失敗，請稍後再試'): string {
    if (error instanceof AxiosError) {
        // 網路錯誤（無回應）
        if (!error.response) {
            if (error.code === 'ECONNABORTED' || error.code === 'ETIMEDOUT') {
                return '請求逾時，請檢查網路連線後再試'
            }
            if (error.code === 'ERR_NETWORK') {
                return '無法連線至伺服器，請確認網路狀態'
            }
            return '網路連線異常，請稍後再試'
        }

        // 有回應時，優先使用後端回傳的訊息
        const data = error.response.data
        if (typeof data === 'string' && data.length > 0 && data.length < 200) return data
        if (data?.error?.message) return data.error.message
        if (data?.message) return data.message

        // 依 HTTP 狀態碼提供預設訊息
        const statusMessages: Record<number, string> = {
            400: '請求格式有誤，請檢查輸入內容',
            401: '登入已過期，請重新登入',
            403: '權限不足，無法執行此操作',
            404: '找不到相關資料',
            409: '資料衝突，請重新整理後再試',
            413: '上傳的檔案太大，請縮小檔案後重試',
            422: '提交的資料不符合格式要求',
            429: '操作過於頻繁，請稍後再試',
            500: '伺服器發生錯誤，請稍後再試',
            502: '伺服器暫時無法服務，請稍後再試',
            503: '系統維護中，請稍後再試',
        }
        const status = error.response.status
        if (statusMessages[status]) return statusMessages[status]
    }
    if (error instanceof Error) return error.message
    return fallback
}
