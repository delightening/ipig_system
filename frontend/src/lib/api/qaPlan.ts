// QA 計畫管理 API（稽查報告、不符合事項、SOP 文件、稽查排程）

import api from './client'

// ========== Types ==========

export type QaInspectionType = 'protocol' | 'equipment' | 'facility' | 'training' | 'general'
export type QaInspectionStatus = 'draft' | 'submitted' | 'closed'
export type QaItemResult = 'pass' | 'fail' | 'not_applicable'
export type NcSeverity = 'critical' | 'major' | 'minor'
export type NcSource = 'inspection' | 'observation' | 'external_audit' | 'self_report'
export type NcStatus = 'open' | 'in_progress' | 'pending_verification' | 'closed'
export type CapaActionType = 'corrective' | 'preventive'
export type CapaStatus = 'open' | 'in_progress' | 'completed' | 'verified'
export type SopStatus = 'draft' | 'active' | 'obsolete'
export type QaScheduleType = 'annual' | 'periodic' | 'ad_hoc'
export type QaScheduleStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled'
export type QaScheduleItemStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled' | 'overdue'

export interface QaInspectionWithInspector {
  id: string
  inspection_number: string
  title: string
  inspection_type: QaInspectionType
  inspection_date: string
  inspector_id: string
  inspector_name: string
  related_entity_type: string | null
  related_entity_id: string | null
  status: QaInspectionStatus
  findings: string | null
  conclusion: string | null
  created_at: string
  updated_at: string
}

export interface QaInspectionItem {
  id: string
  inspection_id: string
  item_order: number
  description: string
  result: QaItemResult
  remarks: string | null
  created_at: string
}

export interface QaInspectionDetail extends QaInspectionWithInspector {
  items: QaInspectionItem[]
}

export interface QaNonConformanceWithDetails {
  id: string
  nc_number: string
  title: string
  description: string
  severity: NcSeverity
  source: NcSource
  related_inspection_id: string | null
  assignee_id: string | null
  assignee_name: string | null
  due_date: string | null
  status: NcStatus
  root_cause: string | null
  closure_notes: string | null
  closed_at: string | null
  created_by: string
  creator_name: string
  created_at: string
  updated_at: string
}

export interface QaCapa {
  id: string
  nc_id: string
  action_type: CapaActionType
  description: string
  assignee_id: string | null
  due_date: string | null
  completed_at: string | null
  status: CapaStatus
  created_at: string
  updated_at: string
}

export interface NcDetail extends QaNonConformanceWithDetails {
  capa: QaCapa[]
}

export interface QaSopDocumentWithAck {
  id: string
  document_number: string
  title: string
  version: string
  category: string | null
  file_path: string | null
  effective_date: string | null
  review_date: string | null
  status: SopStatus
  description: string | null
  created_by: string
  creator_name: string
  acknowledged_by_me: boolean
  ack_count: number
  created_at: string
  updated_at: string
}

export interface QaAuditSchedule {
  id: string
  year: number
  title: string
  schedule_type: QaScheduleType
  description: string | null
  status: QaScheduleStatus
  created_by: string
  created_at: string
  updated_at: string
}

export interface QaScheduleItem {
  id: string
  schedule_id: string
  inspection_type: QaInspectionType
  title: string
  planned_date: string
  actual_date: string | null
  responsible_person_id: string | null
  responsible_name: string | null
  related_inspection_id: string | null
  status: QaScheduleItemStatus
  notes: string | null
  created_at: string
  updated_at: string
}

export interface QaScheduleDetail extends QaAuditSchedule {
  items: QaScheduleItem[]
}

// ========== Inspection API ==========

export const listInspections = async (params?: {
  page?: number
  page_size?: number
  inspection_type?: string
  status?: string
}) => {
  const res = await api.get<{ data: QaInspectionWithInspector[] }>('/qau/inspections', { params })
  return res.data.data
}

export const getInspection = async (id: string) => {
  const res = await api.get<QaInspectionDetail>(`/qau/inspections/${id}`)
  return res.data
}

export const createInspection = async (payload: {
  title: string
  inspection_type: QaInspectionType
  inspection_date: string
  related_entity_type?: string
  related_entity_id?: string
  findings?: string
  conclusion?: string
  items: { item_order: number; description: string; result: QaItemResult; remarks?: string }[]
}) => {
  const res = await api.post<QaInspectionDetail>('/qau/inspections', payload)
  return res.data
}

