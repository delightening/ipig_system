import { useMemo, useState } from 'react'
import { Permission } from '@/lib/api'

// 權限分組結構
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

// 模組配置
const MODULE_CONFIG: Record<string, { name: string; order: number }> = {
  aup: { name: 'AUP（動物使用計畫）', order: 1 },
  pig: { name: '豬隻管理', order: 2 },
  erp: { name: 'ERP管理', order: 3 },
  dev: { name: '程式設計', order: 4 },
  notification: { name: '通知管理', order: 5 },
  report: { name: '報表管理', order: 6 },
  other: { name: '其他', order: 99 },
}

// 子類別名稱映射
const CATEGORY_NAMES: Record<string, Record<string, string>> = {
  aup: {
    protocol: '計畫管理',
    review: '審查流程',
    attachment: '附件管理',
    version: '版本管理',
  },
  pig: {
    pig: '豬隻資料',
    record: '紀錄管理',
    vet: '獸醫師功能',
    export: '匯出功能',
    pathology: '病理報告',
  },
  erp: {
    warehouse: '倉庫管理',
    product: '產品管理',
    partner: '夥伴管理',
    document: '單據管理',
    purchase: '採購作業',
    grn: '採購入庫',
    pr: '採購退貨',
    sales: '銷售作業',
    do: '銷售出庫',
    sr: '銷售退貨',
    stock: '庫存作業',
    stocktake: '盤點作業',
    report: '報表管理',
    po: '採購單',
    so: '銷售單',
    tr: '調撥單',
    stk: '盤點單',
    adj: '調整單',
    inventory: '庫存管理',
    create: '建立作業',
    approve: '核准作業',
    cancel: '作廢作業',
    submit: '送審作業',
    update: '更新作業',
    delete: '刪除作業',
    view: '查看作業',
    read: '檢視作業',
    edit: '編輯作業',
    schedule: '排程作業',
    download: '下載作業',
  },
  dev: {
    user: '使用者管理',
    role: '角色管理',
    permission: '權限管理',
    system: '系統設定',
    audit: '稽核與日誌',
    log: '系統日誌',
    notification: '通知管理',
    database: '資料庫管理',
    create: '建立作業',
    view: '查看作業',
    read: '檢視作業',
    edit: '編輯作業',
    update: '更新作業',
    delete: '刪除作業',
    manage: '管理作業',
    assign: '指派作業',
    reset_password: '重設密碼',
    export: '匯出作業',
    download: '下載作業',
    query: '查詢作業',
    migrate: '遷移作業',
    seed: '種子資料',
    send: '發送作業',
    backup: '備份作業',
    restore: '還原作業',
    trigger: '觸發作業',
    schedule: '排程作業',
    upload: '上傳作業',
  },
  notification: {
    manage: '管理',
    send: '發送',
    trigger: '觸發',
    view: '查看',
  },
  report: {
    download: '下載',
    schedule: '排程',
    view: '查看',
    export: '匯出',
  },
}

// 通用操作名稱映射
const OPERATION_NAMES: Record<string, string> = {
  create: '建立',
  approve: '核准',
  cancel: '作廢',
  submit: '送審',
  update: '更新',
  delete: '刪除',
  view: '查看',
  read: '檢視',
  edit: '編輯',
  manage: '管理',
  assign: '指派',
  reset_password: '重設密碼',
  export: '匯出',
  download: '下載',
  query: '查詢',
  migrate: '遷移',
  seed: '種子資料',
  send: '發送',
  backup: '備份',
  restore: '還原',
  upload: '上傳',
  trigger: '觸發',
  schedule: '排程',
}

/**
 * 獲取權限的模組
 */
function getPermissionModule(perm: Permission): string {
  if (perm.module) {
    // 映射舊模組名稱
    if (perm.module === 'animal') return 'pig'
    if (perm.module === 'notification') return 'dev'
    if (perm.module === 'report') return 'erp'
    return perm.module
  }
  
  // 根據 code 前綴判斷
  const prefix = perm.code.split('.')[0]
  if (prefix === 'aup') return 'aup'
  if (prefix === 'pig' || prefix === 'animal') return 'pig'
  if (prefix === 'erp' || prefix === 'report' || 
      ['warehouse', 'product', 'partner', 'document', 'po', 'grn', 'pr', 
       'so', 'do', 'sr', 'tr', 'stk', 'adj', 'stock'].includes(prefix)) {
    return 'erp'
  }
  if (prefix === 'dev' || prefix === 'admin' || prefix === 'user' || 
      prefix === 'role' || prefix === 'notification') {
    return 'dev'
  }
  return 'other'
}

/**
 * 獲取權限的子類別
 */
function getPermissionCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
  return parts[1]
}

/**
 * 獲取子類別中文名稱
 */
