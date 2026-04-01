import type { PaginatedResponse } from '@/types/common'
import type { UserActivityLog } from '@/types/hr'

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

export const DEMO_ROLES = [
  { id: 'demo-role1', code: 'admin', name: '系統管理員', description: '完整系統管理權限', user_count: 1 },
  { id: 'demo-role2', code: 'VET', name: '獸醫', description: '動物醫療與福利', user_count: 2 },
  { id: 'demo-role3', code: 'EXPERIMENT_STAFF', name: '實驗人員', description: '實驗操作與紀錄', user_count: 5 },
  { id: 'demo-role4', code: 'PI', name: '計畫主持人', description: '研究計畫管理', user_count: 3 },
  { id: 'demo-role5', code: 'IACUC_CHAIR', name: 'IACUC 主席', description: '動物實驗審查', user_count: 1 },
  { id: 'demo-role6', code: 'REVIEWER', name: '審查委員', description: '計畫書審查', user_count: 4 },
]

export const DEMO_AUDIT_LOGS: PaginatedResponse<UserActivityLog> = {
  data: [
    {
      id: 'demo-log1', actor_user_id: 'demo-admin', actor_email: 'admin@example.com',
      actor_display_name: '範例管理員', event_category: 'AUTH', event_type: 'LOGIN',
      event_severity: 'INFO', entity_type: null, entity_id: null,
      entity_display_name: null, before_data: null, after_data: null,
      ip_address: '192.168.1.100', user_agent: 'Mozilla/5.0', request_path: '/api/auth/login',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-04-01T08:00:00Z',
    },
    {
      id: 'demo-log2', actor_user_id: 'demo-staff', actor_email: 'staff@example.com',
      actor_display_name: '範例實驗員', event_category: 'ANIMAL', event_type: 'RECORD_CREATE',
      event_severity: 'INFO', entity_type: 'animal_record', entity_id: 'demo-a1',
      entity_display_name: 'D-001 體重紀錄', before_data: null,
      after_data: { weight: 28.5 }, ip_address: '192.168.1.101',
      user_agent: 'Mozilla/5.0', request_path: '/api/animals/records',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-03-31T10:30:00Z',
    },
    {
      id: 'demo-log3', actor_user_id: 'demo-vet', actor_email: 'vet@example.com',
      actor_display_name: '範例獸醫', event_category: 'ANIMAL', event_type: 'VET_RECOMMEND',
      event_severity: 'INFO', entity_type: 'animal', entity_id: 'demo-a2',
      entity_display_name: 'D-002 獸醫建議', before_data: null,
      after_data: { recommendation: '持續觀察' }, ip_address: '192.168.1.102',
      user_agent: 'Mozilla/5.0', request_path: '/api/animals/vet-recommend',
      is_suspicious: false, suspicious_reason: null, created_at: '2026-03-30T15:00:00Z',
    },
  ],
  total: 3, page: 1, per_page: 50, total_pages: 1,
}
