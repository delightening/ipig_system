import { Permission } from '@/lib/api'

import { usePermissionCategories } from './permission/usePermissionCategories'
import { usePermissionSearch } from './permission/usePermissionSearch'
import { usePermissionExpand } from './permission/usePermissionExpand'

// Re-export types and utilities for backward compatibility
export type { PermissionSubCategory, PermissionCategory, PermissionGroup } from './permission/usePermissionCategories'
export { groupPermissionsByModule } from './permission/usePermissionCategories'

/**
 * 權限管理 Hook — 組合分類、搜尋篩選、展開/收合三個子 Hook
 */
export function usePermissionManager(permissions: Permission[] | undefined) {
  const { groupedPermissions, stats } = usePermissionCategories(permissions)
  const { searchQuery, setSearchQuery, selectedModule, setSelectedModule, filteredGroups } =
    usePermissionSearch(groupedPermissions)
  const {
    toggleModule,
    toggleCategory,
    toggleSubCategory,
    expandAllModules,
    collapseAllModules,
    isModuleExpanded,
    isCategoryExpanded,
    isSubCategoryExpanded,
  } = usePermissionExpand(groupedPermissions)

  return {
    groupedPermissions: filteredGroups,
    stats,
    searchQuery,
    setSearchQuery,
    selectedModule,
    setSelectedModule,
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
