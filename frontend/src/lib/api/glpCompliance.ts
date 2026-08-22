// GLP 合規模組 API

import api from './client'

// ============================================================
// Reference Standards (參考標準器)
// ============================================================

export interface ReferenceStandard {
  id: string
  name: string
  serial_number: string | null
  standard_type: string
  traceable_to: string | null
  national_standard_number: string | null
  calibration_lab: string | null
  calibration_lab_accreditation: string | null
  last_calibrated_at: string | null
  next_due_at: string | null
  certificate_number: string | null
  measurement_uncertainty: string | null
  status: string
  notes: string | null
  created_at: string
  updated_at: string
}

export const listReferenceStandards = async () => {
  const res = await api.get<{ data: ReferenceStandard[] }>('/admin/reference-standards')
  return res.data.data
}

export const getReferenceStandard = async (id: string) => {
  const res = await api.get<ReferenceStandard>(`/admin/reference-standards/${id}`)
  return res.data
}

export const createReferenceStandard = async (payload: Partial<ReferenceStandard>) => {
  const res = await api.post<ReferenceStandard>('/admin/reference-standards', payload)
  return res.data
}

export const updateReferenceStandard = async (id: string, payload: Partial<ReferenceStandard>) => {
  const res = await api.put<ReferenceStandard>(`/admin/reference-standards/${id}`, payload)
  return res.data
}

// ============================================================
// Controlled Documents (文件控制)
// ============================================================

export interface ControlledDocument {
  id: string
  doc_number: string
  title: string
  doc_type: string
  category: string | null
  current_version: number
  status: string
  effective_date: string | null
  review_due_date: string | null
  owner_id: string | null
  owner_name: string | null
  approved_by: string | null
  approved_at: string | null
  retention_years: number | null
  created_at: string
  updated_at: string
}

export interface DocumentRevision {
  id: string
  document_id: string
  version: number
  change_summary: string
  revised_by: string | null
  reviewed_by: string | null
  approved_by: string | null
  approved_at: string | null
  file_path: string | null
  created_at: string
}

export const listControlledDocuments = async (params?: {
  doc_type?: string
  status?: string
  category?: string
}) => {
  const res = await api.get<{ data: ControlledDocument[] }>('/admin/documents', { params })
  return res.data.data
}

export const getControlledDocument = async (id: string) => {
  const res = await api.get<{ document: ControlledDocument; revisions: DocumentRevision[] }>(
    `/admin/documents/${id}`
  )
  return res.data
}

export const createControlledDocument = async (payload: {
  title: string
  doc_type: string
  category?: string
  effective_date?: string
  review_due_date?: string
  retention_years?: number
}) => {
  const res = await api.post<ControlledDocument>('/admin/documents', payload)
  return res.data
}

export const updateControlledDocument = async (id: string, payload: Partial<ControlledDocument>) => {
  const res = await api.put<ControlledDocument>(`/admin/documents/${id}`, payload)
  return res.data
}

export const approveControlledDocument = async (id: string) => {
  const res = await api.post<ControlledDocument>(`/admin/documents/${id}/approve`)
  return res.data
}

export const createRevision = async (id: string, payload: { change_summary: string; file_path?: string }) => {
  const res = await api.post<DocumentRevision>(`/admin/documents/${id}/revisions`, payload)
  return res.data
}

export const acknowledgeDocument = async (id: string) => {
  const res = await api.post(`/admin/documents/${id}/acknowledge`)
  return res.data
}

// ============================================================
// Management Reviews (管理審查)
// ============================================================

export interface ManagementReview {
  id: string
  review_number: string
  title: string
  review_date: string
  status: string
  agenda: string | null
  attendees: unknown[] | null
  minutes: string | null
  decisions: unknown[] | null
  action_items: unknown[] | null
  chaired_by: string | null
  approved_at: string | null
  created_at: string
  updated_at: string
}

export const listManagementReviews = async (params?: { status?: string }) => {
  const res = await api.get<{ data: ManagementReview[] }>('/admin/management-reviews', { params })
  return res.data.data
}

export const getManagementReview = async (id: string) => {
  const res = await api.get<ManagementReview>(`/admin/management-reviews/${id}`)
  return res.data
}

export const createManagementReview = async (payload: {
  title: string
  review_date: string
  agenda?: string
  attendees?: unknown[]
  chaired_by?: string
}) => {
  const res = await api.post<ManagementReview>('/admin/management-reviews', payload)
  return res.data
}

