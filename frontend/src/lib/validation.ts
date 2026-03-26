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

/** 重設密碼表單（token 來自 URL） */
export const resetPasswordSchema = z.object({
    password: passwordField,
    confirmPassword: z.string().min(1, '請確認新密碼'),
}).refine(data => data.password === data.confirmPassword, {
    message: '兩次輸入的密碼不一致',
    path: ['confirmPassword'],
})
export type ResetPasswordFormData = z.infer<typeof resetPasswordSchema>

/** 個人資料設定表單 */
export const profileSettingsSchema = z.object({
    display_name: requiredString('顯示名稱'),
    phone: z.string(),
    phone_ext: z.string(),
    organization: z.string(),
})
export type ProfileSettingsFormData = z.infer<typeof profileSettingsSchema>

/** 合作夥伴（供應商/客戶）表單（舊版，部分欄位） */
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

/** 合作夥伴表單 Zod schema（完整版，搭配 react-hook-form） */
export const partnerFormZodSchema = z.object({
    partner_type: z.enum(['supplier', 'customer']),
    supplier_category: z.enum(['', 'drug', 'consumable', 'feed', 'equipment']),
    customer_category: z.enum(['', 'internal', 'external', 'research', 'other']),
    code: z.string(),
    name: z.string().min(1, '名稱為必填'),
    tax_id: z.string().refine(v => v === '' || /^\d{8}$/.test(v), '統編必須為 8 碼數字'),
    phone: z.string().refine(v => v === '' || /^\d{9,10}$/.test(v), '電話必須為 9-10 碼數字'),
    phone_ext: z.string(),
    email: z.string().refine(v => v === '' || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v), 'Email 格式不正確'),
    address: z.string(),
})

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
    phone: z.string(),
    organization: z.string(),
    role_ids: z.array(z.string()).min(1, '至少選擇一個角色'),
})
export type CreateUserFormData = z.infer<typeof createUserSchema>

/** 使用者編輯表單 */
export const editUserSchema = z.object({
    email: emailField,
    display_name: requiredString('顯示名稱'),
    entry_date: z.string(),
    trainings: z.array(z.object({
        code: z.string(),
        certificate_no: z.string(),
        received_date: z.string(),
    })),
})
export type EditUserFormData = z.infer<typeof editUserSchema>

/** 管理員重設密碼表單 */
export const adminResetPasswordSchema = z.object({
    reauth_password: z.string().min(1, '請輸入您的登入密碼'),
    new_password: passwordField,
    confirm_password: z.string().min(1, '請確認新密碼'),
}).refine(data => data.new_password === data.confirm_password, {
    message: '新密碼與確認密碼不一致',
    path: ['confirm_password'],
})
export type AdminResetPasswordFormData = z.infer<typeof adminResetPasswordSchema>

/** 動物編輯表單（僅可編輯欄位） */
export const animalEditSchema = z.object({
    status: z.string().min(1, '請選擇狀態'),
    pen_location: z.string(),
    iacuc_no: z.string(),
    experiment_date: z.string(),
    remark: z.string(),
}).refine(
    data => !(data.status === 'in_experiment' && !data.iacuc_no),
    { message: '選擇「實驗中」狀態時，IACUC No. 為必填欄位', path: ['iacuc_no'] },
)
export type AnimalEditFormData = z.infer<typeof animalEditSchema>

/** 應付帳款付款表單 */
export const apPaymentSchema = z.object({
    partner_id: z.string().min(1, '請選擇供應商'),
    payment_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, '請選擇付款日期'),
    amount: z.string().refine(
        v => { const n = parseFloat(v); return !isNaN(n) && n > 0 },
        '請輸入有效金額',
    ),
    reference: z.string(),
})
export type ApPaymentFormData = z.infer<typeof apPaymentSchema>

/** 應收帳款收款表單 */
export const arReceiptSchema = z.object({
    partner_id: z.string().min(1, '請選擇客戶'),
    receipt_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, '請選擇收款日期'),
    amount: z.string().refine(
        v => { const n = parseFloat(v); return !isNaN(n) && n > 0 },
        '請輸入有效金額',
    ),
    reference: z.string(),
})
export type ArReceiptFormData = z.infer<typeof arReceiptSchema>

/** 儲位/結構表單 */
export const storageLocationSchema = z.object({
    name: z.string().min(1, '名稱為必填'),
    location_type: z.enum(['shelf', 'rack', 'zone', 'bin', 'wall', 'door', 'window']),
    capacity: z.string(),
    color: z.string().min(1),
})
export type StorageLocationFormData = z.infer<typeof storageLocationSchema>

