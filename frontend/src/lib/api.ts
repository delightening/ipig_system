import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios'
import { useAuthStore } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'

const api = axios.create({
  baseURL: '/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  // HttpOnly Cookie 自動隨請求傳送（SEC-02）
  withCredentials: true,
})

// ============================================
// SEC-24: CSRF Token 自動附加
// ============================================

/** 從 document.cookie 中讀取指定名稱的 cookie 值 */
function getCookie(name: string): string | null {
  const match = document.cookie.match(new RegExp(`(?:^|;\\s*)${name}=([^;]*)`))
  return match ? decodeURIComponent(match[1]) : null
}

// Request interceptor：自動將 csrf_token Cookie 值加到 X-CSRF-Token header
api.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  // FormData 上傳時移除 Content-Type，讓瀏覽器自動設定 multipart/form-data + boundary
  if (config.data instanceof FormData) {
    config.headers.delete('Content-Type')
  }

  const method = (config.method || '').toUpperCase()
  // 只有 POST/PUT/DELETE/PATCH 需要 CSRF token
  if (['POST', 'PUT', 'DELETE', 'PATCH'].includes(method)) {
    const csrfToken = getCookie('csrf_token')
    if (csrfToken) {
      config.headers.set('X-CSRF-Token', csrfToken)
    }
  }
  return config
})

// ============================================
// SEC-25: Refresh Token Queue（防競態）
// ============================================

let isRefreshing = false
let refreshSubscribers: Array<(success: boolean) => void> = []

/** 訂閱 refresh 結果 */
function subscribeTokenRefresh(callback: (success: boolean) => void) {
  refreshSubscribers.push(callback)
}

/** 通知所有等待中的請求 refresh 結果 */
function onRefreshResolved(success: boolean) {
  refreshSubscribers.forEach(cb => cb(success))
  refreshSubscribers = []
}

// 防重複登出鎖：避免多個並行 401 請求同時觸發 logout
let isLoggingOut = false

// Response interceptor - handle errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as typeof error.config & { _retry?: boolean; _503RetryCount?: number }

    // 如果已經在登出流程中，直接拒絕所有 401，不再重試
    if (isLoggingOut) {
      return Promise.reject(error)
    }

    // 503 暫時性錯誤：依 Retry-After 重試（最多 2 次）
    const max503Retries = 2
    const retryCount = originalRequest?._503RetryCount ?? 0
    if (error.response?.status === 503 && retryCount < max503Retries && originalRequest) {
      const retryAfter = error.response?.headers?.['retry-after'] ?? error.response?.headers?.['Retry-After']
      const delayMs = retryAfter ? Math.min(parseInt(String(retryAfter), 10) * 1000, 3000) : 1000
      originalRequest._503RetryCount = retryCount + 1
      await new Promise((r) => setTimeout(r, delayMs))
      return api(originalRequest)
    }

    // If 401 and not already retrying, try to refresh token
    if (error.response?.status === 401 && !originalRequest?._retry) {
      originalRequest._retry = true

      // SEC-25: 如果已經有 refresh 在進行中，等待其結果
      if (isRefreshing) {
        return new Promise((resolve, reject) => {
          subscribeTokenRefresh((success: boolean) => {
            if (success && originalRequest) {
              resolve(api(originalRequest))
            } else {
              reject(error)
            }
          })
        })
      }

      isRefreshing = true

      try {
        await api.post('/auth/refresh')

        // Reset session expiry timer after successful refresh
        try {
          const { useAuthStore } = await import('@/stores/auth')
          useAuthStore.getState().sessionExpiresAt = Date.now() + 6 * 60 * 60 * 1000
        } catch { /* non-critical */ }

        isRefreshing = false
        onRefreshResolved(true)

        if (originalRequest) {
          return api(originalRequest)
        }
      } catch {
        isRefreshing = false
        onRefreshResolved(false)

        // Refresh 也失敗 → 清除 auth 狀態，讓 React Router 自然導向 /login
        // 使用鎖避免多個並行請求重複觸發
        if (!isLoggingOut) {
          isLoggingOut = true
          try {
            // 使用 getState() 在非 React 上下文存取 store
            const store = useAuthStore.getState()
            // 只清 state，不再呼叫後端 logout（token 已失效）
            store.clearAuth()
          } finally {
            // 延遲重置鎖，讓所有排隊的 401 都被靜默拒絕
            setTimeout(() => { isLoggingOut = false }, 1000)
          }
        }
      }
    }

    // Non-401 errors: show global toast for server errors and network issues
    // Skip toast if the request opted-in to silent error handling
    const silent = originalRequest?._silentError
    if (!silent) {
      if (error.response) {
        const status = error.response.status
        if (status >= 500) {
          toast({ variant: 'destructive', title: '伺服器錯誤，請稍後再試' })
        }
      } else if (error.code === 'ECONNABORTED') {
        toast({ variant: 'destructive', title: '請求逾時，請檢查網路連線' })
      } else if (!error.response) {
        toast({ variant: 'destructive', title: '無法連線至伺服器' })
      }
    }

    return Promise.reject(error)
  }
)

