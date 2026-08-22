import { Permission } from '@/lib/api'

// 模組配置 - 中文顯示名稱
export const MODULE_CONFIG: Record<string, { name: string; order: number }> = {
  // 動物使用計畫（包含資料庫中的 AUP 和 aup）
  aup: { name: '動物使用計畫', order: 1 },
  AUP: { name: '動物使用計畫', order: 1 },
  amendment: { name: '動物使用計畫', order: 1 },

  // 動物管理（含物種管理、動物來源、緊急給藥等）
  animal: { name: '動物管理', order: 2 },
  species: { name: '動物管理', order: 2 },
  record: { name: '動物管理', order: 2 },
  source: { name: '動物管理', order: 2 },
  '實驗動物管理': { name: '動物管理', order: 2 },

  // 庫存管理（包含資料庫中的 ERP、erp、設備）
  erp: { name: '庫存管理', order: 3 },
  ERP: { name: '庫存管理', order: 3 },
  storage: { name: '庫存管理', order: 3 },
  storage_location: { name: '庫存管理', order: 3 },
  equipment: { name: '庫存管理', order: 3 },

  // HR（人事管理：出勤、請假、加班）
  hr: { name: '管理階級', order: 4 },
  leave: { name: '管理階級', order: 4 },
  attendance: { name: '管理階級', order: 4 },
  overtime: { name: '管理階級', order: 4 },
  balance: { name: '管理階級', order: 4 },

  // Facility（設施管理：設施、建築物、區域、欄位）
  facility: { name: '管理階級', order: 4 },
  building: { name: '管理階級', order: 4 },
  zone: { name: '管理階級', order: 4 },
  pen: { name: '管理階級', order: 4 },

  // Audit（稽核：報表、稽核紀錄）
  audit: { name: '管理階級', order: 4 },
  report: { name: '管理階級', order: 4 },

  // QAU（GLP 品質保證：檢視計畫、動物、稽核、儀表板）
  qau: { name: '管理階級', order: 4 },

  // Training（人員訓練紀錄，歸於 HR）
  training: { name: '管理階級', order: 4 },

  // 系統管理（使用者、角色、權限）
  admin: { name: '管理階級', order: 4 },
  user: { name: '管理階級', order: 4 },
  role: { name: '管理階級', order: 4 },
  permission: { name: '管理階級', order: 4 },
  system: { name: '管理階級', order: 4 },

  // 其他管理功能（通知、部門、行事曆、儀表板）
  notification: { name: '管理階級', order: 4 },
  department: { name: '管理階級', order: 4 },
  calendar: { name: '管理階級', order: 4 },
  dashboard: { name: '管理階級', order: 4 },

  // 開發工具
  dev: { name: '開發工具', order: 5 },

  // 其他（未分類）
  other: { name: '其他', order: 99 },
}

