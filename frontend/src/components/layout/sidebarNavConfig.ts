import {
  LayoutDashboard,
  Package,
  Settings,
  FileText,
  FolderOpen,
  Users,
  Stethoscope,
  ClipboardCheck,
  BarChart3,
  MessageSquare,
} from 'lucide-react'
import { createElement } from 'react'

export type SubsystemKey = 'aup' | 'erp' | 'animal' | 'hr' | 'admin' | null

export interface NavItem {
  /** 穩定識別字串（語言無關），用於排序、過濾、drag-and-drop。
   *  與 i18n 翻譯後的 `title` 區隔，避免依賴中文字串造成 brittle filter */
  id: string
  /** i18n 鍵（語言包 `nav.<title>`），不是顯示文字；顯示文字由 useSidebarNav.translateTitle 依語言解析 */
  title: string
  href?: string
  icon: React.ReactNode
  children?: NavChildItem[]
  permission?: string
  badge?: number
  /** 子系統色相識別，用於 Sidebar active indicator */
  subsystem?: SubsystemKey
}

export interface NavChildItem {
  /** 穩定識別字串（語言無關），用於需要過濾或標記的子項。可選，未指定則不參與 id-based filter */
  id?: string
  title: string
  href?: string
  permission?: string
  children?: NavChildItem[]
}

const icon = (Icon: React.ComponentType<{ className?: string }>) =>
  createElement(Icon, { className: 'h-6 w-6' })

// 排序用的 stable id 陣列（DB 也以 id 形式存儲；舊版 nav_order 若殘留中文 title 由
// useSidebarNav 內 LEGACY_TITLE_TO_ID 映射相容）
export const DEFAULT_NAV_ORDER = [
  'dashboard',
  'messaging',
  'reports',
  'qau',
  'myProjects',
  'aupReview',
  'animalManagement',
  'hr',
  'erp',
  'admin',
]

export const GUEST_NAV_ORDER = [
  'dashboard',
  'reports',
  'aupReview',
  'animalManagement',
  'qau',
  'myProjects',
  'hr',
  'erp',
  'admin',
]

/** 客戶專屬（PI-only）可見的導航項目 id */
export const CLIENT_ONLY_NAV_IDS = new Set(['myProjects'])

/** Guest 模式需隱藏的子項 id（infra / 寫入專用，與 GLP read-only demo 無關） */
export const GUEST_HIDDEN_CHILD_IDS = new Set([
  'admin.users',           // PII
  'admin.settings',        // infra 細節
  'admin.notificationRouting', // infra
  'hr.invitations',        // 寫入流程（客戶邀請開通，已移至人員管理群組）
  'animalManagement.fieldCorrections', // R49 follow-up：修正審核流程僅管理員，guest 無意義
])

/** 舊版 DB 存的 nav_order 可能含中文 title — 升級時對齊到新 id */
export const LEGACY_TITLE_TO_ID: Record<string, string> = {
  'QAU 品質保證': 'qau',
  '人員管理': 'hr',
  'ERP': 'erp',
  '系統管理': 'admin',
}

