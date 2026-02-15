import { useMemo, useState } from 'react'
import { Permission } from '@/lib/api'

// 子類別介面（用於三層結構）
export interface PermissionSubCategory {
  subCategory: string
  subCategoryName: string
  permissions: Permission[]
}

// 類別介面
export interface PermissionCategory {
  category: string
  categoryName: string
  permissions: Permission[]
  // 子類別（用於三層結構）- 如果有 subCategories，直接 permissions 應為空
  subCategories?: PermissionSubCategory[]
}

// 權限群組介面
export interface PermissionGroup {
  module: string
  moduleName: string
  moduleOrder: number
  categories: PermissionCategory[]
}

// 模組配置 - 中文顯示名稱
const MODULE_CONFIG: Record<string, { name: string; order: number }> = {
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

  // 庫存管理（包含資料庫中的 ERP 和 erp）
  erp: { name: '庫存管理', order: 3 },
  ERP: { name: '庫存管理', order: 3 },
  storage: { name: '庫存管理', order: 3 },
  storage_location: { name: '庫存管理', order: 3 },

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
const CATEGORY_NAMES: Record<string, Record<string, string>> = {
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
    vet: '獸醫',
    export: '匯出',
    pathology: '病理',
    source: '來源',
  },
  erp: {
    warehouse: '倉庫',
    product: '產品',
    partner: '合作夥伴',
    document: '文件',
    purchase: '採購',
    grn: '收貨',
    sales: '銷售',
    do: '出貨',
    stock: '庫存',
    stocktake: '盤點',
    report: '報表',
    po: '採購單',
    so: '銷售單',
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
const OPERATION_NAMES: Record<string, string> = {
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
/**
 * 取得權限所屬模組
 */
function getPermissionModule(perm: Permission): string {
  if (perm.module) {
    return perm.module
  }

  const prefix = perm.code.split('.')[0]

  // 檢查是否為已知模組
  if (MODULE_CONFIG[prefix]) {
    return prefix
  }

  // 回退至 other
  return 'other'
}

// 類別合併映射 - 將某些類別合併到其他類別
const CATEGORY_MERGE_MAP: Record<string, string> = {
  // 庫存管理：文件類別
  po: '文件', // 採購單
  so: '文件', // 銷售單
  grn: '文件', // 收貨單
  do: '文件', // 出貨單
  purchase: '文件', // 採購
  sales: '文件', // 銷售
  pr: '文件', // 採購申請
  document: '文件', // 文件
  // 庫存管理：庫存類別
  stock: '庫存', // 庫存
  stocktake: '庫存', // 盤點
  stk: '庫存', // 盤點
  tr: '庫存', // 調撥
  adj: '庫存', // 庫存調整
  inventory: '庫存', // 庫存現況
  // 動物管理：匯入類別
  weight: '匯入', // 將體重匯入歸類到匯入
  info: '匯入', // 將基本資料匯入歸類到匯入
  // HR 相關類別合併
  leave: 'HR',
  attendance: 'HR',
  overtime: 'HR',
  balance: 'HR',
  hr: 'HR',
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
// key: 合併後的類別名稱, value: 原始子類別列表
const SUB_CATEGORY_CONFIG: Record<string, string[]> = {
  // 管理階級 > Facility > [區域, 設施, 棟舍, 欄位]
  Facility: ['zone', 'facility', 'building', 'pen'],
  // 管理階級 > HR > [出勤, 請假, 加班, 假期餘額, 人事]
  HR: ['attendance', 'leave', 'overtime', 'balance', 'hr'],
}

// 子類別顯示名稱
const SUB_CATEGORY_NAMES: Record<string, string> = {
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
  // 基於名稱的子群組
  '新增紀錄': '新增紀錄',
  '建立單據': '建立單據',
}

// 基於權限名稱的子群組配置
// key: 類別名稱, value: { subGroupName: [包含的權限名稱模式] }
const PERMISSION_NAME_SUBGROUPS: Record<string, Record<string, string[]>> = {
  // 紀錄類別：將新增__紀錄分組到「新增紀錄」
  record: {
    '新增紀錄': ['新增手術紀錄', '新增疫苗紀錄', '新增犧牲紀錄', '新增體重紀錄', '新增觀察紀錄'],
  },
  // 文件類別：將建立__分組到「建立單據」
  '文件': {
    '建立單據': ['建立採購退貨', '建立採購單', '建立進貨單'],
  },
}

// 獲取權限的原始子類別（用於三層結構分組）
function getPermissionRawCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
  if (parts.length === 2) {
    return parts[0]
  }
  return parts[1]
}

function getPermissionCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
  // For simple module.action permissions (2 parts), use module as category
  // e.g., species.create -> category 'species' (same as module)
  // For complex module.entity.action permissions (3+ parts), use entity as category
  // e.g., animal.record.create -> category 'record'
  let category: string
  if (parts.length === 2) {
    category = parts[0]
  } else {
    category = parts[1]
  }

  // 應用類別合併映射
  if (CATEGORY_MERGE_MAP[category]) {
    return CATEGORY_MERGE_MAP[category]
  }

  return category
}

/**
 * ??????????????
 */
function getCategoryName(module: string, category: string): string {
  // ?????????????????
  if (CATEGORY_NAMES[module]?.[category]) {
    return CATEGORY_NAMES[module][category]
  }

  // ?????????
  for (const mod in CATEGORY_NAMES) {
    if (CATEGORY_NAMES[mod][category]) {
      return CATEGORY_NAMES[mod][category]
    }
  }

  // ????????????
  if (OPERATION_NAMES[category]) {
    return OPERATION_NAMES[category]
  }

  // ???????????
  return category
    .replace(/_/g, ' ')
    .replace(/([a-z])([A-Z])/g, '$1 $2')
    .toLowerCase()
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ') || category
}

/**
 * 權限管理 Hook
 */
export function usePermissionManager(permissions: Permission[] | undefined) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedModule, setSelectedModule] = useState<string | null>(null)
  const [expandedModules, setExpandedModules] = useState<Set<string>>(new Set())
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set())
  const [expandedSubCategories, setExpandedSubCategories] = useState<Set<string>>(new Set())

  // ??????
  const uniquePermissions = useMemo(() => {
    if (!permissions) return []
    const seen = new Set<string>()
    return permissions.filter(perm => {
      if (seen.has(perm.id)) return false
      seen.add(perm.id)
      return true
    })
  }, [permissions])

  // 根據模組顯示名稱分組權限（相同顯示名稱的模組會合併）
  // 例如 notification、hr、facility 都映射到「管理階級」會合併成一組
  const groupedPermissions = useMemo(() => {
    // 第一步：按模組顯示名稱 > 類別 > 子類別分組
    const groupsByName = new Map<string, {
      moduleOrder: number
      categories: Map<string, Map<string, Permission[]>> // category -> subCategory -> permissions
    }>()

    uniquePermissions.forEach(perm => {
      const moduleCode = getPermissionModule(perm)
      const moduleConfig = MODULE_CONFIG[moduleCode] || MODULE_CONFIG.other
      const moduleName = moduleConfig.name
      const moduleOrder = moduleConfig.order

      // 獲取合併後的類別
      const rawCategory = getPermissionCategory(perm.code)
      const category = rawCategory === moduleCode ? moduleCode : rawCategory

      // 獲取原始子類別（未合併）
      const rawSubCategory = getPermissionRawCategory(perm.code)

      if (!groupsByName.has(moduleName)) {
        groupsByName.set(moduleName, {
          moduleOrder,
          categories: new Map()
        })
      }
      const group = groupsByName.get(moduleName)!

      if (!group.categories.has(category)) {
        group.categories.set(category, new Map())
      }
      const categoryMap = group.categories.get(category)!

      if (!categoryMap.has(rawSubCategory)) {
        categoryMap.set(rawSubCategory, [])
      }
      categoryMap.get(rawSubCategory)!.push(perm)
    })

    // 第二步：轉換為結果格式，處理三層結構
    const result: PermissionGroup[] = Array.from(groupsByName.entries())
      .map(([moduleName, { moduleOrder, categories }]) => {
        return {
          module: moduleName, // 使用顯示名稱作為 module key
          moduleName: moduleName,
          moduleOrder: moduleOrder,
          categories: Array.from(categories.entries())
            .map(([category, subCategoryMap]) => {
              // 獲取類別顯示名稱
              let categoryName = getCategoryName(category, category)
              if (categoryName === category && CATEGORY_NAMES[category]?.[category]) {
                categoryName = CATEGORY_NAMES[category][category]
              }

              // 檢查是否需要三層結構（基於權限碼類別）
              const needsSubCategories = SUB_CATEGORY_CONFIG[category] !== undefined

              // 檢查是否需要基於名稱的子群組
              const nameSubgroups = PERMISSION_NAME_SUBGROUPS[category]

              if (needsSubCategories) {
                // 三層結構：按子類別分組（基於權限碼）
                const subCategories: PermissionSubCategory[] = Array.from(subCategoryMap.entries())
                  .map(([subCat, perms]) => {
                    const seen = new Set<string>()
                    const uniquePerms = perms.filter(perm => {
                      if (seen.has(perm.id)) return false
                      seen.add(perm.id)
                      return true
                    })

                    return {
                      subCategory: subCat,
                      subCategoryName: SUB_CATEGORY_NAMES[subCat] || subCat,
                      permissions: uniquePerms.sort((a, b) => a.name.localeCompare(b.name)),
                    }
                  })
                  .filter(subCat => subCat.permissions.length > 0)
                  .sort((a, b) => a.subCategoryName.localeCompare(b.subCategoryName))

                return {
                  category,
                  categoryName,
                  permissions: [], // 三層結構時直接 permissions 為空
                  subCategories,
                }
              } else if (nameSubgroups) {
                // 三層結構：基於權限名稱分組
                const allPerms: Permission[] = []
                subCategoryMap.forEach(perms => allPerms.push(...perms))

                const seen = new Set<string>()
                const uniquePerms = allPerms.filter(perm => {
                  if (seen.has(perm.id)) return false
                  seen.add(perm.id)
                  return true
                })

                // 建立子群組
                const subCategories: PermissionSubCategory[] = []
                const ungroupedPerms: Permission[] = []

                // 遍歷每個權限，分配到對應的子群組
                uniquePerms.forEach(perm => {
                  let assigned = false
                  for (const [subGroupName, permNames] of Object.entries(nameSubgroups)) {
                    if (permNames.includes(perm.name)) {
                      // 找到或建立子群組
                      let subCat = subCategories.find(s => s.subCategory === subGroupName)
                      if (!subCat) {
                        subCat = {
                          subCategory: subGroupName,
                          subCategoryName: SUB_CATEGORY_NAMES[subGroupName] || subGroupName,
                          permissions: [],
                        }
                        subCategories.push(subCat)
                      }
                      subCat.permissions.push(perm)
                      assigned = true
                      break
                    }
                  }
                  if (!assigned) {
                    ungroupedPerms.push(perm)
                  }
                })

                // 排序子群組內的權限
                subCategories.forEach(subCat => {
                  subCat.permissions.sort((a, b) => a.name.localeCompare(b.name))
                })
                subCategories.sort((a, b) => a.subCategoryName.localeCompare(b.subCategoryName))

                return {
                  category,
                  categoryName,
                  permissions: ungroupedPerms.sort((a, b) => a.name.localeCompare(b.name)), // 未分組的權限
                  subCategories,
                }
              } else {
                // 兩層結構：合併所有子類別的權限
                const allPerms: Permission[] = []
                subCategoryMap.forEach(perms => allPerms.push(...perms))

                const seen = new Set<string>()
                const uniquePerms = allPerms.filter(perm => {
                  if (seen.has(perm.id)) return false
                  seen.add(perm.id)
                  return true
                })

                return {
                  category,
                  categoryName,
                  permissions: uniquePerms.sort((a, b) => a.name.localeCompare(b.name)),
                }
              }
            })
            .filter(cat => cat.permissions.length > 0 || (cat.subCategories && cat.subCategories.length > 0))
            .sort((a, b) => a.categoryName.localeCompare(b.categoryName)),
        }
      })
      .filter(group => group.categories.length > 0)
      .sort((a, b) => a.moduleOrder - b.moduleOrder)

    return result
  }, [uniquePermissions])

  // ???????????????????????
  const filteredGroups = useMemo(() => {
    let filtered = groupedPermissions

    // ????????
    if (selectedModule) {
      filtered = filtered.filter(g => g.module === selectedModule)
    }

    // ????????????
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase().trim()
      filtered = filtered.map(group => ({
        ...group,
        categories: group.categories
          .map(cat => ({
            ...cat,
            permissions: cat.permissions.filter(perm =>
              perm.name.toLowerCase().includes(query) ||
              perm.code.toLowerCase().includes(query) ||
              perm.description?.toLowerCase().includes(query)
            ),
          }))
          .filter(cat => cat.permissions.length > 0),
      })).filter(group => group.categories.length > 0)
    }

    return filtered
  }, [groupedPermissions, searchQuery, selectedModule])

  // ??????
  const stats = useMemo(() => {
    const total = uniquePermissions.length
    const moduleCounts = new Map<string, number>()

    uniquePermissions.forEach(perm => {
      const module = getPermissionModule(perm)
      moduleCounts.set(module, (moduleCounts.get(module) || 0) + 1)
    })

    return {
      total,
      moduleCounts: Object.fromEntries(moduleCounts),
    }
  }, [uniquePermissions])

  // ?????????????
  const toggleModule = (module: string) => {
    setExpandedModules(prev => {
      const next = new Set(prev)
      if (next.has(module)) {
        next.delete(module)
      } else {
        next.add(module)
      }
      return next
    })
  }

  // ??????????????
  const toggleCategory = (module: string, category: string) => {
    const key = `${module}.${category}`
    setExpandedCategories(prev => {
      const next = new Set(prev)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return next
    })
  }

  // ???/??????????
  const expandAllModules = () => {
    setExpandedModules(new Set(groupedPermissions.map(g => g.module)))
  }

  const collapseAllModules = () => {
    setExpandedModules(new Set())
  }

  // ????????????
  const isModuleExpanded = (module: string) => expandedModules.has(module)

  // 檢查類別是否展開
  const isCategoryExpanded = (module: string, category: string) => {
    return expandedCategories.has(`${module}.${category}`)
  }

  // 切換子類別展開狀態
  const toggleSubCategory = (module: string, category: string, subCategory: string) => {
    const key = `${module}.${category}.${subCategory}`
    setExpandedSubCategories(prev => {
      const next = new Set(prev)
      if (next.has(key)) {
        next.delete(key)
      } else {
        next.add(key)
      }
      return next
    })
  }

  // 檢查子類別是否展開
  const isSubCategoryExpanded = (module: string, category: string, subCategory: string) => {
    return expandedSubCategories.has(`${module}.${category}.${subCategory}`)
  }

  return {
    // 資料
    groupedPermissions: filteredGroups,
    stats,

    // 搜尋
    searchQuery,
    setSearchQuery,

    // 篩選
    selectedModule,
    setSelectedModule,

    // 展開/收合
    toggleModule,
    toggleCategory,
    toggleSubCategory,
    expandAllModules,
    collapseAllModules,
    isModuleExpanded,
    isCategoryExpanded,
    isSubCategoryExpanded,
  }
}