// 類別顯示名稱 - 中文
export const CATEGORY_NAMES: Record<string, Record<string, string>> = {
  admin: {
    user: '使用者',
    role: '角色',
    permission: '權限',
    audit: '稽核',
  },
  aup: {
    protocol: '計畫',
    review: '審查',
    attachment: '附件',
    version: '版本',
  },
  animal: {
    animal: '動物',
    record: '紀錄',
    blood_test_template: '血檢項目',
    vet: '獸醫',
    export: '匯出',
    pathology: '病理',
    source: '來源',
  },
  equipment: {
    equipment: '設備',
  },
  qau: {
    QAU: 'QAU',
    dashboard: '儀表板',
    protocol: '計畫',
    audit: '稽核',
    animal: '動物',
    inspection: '稽查報告',
    nc: '不符合事項',
    sop: 'SOP 文件',
    schedule: '稽查排程',
  },
  erp: {
    warehouse: '倉庫',
    product: '產品',
    partner: '合作夥伴',
    document: '文件',
    purchase: '採購',
    grn: '收貨',
    sales: '銷貨',
    do: '出貨',
    stock: '庫存',
    stocktake: '盤點',
    report: '報表',
    po: '採購單',
    so: '銷貨單',
    tr: '調撥',
    stk: '盤點',
    adj: '庫存調整',
    inventory: '庫存現況',
    create: '新增',
    approve: '審核',
    cancel: '取消',
    submit: '提交',
    update: '更新',
    delete: '刪除',
    view: '檢視',
    read: '讀取',
    edit: '編輯',
    schedule: '排程',
    download: '下載',
  },
  dev: {
    user: '使用者',
    role: '角色',
    permission: '權限',
    system: '系統',
    audit: '稽核',
    log: '日誌',
    notification: '通知',
    database: '資料庫',
    create: '新增',
    view: '檢視',
    read: '讀取',
    edit: '編輯',
    update: '更新',
    delete: '刪除',
    manage: '管理',
    assign: '指派',
    reset_password: '重設密碼',
    export: '匯出',
    download: '下載',
    query: '查詢',
    migrate: '遷移',
    seed: '初始資料',
    send: '發送',
    backup: '備份',
    restore: '還原',
    trigger: '觸發',
    schedule: '排程',
    upload: '上傳',
  },
  notification: {
    manage: '管理',
    send: '發送',
    trigger: '觸發',
    view: '檢視',
    notification: '通知',
  },
  report: {
    download: '下載',
    schedule: '排程',
    view: '檢視',
    export: '匯出',
    report: '報表',
  },
  // 合併模組類別 (兩段式權限碼如 species.create)
  species: {
    species: '物種',
    create: '新增',
    read: '讀取',
    update: '更新',
    delete: '刪除',
  },
  department: {
    department: '部門',
  },
  facility: {
    facility: '設施',
    building: '建築物',
    zone: '區域',
    pen: '欄位',
  },
  building: {
    building: '建築物',
  },
  zone: {
    zone: '區域',
  },
  pen: {
    pen: '欄位',
  },
  record: {
    record: '紀錄',
  },
  source: {
    source: '來源',
  },
  audit: {
    audit: '稽核',
    alerts: '警示',
    logs: '日誌',
    timeline: '時間軸',
  },
  user: {
    user: '使用者',
  },
  role: {
    role: '角色',
  },
  permission: {
    permission: '權限',
  },
  // HR 人事模組類別
  hr: {
    leave: '請假',
    attendance: '出勤',
    overtime: '加班',
    balance: '假期餘額',
    hr: '人事',
  },
  // 行事曆模組
  calendar: {
    calendar: '行事曆',
  },
  // 通用類別（用於合併模組顯示）
  leave: {
    leave: '請假',
  },
  attendance: {
    attendance: '出勤',
  },
  overtime: {
    overtime: '加班',
  },
  balance: {
    balance: '假期餘額',
  },
  // 合併類別顯示名稱
  merged: {
    HR: 'HR',
    Facility: 'Facility',
    Audit: 'Audit',
    '系統管理': '系統管理',
    '匯入': '匯入',
    '文件': '文件',
    '庫存': '庫存',
  },
}

// 操作顯示名稱 - 中文
export const OPERATION_NAMES: Record<string, string> = {
  create: '新增',
  approve: '審核',
  cancel: '取消',
  submit: '提交',
  update: '更新',
  delete: '刪除',
  view: '檢視',
  read: '讀取',
  edit: '編輯',
  manage: '管理',
  assign: '指派',
  reset_password: '重設密碼',
  export: '匯出',
  download: '下載',
  query: '查詢',
  migrate: '遷移',
  seed: '初始資料',
  send: '發送',
  backup: '備份',
  restore: '還原',
  upload: '上傳',
  trigger: '觸發',
  schedule: '排程',
}

// 類別合併映射 - 將某些類別合併到其他類別
export const CATEGORY_MERGE_MAP: Record<string, string> = {
  // 庫存管理：文件類別
  po: '文件',
  so: '文件',
  grn: '文件',
  do: '文件',
  purchase: '文件',
  sales: '文件',
  pr: '文件',
  document: '文件',
  // 庫存管理：庫存類別
  stock: '庫存',
  stocktake: '庫存',
  stk: '庫存',
  tr: '庫存',
  adj: '庫存',
  inventory: '庫存',
  // 動物管理：匯入類別
  weight: '匯入',
  info: '匯入',
  // HR 相關類別合併（含 Training）
  leave: 'HR',
  attendance: 'HR',
  overtime: 'HR',
  balance: 'HR',
  hr: 'HR',
  training: 'HR',
  // Facility 相關類別合併
  facility: 'Facility',
  building: 'Facility',
  zone: 'Facility',
  pen: 'Facility',
  // Audit 相關類別合併（包含報表）
  audit: 'Audit',
  report: 'Audit',
  logs: 'Audit',
  alerts: 'Audit',
  timeline: 'Audit',
  // 系統管理相關類別合併
  user: '系統管理',
  role: '系統管理',
  permission: '系統管理',
  admin: '系統管理',
  // 其他類別映射
  storage_location: 'Storage Location',
  dashboard: 'Dashboard',
  amendment: 'Amendment',
}