export const navItemsConfig: NavItem[] = [
  {
    id: 'dashboard',
    title: 'dashboard',
    href: '/dashboard',
    icon: icon(LayoutDashboard),
    permission: 'dashboard.view',
  },
  {
    id: 'messaging',
    title: 'messaging',
    href: '/messaging',
    icon: icon(MessageSquare),
    permission: 'messaging.send',
  },
  {
    // 跨子系統 hub（含 ERP / AUP / 動物管理 / audit）— 必為 top-level，
    // 不可嵌在 ERP 父項下（父項 permission='erp' 會擋掉只有 AUP / 動物管理權限的使用者）
    id: 'reports',
    title: 'reports',
    href: '/reports',
    icon: icon(BarChart3),
  },
  {
    id: 'qau',
    title: 'qau',
    icon: icon(ClipboardCheck),
    permission: 'qau.dashboard.view',
    subsystem: 'admin',
    children: [
      { title: 'qauDashboard', href: '/admin/qau', permission: 'qau.dashboard.view' },
      { title: 'qauInspections', href: '/admin/qau/inspections', permission: 'qau.inspection.view' },
      { title: 'qauNonConformances', href: '/admin/qau/non-conformances', permission: 'qau.nc.view' },
      { title: 'qauSop', href: '/admin/qau/sop', permission: 'qau.sop.view' },
      { title: 'qauSchedules', href: '/admin/qau/schedules', permission: 'qau.schedule.view' },
    ],
  },
  {
    id: 'myProjects',
    title: 'myProjects',
    href: '/my-projects',
    icon: icon(FolderOpen),
  },
  {
    id: 'aupReview',
    title: 'aupReview',
    icon: icon(FileText),
    subsystem: 'aup',
    children: [
      { title: 'protocolManagement', href: '/protocols' },
      { title: 'newProtocol', href: '/protocols/new' },
      { title: 'myAmendments', href: '/my-amendments' },
    ],
  },
  {
    id: 'hr',
    title: 'hr',
    icon: icon(Users),
    subsystem: 'hr',
    children: [
      { title: 'hrAttendance', href: '/hr/attendance' },
      { title: 'hrLeaves', href: '/hr/leaves' },
      { title: 'hrOvertime', href: '/hr/overtime' },
      { title: 'hrAnnualLeave', href: '/hr/annual-leave', permission: 'hr.balance.manage' },
      { title: 'hrTraining', href: '/hr/training-records', permission: 'training.view' },
      { id: 'hr.invitations', title: 'hrInvitations', href: '/hr/invitations', permission: 'invitation.view' },
      { title: 'hrCalendar', href: '/hr/calendar' },
    ],
  },
  {
    id: 'animalManagement',
    title: 'animalManagement',
    icon: icon(Stethoscope),
    subsystem: 'animal',
    children: [
      { title: 'animalList', href: '/animals' },
      // 選單閘與路由閘一致用「檢視」權限；頁內操作另由 animal.planning.manage 個別守。
      // 用 animal.info.assign 會讓 SD / 試驗工作人員連選單入口都看不到。
      { title: 'animalPlanning', href: '/animals/reservation-planning', permission: 'animal.planning.view' },
      { title: 'vetPatrol', href: '/vet-patrol-reports', permission: 'animal.record.view' },
      { title: 'bloodTestAnalysis', href: '/blood-test-analysis' },
      { title: 'bloodTestTemplates', href: '/blood-test-templates', permission: 'animal.blood_test_template.manage' },
      { title: 'animalSources', href: '/animal-sources', permission: 'animal.source.manage' },
      { id: 'animalManagement.fieldCorrections', title: 'fieldCorrections', href: '/animals/animal-field-corrections', permission: 'admin' },
    ],
  },
  {
    id: 'erp',
    title: 'erp',
    icon: icon(Package),
    permission: 'erp',
    subsystem: 'erp',
    children: [
      { title: 'erpProducts', href: '/products' },
      { title: 'erpDocuments', href: '/documents' },
      {
        title: 'erpWarehouseOps',
        children: [
          { title: 'erpWarehouses', href: '/warehouses' },
          { title: 'erpInventory', href: '/inventory' },
          { title: 'erpInventoryLedger', href: '/inventory/ledger' },
        ],
      },
      { title: 'erpEquipment', href: '/equipment', permission: 'equipment.view' },
      { title: 'erpPartners', href: '/partners' },
    ],
  },
  {
    id: 'admin',
    title: 'admin',
    icon: icon(Settings),
    subsystem: 'admin',
    children: [
      { id: 'admin.users', title: 'adminUsers', href: '/admin/users' },
      { title: 'adminRoles', href: '/admin/roles' },
      { id: 'admin.settings', title: 'adminSettings', href: '/admin/settings' },
      { title: 'adminAuditLogs', href: '/admin/audit-logs' },
      { title: 'adminSecurityAudit', href: '/admin/audit' },
      { id: 'admin.notificationRouting', title: 'adminNotificationRouting', href: '/admin/notification-routing' },
      { title: 'adminTreatmentDrugs', href: '/admin/treatment-drugs' },
      { title: 'adminFacilities', href: '/admin/facilities' },
      {
        id: 'admin.glp',
        title: 'adminGlp',
        children: [
          { title: 'glpChangeControl', href: '/admin/change-control', permission: 'change.request.view' },
          { title: 'glpDocumentControl', href: '/admin/document-control', permission: 'dms.document.view' },
          { title: 'glpRiskRegister', href: '/admin/risk-register', permission: 'risk.register.view' },
          { title: 'glpManagementReview', href: '/admin/management-reviews', permission: 'glp.management_review.view' },
          { title: 'glpFormulationRecords', href: '/admin/formulation-records', permission: 'formulation.record.view' },
          { title: 'glpCompetency', href: '/admin/competency-assessments', permission: 'competency.assessment.view' },
          { title: 'glpStudyReports', href: '/admin/study-reports', permission: 'study.report.view' },
          { title: 'glpEnvMonitoring', href: '/admin/environment-monitoring', permission: 'env.monitoring.view' },
        ],
      },
    ],
    permission: 'admin',
  },
]
