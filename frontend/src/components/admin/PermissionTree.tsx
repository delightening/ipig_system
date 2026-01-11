import { Permission } from '@/lib/api'
import { PermissionGroup } from './PermissionGroup'
import { PermissionSearch } from './PermissionSearch'
import { usePermissionManager } from '@/hooks/usePermissionManager'
import { Button } from '@/components/ui/button'
import { ChevronsDownUp, ChevronsUpDown } from 'lucide-react'

interface PermissionTreeProps {
  permissions: Permission[] | undefined
  selectedPermissionIds: string[]
  onTogglePermission: (permId: string) => void
  showSearch?: boolean
}

export function PermissionTree({
  permissions,
  selectedPermissionIds,
  onTogglePermission,
  showSearch = true,
}: PermissionTreeProps) {
  const {
    groupedPermissions,
    stats,
    searchQuery,
    setSearchQuery,
    selectedModule,
    setSelectedModule,
    toggleModule,
    toggleCategory,
    expandAllModules,
    collapseAllModules,
    isModuleExpanded,
    isCategoryExpanded,
  } = usePermissionManager(permissions)

  // 模組選項
  const moduleOptions = groupedPermissions.map(g => ({
    value: g.module,
    label: g.moduleName,
  }))

  return (
    <div className="space-y-4">
      {/* 搜索和過濾 */}
      {showSearch && (
        <PermissionSearch
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          selectedModule={selectedModule}
          onModuleChange={setSelectedModule}
          moduleOptions={moduleOptions}
          stats={stats}
        />
      )}

      {/* 批量操作按鈕 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={expandAllModules}
            className="h-8 text-xs"
          >
            <ChevronsDownUp className="h-3 w-3 mr-1" />
            展開全部
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={collapseAllModules}
            className="h-8 text-xs"
          >
            <ChevronsUpDown className="h-3 w-3 mr-1" />
            摺疊全部
          </Button>
        </div>
        <div className="text-sm text-muted-foreground">
          共 {stats.total} 個權限，已選 {selectedPermissionIds.length} 個
        </div>
      </div>

      {/* 權限樹 */}
      <div className="space-y-2">
        {groupedPermissions.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            {searchQuery ? '沒有找到符合條件的權限' : '沒有權限'}
          </div>
        ) : (
          groupedPermissions.map((group) => (
            <PermissionGroup
              key={group.module}
              module={group.module}
              moduleName={group.moduleName}
              categories={group.categories}
              selectedPermissionIds={selectedPermissionIds}
              onTogglePermission={onTogglePermission}
              onToggleCategory={(category) => toggleCategory(group.module, category)}
              isExpanded={isModuleExpanded(group.module)}
              onToggleExpand={() => toggleModule(group.module)}
              isCategoryExpanded={(module, category) => isCategoryExpanded(module, category)}
              searchQuery={searchQuery}
            />
          ))
        )}
      </div>
    </div>
  )
}