// Utility function to format ear tag: if it's a number < 100, pad to 3 digits
export function formatEarTag(earTag: string): string {
  if (!earTag) return earTag
  // Check if it's a pure number
  if (/^\d+$/.test(earTag)) {
    const num = parseInt(earTag, 10)
    if (num < 100) {
      return earTag.padStart(3, '0')
    }
  }
  return earTag
}

export default api

// ============================================
// 型別與常數 re-export（向後相容）
// ============================================
// 所有型別已拆分至 @/types/ 目錄，此處 re-export 以維持既有 import 路徑
export type * from '@/types/auth'
export type * from '@/types/erp'
export type * from '@/types/animal'
export type * from '@/types/aup'
export type * from '@/types/report'
export type * from '@/types/audit'
export type * from '@/types/notification'
export type * from '@/types/amendment'
export type * from '@/types/upload'
export type { ProtocolWorkingContent } from '@/types/protocol'

// 常值 re-export
export {
  storageLocationTypeNames,
} from '@/types/erp'
export {
  animalStatusNames, allAnimalStatusNames, animalBreedNames, animalGenderNames, recordTypeNames,
  CORRECTABLE_FIELDS,
} from '@/types/animal'
export {
  protocolStatusNames,
} from '@/types/aup'
export {
  notificationTypeNames,
} from '@/types/notification'
export {
  amendmentStatusNames, amendmentStatusColors, amendmentTypeNames,
  AMENDMENT_CHANGE_ITEM_OPTIONS,
} from '@/types/amendment'

// ============================================
// SEC-33：敏感操作二級認證
// ============================================
/** 以密碼換取短期 reauth token，供敏感操作 API 帶入 X-Reauth-Token header */
export async function confirmPassword(password: string): Promise<{ reauth_token: string; expires_in: number }> {
  const { data } = await api.post<{ reauth_token: string; expires_in: number }>('/auth/confirm-password', { password })
  return data
}

// ============================================
// API 函數
// ============================================

import type {
  BloodTestListItem, AnimalBloodTestWithItems, CreateBloodTestRequest,
  UpdateBloodTestRequest, BloodTestTemplate, CreateBloodTestTemplateRequest,
  UpdateBloodTestTemplateRequest, BloodTestPanel, CreateBloodTestPanelRequest,
  UpdateBloodTestPanelRequest, UpdateBloodTestPanelItemsRequest,
  ProtocolActivity, BloodTestAnalysisRow,
} from '@/types'

// 血液檢查 API 函數
export const bloodTestApi = {
  listByAnimal: (animalId: string) =>
    api.get<BloodTestListItem[]>(`/animals/${animalId}/blood-tests`),
  getById: (id: string) =>
    api.get<AnimalBloodTestWithItems>(`/blood-tests/${id}`),
  create: (animalId: string, data: CreateBloodTestRequest) =>
    api.post<AnimalBloodTestWithItems>(`/animals/${animalId}/blood-tests`, data),
  update: (id: string, data: UpdateBloodTestRequest) =>
    api.put<AnimalBloodTestWithItems>(`/blood-tests/${id}`, data),
  delete: (id: string, reason: string) =>
    api.delete(`/blood-tests/${id}`, { data: { reason } }),
}

