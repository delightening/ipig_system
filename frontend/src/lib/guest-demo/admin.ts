import type { PaginatedResponse } from '@/types/common'
import type { UserActivityLog } from '@/types/hr'
import type { Permission, Role } from '@/types/auth'

export interface DemoUser {
  id: string
  email: string
  display_name: string
  roles: string[]
  is_active: boolean
  created_at: string
  last_login_at?: string
}

export const DEMO_USERS: PaginatedResponse<DemoUser> = {
  data: [
    {
      id: 'demo-admin', email: 'admin@example.com', display_name: '範例管理員',
      roles: ['admin'], is_active: true,
      created_at: '2025-01-01T08:00:00Z', last_login_at: '2026-04-01T08:00:00Z',
    },
    {
      id: 'demo-vet', email: 'vet@example.com', display_name: '範例獸醫',
      roles: ['VET'], is_active: true,
      created_at: '2025-02-01T08:00:00Z', last_login_at: '2026-03-31T09:00:00Z',
    },
    {
      id: 'demo-staff', email: 'staff@example.com', display_name: '範例實驗員',
      roles: ['EXPERIMENT_STAFF'], is_active: true,
      created_at: '2025-03-01T08:00:00Z', last_login_at: '2026-03-30T08:30:00Z',
    },
    {
      id: 'demo-pi', email: 'pi@example.com', display_name: '範例研究員',
      roles: ['PI'], is_active: true,
      created_at: '2025-04-01T08:00:00Z', last_login_at: '2026-03-28T14:00:00Z',
    },
  ],
  total: 4, page: 1, per_page: 20, total_pages: 1,
}

// R49 follow-up：DEMO_PERMISSIONS 為 RolesPage 角色詳情提供範例權限資料。
// 真實系統權限約百項；此處挑各模組代表性權限讓 guest 可看出角色範圍 + module 分組。
const DEMO_TIMESTAMP = '2026-01-01T00:00:00Z'
const demoPerm = (code: string, name: string, module: string): Permission => ({
  id: `demo-perm-${code}`,
  code,
  name,
  module,
  created_at: DEMO_TIMESTAMP,
})

export const DEMO_PERMISSIONS: Permission[] = [
  // 系統管理
  demoPerm('admin.users', '使用者管理', 'admin'),
  demoPerm('admin.roles', '角色管理', 'admin'),
  demoPerm('admin.settings', '系統設定', 'admin'),
  demoPerm('admin.audit.view', '稽核日誌檢視', 'admin'),
  // 計畫書（AUP）
  demoPerm('protocol.view', '計畫書檢視', 'protocol'),
  demoPerm('protocol.create', '計畫書建立', 'protocol'),
  demoPerm('protocol.review', '計畫書審查', 'protocol'),
  demoPerm('protocol.approve', '計畫書核准', 'protocol'),
  // 動物管理
  demoPerm('animal.record.view', '動物紀錄檢視', 'animal'),
  demoPerm('animal.record.edit', '動物紀錄編輯', 'animal'),
  demoPerm('animal.vet_advice.manage', '獸醫建議管理', 'animal'),
  demoPerm('animal.source.manage', '動物來源管理', 'animal'),
  // 人員管理
  demoPerm('hr.attendance.manage', '出勤管理', 'hr'),
  demoPerm('hr.leave.approve', '請假審核', 'hr'),
  demoPerm('hr.balance.manage', '特休管理', 'hr'),
  // QAU
  demoPerm('qau.inspection.view', '稽查報告檢視', 'qau'),
  demoPerm('qau.nc.manage', '不符合事項管理', 'qau'),
  demoPerm('qau.sop.view', 'SOP 文件檢視', 'qau'),
]

const pickPerms = (...codes: string[]): Permission[] =>
  DEMO_PERMISSIONS.filter(p => codes.includes(p.code))

const demoRole = (
  id: string,
  code: string,
  name: string,
  description: string,
  is_system: boolean,
  permCodes: string[],
): Role => ({
  id,
  code,
  name,
  description,
  is_internal: true,
  is_system,
  is_active: true,
  permissions: pickPerms(...permCodes),
  created_at: DEMO_TIMESTAMP,
  updated_at: DEMO_TIMESTAMP,
})