// 三層結構配置 - 定義哪些類別需要拆分成子類別
export const SUB_CATEGORY_CONFIG: Record<string, string[]> = {
  Facility: ['zone', 'facility', 'building', 'pen'],
  HR: ['attendance', 'leave', 'overtime', 'balance', 'hr', 'training'],
  QAU: ['protocol', 'animal', 'audit', 'dashboard', 'inspection', 'nc', 'sop', 'schedule'],
}

// 子類別顯示名稱
export const SUB_CATEGORY_NAMES: Record<string, string> = {
  // Facility
  zone: '區域',
  facility: '設施',
  building: '棟舍',
  pen: '欄位',
  // HR
  attendance: '出勤',
  leave: '請假',
  overtime: '加班',
  balance: '假期餘額',
  hr: '人事',
  training: 'Training',
  // QAU
  protocol: '檢視計畫',
  animal: '檢視動物',
  audit: '檢視稽核',
  dashboard: '查看QAU儀表板',
  inspection: '稽查報告',
  nc: '不符合事項',
  sop: 'SOP 文件',
  schedule: '稽查排程',
  // 基於名稱的子群組
  '新增紀錄': '新增紀錄',
  '建立單據': '建立單據',
}

// 基於權限名稱的子群組配置
export const PERMISSION_NAME_SUBGROUPS: Record<string, Record<string, string[]>> = {
  record: {
    '新增紀錄': ['新增手術紀錄', '新增疫苗紀錄', '新增犧牲紀錄', '新增體重紀錄', '新增觀察紀錄'],
  },
  '文件': {
    '建立單據': ['建立採購退貨', '建立採購單', '建立進貨單'],
  },
}

// 模組顯示名稱 -> 排序用 order（取該名稱下最小 order）
export const MODULE_NAME_ORDER = (() => {
  const map = new Map<string, number>()
  for (const config of Object.values(MODULE_CONFIG)) {
    const current = map.get(config.name)
    if (current === undefined || config.order < current) {
      map.set(config.name, config.order)
    }
  }
  return map
})()

/**
 * 取得權限所屬模組（用於 UI 分組）
 * dev.role.* 在畫面上歸到「管理階級－系統管理」，與 admin 使用者/角色/權限同一區塊
 */
export function getPermissionModule(perm: Permission): string {
  if (perm.code.startsWith('dev.role.')) {
    return 'admin'
  }
  if (perm.module) {
    return perm.module
  }

  const prefix = perm.code.split('.')[0]
  if (MODULE_CONFIG[prefix]) {
    return prefix
  }

  return 'other'
}

// 獲取權限的原始子類別（用於三層結構分組）
export function getPermissionRawCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
  if (parts.length === 2) {
    return parts[0]
  }
  return parts[1]
}

export function getPermissionCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'

  let category: string
  if (parts.length === 2) {
    category = parts[0]
  } else {
    category = parts[1]
  }

  if (CATEGORY_MERGE_MAP[category]) {
    return CATEGORY_MERGE_MAP[category]
  }

  return category
}

/**
 * 取得類別顯示名稱
 */
export function getCategoryName(module: string, category: string): string {
  if (CATEGORY_NAMES[module]?.[category]) {
    return CATEGORY_NAMES[module][category]
  }

  for (const mod in CATEGORY_NAMES) {
    if (CATEGORY_NAMES[mod][category]) {
      return CATEGORY_NAMES[mod][category]
    }
  }

  if (OPERATION_NAMES[category]) {
    return OPERATION_NAMES[category]
  }

  return category
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .toLowerCase()
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ') || category
}