// 血液檢查項目模板 API 函數
export const bloodTestTemplateApi = {
  list: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates'),
  listAll: () =>
    api.get<BloodTestTemplate[]>('/blood-test-templates/all'),
  create: (data: CreateBloodTestTemplateRequest) =>
    api.post<BloodTestTemplate>('/blood-test-templates', data),
  update: (id: string, data: UpdateBloodTestTemplateRequest) =>
    api.put<BloodTestTemplate>(`/blood-test-templates/${id}`, data),
  delete: (id: string) =>
    api.delete(`/blood-test-templates/${id}`),
}

// 血液檢查組合 API 函數
export const bloodTestPanelApi = {
  list: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels'),
  listAll: () =>
    api.get<BloodTestPanel[]>('/blood-test-panels/all'),
  create: (data: CreateBloodTestPanelRequest) =>
    api.post<BloodTestPanel>('/blood-test-panels', data),
  update: (id: string, data: UpdateBloodTestPanelRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}`, data),
  updateItems: (id: string, data: UpdateBloodTestPanelItemsRequest) =>
    api.put<BloodTestPanel>(`/blood-test-panels/${id}/items`, data),
  delete: (id: string) =>
    api.delete(`/blood-test-panels/${id}`),
}

// 血液檢查結果分析 API 函數
export const bloodTestAnalysisApi = {
  query: (params: string) =>
    api.get<BloodTestAnalysisRow[]>(`/reports/blood-test-analysis?${params}`),
}

// 計畫書活動紀錄 API
export const getProtocolActivities = async (id: string): Promise<ProtocolActivity[]> => {
  const response = await api.get<ProtocolActivity[]>(`/protocols/${id}/activities`)
  return response.data
}

// 通知路由管理 API
import type {
  NotificationRouting, CreateNotificationRoutingRequest,
  UpdateNotificationRoutingRequest, EventTypeCategory, RoleInfo,
} from '@/types/notification'

export const notificationRoutingApi = {
  list: () =>
    api.get<NotificationRouting[]>('/admin/notification-routing'),
  create: (data: CreateNotificationRoutingRequest) =>
    api.post<NotificationRouting>('/admin/notification-routing', data),
  update: (id: string, data: UpdateNotificationRoutingRequest) =>
    api.put<NotificationRouting>(`/admin/notification-routing/${id}`, data),
  delete: (id: string) =>
    api.delete(`/admin/notification-routing/${id}`),
  getEventTypes: () =>
    api.get<EventTypeCategory[]>('/admin/notification-routing/event-types'),
  getRoles: () =>
    api.get<RoleInfo[]>('/admin/notification-routing/roles'),
}

// 通知路由常數 re-export
export {
  eventTypeNames, channelNames,
} from '@/types/notification'
export {
  transferStatusNames,
} from '@/types/animal'

// ============================================
// 轉讓流程 API
// ============================================

import type {
  AnimalTransfer, TransferVetEvaluation,
  CreateTransferRequest, VetEvaluateTransferRequest,
  AssignTransferPlanRequest, RejectTransferRequest,
} from '@/types/animal'

export const transferApi = {
  getDataBoundary: (animalId: string) =>
    api.get<{ boundary: string | null }>(`/animals/${animalId}/data-boundary`),
  list: (animalId: string) =>
    api.get<AnimalTransfer[]>(`/animals/${animalId}/transfers`),
  get: (transferId: string) =>
    api.get<AnimalTransfer>(`/transfers/${transferId}`),
  initiate: (animalId: string, data: CreateTransferRequest) =>
    api.post<AnimalTransfer>(`/animals/${animalId}/transfers`, data),
  vetEvaluate: (transferId: string, data: VetEvaluateTransferRequest) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/vet-evaluate`, data),
  getVetEvaluation: (transferId: string) =>
    api.get<TransferVetEvaluation | null>(`/transfers/${transferId}/vet-evaluation`),
  assignPlan: (transferId: string, data: AssignTransferPlanRequest) =>
    api.put<AnimalTransfer>(`/transfers/${transferId}/assign-plan`, data),
  approve: (transferId: string) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/approve`),
  complete: (transferId: string) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/complete`),
  reject: (transferId: string, data: RejectTransferRequest) =>
    api.post<AnimalTransfer>(`/transfers/${transferId}/reject`, data),
}

