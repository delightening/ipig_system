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

// ??????
const MODULE_CONFIG: Record<string, { name: string; order: number }> = {
  aup: { name: 'AUP', order: 1 },
  pig: { name: 'Pig', order: 2 },
  erp: { name: 'ERP', order: 3 },
  dev: { name: 'Dev', order: 4 },
  notification: { name: 'Notification', order: 5 },
  report: { name: 'Report', order: 6 },
  other: { name: 'Other', order: 99 },
}

// Category display names
const CATEGORY_NAMES: Record<string, Record<string, string>> = {
  aup: {
    protocol: 'Protocols',
    review: 'Reviews',
    attachment: 'Attachments',
    version: 'Versions',
  },
  pig: {
    pig: 'Pigs',
    record: 'Records',
    vet: 'Vet',
    export: 'Export',
    pathology: 'Pathology',
  },
  erp: {
    warehouse: 'Warehouses',
    product: 'Products',
    partner: 'Partners',
    document: 'Documents',
    purchase: 'Purchasing',
    grn: 'GRN',
    pr: 'PR',
    sales: 'Sales',
    do: 'DO',
    stock: 'Stock',
    stocktake: 'Stocktake',
    report: 'Reports',
    po: 'PO',
    so: 'SO',
    tr: 'Transfer',
    stk: 'Stocktake',
    adj: 'Adjustment',
    inventory: 'Inventory',
    create: 'Create',
    approve: 'Approve',
    cancel: 'Cancel',
    submit: 'Submit',
    update: 'Update',
    delete: 'Delete',
    view: 'View',
    read: 'Read',
    edit: 'Edit',
    schedule: 'Schedule',
    download: 'Download',
  },
  dev: {
    user: 'Users',
    role: 'Roles',
    permission: 'Permissions',
    system: 'System',
    audit: 'Audit',
    log: 'Logs',
    notification: 'Notifications',
    database: 'Database',
    create: 'Create',
    view: 'View',
    read: 'Read',
    edit: 'Edit',
    update: 'Update',
    delete: 'Delete',
    manage: 'Manage',
    assign: 'Assign',
    reset_password: 'Reset Password',
    export: 'Export',
    download: 'Download',
    query: 'Query',
    migrate: 'Migrate',
    seed: 'Seed',
    send: 'Send',
    backup: 'Backup',
    restore: 'Restore',
    trigger: 'Trigger',
    schedule: 'Schedule',
    upload: 'Upload',
  },
  notification: {
    manage: 'Manage',
    send: 'Send',
    trigger: 'Trigger',
    view: 'View',
  },
  report: {
    download: 'Download',
    schedule: 'Schedule',
    view: 'View',
    export: 'Export',
  },
}

// Operation display names
const OPERATION_NAMES: Record<string, string> = {
  create: 'Create',
  approve: 'Approve',
  cancel: 'Cancel',
  submit: 'Submit',
  update: 'Update',
  delete: 'Delete',
  view: 'View',
  read: 'Read',
  edit: 'Edit',
  manage: 'Manage',
  assign: 'Assign',
  reset_password: 'Reset Password',
  export: 'Export',
  download: 'Download',
  query: 'Query',
  migrate: 'Migrate',
  seed: 'Seed',
  send: 'Send',
  backup: 'Backup',
  restore: 'Restore',
  upload: 'Upload',
  trigger: 'Trigger',
  schedule: 'Schedule',
}
/**
 * ???????????
 */
function getPermissionModule(perm: Permission): string {
  if (perm.module) {
    return perm.module
  }

  const prefix = perm.code.split('.')[0]
  if (prefix === 'aup') return 'aup'
  if (prefix === 'pig') return 'pig'
  if (prefix === 'erp') return 'erp'
  if (prefix === 'dev') return 'dev'
  return 'other'
}
function getPermissionCategory(code: string): string {
  const parts = code.split('.')
  if (parts.length < 2) return 'other'
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

  // ?????????????????ID ?????????????????
  // ???????????? code?????pig.record.* ?????animal.record.*??
  const groupedPermissions = useMemo(() => {
    const groups = new Map<string, Map<string, Permission[]>>()

    uniquePermissions.forEach(perm => {
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
    })

    const result: PermissionGroup[] = Array.from(groups.entries())
      .map(([module, categories]) => {
        const moduleConfig = MODULE_CONFIG[module] || MODULE_CONFIG.other
        return {
          module,
          moduleName: moduleConfig.name,
          moduleOrder: moduleConfig.order,
          categories: Array.from(categories.entries())
            .map(([category, perms]) => {
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




