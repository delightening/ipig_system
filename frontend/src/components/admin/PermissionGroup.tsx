import { Permission } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Check, ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

interface PermissionGroupProps {
  module: string
  moduleName: string
  categories: {
    category: string
    categoryName: string
    permissions: Permission[]
  }[]
  selectedPermissionIds: string[]
  onTogglePermission: (permId: string) => void
  onToggleCategory?: (category: string) => void
  isExpanded: boolean
  onToggleExpand: () => void
  isCategoryExpanded?: (module: string, category: string) => boolean
  searchQuery?: string
}

export function PermissionGroup({
  module,
  moduleName,
  categories,
  selectedPermissionIds,
  onTogglePermission,
  onToggleCategory,
  isExpanded,
  onToggleExpand,
  isCategoryExpanded,
  searchQuery = '',
}: PermissionGroupProps) {
  // 計算統計
  const totalPermissions = categories.reduce((sum, cat) => sum + cat.permissions.length, 0)
  const selectedCount = categories.reduce(
    (sum, cat) => sum + cat.permissions.filter(p => selectedPermissionIds.includes(p.id)).length,
    0
  )

  // 高亮搜索關鍵字
  const highlightText = (text: string) => {
    if (!searchQuery.trim()) return text
    
    const query = searchQuery.trim().toLowerCase()
    const index = text.toLowerCase().indexOf(query)
    
    if (index === -1) return text
    
    return (
      <>
        {text.substring(0, index)}
        <mark className="bg-yellow-200 dark:bg-yellow-900">{text.substring(index, index + query.length)}</mark>
        {text.substring(index + query.length)}
      </>
    )
  }

  return (
    <div className="border rounded-lg overflow-hidden">
      {/* 模組標題 */}
      <button
        type="button"
        onClick={onToggleExpand}
        className="w-full flex items-center justify-between p-4 bg-slate-50 hover:bg-slate-100 transition-colors"
      >
        <div className="flex items-center gap-3">
          {isExpanded ? (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 text-muted-foreground" />
          )}
          <h3 className="font-semibold text-base text-primary">{moduleName}</h3>
          <Badge variant="secondary" className="text-xs">
            {selectedCount}/{totalPermissions}
          </Badge>
        </div>
        <div className="text-sm text-muted-foreground">
          {categories.length} 個分類
        </div>
      </button>

      {/* 模組內容 */}
      {isExpanded && (
        <div className="p-4 space-y-4 bg-white">
          {categories.map(({ category, categoryName, permissions }) => {
            const categoryKey = `${module}.${category}`
            const isCatExpanded = isCategoryExpanded ? isCategoryExpanded(module, category) : true
            const catSelectedCount = permissions.filter(p => selectedPermissionIds.includes(p.id)).length

            return (
              <div key={categoryKey} className="space-y-2">
                {/* 子類別標題 */}
                {onToggleCategory && (
                  <button
                    type="button"
                    onClick={() => onToggleCategory(category)}
                    className="flex items-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                  >
                    {isCatExpanded ? (
                      <ChevronDown className="h-3 w-3" />
                    ) : (
                      <ChevronRight className="h-3 w-3" />
                    )}
                    <span>{categoryName}</span>
                    <Badge variant="outline" className="text-xs">
                      {catSelectedCount}/{permissions.length}
                    </Badge>
                  </button>
                )}
                {!onToggleCategory && (
                  <h4 className="text-sm font-medium text-muted-foreground">
                    {categoryName}
                    <Badge variant="outline" className="ml-2 text-xs">
                      {catSelectedCount}/{permissions.length}
                    </Badge>
                  </h4>
                )}

                {/* 權限列表 */}
                {isCatExpanded && (
                  <div className="flex flex-wrap gap-2 ml-5">
                    {permissions.map((perm) => {
                      const isSelected = selectedPermissionIds.includes(perm.id)
                      return (
                        <Badge
                          key={perm.id}
                          variant={isSelected ? 'default' : 'outline'}
                          className={cn(
                            'cursor-pointer hover:bg-primary/10 transition-colors',
                            isSelected && 'bg-primary text-primary-foreground'
                          )}
                          onClick={() => onTogglePermission(perm.id)}
                          title={perm.description || perm.code}
                        >
                          {isSelected && <Check className="h-3 w-3 mr-1" />}
                          {highlightText(perm.name)}
                        </Badge>
                      )
                    })}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