export const updateInspection = async (
  id: string,
  payload: Partial<{
    title: string
    inspection_date: string
    findings: string
    conclusion: string
    status: QaInspectionStatus
    items: { item_order: number; description: string; result: QaItemResult; remarks?: string }[]
  }>
) => {
  const res = await api.put<QaInspectionDetail>(`/qau/inspections/${id}`, payload)
  return res.data
}

// ========== Non-Conformance API ==========

export const listNonConformances = async (params?: {
  page?: number
  page_size?: number
  severity?: string
  status?: string
}) => {
  const res = await api.get<{ data: QaNonConformanceWithDetails[] }>('/qau/non-conformances', { params })
  return res.data.data
}

export const getNonConformance = async (id: string) => {
  const res = await api.get<NcDetail>(`/qau/non-conformances/${id}`)
  return res.data
}

export const createNonConformance = async (payload: {
  title: string
  description: string
  severity: NcSeverity
  source: NcSource
  related_inspection_id?: string
  assignee_id?: string
  due_date?: string
}) => {
  const res = await api.post<NcDetail>('/qau/non-conformances', payload)
  return res.data
}

export const updateNonConformance = async (
  id: string,
  payload: Partial<{
    title: string
    description: string
    assignee_id: string
    due_date: string
    status: NcStatus
    root_cause: string
    closure_notes: string
  }>
) => {
  const res = await api.put<NcDetail>(`/qau/non-conformances/${id}`, payload)
  return res.data
}

export const createCapa = async (
  ncId: string,
  payload: { action_type: CapaActionType; description: string; assignee_id?: string; due_date?: string }
) => {
  const res = await api.post<QaCapa>(`/qau/non-conformances/${ncId}/capa`, payload)
  return res.data
}

export const updateCapa = async (
  ncId: string,
  capaId: string,
  payload: Partial<{ description: string; assignee_id: string; due_date: string; status: CapaStatus }>
) => {
  const res = await api.put<QaCapa>(`/qau/non-conformances/${ncId}/capa/${capaId}`, payload)
  return res.data
}

// ========== SOP API ==========

export const listSopDocuments = async (params?: {
  page?: number
  page_size?: number
  status?: string
  category?: string
}) => {
  const res = await api.get<{ data: QaSopDocumentWithAck[] }>('/qau/sop', { params })
  return res.data.data
}

export const getSopDocument = async (id: string) => {
  const res = await api.get<QaSopDocumentWithAck>(`/qau/sop/${id}`)
  return res.data
}

export const createSopDocument = async (payload: {
  title: string
  version: string
  category?: string
  file_path?: string
  effective_date?: string
  review_date?: string
  description?: string
}) => {
  const res = await api.post<QaSopDocumentWithAck>('/qau/sop', payload)
  return res.data
}

export const updateSopDocument = async (
  id: string,
  payload: Partial<{
    title: string
    version: string
    category: string
    file_path: string
    effective_date: string
    review_date: string
    status: SopStatus
    description: string
  }>
) => {
  const res = await api.put<QaSopDocumentWithAck>(`/qau/sop/${id}`, payload)
  return res.data
}

export const acknowledgeSop = async (id: string) => {
  const res = await api.post<QaSopDocumentWithAck>(`/qau/sop/${id}/acknowledge`)
  return res.data
}

// ========== Schedule API ==========

export const listSchedules = async (params?: { year?: number; status?: string }) => {
  const res = await api.get<{ data: QaAuditSchedule[] }>('/qau/schedules', { params })
  return res.data.data
}

export const getSchedule = async (id: string) => {
  const res = await api.get<QaScheduleDetail>(`/qau/schedules/${id}`)
  return res.data
}

export const createSchedule = async (payload: {
  year: number
  title: string
  schedule_type: QaScheduleType
  description?: string
  items: {
    inspection_type: QaInspectionType
    title: string
    planned_date: string
    responsible_person_id?: string
    notes?: string
  }[]
}) => {
  const res = await api.post<QaScheduleDetail>('/qau/schedules', payload)
  return res.data
}

export const updateSchedule = async (
  id: string,
  payload: Partial<{ title: string; description: string; status: QaScheduleStatus }>
) => {
  const res = await api.put<QaScheduleDetail>(`/qau/schedules/${id}`, payload)
  return res.data
}

export const updateScheduleItem = async (
  scheduleId: string,
  itemId: string,
  payload: Partial<{
    actual_date: string
    responsible_person_id: string
    related_inspection_id: string
    status: QaScheduleItemStatus
    notes: string
  }>
) => {
  const res = await api.put<QaScheduleDetail>(`/qau/schedules/${scheduleId}/items/${itemId}`, payload)
  return res.data
}