export const DEMO_ROLES: Role[] = [
  demoRole('demo-role1', 'admin', '系統管理員', '完整系統管理權限', true,
    ['admin.users', 'admin.roles', 'admin.settings', 'admin.audit.view',
     'protocol.approve', 'animal.record.edit', 'hr.balance.manage', 'qau.nc.manage']),
  demoRole('demo-role2', 'VET', '獸醫', '動物醫療與福利', true,
    ['animal.record.view', 'animal.record.edit', 'animal.vet_advice.manage', 'animal.source.manage']),
  demoRole('demo-role3', 'EXPERIMENT_STAFF', '實驗人員', '實驗操作與紀錄', true,
    ['protocol.view', 'animal.record.view', 'animal.record.edit']),
  demoRole('demo-role4', 'PI', '計畫主持人', '研究計畫管理', true,
    ['protocol.view', 'protocol.create', 'animal.record.view']),
  demoRole('demo-role5', 'IACUC_CHAIR', 'IACUC 主席', '動物實驗審查', true,
    ['protocol.view', 'protocol.review', 'protocol.approve', 'admin.audit.view']),
  demoRole('demo-role6', 'REVIEWER', '審查委員', '計畫書審查', true,
    ['protocol.view', 'protocol.review']),
]

export const DEMO_AUDIT_LOGS: PaginatedResponse<UserActivityLog> = {
  data: [
    {
      id: 'demo-log1', actor_user_id: 'demo-admin', actor_email: 'admin@example.com',
      actor_display_name: '範例管理員', event_category: 'AUTH', event_type: 'LOGIN',
      event_severity: 'INFO', entity_type: null, entity_id: null,
      entity_display_name: null, before_data: null, after_data: null,
      changed_fields: null,
      ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0', request_path: '/api/auth/login',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-04-01T08:00:00Z',
      integrity_hash: null, previous_hash: null, impersonated_by_user_id: null, hmac_version: null,
    },
    {
      id: 'demo-log2', actor_user_id: 'demo-staff', actor_email: 'staff@example.com',
      actor_display_name: '範例實驗員', event_category: 'ANIMAL', event_type: 'WEIGHT_CREATE',
      event_severity: 'INFO', entity_type: 'animal_weight', entity_id: 'demo-w-1',
      entity_display_name: 'D-001 體重 28.5 kg', before_data: null,
      after_data: { weight: 28.5, measure_date: '2026-03-28' }, changed_fields: ['weight', 'measure_date'],
      ip_address: '192.168.1.101', user_agent: 'Mozilla/5.0', request_path: '/api/animals/demo-a1/weights',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-03-28T10:00:00Z',
      integrity_hash: 'demo-hash-2', previous_hash: 'demo-hash-1', impersonated_by_user_id: null, hmac_version: 2,
    },
    {
      id: 'demo-log3', actor_user_id: 'demo-vet', actor_email: 'vet@example.com',
      actor_display_name: '範例獸醫', event_category: 'ANIMAL', event_type: 'OBSERVATION_CREATE',
      event_severity: 'INFO', entity_type: 'animal_observation', entity_id: 'demo-obs-3',
      entity_display_name: 'D-002 異常觀察 — 輕微跛行', before_data: null,
      after_data: { record_type: 'abnormal', content: '輕微跛行（左後肢）...' }, changed_fields: ['content', 'record_type'],
      ip_address: '192.168.1.102', user_agent: 'Mozilla/5.0', request_path: '/api/animals/demo-a2/observations',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-03-25T14:00:00Z',
      integrity_hash: 'demo-hash-3', previous_hash: 'demo-hash-2', impersonated_by_user_id: null, hmac_version: 2,
    },
    {
      id: 'demo-log4', actor_user_id: 'demo-vet', actor_email: 'vet@example.com',
      actor_display_name: '範例獸醫', event_category: 'ANIMAL', event_type: 'SURGERY_CREATE',
      event_severity: 'INFO', entity_type: 'animal_surgery', entity_id: 'demo-surg-1',
      entity_display_name: 'D-001 左眼手術', before_data: null,
      after_data: { surgery_site: '左眼', surgery_date: '2026-01-15' }, changed_fields: ['surgery_site', 'surgery_date'],
      ip_address: '192.168.1.102', user_agent: 'Mozilla/5.0', request_path: '/api/animals/demo-a1/surgeries',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-01-15T11:00:00Z',
      integrity_hash: 'demo-hash-4', previous_hash: 'demo-hash-3', impersonated_by_user_id: null, hmac_version: 2,
    },
    {
      id: 'demo-log5', actor_user_id: 'demo-pi', actor_email: 'pi@example.com',
      actor_display_name: '範例研究員', event_category: 'AUP', event_type: 'PROTOCOL_SUBMIT',
      event_severity: 'INFO', entity_type: 'protocol', entity_id: 'demo-p1',
      entity_display_name: 'AUP-2025-001 心血管藥物安全性評估', before_data: { status: 'DRAFT' },
      after_data: { status: 'SUBMITTED' }, changed_fields: ['status'],
      ip_address: '192.168.1.105', user_agent: 'Mozilla/5.0', request_path: '/api/protocols/demo-p1/submit',
      is_suspicious: false, suspicious_reason: null, created_at: '2025-09-20T10:30:00Z',
      integrity_hash: 'demo-hash-5', previous_hash: 'demo-hash-4', impersonated_by_user_id: null, hmac_version: 2,
    },
    {
      id: 'demo-log6', actor_user_id: 'demo-admin', actor_email: 'admin@example.com',
      actor_display_name: '範例 IACUC 主席', event_category: 'AUP', event_type: 'PROTOCOL_APPROVE',
      event_severity: 'INFO', entity_type: 'protocol', entity_id: 'demo-p1',
      entity_display_name: 'AUP-2025-001 心血管藥物安全性評估', before_data: { status: 'UNDER_REVIEW' },
      after_data: { status: 'APPROVED' }, changed_fields: ['status'],
      ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0', request_path: '/api/protocols/demo-p1/approve',
      is_suspicious: false, suspicious_reason: null, created_at: '2025-11-01T14:00:00Z',
      integrity_hash: 'demo-hash-6', previous_hash: 'demo-hash-5', impersonated_by_user_id: null, hmac_version: 2,
    },
    {
      id: 'demo-log7', actor_user_id: null, actor_email: null,
      actor_display_name: null, event_category: 'SECURITY', event_type: 'LOGIN_FAILED',
      event_severity: 'WARN', entity_type: null, entity_id: null,
      entity_display_name: '密碼錯誤 (admin@example.com)', before_data: null,
      after_data: { reason: 'invalid_credentials' }, changed_fields: null,
      ip_address: '203.0.113.45', user_agent: 'curl/7.81', request_path: '/api/auth/login',
      is_suspicious: true, suspicious_reason: '同 IP 連續失敗 5 次',
      created_at: '2026-04-02T03:14:00Z',
      integrity_hash: null, previous_hash: null, impersonated_by_user_id: null, hmac_version: null,
    },
    {
      id: 'demo-log8', actor_user_id: 'demo-vet', actor_email: 'vet@example.com',
      actor_display_name: '範例獸醫', event_category: 'ANIMAL', event_type: 'EXPORT_MEDICAL',
      event_severity: 'INFO', entity_type: 'animal', entity_id: 'demo-a1',
      entity_display_name: 'D-001 病歷 PDF 匯出', before_data: null,
      after_data: { format: 'pdf', export_type: 'medical_summary' }, changed_fields: null,
      ip_address: '192.168.1.102', user_agent: 'Mozilla/5.0', request_path: '/api/animals/demo-a1/export-pdf',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-04-03T11:20:00Z',
      integrity_hash: 'demo-hash-8', previous_hash: 'demo-hash-7', impersonated_by_user_id: null, hmac_version: 2,
    },
  ],
  total: 8, page: 1, per_page: 50, total_pages: 1,
}