function getCategoryName(module: string, category: string): string {
  // 優先使用模組特定的映射
  if (CATEGORY_NAMES[module]?.[category]) {
    return CATEGORY_NAMES[module][category]
  }
  
  // 檢查其他模組
  for (const mod in CATEGORY_NAMES) {
    if (CATEGORY_NAMES[mod][category]) {
      return CATEGORY_NAMES[mod][category]
    }
  }
  
  // 使用通用操作名稱
  if (OPERATION_NAMES[category]) {
    return OPERATION_NAMES[category]
  }
  
  // 格式化英文名稱
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

  // 去重權限
  const uniquePermissions = useMemo(() => {
    if (!permissions) return []
    const seen = new Set<string>()
    return permissions.filter(perm => {
      if (seen.has(perm.id)) return false
      seen.add(perm.id)
      return true
    })
  }, [permissions])

  // 分組權限（確保每個權限 ID 在整個樹中只出現一次）
  // 優先使用更完整的 code（例如 pig.record.* 優先於 animal.record.*）
  const groupedPermissions = useMemo(() => {
    const groups = new Map<string, Map<string, Permission[]>>()
    const usedPermissionIds = new Set<string>() // 追蹤已使用的權限 ID
    const permissionByCode = new Map<string, Permission>() // 按 code 索引，用於去重

    // 首先按 code 去重，保留更完整的 code（更長的優先，或優先保留 pig.* 而非 animal.*）
    uniquePermissions.forEach(perm => {
      const existing = permissionByCode.get(perm.code)
      if (existing) {
        // 如果已存在，優先保留 pig.* 而非 animal.*，或保留 code 更長的那個
        const shouldReplace = 
          perm.code.startsWith('pig.') && existing.code.startsWith('animal.') ||
          (!existing.code.startsWith('pig.') && perm.code.length > existing.code.length)
        
        if (shouldReplace) {
          permissionByCode.set(perm.code, perm)
        }
      } else {
        permissionByCode.set(perm.code, perm)
      }
    })

    // 使用去重後的權限進行分組
    Array.from(permissionByCode.values()).forEach(perm => {
      // 如果這個權限已經被使用過，跳過
      if (usedPermissionIds.has(perm.id)) {
        return
      }

      const module = getPermissionModule(perm)
      const category = getPermissionCategory(perm.code)

      if (!groups.has(module)) {
        groups.set(module, new Map())
      }
      const moduleMap = groups.get(module)!
      
      if (!moduleMap.has(category)) {
        moduleMap.set(category, [])
      }
      
      const categoryPerms = moduleMap.get(category)!
      categoryPerms.push(perm)
      usedPermissionIds.add(perm.id)
    })

    // 轉換為結構化格式，並在每個子類別中去重
    const result: PermissionGroup[] = Array.from(groups.entries())
      .map(([module, categories]) => {
        const moduleConfig = MODULE_CONFIG[module] || MODULE_CONFIG.other
        return {
          module,
          moduleName: moduleConfig.name,
          moduleOrder: moduleConfig.order,
          categories: Array.from(categories.entries())
            .map(([category, perms]) => {
              // 在子類別層級再次去重（基於 ID）
              const seen = new Set<string>()
              const uniquePerms = perms.filter(perm => {
                if (seen.has(perm.id)) return false
                seen.add(perm.id)
                return true
              })
              
              return {
                category,
                categoryName: getCategoryName(module, category),
                permissions: uniquePerms.sort((a, b) => a.name.localeCompare(b.name)),
              }
            })
            .filter(cat => cat.permissions.length > 0) // 移除空的分類
            .sort((a, b) => a.categoryName.localeCompare(b.categoryName)),
        }
      })
      .filter(group => group.categories.length > 0) // 移除空的分組
      .sort((a, b) => a.moduleOrder - b.moduleOrder)

    return result
  }, [uniquePermissions])

  // 過濾權限（根據搜索和模組選擇）
  const filteredGroups = useMemo(() => {
    let filtered = groupedPermissions

    // 按模組過濾
    if (selectedModule) {
      filtered = filtered.filter(g => g.module === selectedModule)
    }

    // 按搜索關鍵字過濾
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

  // 統計資訊
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

  // 切換模組展開狀態
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

  // 切換子類別展開狀態
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

  // 展開/摺疊所有模組
  const expandAllModules = () => {
    setExpandedModules(new Set(groupedPermissions.map(g => g.module)))
  }

  const collapseAllModules = () => {
    setExpandedModules(new Set())
  }

  // 檢查模組是否展開
  const isModuleExpanded = (module: string) => expandedModules.has(module)

  // 檢查子類別是否展開
  const isCategoryExpanded = (module: string, category: string) => {
    return expandedCategories.has(`${module}.${category}`)
  }

  return {
    // 資料
    groupedPermissions: filteredGroups,
    stats,
    
    // 搜索
    searchQuery,
    setSearchQuery,
    
    // 過濾
    selectedModule,
    setSelectedModule,
    
    // 展開/摺疊
    toggleModule,
    toggleCategory,
    expandAllModules,
    collapseAllModules,
    isModuleExpanded,
    isCategoryExpanded,
  }
}
