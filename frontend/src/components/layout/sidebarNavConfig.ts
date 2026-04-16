import {
  LayoutDashboard,
  Package,
  Settings,
  FileText,
  FolderOpen,
  Users,
  Stethoscope,
  ClipboardCheck,
} from 'lucide-react'
import { createElement } from 'react'

export type SubsystemKey = 'aup' | 'erp' | 'animal' | 'hr' | 'admin' | null

export interface NavItem {
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
  title: string
  href?: string
  permission?: string
  translate?: boolean
  children?: NavChildItem[]
}

const icon = (Icon: React.ComponentType<{ className?: string }>) =>
  createElement(Icon, { className: 'h-6 w-6' })

export const DEFAULT_NAV_ORDER = [
  'dashboard',
  'QAU 品質保證',
  'myProjects',
  'aupReview',
  'animalManagement',
  '人員管理',
  'ERP',
  '系統管理',
]

export const GUEST_NAV_ORDER = [
  'dashboard',
  'myProjects',
  'aupReview',
  'animalManagement',
  '人員管理',
  'ERP',
  'QAU 品質保證',
]

export const navItemsConfig: NavItem[] = [
  {
    title: 'dashboard',
    href: '/dashboard',
    icon: icon(LayoutDashboard),
    permission: 'dashboard.view',
    translate: true,
  },
  {
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
    title: 'myProjects',
    href: '/my-projects',
    icon: icon(FolderOpen),
    translate: true,
  },
  {
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
      { title: '日曆', href: '/hr/calendar', translate: false },
    ],
  },
  {
    title: 'animalManagement',
    icon: icon(Stethoscope),
    translate: true,
    subsystem: 'animal',
    children: [
      { title: 'animalList', href: '/animals', translate: true },
      { title: '血檢分析', href: '/blood-test-analysis', translate: false },
      { title: '血檢項目', href: '/blood-test-templates', permission: 'animal.blood_test_template.manage', translate: false },
      { title: '來源管理', href: '/animal-sources', permission: 'animal.source.manage', translate: false },
      { title: '修正審核', href: '/animals/animal-field-corrections', permission: 'admin', translate: false },
    ],
  },
  {
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
      { title: '報表中心', href: '/erp/reports', permission: 'admin', translate: false },
    ],
  },
  {
    title: '系統管理',
    icon: icon(Settings),
    translate: false,
    subsystem: 'admin',
    children: [
      { title: '使用者管理', href: '/admin/users', translate: false },
      { title: '角色權限', href: '/admin/roles', translate: false },
      { title: '系統設定', href: '/admin/settings', translate: false },
      { title: '操作日誌', href: '/admin/audit-logs', translate: false },
      { title: '安全審計', href: '/admin/audit', translate: false },
      { title: '通知路由', href: '/admin/notification-routing', translate: false },
      { title: '藥物選單', href: '/admin/treatment-drugs', translate: false },
      { title: '設施管理', href: '/admin/facilities', translate: false },
      { title: '邀請管理', href: '/admin/invitations', permission: 'invitation.view', translate: false },
    ],
    permission: 'admin',
  },
]

/** 客戶專屬（PI-only）可見的導航項目 title */
export const CLIENT_ONLY_NAV_TITLES = new Set(['myProjects'])
