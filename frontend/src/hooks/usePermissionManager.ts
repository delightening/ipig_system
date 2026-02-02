import { useMemo, useState } from 'react'
import { Permission } from '@/lib/api'

// ?????????
export interface PermissionGroup {
  module: string
  moduleName: string
  moduleOrder: number
  categories: {
    category: string
    categoryName: string
    permissions: Permission[]
  }[]
}

// 模組配置 - 中文顯示名稱
const MODULE_CONFIG: Record<string, { name: string; order: number }> = {
  // 動物使用計畫
  aup: { name: '動物使用計畫', order: 1 },

  // 動物管理（含物種管理、動物來源、緊急給藥等）
  pig: { name: '動物管理', order: 2 },
  animal: { name: '動物管理', order: 2 },
  species: { name: '動物管理', order: 2 },

  // 庫存管理
  erp: { name: '庫存管理', order: 3 },

  // 管理階級（通知、報表、人事、設施、部門、稽核等）
  notification: { name: '管理階級', order: 4 },
  report: { name: '管理階級', order: 4 },
  hr: { name: '管理階級', order: 4 },
  facility: { name: '管理階級', order: 4 },
  building: { name: '管理階級', order: 4 },
  zone: { name: '管理階級', order: 4 },
  pen: { name: '管理階級', order: 4 },
  department: { name: '管理階級', order: 4 },
  audit: { name: '管理階級', order: 4 },
  calendar: { name: '管理階級', order: 4 },

  // 系統管理（使用者、角色、權限）
  admin: { name: '系統管理', order: 5 },
  user: { name: '系統管理', order: 5 },
  role: { name: '系統管理', order: 5 },
  permission: { name: '系統管理', order: 5 },
  system: { name: '系統管理', order: 5 },

  // 開發工具
  dev: { name: '開發工具', order: 6 },

  // 其他
  record: { name: '其他', order: 99 },
  source: { name: '其他', order: 99 },
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
  pig: {
    pig: '動物',
    record: '紀錄',
    vet: '獸醫',
    export: '匯出',
    pathology: '病理',
    source: '來源',
  },
  animal: {
    record: '紀錄',
    source: '來源',
  },
  erp: {
    warehouse: '倉庫',
    product: '產品',
    partner: '合作夥伴',
    document: '文件',
    purchase: '採購',
    grn: '收貨',
    pr: '採購申請',
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
    inventory: '庫存',
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
function getPermissionCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
  // For simple module.action permissions (2 parts), use module as category
  // e.g., species.create -> category 'species' (same as module)
  // For complex module.entity.action permissions (3+ parts), use entity as category
  // e.g., pig.record.create -> category 'record'
  if (parts.length === 2) {
    return parts[0]
  }
  return parts[1]
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
 * ?????? Hook
 */
export function usePermissionManager(permissions: Permission[] | undefined) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedModule, setSelectedModule] = useState<string | null>(null)
  const [expandedModules, setExpandedModules] = useState<Set<string>>(new Set())
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set())

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
    // 第一步：按模組顯示名稱分組
    const groupsByName = new Map<string, {
      moduleOrder: number
      categories: Map<string, Permission[]>
    }>()

    uniquePermissions.forEach(perm => {
      const moduleCode = getPermissionModule(perm)
      const moduleConfig = MODULE_CONFIG[moduleCode] || MODULE_CONFIG.other
      const moduleName = moduleConfig.name
      const moduleOrder = moduleConfig.order

      // 對於合併的模組（相同 moduleName），使用原始 moduleCode 作為類別
      // 對於 2 段式權限碼，類別就是 moduleCode 本身
      // 對於 3+ 段式權限碼，類別是第二段
      const rawCategory = getPermissionCategory(perm.code)
      // 如果類別和模組碼相同（2段式權限），使用模組碼作為類別
      const category = rawCategory === moduleCode ? moduleCode : rawCategory

      if (!groupsByName.has(moduleName)) {
        groupsByName.set(moduleName, {
          moduleOrder,
          categories: new Map()
        })
      }
      const group = groupsByName.get(moduleName)!

      if (!group.categories.has(category)) {
        group.categories.set(category, [])
      }

      group.categories.get(category)!.push(perm)
    })

    // 第二步：轉換為結果格式
    const result: PermissionGroup[] = Array.from(groupsByName.entries())
      .map(([moduleName, { moduleOrder, categories }]) => {
        return {
          module: moduleName, // 使用顯示名稱作為 module key
          moduleName: moduleName,
          moduleOrder: moduleOrder,
          categories: Array.from(categories.entries())
            .map(([category, perms]) => {
              const seen = new Set<string>()
              const uniquePerms = perms.filter(perm => {
                if (seen.has(perm.id)) return false
                seen.add(perm.id)
                return true
              })

              // 獲取類別顯示名稱 - 優先從原始模組碼查找
              let categoryName = getCategoryName(category, category)
              // 如果找不到，嘗試從 CATEGORY_NAMES 中查找（使用類別作為模組鍵）
              if (categoryName === category && CATEGORY_NAMES[category]?.[category]) {
                categoryName = CATEGORY_NAMES[category][category]
              }

              return {
                category,
                categoryName,
                permissions: uniquePerms.sort((a, b) => a.name.localeCompare(b.name)),
              }
            })
            .filter(cat => cat.permissions.length > 0)
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

  // ??????????????
  const isCategoryExpanded = (module: string, category: string) => {
    return expandedCategories.has(`${module}.${category}`)
  }

  return {
    // ???
    groupedPermissions: filteredGroups,
    stats,

    // ???
    searchQuery,
    setSearchQuery,

    // ???
    selectedModule,
    setSelectedModule,

    // ???/???
    toggleModule,
    toggleCategory,
    expandAllModules,
    collapseAllModules,
    isModuleExpanded,
    isCategoryExpanded,
  }
}




