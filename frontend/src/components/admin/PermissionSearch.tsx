import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Search, X } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface PermissionSearchProps {
  searchQuery: string
  onSearchChange: (query: string) => void
  selectedModule: string | null
  onModuleChange: (module: string | null) => void
  moduleOptions: { value: string; label: string }[]
  stats: {
    total: number
    moduleCounts: Record<string, number>
  }
}

export function PermissionSearch({
  searchQuery,
  onSearchChange,
  selectedModule,
  onModuleChange,
  moduleOptions,
  stats,
}: PermissionSearchProps) {
  return (
    <div className="space-y-3">
      {/* 搜索框 */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          type="text"
          placeholder="搜索權限名稱、代碼或描述..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-9 pr-9"
        />
        {searchQuery && (
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-1 top-1/2 transform -translate-y-1/2 h-7 w-7"
            onClick={() => onSearchChange('')}
          >
            <X className="h-4 w-4" />
          </Button>
        )}
      </div>

      {/* 模組過濾 */}
      <div className="flex items-center gap-2">
        <Label className="text-sm text-muted-foreground whitespace-nowrap">模組篩選：</Label>
        <div className="flex flex-wrap gap-2">
          <Button
            variant={selectedModule === null ? 'default' : 'outline'}
            size="sm"
            onClick={() => onModuleChange(null)}
            className="h-7 text-xs"
          >
            全部 ({stats.total})
          </Button>
          {moduleOptions.map((option) => (
            <Button
              key={option.value}
              variant={selectedModule === option.value ? 'default' : 'outline'}
              size="sm"
              onClick={() => onModuleChange(option.value)}
              className="h-7 text-xs"
            >
              {option.label} ({stats.moduleCounts[option.value] || 0})
            </Button>
          ))}
        </div>
      </div>
    </div>
  )
}