// ============================================
// 動物欄位修正申請 API（耳號、出生日期、性別、品種需 admin 批准）
// ============================================

import type {
  AnimalFieldCorrectionRequest,
  CreateAnimalFieldCorrectionRequest,
  ReviewAnimalFieldCorrectionRequest,
} from '@/types/animal'

export const animalFieldCorrectionApi = {
  listByAnimal: (animalId: string) =>
    api.get<AnimalFieldCorrectionRequest[]>(`/animals/${animalId}/field-corrections`),
  create: (animalId: string, data: CreateAnimalFieldCorrectionRequest) =>
    api.post<{ id: string }>(`/animals/${animalId}/field-corrections`, data),
  listPending: () =>
    api.get<AnimalFieldCorrectionRequest[]>('/admin/animal-field-corrections/pending'),
  review: (requestId: string, data: ReviewAnimalFieldCorrectionRequest) =>
    api.post<{ message: string }>(`/admin/animal-field-corrections/${requestId}/review`, data),
}

// ============================================
// 電子簽章 API（手寫簽名 / 密碼驗證）
// ============================================

export interface SignRecordRequest {
  password?: string
  signature_type?: string
  handwriting_svg?: string
  stroke_data?: object[]
}

export interface SignRecordResponse {
  signature_id: string
  signed_at: string
  is_locked: boolean
}

export interface SignatureInfo {
  id: string
  signature_type: string
  signer_name: string | null
  signed_at: string
  signature_method: string | null
  handwriting_svg: string | null
}

export interface SignatureStatusResponse {
  is_signed: boolean
  is_locked: boolean
  signatures: SignatureInfo[]
}

export const signatureApi = {
  // 犧牲紀錄簽章
  signSacrifice: (sacrificeId: number, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/sacrifice/${sacrificeId}`, data),
  getSacrificeStatus: (sacrificeId: number) =>
    api.get<SignatureStatusResponse>(`/signatures/sacrifice/${sacrificeId}`),

  // 觀察紀錄簽章
  signObservation: (observationId: number, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/observation/${observationId}`, data),

  // 安樂死單據簽章
  signEuthanasia: (orderId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/euthanasia/${orderId}`, data),
  getEuthanasiaStatus: (orderId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/euthanasia/${orderId}`),

  // 轉讓記錄簽章
  signTransfer: (transferId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/transfer/${transferId}`, data),
  getTransferStatus: (transferId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/transfer/${transferId}`),

  // 計劃審查簽章
  signProtocol: (protocolId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/protocol/${protocolId}`, data),
  getProtocolStatus: (protocolId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/protocol/${protocolId}`),
}

// ============================================
// 治療方式藥物選項 API
// ============================================

import type {
  TreatmentDrugOption, CreateTreatmentDrugRequest,
  UpdateTreatmentDrugRequest, ImportFromErpRequest,
} from '@/types/treatment-drug'

export const treatmentDrugApi = {
  /** 列表（僅啟用項目，供一般使用者） */
  list: () =>
    api.get<TreatmentDrugOption[]>('/treatment-drugs'),
  /** 管理員列表（含停用項目、篩選） */
  adminList: (params?: { keyword?: string; category?: string; is_active?: boolean }) =>
    api.get<TreatmentDrugOption[]>('/admin/treatment-drugs', { params }),
  /** 建立 */
  create: (data: CreateTreatmentDrugRequest) =>
    api.post<TreatmentDrugOption>('/admin/treatment-drugs', data),
  /** 更新 */
  update: (id: string, data: UpdateTreatmentDrugRequest) =>
    api.put<TreatmentDrugOption>(`/admin/treatment-drugs/${id}`, data),
  /** 刪除（軟刪除） */
  delete: (id: string) =>
    api.delete(`/admin/treatment-drugs/${id}`),
  /** 從 ERP 匯入 */
  importFromErp: (data: ImportFromErpRequest) =>
    api.post<TreatmentDrugOption[]>('/admin/treatment-drugs/import-erp', data),
}