export const updateManagementReview = async (id: string, payload: Partial<ManagementReview>) => {
  const res = await api.put<ManagementReview>(`/admin/management-reviews/${id}`, payload)
  return res.data
}

// ============================================================
// Risk Register (風險管理)
// ============================================================

export interface RiskEntry {
  id: string
  risk_number: string
  title: string
  description: string | null
  category: string | null
  source: string | null
  severity: number
  likelihood: number
  detectability: number | null
  risk_score: number | null
  status: string
  mitigation_plan: string | null
  residual_risk_score: number | null
  owner_id: string | null
  owner_name: string | null
  review_date: string | null
  created_at: string
  updated_at: string
}

export const listRisks = async (params?: { status?: string; category?: string }) => {
  const res = await api.get<{ data: RiskEntry[] }>('/admin/risks', { params })
  return res.data.data
}

export const getRisk = async (id: string) => {
  const res = await api.get<RiskEntry>(`/admin/risks/${id}`)
  return res.data
}

export const createRisk = async (payload: {
  title: string
  severity: number
  likelihood: number
  description?: string
  category?: string
  source?: string
  detectability?: number
  mitigation_plan?: string
  owner_id?: string
  review_date?: string
}) => {
  const res = await api.post<RiskEntry>('/admin/risks', payload)
  return res.data
}

export const updateRisk = async (id: string, payload: Partial<RiskEntry>) => {
  const res = await api.put<RiskEntry>(`/admin/risks/${id}`, payload)
  return res.data
}

// ============================================================
// Change Requests (變更控制)
// ============================================================

export interface ChangeRequest {
  id: string
  change_number: string
  title: string
  change_type: string
  description: string
  justification: string | null
  impact_assessment: string | null
  status: string
  requested_by: string
  requester_name: string | null
  approved_by: string | null
  approved_at: string | null
  created_at: string
  updated_at: string
}

export const listChangeRequests = async (params?: { status?: string; change_type?: string }) => {
  const res = await api.get<{ data: ChangeRequest[] }>('/admin/change-requests', { params })
  return res.data.data
}

export const getChangeRequest = async (id: string) => {
  const res = await api.get<ChangeRequest>(`/admin/change-requests/${id}`)
  return res.data
}

export const createChangeRequest = async (payload: {
  title: string
  change_type: string
  description: string
  justification?: string
  impact_assessment?: string
}) => {
  const res = await api.post<ChangeRequest>('/admin/change-requests', payload)
  return res.data
}

export const updateChangeRequest = async (id: string, payload: Partial<ChangeRequest>) => {
  const res = await api.put<ChangeRequest>(`/admin/change-requests/${id}`, payload)
  return res.data
}

export const approveChangeRequest = async (id: string) => {
  const res = await api.post<ChangeRequest>(`/admin/change-requests/${id}/approve`)
  return res.data
}

// ============================================================
// Environment Monitoring (環境監控)
// ============================================================

