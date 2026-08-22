import { useState } from 'react'

import { PermissionGroup } from './usePermissionCategories'

/**
 * 權限展開/收合狀態管理 Hook
 */
export function usePermissionExpand(groupedPermissions: PermissionGroup[]) {
  const [expandedModules, setExpandedModules] = useState<Set<string>>(new Set())
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set())
  const [expandedSubCategories, setExpandedSubCategories] = useState<Set<string>>(new Set())

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

  const expandAllModules = () => {
    setExpandedModules(new Set(groupedPermissions.map(g => g.module)))
  }

  const collapseAllModules = () => {
    setExpandedModules(new Set())
  }

  const isModuleExpanded = (module: string) => expandedModules.has(module)

  const isCategoryExpanded = (module: string, category: string) => {
    return expandedCategories.has(`${module}.${category}`)
  }

  const isSubCategoryExpanded = (module: string, category: string, subCategory: string) => {
    return expandedSubCategories.has(`${module}.${category}.${subCategory}`)
  }

  return {
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
