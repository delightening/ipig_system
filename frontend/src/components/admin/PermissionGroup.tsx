import { Permission } from '@/lib/api'
import { PermissionCategory, PermissionSubCategory } from '@/hooks/usePermissionManager'
import { Badge } from '@/components/ui/badge'
import { Check, ChevronDown, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

interface PermissionGroupProps {
  module: string
  moduleName: string
  categories: PermissionCategory[]
  selectedPermissionIds: string[]
  onTogglePermission: (permId: string) => void
  onToggleCategory?: (category: string) => void
  onToggleSubCategory?: (category: string, subCategory: string) => void
  isExpanded: boolean
  onToggleExpand: () => void
  isCategoryExpanded?: (module: string, category: string) => boolean
  isSubCategoryExpanded?: (module: string, category: string, subCategory: string) => boolean
  searchQuery?: string
}

// 計算類別權限總數（包含子類別和直接權限）
function getCategoryPermissionCount(category: PermissionCategory): number {
  let count = category.permissions.length
  if (category.subCategories && category.subCategories.length > 0) {
    count += category.subCategories.reduce((sum, sub) => sum + sub.permissions.length, 0)
  }
  return count
}

// 計算類別已選權限數（包含子類別和直接權限）
function getCategorySelectedCount(category: PermissionCategory, selectedIds: string[]): number {
  let count = category.permissions.filter(p => selectedIds.includes(p.id)).length
  if (category.subCategories && category.subCategories.length > 0) {
    count += category.subCategories.reduce(
      (sum, sub) => sum + sub.permissions.filter(p => selectedIds.includes(p.id)).length,
      0
    )
  }
  return count
}

export function PermissionGroup({
  module,
  moduleName,
  categories,
  selectedPermissionIds,
  onTogglePermission,
  onToggleCategory,
  onToggleSubCategory,
  isExpanded,
  onToggleExpand,
  isCategoryExpanded,
  isSubCategoryExpanded,
  searchQuery = '',
}: PermissionGroupProps) {
  // 計算統計
  const totalPermissions = categories.reduce((sum, cat) => sum + getCategoryPermissionCount(cat), 0)
  const selectedCount = categories.reduce(
    (sum, cat) => sum + getCategorySelectedCount(cat, selectedPermissionIds),
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

  // 渲染權限 badge
  const renderPermissionBadge = (perm: Permission) => {
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
  }

  // 渲染子類別（第三層）
  const renderSubCategory = (
    category: string,
    subCat: PermissionSubCategory
  ) => {
    const subCatSelectedCount = subCat.permissions.filter(p => selectedPermissionIds.includes(p.id)).length
    const isSubExpanded = isSubCategoryExpanded
      ? isSubCategoryExpanded(module, category, subCat.subCategory)
      : true

    return (
      <div key={`${category}.${subCat.subCategory}`} className="space-y-1">
        {onToggleSubCategory ? (
          <button
            type="button"
            onClick={() => onToggleSubCategory(category, subCat.subCategory)}
            className="flex items-center gap-2 text-xs font-medium text-slate-500 hover:text-slate-700 transition-colors"
          >
            {isSubExpanded ? (
              <ChevronDown className="h-3 w-3" />
            ) : (
              <ChevronRight className="h-3 w-3" />
            )}
            <span>{subCat.subCategoryName}</span>
            <Badge variant="outline" className="text-xs py-0 px-1.5">
              {subCatSelectedCount}/{subCat.permissions.length}
            </Badge>
          </button>
        ) : (
          <div className="flex items-center gap-2 text-xs font-medium text-slate-500">
            <span>{subCat.subCategoryName}</span>
            <Badge variant="outline" className="text-xs py-0 px-1.5">
              {subCatSelectedCount}/{subCat.permissions.length}
            </Badge>
          </div>
        )}

        {isSubExpanded && (
          <div className="flex flex-wrap gap-2 ml-5">
            {subCat.permissions.map(renderPermissionBadge)}
          </div>
        )}
      </div>
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
          {categories.map((cat) => {
            const categoryKey = `${module}.${cat.category}`
            const isCatExpanded = isCategoryExpanded ? isCategoryExpanded(module, cat.category) : true
            const catTotalCount = getCategoryPermissionCount(cat)
            const catSelectedCount = getCategorySelectedCount(cat, selectedPermissionIds)
            const hasSubCategories = cat.subCategories && cat.subCategories.length > 0

            return (
              <div key={categoryKey} className="space-y-2">
                {/* 類別標題 */}
                {onToggleCategory && (
                  <button
                    type="button"
                    onClick={() => onToggleCategory(cat.category)}
                    className="flex items-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                  >
                    {isCatExpanded ? (
                      <ChevronDown className="h-3 w-3" />
                    ) : (
                      <ChevronRight className="h-3 w-3" />
                    )}
                    <span>{cat.categoryName}</span>
                    <Badge variant="outline" className="text-xs">
                      {catSelectedCount}/{catTotalCount}
                    </Badge>
                  </button>
                )}
                {!onToggleCategory && (
                  <h4 className="text-sm font-medium text-muted-foreground">
                    {cat.categoryName}
                    <Badge variant="outline" className="ml-2 text-xs">
                      {catSelectedCount}/{catTotalCount}
                    </Badge>
                  </h4>
                )}

                {/* 類別內容 */}
                {isCatExpanded && (
                  <div className="ml-5 space-y-3">
                    {/* 渲染子類別（如果有） */}
                    {hasSubCategories && cat.subCategories!.map(subCat => renderSubCategory(cat.category, subCat))}
                    {/* 渲染直接權限（未分組的權限） */}
                    {cat.permissions.length > 0 && (
                      <div className="flex flex-wrap gap-2">
                        {cat.permissions.map(renderPermissionBadge)}
                      </div>
                    )}
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
