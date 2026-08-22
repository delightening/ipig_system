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
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Search className="h-5 w-5" />
          {t('admin.auditLogFilters.title')}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="audit-search">{t('admin.auditLogFilters.keywordSearch')}</Label>
          <Input
            id="audit-search"
            type="text"
            placeholder={t('admin.auditLogFilters.searchPlaceholder')}
            value={searchQuery}
            maxLength={100}
            onChange={(e) => onSearchQueryChange(e.target.value)}
          />
        </div>
        <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
          <div className="space-y-2">
            <Label>{t('admin.auditLogFilters.actor')}</Label>
            <Select value={userFilter} onValueChange={onUserChange}>
              <SelectTrigger>
                <SelectValue placeholder={t('admin.auditLogFilters.allActors')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('admin.auditLogFilters.allActors')}</SelectItem>
                {users.map((u) => (
                  <SelectItem key={u.id} value={u.id}>
                    {u.display_name || u.email}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t('admin.auditLogFilters.eventCategory')}</Label>
            <Select value={categoryFilter} onValueChange={onCategoryChange}>
              <SelectTrigger>
                <SelectValue placeholder={t('admin.auditLogFilters.allCategories')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('admin.auditLogFilters.allCategories')}</SelectItem>
                <SelectItem value="ERP">ERP</SelectItem>
                <SelectItem value="AUP">{t('admin.auditLogFilters.categoryAup')}</SelectItem>
                <SelectItem value="ANIMAL">{t('admin.auditLogFilters.categoryAnimal')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t('admin.auditLogFilters.entityType')}</Label>
            <Select value={entityTypeFilter} onValueChange={onEntityTypeChange}>
              <SelectTrigger>
                <SelectValue placeholder={t('admin.auditLogFilters.allEntityTypes')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('admin.auditLogFilters.allEntityTypes')}</SelectItem>
                {availableEntityTypes.map((et) => (
                  <SelectItem key={et.value} value={et.value}>{et.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>{t('admin.auditLogFilters.dateFrom')}</Label>
            <Input
              type="date"
              value={dateFrom}
              onChange={(e) => onDateFromChange(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('admin.auditLogFilters.dateTo')}</Label>
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
