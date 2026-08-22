import { useMemo, useState } from 'react'

import { PermissionGroup } from './usePermissionCategories'

/**
 * 權限搜尋篩選 Hook — 管理搜尋文字與模組篩選邏輯
 */
export function usePermissionSearch(groupedPermissions: PermissionGroup[]) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedModule, setSelectedModule] = useState<string | null>(null)

  const filteredGroups = useMemo(() => {
    let filtered = groupedPermissions

    if (selectedModule) {
      filtered = filtered.filter(g => g.module === selectedModule)
    }

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

  return {
    searchQuery,
    setSearchQuery,
    selectedModule,
    setSelectedModule,
    filteredGroups,
  }
}
