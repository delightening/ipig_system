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
  title: string
  href?: string
  icon: React.ReactNode
  children?: NavChildItem[]
  permission?: string
  badge?: number
  translate?: boolean
  /** 子系統色相識別，用於 Sidebar active indicator */
  subsystem?: SubsystemKey
}

export interface NavChildItem {
  /** 穩定識別字串（語言無關），用於需要過濾或標記的子項。可選，未指定則不參與 id-based filter */
  id?: string
  title: string
  href?: string
  permission?: string
  translate?: boolean
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
    translate: true,
  },
  {
    id: 'messaging',
    title: '站內信',
    href: '/messaging',
    icon: icon(MessageSquare),
    permission: 'messaging.send',
    translate: false,
  },
  {
    // 跨子系統 hub（含 ERP / AUP / 動物管理 / audit）— 必為 top-level，
    // 不可嵌在 ERP 父項下（父項 permission='erp' 會擋掉只有 AUP / 動物管理權限的使用者）
    id: 'reports',
    title: '報表中心',
    href: '/reports',
    icon: icon(BarChart3),
    translate: false,
  },
  {
    id: 'qau',
    title: 'QAU 品質保證',
    icon: icon(ClipboardCheck),
    permission: 'qau.dashboard.view',
    translate: false,
    subsystem: 'admin',
    children: [
      { title: '品質保證儀表板', href: '/admin/qau', permission: 'qau.dashboard.view', translate: false },
      { title: '稽查報告', href: '/admin/qau/inspections', permission: 'qau.inspection.view', translate: false },
      { title: '不符合事項（NC）', href: '/admin/qau/non-conformances', permission: 'qau.nc.view', translate: false },
      { title: 'SOP 文件', href: '/admin/qau/sop', permission: 'qau.sop.view', translate: false },
      { title: '稽查排程', href: '/admin/qau/schedules', permission: 'qau.schedule.view', translate: false },
    ],
  },
  {
    id: 'myProjects',
    title: 'myProjects',
    href: '/my-projects',
    icon: icon(FolderOpen),
    translate: true,
  },
  {
    id: 'aupReview',
    title: 'aupReview',
    icon: icon(FileText),
    translate: true,
    subsystem: 'aup',
    children: [
      { title: 'protocolManagement', href: '/protocols', translate: true },
      { title: 'newProtocol', href: '/protocols/new', translate: true },
      { title: 'myAmendments', href: '/my-amendments', translate: true },
    ],
  },
  {
    id: 'hr',
    title: '人員管理',
    icon: icon(Users),
    translate: false,
    subsystem: 'hr',
    children: [
      { title: '出勤打卡', href: '/hr/attendance', translate: false },
      { title: '請假管理', href: '/hr/leaves', translate: false },
      { title: '加班管理', href: '/hr/overtime', translate: false },
      { title: '特休管理', href: '/hr/annual-leave', permission: 'hr.balance.manage', translate: false },
      { title: '人員訓練', href: '/hr/training-records', permission: 'training.view', translate: false },
      { id: 'hr.invitations', title: '邀請管理', href: '/hr/invitations', permission: 'invitation.view', translate: false },
      { title: '行事曆', href: '/hr/calendar', translate: false },
    ],
  },
  {
    id: 'animalManagement',
    title: 'animalManagement',
    icon: icon(Stethoscope),
    translate: true,
    subsystem: 'animal',
    children: [
      { title: 'animalList', href: '/animals', translate: true },
      // 選單閘與路由閘一致用「檢視」權限；頁內操作另由 animal.planning.manage 個別守。
      // 用 animal.info.assign 會讓 SD / 試驗工作人員連選單入口都看不到。
      { title: '預約與試驗規劃', href: '/animals/reservation-planning', permission: 'animal.planning.view', translate: false },
      { title: '巡場報告', href: '/vet-patrol-reports', permission: 'animal.record.view', translate: false },
      { title: '血檢分析', href: '/blood-test-analysis', translate: false },
      { title: '血檢項目', href: '/blood-test-templates', permission: 'animal.blood_test_template.manage', translate: false },
      { title: '來源管理', href: '/animal-sources', permission: 'animal.source.manage', translate: false },
      { id: 'animalManagement.fieldCorrections', title: '修正審核', href: '/animals/animal-field-corrections', permission: 'admin', translate: false },
    ],
  },
  {
    id: 'erp',
    title: 'ERP',
    icon: icon(Package),
    translate: false,
    permission: 'erp',
    subsystem: 'erp',
    children: [
      { title: '產品管理', href: '/products', translate: false },
      { title: '單據管理', href: '/documents', translate: false },
      {
        title: '倉儲作業',
        translate: false,
        children: [
          { title: '倉庫', href: '/warehouses', translate: false },
          { title: '庫存查詢', href: '/inventory', translate: false },
          { title: '庫存流水', href: '/inventory/ledger', translate: false },
        ],
      },
      { title: '設備維護', href: '/equipment', permission: 'equipment.view', translate: false },
      { title: '供應商／客戶', href: '/partners', translate: false },
    ],
  },
  {
    id: 'admin',
    title: '系統管理',
    icon: icon(Settings),
    translate: false,
    subsystem: 'admin',
    children: [
      { id: 'admin.users', title: '使用者管理', href: '/admin/users', translate: false },
      { title: '角色權限', href: '/admin/roles', translate: false },
      { id: 'admin.settings', title: '系統設定', href: '/admin/settings', translate: false },
      { title: '操作日誌', href: '/admin/audit-logs', translate: false },
      { title: '安全審計', href: '/admin/audit', translate: false },
      { id: 'admin.notificationRouting', title: '通知路由', href: '/admin/notification-routing', translate: false },
      { title: '藥物選單', href: '/admin/treatment-drugs', translate: false },
      { title: '設施管理', href: '/admin/facilities', translate: false },
      {
        id: 'admin.glp',
        title: 'GLP 合規',
        translate: false,
        children: [
          { title: '變更控制', href: '/admin/change-control', permission: 'change.request.view', translate: false },
          { title: '文件控制', href: '/admin/document-control', permission: 'dms.document.view', translate: false },
          { title: '風險登記簿', href: '/admin/risk-register', permission: 'risk.register.view', translate: false },
          { title: '管理審查', href: '/admin/management-reviews', permission: 'glp.management_review.view', translate: false },
          { title: '配製紀錄', href: '/admin/formulation-records', permission: 'formulation.record.view', translate: false },
          { title: '能力評鑑', href: '/admin/competency-assessments', permission: 'competency.assessment.view', translate: false },
          { title: '研究最終報告', href: '/admin/study-reports', permission: 'study.report.view', translate: false },
          { title: '環境監控', href: '/admin/environment-monitoring', permission: 'env.monitoring.view', translate: false },
        ],
      },
    ],
    permission: 'admin',
  },
]
