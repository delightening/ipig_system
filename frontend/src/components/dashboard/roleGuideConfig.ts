/**
 * 角色歡迎指引設定：每個角色對應的指引文字與頁面連結
 *
 * - admin 不需要指引
 * - 多角色使用者會合併顯示所有角色的指引
 * - 連結指向實際存在的路由
 */

export interface GuideLink {
  /** i18n key 中的 label（如 "animalList"） */
  labelKey: string
  /** 連結路徑 */
  href: string
}

export interface RoleGuide {
  /** 角色代碼 */
  role: string
  /** i18n key prefix: dashboard.welcome.roles.{key} */
  i18nKey: string
  /** 排序優先順序（數字越小越前面） */
  priority: number
  /** 指引中的可點擊連結 */
  links: GuideLink[]
}

/**
 * 角色指引設定表
 * priority 決定多角色時的排列順序
 */
export const roleGuideConfigs: RoleGuide[] = [
  {
    role: 'VET',
    i18nKey: 'vet',
    priority: 1,
    links: [
      { labelKey: 'animalList', href: '/animals' },
    ],
  },
  {
    role: 'IACUC_CHAIR',
    i18nKey: 'iacucChair',
    priority: 2,
    links: [
      { labelKey: 'protocols', href: '/protocols' },
    ],
  },
  {
    role: 'IACUC_STAFF',
    i18nKey: 'iacucStaff',
    priority: 3,
    links: [
      { labelKey: 'protocols', href: '/protocols' },
    ],
  },
  {
    role: 'REVIEWER',
    i18nKey: 'reviewer',
    priority: 4,
    links: [
      { labelKey: 'protocols', href: '/protocols' },
    ],
  },
  {
    role: 'PI',
    i18nKey: 'pi',
    priority: 5,
    links: [
      { labelKey: 'myProjects', href: '/my-projects' },
      { labelKey: 'animalList', href: '/animals' },
      { labelKey: 'myAmendments', href: '/my-amendments' },
    ],
  },
  {
    role: 'EXPERIMENT_STAFF',
    i18nKey: 'experimentStaff',
    priority: 6,
    links: [
      { labelKey: 'animalList', href: '/animals' },
    ],
  },
  {
    role: 'INTERN',
    i18nKey: 'intern',
    priority: 7,
    links: [
      { labelKey: 'animalList', href: '/animals' },
    ],
  },
  {
    role: 'WAREHOUSE_MANAGER',
    i18nKey: 'warehouseManager',
    priority: 8,
    links: [
      { labelKey: 'inventory', href: '/inventory' },
      { labelKey: 'documents', href: '/documents' },
    ],
  },
  {
    role: 'PURCHASING',
    i18nKey: 'purchasing',
    priority: 9,
    links: [
      { labelKey: 'documents', href: '/documents' },
      { labelKey: 'partners', href: '/partners' },
    ],
  },
  {
    role: 'ADMIN_STAFF',
    i18nKey: 'adminStaff',
    priority: 10,
    links: [
      { labelKey: 'hrAttendance', href: '/hr/attendance' },
      { labelKey: 'auditLogs', href: '/admin/audit-logs' },
    ],
  },
  {
    role: 'QAU',
    i18nKey: 'qau',
    priority: 11,
    links: [
      { labelKey: 'qauDashboard', href: '/admin/qau' },
    ],
  },
  {
    role: 'EQUIPMENT_MAINTENANCE',
    i18nKey: 'equipmentMaintenance',
    priority: 12,
    links: [
      { labelKey: 'equipment', href: '/equipment' },
    ],
  },
  {
    role: 'CLIENT',
    i18nKey: 'client',
    priority: 13,
    links: [
      { labelKey: 'myProjects', href: '/my-projects' },
      { labelKey: 'animalList', href: '/animals' },
    ],
  },
]

/** 依使用者角色取得適用的指引，按 priority 排序 */
export function getGuidesForRoles(userRoles: string[]): RoleGuide[] {
  return roleGuideConfigs
    .filter((g) => userRoles.includes(g.role))
    .sort((a, b) => a.priority - b.priority)
}
