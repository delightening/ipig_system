export interface DemoQauDashboard {
  protocol_status_summary: { status: string; display_name: string; count: number }[]
  review_progress: {
    status_changes_last_7_days: number
    protocols_in_review: number
    protocols_pending_pi_response: number
  }
  audit_summary: { entity_type: string; count: number }[]
  animal_summary: {
    total: number
    by_status: { status: string; display_name: string; count: number }[]
    in_experiment: number
    euthanized: number
    completed: number
  }
  qa_plan_summary: {
    open_nc_count: number
    overdue_nc_count: number
    active_sop_count: number
    inspection_by_status: { status: string; count: number }[]
    schedule_items_by_status: { status: string; count: number }[]
  }
}

export const DEMO_QAU_DASHBOARD: DemoQauDashboard = {
  protocol_status_summary: [
    { status: 'APPROVED', display_name: '已核准', count: 8 },
    { status: 'UNDER_REVIEW', display_name: '審查中', count: 3 },
    { status: 'DRAFT', display_name: '草稿', count: 2 },
    { status: 'SUBMITTED', display_name: '已送審', count: 1 },
  ],
  review_progress: {
    status_changes_last_7_days: 5,
    protocols_in_review: 3,
    protocols_pending_pi_response: 1,
  },
  audit_summary: [
    { entity_type: 'protocol', count: 14 },
    { entity_type: 'animal', count: 45 },
    { entity_type: 'facility', count: 6 },
  ],
  animal_summary: {
    total: 45,
    by_status: [
      { status: 'in_experiment', display_name: '實驗中', count: 20 },
      { status: 'unassigned', display_name: '未指派', count: 10 },
      { status: 'completed', display_name: '已完成', count: 12 },
      { status: 'euthanized', display_name: '已安樂死', count: 3 },
    ],
    in_experiment: 20,
    euthanized: 3,
    completed: 12,
  },
  qa_plan_summary: {
    open_nc_count: 2,
    overdue_nc_count: 0,
    active_sop_count: 15,
    inspection_by_status: [
      { status: 'completed', count: 8 },
      { status: 'scheduled', count: 4 },
      { status: 'overdue', count: 1 },
    ],
    schedule_items_by_status: [
      { status: 'completed', count: 12 },
      { status: 'pending', count: 3 },
    ],
  },
}