/** Amendment（計畫變更）表單 */
export const amendmentSchema = z.object({
    protocol_id: uuidField,
    title: z.string().min(1, '標題為必填').max(200, '標題不得超過 200 字元'),
    description: optionalString,
    change_items: z.array(z.string()).optional(),
    changes_content: z.record(z.string(), z.unknown()).optional(),
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
  code: z.string(),
  address: z.string(),
  description: z.string(),
  is_active: z.boolean(),
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

/** 血液檢查模板表單 schema */
export const bloodTestTemplateFormSchema = z.object({
  code: nonEmptyString('validation.required'),
  name: nonEmptyString('validation.required'),
  default_unit: z.string(),
  reference_range: z.string(),
  default_price: z.number().min(0),
  sort_order: z.number().int(),
  panel_id: z.string().optional(),
})

export type BloodTestTemplateFormData = z.infer<typeof bloodTestTemplateFormSchema>

/** 血液檢查分類表單 schema */
export const bloodTestPanelFormSchema = z.object({
  key: nonEmptyString('validation.required'),
  name: nonEmptyString('validation.required'),
  icon: z.string(),
  sort_order: z.number().int(),
})

export type BloodTestPanelFormData = z.infer<typeof bloodTestPanelFormSchema>

/** 血液檢查常用組合表單 schema */
export const bloodTestPresetFormSchema = z.object({
  name: nonEmptyString('validation.required'),
  icon: z.string(),
  panel_keys: z.array(z.string()),
  sort_order: z.number().int(),
})

export type BloodTestPresetFormData = z.infer<typeof bloodTestPresetFormSchema>

/** 動物來源表單 schema */
export const animalSourceFormSchema = z.object({
  code: nonEmptyString('validation.required'),
  name: nonEmptyString('validation.required'),
  address: z.string(),
  contact: z.string(),
  phone: z.string(),
  phone_ext: z.string(),
  is_active: z.boolean(),
  sort_order: z.number().int(),
})

export type AnimalSourceFormData = z.infer<typeof animalSourceFormSchema>

// ============================================
// HR 模組 Schema
// ============================================

/** 請假申請表單 */
export const leaveRequestSchema = z.object({
  leaveType: requiredString('假別'),
  startDate: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, '請選擇開始日期'),
  endDate: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, '請選擇結束日期'),
  totalHours: z.string().refine(
    (v) => { const n = parseFloat(v); return !isNaN(n) && n >= 0.5 },
    '時數至少 0.5 小時'
  ),
  reason: z.string(),
  proxyUserId: z.string(),
  supportingImages: z.array(z.string()),
}).refine(
  (data) => new Date(data.endDate) >= new Date(data.startDate),
  { message: '結束日期不得早於開始日期', path: ['endDate'] }
)
export type LeaveRequestFormData = z.infer<typeof leaveRequestSchema>

/** 加班申請表單 */
export const overtimeRequestSchema = z.object({
  overtimeDate: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, '請選擇加班日期'),
  startTime: z.string().regex(/^\d{2}:\d{2}$/, '請選擇開始時間'),
  endTime: z.string().regex(/^\d{2}:\d{2}$/, '請選擇結束時間'),
  overtimeType: z.string(),
  reason: requiredString('加班事由'),
}).refine(
  (data) => data.endTime > data.startTime,
  { message: '結束時間必須晚於開始時間（不支援跨午夜）', path: ['endTime'] }
)
export type OvertimeRequestFormData = z.infer<typeof overtimeRequestSchema>


/** 特休額度建立表單 */
export const annualLeaveEntitlementSchema = z.object({
    userId: requiredString('員工'),
    year: z.number().int().min(2020, '年度不得小於 2020').max(2100, '年度不得大於 2100'),
    days: z.number({ error: '請輸入特休天數' }).positive('特休天數必須為正數'),
    hireDate: z.string(),
    notes: z.string(),
})
export type AnnualLeaveEntitlementFormData = z.infer<typeof annualLeaveEntitlementSchema>

/** AI API Key 建立表單 */
export const createAiKeySchema = z.object({
    name: z.string().min(1, '名稱為必填'),
    scopes: z.array(z.string()).min(1, '至少選擇一項權限'),
    rateLimit: z.number({ error: '請輸入速率限制' }).int().min(1, '最小為 1').max(600, '最大為 600'),
    expiresInDays: z.string(),
})
export type CreateAiKeyFormData = z.infer<typeof createAiKeySchema>

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
