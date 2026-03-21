/**
 * TanStack Query Key Factory
 *
 * Centralises all query key definitions so that:
 * - keys are type-safe and consistent across the app
 * - invalidation via prefix is simple (e.g. `queryKeys.animals.all`)
 * - adding filters preserves the parent key prefix for partial invalidation
 */

export const queryKeys = {
  // ───── Animals ─────
  animals: {
    all: ['animals'] as const,
    stats: ['animals-stats'] as const,
    byPen: ['animals-by-pen'] as const,
    list: (filters: Record<string, unknown>) => ['animals', filters] as const,
    detail: (id: string) => ['animal', id] as const,
    dataBoundary: (id: string) => ['animal-data-boundary', id] as const,
    observations: (id: string, after?: string) => ['animal-observations', id, after] as const,
    surgeries: (id: string, after?: string) => ['animal-surgeries', id, after] as const,
    weights: (id: string, after?: string) => ['animal-weights', id, after] as const,
    vaccinations: (id: string, after?: string) => ['animal-vaccinations', id, after] as const,
    sacrifice: (id: string) => ['animal-sacrifice', id] as const,
    suddenDeath: (id: string) => ['animal-sudden-death', id] as const,
    events: (id: string) => ['animal-iacuc-events', id] as const,
    transfers: (id: string) => ['animal-transfers', id] as const,
    sources: ['animal-sources'] as const,
  },

  // ───── Protocols ─────
  protocols: {
    all: ['protocols'] as const,
    detail: (id: string) => ['protocol', id] as const,
    versions: (id: string) => ['protocol-versions', id] as const,
    comments: (id: string) => ['protocol-comments', id] as const,
    activities: (id: string) => ['protocol-activities', id] as const,
    attachments: (id: string) => ['protocol-attachments', id] as const,
    reviewers: (id: string) => ['protocol-reviewers', id] as const,
    coEditors: (id: string) => ['protocol-co-editors', id] as const,
    statusHistory: (id: string) => ['protocol-status-history', id] as const,
    approvedList: ['approved-protocols'] as const,
    availableReviewers: ['available-reviewers'] as const,
    availableStaff: ['available-experiment-staff'] as const,
    staff: ['staff'] as const,
  },

  // ───── Users / Auth ─────
  users: {
    all: ['users'] as const,
    detail: (id: string) => ['user', id] as const,
    preferences: (key: string) => ['user-preferences', key] as const,
  },

  // ───── Products ─────
  products: {
    all: ['products'] as const,
    detail: (id: string) => ['product', id] as const,
  },

  // ───── Warehouses ─────
  warehouses: {
    all: ['warehouses'] as const,
    list: (search?: string) => ['warehouses', search] as const,
  },

  // ───── Documents ─────
  documents: {
    all: ['documents'] as const,
    list: (filters: Record<string, unknown>) => ['documents', filters] as const,
    detail: (id: string) => ['document', id] as const,
    recent: ['recent-documents'] as const,
  },

  // ───── Notifications ─────
  notifications: {
    all: ['notifications'] as const,
    unreadCount: ['notifications-unread-count'] as const,
    recent: ['notifications-recent'] as const,
    settings: ['notification-settings'] as const,
    routing: ['notification-routing'] as const,
    routingEventTypes: ['notification-routing-event-types'] as const,
    routingRoles: ['notification-routing-roles'] as const,
  },

  // ───── HR ─────
  hr: {
    myOvertime: ['hr-my-overtime'] as const,
    pendingOvertime: ['hr-pending-overtime'] as const,
    allOvertime: (filters: Record<string, unknown>) => ['hr-all-overtime', filters] as const,
    myLeaves: ['hr-my-leaves'] as const,
    pendingLeaves: ['hr-pending-leaves'] as const,
    internalUsers: ['hr-internal-users'] as const,
    balanceSummary: ['hr-balance-summary-expiring'] as const,
    attendance: (filters: Record<string, unknown>) => ['hr-attendance', filters] as const,
    staffList: ['hr-staff'] as const,
  },

  // ───── Calendar ─────
  calendar: {
    status: ['calendar-status'] as const,
    history: ['calendar-history'] as const,
    conflicts: ['calendar-conflicts'] as const,
    events: (range: unknown) => ['calendar-events', range] as const,
    dashboard: ['dashboard-calendar'] as const,
  },

  // ───── Admin ─────
  admin: {
    systemSettings: ['system-settings'] as const,
    configWarnings: ['admin-config-warnings'] as const,
    auditLogs: (filters: Record<string, unknown>) => ['audit-logs', filters] as const,
  },

  // ───── Inventory / Reports ─────
  inventory: {
    stockLedger: ['stock-ledger'] as const,
    lowStockAlerts: ['low-stock-alerts'] as const,
  },

  // ───── Signatures / Euthanasia ─────
  signatures: {
    sacrifice: (id?: string) => ['sacrifice-signature', id] as const,
  },
  euthanasia: {
    orders: ['euthanasia-orders'] as const,
  },

  // ───── My Projects ─────
  myProjects: {
    all: ['my-projects'] as const,
    detail: (id: string) => ['my-project', id] as const,
  },

  // ───── Vet ─────
  vet: {
    recentComments: ['recent-vet-comments'] as const,
  },

  // ───── Blood Tests ─────
  bloodTests: {
    analysis: (params: unknown) => ['blood-test-analysis', params] as const,
  },

  // ───── Reports ─────
  reports: {
    salesLines: (filters: unknown) => ['report-sales-lines', filters] as const,
  },
} as const
