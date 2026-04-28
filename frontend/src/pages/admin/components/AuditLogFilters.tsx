import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Search } from 'lucide-react'
import type { User } from '@/types/auth'

interface AuditLogFiltersProps {
  users: User[]
  userFilter: string
  categoryFilter: string
  entityTypeFilter: string
  dateFrom: string
  dateTo: string
  /** R30-14: 自由文字搜尋（操作者 / 實體 / 事件 / IP） */
  searchQuery: string
  availableEntityTypes: { value: string; label: string }[]
  onUserChange: (val: string) => void
  onCategoryChange: (val: string) => void
  onEntityTypeChange: (val: string) => void
  onDateFromChange: (val: string) => void
  onDateToChange: (val: string) => void
  onSearchQueryChange: (val: string) => void
}

export function AuditLogFilters({
  users,
  userFilter,
  categoryFilter,
  entityTypeFilter,
  dateFrom,
  dateTo,
  searchQuery,
  availableEntityTypes,
  onUserChange,
  onCategoryChange,
  onEntityTypeChange,
  onDateFromChange,
  onDateToChange,
  onSearchQueryChange,
}: AuditLogFiltersProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Search className="h-5 w-5" />
          搜尋條件
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="audit-search">關鍵字搜尋</Label>
          <Input
            id="audit-search"
            type="text"
            placeholder="搜尋操作者 / 實體名稱 / 事件 / IP（最多 100 字）"
            value={searchQuery}
            maxLength={100}
            onChange={(e) => onSearchQueryChange(e.target.value)}
          />
        </div>
        <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
          <div className="space-y-2">
            <Label>操作者</Label>
            <Select value={userFilter} onValueChange={onUserChange}>
              <SelectTrigger>
                <SelectValue placeholder="全部操作者" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部操作者</SelectItem>
                {users.map((u) => (
                  <SelectItem key={u.id} value={u.id}>
                    {u.display_name || u.email}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>事件類別</Label>
            <Select value={categoryFilter} onValueChange={onCategoryChange}>
              <SelectTrigger>
                <SelectValue placeholder="全部類別" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部類別</SelectItem>
                <SelectItem value="ERP">ERP</SelectItem>
                <SelectItem value="AUP">計畫書</SelectItem>
                <SelectItem value="ANIMAL">實驗動物</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>實體類型</Label>
            <Select value={entityTypeFilter} onValueChange={onEntityTypeChange}>
              <SelectTrigger>
                <SelectValue placeholder="全部類型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部類型</SelectItem>
                {availableEntityTypes.map((et) => (
                  <SelectItem key={et.value} value={et.value}>{et.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>開始日期</Label>
            <Input
              type="date"
              value={dateFrom}
              onChange={(e) => onDateFromChange(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label>結束日期</Label>
            <Input
              type="date"
              value={dateTo}
              onChange={(e) => onDateToChange(e.target.value)}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
