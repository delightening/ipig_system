import api, { deleteResource } from './client'

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
    deleteResource(`/admin/treatment-drugs/${id}`),
  /** 從 ERP 匯入 */
  importFromErp: (data: ImportFromErpRequest) =>
    api.post<TreatmentDrugOption[]>('/admin/treatment-drugs/import-erp', data),
}