export interface MonitoringPoint {
  id: string
  name: string
  location_type: string
  building_id: string | null
  zone_id: string | null
  parameters: { name: string; unit: string; min?: number; max?: number }[]
  monitoring_interval: string | null
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface EnvironmentReading {
  id: string
  monitoring_point_id: string
  reading_time: string
  readings: Record<string, number>
  is_out_of_range: boolean
  out_of_range_params: string[] | null
  recorded_by: string | null
  source: string
  notes: string | null
  created_at: string
}

export const listMonitoringPoints = async (activeOnly?: boolean) => {
  const res = await api.get<{ data: MonitoringPoint[] }>('/admin/env-monitoring/points', {
    params: { active_only: activeOnly },
  })
  return res.data.data
}

export const getMonitoringPoint = async (id: string) => {
  const res = await api.get<MonitoringPoint>(`/admin/env-monitoring/points/${id}`)
  return res.data
}

export const createMonitoringPoint = async (payload: {
  name: string
  location_type: string
  building_id?: string
  zone_id?: string
  parameters: { name: string; unit: string; min?: number; max?: number }[]
  monitoring_interval?: string
}) => {
  const res = await api.post<MonitoringPoint>('/admin/env-monitoring/points', payload)
  return res.data
}

export const updateMonitoringPoint = async (id: string, payload: Partial<MonitoringPoint>) => {
  const res = await api.put<MonitoringPoint>(`/admin/env-monitoring/points/${id}`, payload)
  return res.data
}

export const listReadings = async (params?: {
  monitoring_point_id?: string
  is_out_of_range?: boolean
}) => {
  const res = await api.get<{ data: EnvironmentReading[] }>('/admin/env-monitoring/readings', { params })
  return res.data.data
}

export const createReading = async (payload: {
  monitoring_point_id: string
  reading_time: string
  readings: Record<string, number>
  notes?: string
}) => {
  const res = await api.post<EnvironmentReading>('/admin/env-monitoring/readings', payload)
  return res.data
}

// ============================================================
// Competency Assessments (能力評鑑)
// ============================================================

export interface CompetencyAssessment {
  id: string
  user_id: string
  user_name: string | null
  assessment_type: string
  skill_area: string
  assessment_date: string
  assessor_id: string
  assessor_name: string | null
  result: string
  score: number | null
  method: string | null
  valid_until: string | null
  notes: string | null
  created_at: string
}

export interface TrainingRequirement {
  id: string
  role_code: string
  training_topic: string
  is_mandatory: boolean
  recurrence_months: number | null
  created_at: string
}

export const listCompetencyAssessments = async (params?: { user_id?: string; result?: string }) => {
  const res = await api.get<{ data: CompetencyAssessment[] }>('/admin/competency-assessments', { params })
  return res.data.data
}

export const createCompetencyAssessment = async (payload: {
  user_id: string
  assessment_type: string
  skill_area: string
  assessment_date: string
  result: string
  score?: number
  method?: string
  valid_until?: string
  notes?: string
}) => {
  const res = await api.post<CompetencyAssessment>('/admin/competency-assessments', payload)
  return res.data
}

export const updateCompetencyAssessment = async (id: string, payload: Partial<CompetencyAssessment>) => {
  const res = await api.put<CompetencyAssessment>(`/admin/competency-assessments/${id}`, payload)
  return res.data
}

export const listTrainingRequirements = async (roleCode?: string) => {
  const res = await api.get<{ data: TrainingRequirement[] }>('/admin/training-requirements', {
    params: { role_code: roleCode },
  })
  return res.data.data
}

export const createTrainingRequirement = async (payload: {
  role_code: string
  training_topic: string
  is_mandatory?: boolean
  recurrence_months?: number
}) => {
  const res = await api.post<TrainingRequirement>('/admin/training-requirements', payload)
  return res.data
}

export const deleteTrainingRequirement = async (id: string) => {
  await api.delete(`/admin/training-requirements/${id}`)
}

// ============================================================
// Study Final Reports (最終報告)
// ============================================================

export interface StudyFinalReport {
  id: string
  report_number: string
  protocol_id: string
  title: string
  status: string
  summary: string | null
  methods: string | null
  results: string | null
  conclusions: string | null
  deviations: string | null
  signed_by: string | null
  signed_at: string | null
  qau_statement: string | null
  qau_signed_by: string | null
  qau_signed_at: string | null
  created_at: string
  updated_at: string
}

export const listStudyReports = async (params?: { status?: string; protocol_id?: string }) => {
  const res = await api.get<{ data: StudyFinalReport[] }>('/admin/study-reports', { params })
  return res.data.data
}

export const getStudyReport = async (id: string) => {
  const res = await api.get<StudyFinalReport>(`/admin/study-reports/${id}`)
  return res.data
}

export const createStudyReport = async (payload: {
  protocol_id: string
  title: string
  summary?: string
  methods?: string
  results?: string
  conclusions?: string
  deviations?: string
}) => {
  const res = await api.post<StudyFinalReport>('/admin/study-reports', payload)
  return res.data
}

export const updateStudyReport = async (id: string, payload: Partial<StudyFinalReport>) => {
  const res = await api.put<StudyFinalReport>(`/admin/study-reports/${id}`, payload)
  return res.data
}

// ============================================================
// Formulation Records (配製紀錄)
// ============================================================

export interface FormulationRecord {
  id: string
  product_id: string
  product_name: string | null
  protocol_id: string | null
  formulation_date: string
  batch_number: string | null
  concentration: string | null
  volume: string | null
  prepared_by: string
  preparer_name: string | null
  verified_by: string | null
  verified_at: string | null
  expiry_date: string | null
  notes: string | null
  created_at: string
}

export const listFormulationRecords = async (params?: { product_id?: string; protocol_id?: string }) => {
  const res = await api.get<{ data: FormulationRecord[] }>('/admin/formulation-records', { params })
  return res.data.data
}

export const createFormulationRecord = async (payload: {
  product_id: string
  protocol_id?: string
  formulation_date: string
  batch_number?: string
  concentration?: string
  volume?: string
  expiry_date?: string
  notes?: string
}) => {
  const res = await api.post<FormulationRecord>('/admin/formulation-records', payload)
  return res.data
}
