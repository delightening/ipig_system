import type { PaginatedResponse } from '@/types/common'
import type {
  LeaveRequestWithUser,
  AttendanceWithUser,
  OvertimeWithUser,
  BalanceSummary,
} from '@/types/hr'

export const DEMO_BALANCE_SUMMARY: BalanceSummary = {
  user_id: 'demo-guest',
  user_name: '範例員工',
  annual_leave_total: 14,
  annual_leave_used: 5,
  annual_leave_remaining: 9,
  comp_time_total: 16,
  comp_time_used: 4,
  comp_time_remaining: 12,
  expiring_soon_days: 3,
  expiring_soon_hours: 0,
}

export const DEMO_LEAVES: PaginatedResponse<LeaveRequestWithUser> = {
  data: [
    {
      id: 'demo-l1', user_id: 'demo-u1', user_email: 'demo-a@example.com',
      user_name: '範例員工 A', proxy_user_id: null, proxy_user_name: null,
      leave_type: 'ANNUAL', start_date: '2026-03-25', end_date: '2026-03-26',
      total_days: 2, total_hours: null, reason: '範例休假事由',
      is_urgent: false, is_retroactive: false, status: 'APPROVED',
      current_approver_id: null, current_approver_name: null,
      submitted_at: '2026-03-20T09:00:00Z', created_at: '2026-03-20T09:00:00Z',
    },
    {
      id: 'demo-l2', user_id: 'demo-u2', user_email: 'demo-b@example.com',
      user_name: '範例員工 B', proxy_user_id: 'demo-u1', proxy_user_name: '範例員工 A',
      leave_type: 'SICK', start_date: '2026-03-28', end_date: '2026-03-28',
      total_days: 1, total_hours: null, reason: '範例病假事由',
      is_urgent: false, is_retroactive: false, status: 'PENDING_L1',
      current_approver_id: 'demo-u3', current_approver_name: '範例主管',
      submitted_at: '2026-03-27T14:00:00Z', created_at: '2026-03-27T14:00:00Z',
    },
  ],
  total: 2, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_ATTENDANCE: PaginatedResponse<AttendanceWithUser> = {
  data: [
    {
      id: 'demo-att1', user_id: 'demo-u1', user_email: 'demo-a@example.com',
      user_name: '範例員工 A', work_date: '2026-04-01',
      clock_in_time: '2026-04-01T08:02:00Z', clock_out_time: null,
      regular_hours: null, overtime_hours: null, status: 'present',
      remark: null, is_corrected: false,
    },
  ],
  total: 1, page: 1, per_page: 20, total_pages: 1,
}

export const DEMO_OVERTIME: PaginatedResponse<OvertimeWithUser> = {
  data: [
    {
      id: 'demo-ot1', user_id: 'demo-u1', user_email: 'demo-a@example.com',
      user_name: '範例員工 A', overtime_date: '2026-03-22',
      start_time: '18:00', end_time: '20:00', hours: 2,
      overtime_type: 'weekday', multiplier: 1.34, comp_time_hours: 2.68,
      comp_time_expires_at: '2026-09-22T00:00:00Z', status: 'APPROVED',
      reason: '範例加班事由',
    },
  ],
  total: 1, page: 1, per_page: 20, total_pages: 1,
}
