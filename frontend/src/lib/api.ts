import axios, { AxiosError } from 'axios'

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
  // HttpOnly Cookie 自動隨請求傳送（SEC-02）
  withCredentials: true,
})

// Response interceptor - handle errors and token refresh
api.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as typeof error.config & { _retry?: boolean }

    // If 401 and not already retrying, try to refresh token
    if (error.response?.status === 401 && !originalRequest?._retry) {
      originalRequest._retry = true

      try {
        // refresh_token Cookie 自動傳送，不需手動附帶
        await api.post('/auth/refresh')

        if (originalRequest) {
          return api(originalRequest)
        }
      } catch {
        // Refresh failed, redirect to login (only once)
        if (!sessionStorage.getItem('auth_redirecting')) {
          sessionStorage.setItem('auth_redirecting', 'true')
          window.location.href = '/login'
        }
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
  pigStatusNames, allPigStatusNames, pigBreedNames, pigGenderNames, recordTypeNames,
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
// API 函數
// ============================================

import type {
  BloodTestListItem, PigBloodTestWithItems, CreateBloodTestRequest,
  UpdateBloodTestRequest, BloodTestTemplate, CreateBloodTestTemplateRequest,
  UpdateBloodTestTemplateRequest, BloodTestPanel, CreateBloodTestPanelRequest,
  UpdateBloodTestPanelRequest, UpdateBloodTestPanelItemsRequest,
  ProtocolActivity,
} from '@/types'

// 血液檢查 API 函數
export const bloodTestApi = {
  listByPig: (pigId: string) =>
    api.get<BloodTestListItem[]>(`/pigs/${pigId}/blood-tests`),
  getById: (id: string) =>
    api.get<PigBloodTestWithItems>(`/blood-tests/${id}`),
  create: (pigId: string, data: CreateBloodTestRequest) =>
    api.post<PigBloodTestWithItems>(`/pigs/${pigId}/blood-tests`, data),
  update: (id: string, data: UpdateBloodTestRequest) =>
    api.put<PigBloodTestWithItems>(`/blood-tests/${id}`, data),
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

// 計畫書活動紀錄 API
export const getProtocolActivities = async (id: string): Promise<ProtocolActivity[]> => {
  const response = await api.get<ProtocolActivity[]>(`/protocols/${id}/activities`)
  return response.data
}