// QAU 稽查樣本（顯示 GLP 品保流程）
export const DEMO_QAU_INSPECTIONS = {
  data: [
    {
      id: 'demo-insp-1', inspection_no: 'QA-2026-001',
      inspection_type: 'PROTOCOL',
      protocol_id: 'demo-p1', protocol_no: 'AUP-2025-001',
      title: 'AUP-2025-001 期中稽查',
      status: 'COMPLETED', planned_date: '2026-02-15',
      completed_at: '2026-02-15T16:00:00Z',
      inspector_name: '範例 QAU 稽查員',
      created_at: '2026-02-01T09:00:00Z',
    },
    {
      id: 'demo-insp-2', inspection_no: 'QA-2026-002',
      inspection_type: 'FACILITY',
      title: '動物房環境每月稽查',
      status: 'IN_PROGRESS', planned_date: '2026-04-15',
      inspector_name: '範例 QAU 稽查員',
      created_at: '2026-04-01T09:00:00Z',
    },
    {
      id: 'demo-insp-3', inspection_no: 'QA-2026-003',
      inspection_type: 'EQUIPMENT',
      title: '麻醉機 / 生理監測儀年度校正稽查',
      status: 'PLANNED', planned_date: '2026-05-20',
      inspector_name: '範例 QAU 稽查員',
      created_at: '2026-04-10T09:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_QAU_NCS = {
  data: [
    {
      id: 'demo-nc-1', nc_no: 'NC-2026-001',
      severity: 'MINOR', status: 'CLOSED',
      title: '採血紀錄少填日期欄位',
      description: 'D-002 動物 2026-03-15 採血未填寫採血時刻，已補正',
      raised_by: '範例 QAU 稽查員', closed_at: '2026-03-20T15:00:00Z',
      created_at: '2026-03-15T16:00:00Z',
    },
    {
      id: 'demo-nc-2', nc_no: 'NC-2026-002',
      severity: 'MAJOR', status: 'OPEN',
      title: '麻醉機 SpO₂ 探頭超期未校正',
      description: '校正紀錄顯示上次校正 2025-04，超過年度上限。已下架待校正完成',
      raised_by: '範例 QAU 稽查員',
      created_at: '2026-04-12T10:00:00Z',
    },
  ],
  total: 2, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_QAU_SOPS = {
  data: [
    {
      id: 'demo-sop-1', doc_no: 'SOP-AUP-001',
      title: 'IACUC 計畫書審核標準作業程序',
      version: '2.1', status: 'EFFECTIVE',
      effective_date: '2025-06-01', review_date: '2027-06-01',
      created_at: '2025-05-15T08:00:00Z',
    },
    {
      id: 'demo-sop-2', doc_no: 'SOP-VET-003',
      title: '迷你豬麻醉與術後疼痛評估 SOP',
      version: '1.4', status: 'EFFECTIVE',
      effective_date: '2024-09-01', review_date: '2026-09-01',
      created_at: '2024-08-15T08:00:00Z',
    },
    {
      id: 'demo-sop-3', doc_no: 'SOP-FAC-002',
      title: '動物房環境溫濕度監控 SOP',
      version: '1.2', status: 'UNDER_REVIEW',
      effective_date: '2025-01-01', review_date: '2026-12-31',
      created_at: '2024-12-15T08:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 20, total_pages: 1,
}

// HR 訓練紀錄樣本
export const DEMO_TRAINING_RECORDS = {
  data: [
    {
      id: 'demo-tr-1',
      user_id: 'demo-vet',
      user_email: 'vet@example.com',
      user_name: '範例獸醫',
      course_name: 'GLP 法規與品保系統年度訓練',
      completed_at: '2026-01-15',
      expires_at: '2027-01-15',
      notes: '通過後測 95 分',
      created_at: '2026-01-15T10:00:00Z',
    },
    {
      id: 'demo-tr-2',
      user_id: 'demo-vet',
      user_email: 'vet@example.com',
      user_name: '範例獸醫',
      course_name: '迷你豬麻醉與術後疼痛評估',
      completed_at: '2025-11-20',
      expires_at: '2027-11-20',
      notes: null,
      created_at: '2025-11-20T14:00:00Z',
    },
    {
      id: 'demo-tr-3',
      user_id: 'demo-staff',
      user_email: 'staff@example.com',
      user_name: '範例實驗員',
      course_name: '動物福利與 3R 原則',
      completed_at: '2026-02-08',
      expires_at: '2027-02-08',
      notes: null,
      created_at: '2026-02-08T09:30:00Z',
    },
    {
      id: 'demo-tr-4',
      user_id: 'demo-staff',
      user_email: 'staff@example.com',
      user_name: '範例實驗員',
      course_name: '電子簽章與資料完整性 (ALCOA+)',
      completed_at: '2026-03-12',
      expires_at: null,
      notes: '一次性課程',
      created_at: '2026-03-12T13:00:00Z',
    },
    {
      id: 'demo-tr-5',
      user_id: 'demo-pi',
      user_email: 'pi@example.com',
      user_name: '範例研究員',
      course_name: 'IACUC 計畫書撰寫與審查流程',
      completed_at: '2025-08-25',
      expires_at: '2027-08-25',
      notes: null,
      created_at: '2025-08-25T15:00:00Z',
    },
    {
      id: 'demo-tr-6',
      user_id: 'demo-admin',
      user_email: 'admin@example.com',
      user_name: '範例管理員',
      course_name: '個資保護與資訊安全年度訓練',
      completed_at: '2026-01-20',
      expires_at: '2027-01-20',
      notes: '通過後測 92 分',
      created_at: '2026-01-20T11:00:00Z',
    },
  ],
  total: 6, page: 1, per_page: 20, total_pages: 1,
}
