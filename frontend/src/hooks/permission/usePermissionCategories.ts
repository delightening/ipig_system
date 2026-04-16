import { useMemo } from 'react'

import { Permission } from '@/lib/api'

import {
  MODULE_CONFIG,
  MODULE_NAME_ORDER,
  CATEGORY_NAMES,
  SUB_CATEGORY_CONFIG,
  SUB_CATEGORY_NAMES,
  PERMISSION_NAME_SUBGROUPS,
  getPermissionModule,
  getPermissionRawCategory,
  getPermissionCategory,
  getCategoryName,
} from './permissionConfig'

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
  subCategories?: PermissionSubCategory[]
}

// 權限群組介面
export interface PermissionGroup {
  module: string
  moduleName: string
  moduleOrder: number
  categories: PermissionCategory[]
}

/**
 * 去重權限列表
 */
function deduplicatePermissions(perms: Permission[]): Permission[] {
  const seen = new Set<string>()
  return perms.filter(perm => {
    if (seen.has(perm.id)) return false
    seen.add(perm.id)
    return true
  })
}

/**
 * 建立三層結構（基於權限碼子類別）
 */
function buildCodeBasedSubCategories(
  subCategoryMap: Map<string, Permission[]>,
): PermissionSubCategory[] {
  return Array.from(subCategoryMap.entries())
    .map(([subCat, perms]) => ({
      subCategory: subCat,
      subCategoryName: SUB_CATEGORY_NAMES[subCat] || subCat,
      permissions: deduplicatePermissions(perms).sort((a, b) => a.name.localeCompare(b.name)),
    }))
    .filter(subCat => subCat.permissions.length > 0)
    .sort((a, b) => a.subCategoryName.localeCompare(b.subCategoryName))
}

/**
 * 建立三層結構（基於權限名稱分組）
 */
function buildNameBasedSubCategories(
  subCategoryMap: Map<string, Permission[]>,
  nameSubgroups: Record<string, string[]>,
): { subCategories: PermissionSubCategory[]; ungrouped: Permission[] } {
  const allPerms: Permission[] = []
  subCategoryMap.forEach(perms => allPerms.push(...perms))
  const uniquePerms = deduplicatePermissions(allPerms)

  const subCategories: PermissionSubCategory[] = []
  const ungrouped: Permission[] = []

  uniquePerms.forEach(perm => {
    let assigned = false
    for (const [subGroupName, permNames] of Object.entries(nameSubgroups)) {
      if (permNames.includes(perm.name)) {
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
      ungrouped.push(perm)
    }
  })

  subCategories.forEach(subCat => {
    subCat.permissions.sort((a, b) => a.name.localeCompare(b.name))
  })
  subCategories.sort((a, b) => a.subCategoryName.localeCompare(b.subCategoryName))

  return { subCategories, ungrouped: ungrouped.sort((a, b) => a.name.localeCompare(b.name)) }
}

/**
 * 將類別 Map 轉換為 PermissionCategory 陣列
 */
function buildCategories(
  categories: Map<string, Map<string, Permission[]>>,
): PermissionCategory[] {
  return Array.from(categories.entries())
    .map(([category, subCategoryMap]) => {
      let categoryName = getCategoryName(category, category)
      if (categoryName === category && CATEGORY_NAMES[category]?.[category]) {
        categoryName = CATEGORY_NAMES[category][category]
      }

      const needsSubCategories = SUB_CATEGORY_CONFIG[category] !== undefined
      const nameSubgroups = PERMISSION_NAME_SUBGROUPS[category]

      if (needsSubCategories) {
        return {
          category,
          categoryName,
          permissions: [],
          subCategories: buildCodeBasedSubCategories(subCategoryMap),
        }
      }

      if (nameSubgroups) {
        const { subCategories, ungrouped } = buildNameBasedSubCategories(subCategoryMap, nameSubgroups)
        return {
          category,
          categoryName,
          permissions: ungrouped,
          subCategories,
        }
      }

      // 兩層結構：合併所有子類別的權限
      const allPerms: Permission[] = []
      subCategoryMap.forEach(perms => allPerms.push(...perms))

      return {
        category,
        categoryName,
        permissions: deduplicatePermissions(allPerms).sort((a, b) => a.name.localeCompare(b.name)),
      }
    })
    .filter(cat => cat.permissions.length > 0 || (cat.subCategories && cat.subCategories.length > 0))
    .sort((a, b) => a.categoryName.localeCompare(b.categoryName))
}

/**
 * 權限分類 Hook — 將權限列表分組為模組 > 類別 > 子類別結構
 */
export function usePermissionCategories(permissions: Permission[] | undefined) {
  const uniquePermissions = useMemo(() => {
    if (!permissions) return []
    return deduplicatePermissions(permissions)
  }, [permissions])

  const groupedPermissions = useMemo(() => {
    const groupsByName = new Map<string, {
      moduleOrder: number
      categories: Map<string, Map<string, Permission[]>>
    }>()

    uniquePermissions.forEach(perm => {
      const moduleCode = getPermissionModule(perm)
      const moduleConfig = MODULE_CONFIG[moduleCode] || MODULE_CONFIG.other
      const moduleName = moduleConfig.name
      const moduleOrder = moduleConfig.order

      let rawCategory = getPermissionCategory(perm.code)
      if (moduleCode === 'qau') {
        rawCategory = 'QAU'
      }
      const category = rawCategory === moduleCode ? moduleCode : rawCategory
      const rawSubCategory = getPermissionRawCategory(perm.code)

      if (!groupsByName.has(moduleName)) {
        groupsByName.set(moduleName, { moduleOrder, categories: new Map() })
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

    return Array.from(groupsByName.entries())
      .map(([moduleName, { moduleOrder, categories }]) => ({
        module: moduleName,
        moduleName,
        moduleOrder,
        categories: buildCategories(categories),
      }))
      .filter(group => group.categories.length > 0)
      .sort((a, b) => a.moduleOrder - b.moduleOrder)
  }, [uniquePermissions])

  const stats = useMemo(() => {
    const total = uniquePermissions.length
    const moduleCounts = new Map<string, number>()

    uniquePermissions.forEach(perm => {
      const moduleCode = getPermissionModule(perm)
      const moduleConfig = MODULE_CONFIG[moduleCode] || MODULE_CONFIG.other
      const moduleName = moduleConfig.name
      moduleCounts.set(moduleName, (moduleCounts.get(moduleName) || 0) + 1)
    })

    return { total, moduleCounts: Object.fromEntries(moduleCounts) }
  }, [uniquePermissions])

  return { groupedPermissions, stats }
}

/**
 * 將權限依模組分組並回傳摘要（模組名稱 + 數量）
 * 用於角色卡片的模組摘要顯示
 */
export function groupPermissionsByModule(permissions: Permission[]): { moduleName: string; count: number }[] {
  const counts = new Map<string, number>()
  for (const perm of permissions) {
    const moduleCode = getPermissionModule(perm)
    const config = MODULE_CONFIG[moduleCode] || MODULE_CONFIG.other
    const name = config.name
    counts.set(name, (counts.get(name) || 0) + 1)
  }
  return Array.from(counts.entries())
    .map(([moduleName, count]) => ({ moduleName, count }))
    .sort((a, b) => {
      const orderA = MODULE_NAME_ORDER.get(a.moduleName) ?? 99
      const orderB = MODULE_NAME_ORDER.get(b.moduleName) ?? 99
      if (orderA !== orderB) return orderA - orderB
      return a.moduleName.localeCompare(b.moduleName)
    })
}
