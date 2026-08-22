import React, { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Edit, Power, PowerOff, Droplets, ArrowUpDown, ChevronDown, ChevronRight } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { cn } from '@/lib/utils'
import { PanelIcon } from '@/components/ui/panel-icon'
import type { BloodTestTemplate, BloodTestPanel } from '@/lib/api'
import type { SortField } from '../hooks/useBloodTestTemplates'

interface BloodTestTemplateTableProps {
  groupedData: { panel: BloodTestPanel | null; items: BloodTestTemplate[] }[]
  flatFiltered: BloodTestTemplate[]
  isLoading: boolean
  search: string
  sortField: SortField
  onSort: (field: SortField) => void
  onEdit: (template: BloodTestTemplate) => void
  onToggle: (template: BloodTestTemplate) => void
}

export function BloodTestTemplateTable({
  groupedData,
  flatFiltered,
  isLoading,
  search,
  sortField,
  onSort,
  onEdit,
  onToggle,
}: BloodTestTemplateTableProps) {
  // 預設全部收合；key 為 group.panel?.key ?? '__uncategorized__'
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set())

  const getGroupKey = (group: { panel: BloodTestPanel | null }) =>
    group.panel?.key ?? '__uncategorized__'

  const toggleGroup = (key: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }

  const SortIndicator = ({ field }: { field: SortField }) => (
    <ArrowUpDown
      className={cn(
        'ml-1 h-3 w-3 inline-block cursor-pointer',
        sortField === field ? 'text-primary' : 'text-muted-foreground'
      )}
    />
  )

  const renderTemplateRow = (template: BloodTestTemplate) => (
    <TableRow key={template.id} className={cn(!template.is_active && 'opacity-50')}>
      <TableCell className="font-mono text-sm font-semibold">{template.code}</TableCell>
      <TableCell className="font-medium">{template.name}</TableCell>
      <TableCell className="text-muted-foreground">{template.default_unit || '—'}</TableCell>
      <TableCell className="text-sm text-muted-foreground">
        {template.reference_range || '—'}
      </TableCell>
      <TableCell className="text-right font-mono text-sm">
        {template.default_price ? `$${Number(template.default_price).toFixed(0)}` : '—'}
      </TableCell>
      <TableCell className="text-center">
        {template.is_active ? (
          <Badge variant="success">啟用</Badge>
        ) : (
          <Badge variant="secondary">停用</Badge>
        )}
      </TableCell>
      <TableCell className="text-right">
        <div className="flex items-center justify-end gap-1">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onEdit(template)}
            title="編輯"
            aria-label="編輯"
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onToggle(template)}
            title={template.is_active ? '停用' : '恢復'}
            aria-label={template.is_active ? '停用' : '恢復'}
          >
            {template.is_active ? (
              <PowerOff className="h-4 w-4 text-status-warning-text" />
            ) : (
              <Power className="h-4 w-4 text-status-success-text" />
            )}
          </Button>
        </div>
      </TableCell>
    </TableRow>
  )

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            <TableHead className="w-[120px] cursor-pointer" onClick={() => onSort('code')}>
              代碼 <SortIndicator field="code" />
            </TableHead>
            <TableHead className="cursor-pointer" onClick={() => onSort('name')}>
              名稱 <SortIndicator field="name" />
            </TableHead>
            <TableHead className="w-[100px] cursor-pointer" onClick={() => onSort('default_unit')}>
              單位 <SortIndicator field="default_unit" />
            </TableHead>
            <TableHead className="w-[140px]">參考範圍</TableHead>
            <TableHead
              className="w-[100px] cursor-pointer text-right"
              onClick={() => onSort('default_price')}
            >
              價格 <SortIndicator field="default_price" />
            </TableHead>
            <TableHead className="w-[80px] text-center">狀態</TableHead>
            <TableHead className="w-[120px] text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={7} className="p-0">
                <TableSkeleton rows={5} cols={7} />
              </TableCell>
            </TableRow>
          ) : flatFiltered.length > 0 ? (
            groupedData.map((group, gi) => {
              const groupKey = getGroupKey(group)
              const isExpanded = expandedGroups.has(groupKey)
              const hasGroups = groupedData.length > 1

              return (
                <React.Fragment key={`group-${gi}`}>
                  {hasGroups && (
                    <TableRow
                      key={`group-header-${gi}`}
                      className="bg-muted/50 hover:bg-muted/70 cursor-pointer"
                      onClick={() => toggleGroup(groupKey)}
                    >
                      <TableCell colSpan={7} className="py-2">
                        <div className="flex items-center gap-2 font-semibold text-sm">
                          {isExpanded ? (
                            <ChevronDown className="h-4 w-4 shrink-0" />
                          ) : (
                            <ChevronRight className="h-4 w-4 shrink-0" />
                          )}
                          {group.panel ? (
                            <>
                              <PanelIcon icon={group.panel.icon} className="text-base" />
                              <span>{group.panel.name}</span>
                              <Badge variant="outline" className="ml-1 text-xs">
                                {group.items.length} 項
                              </Badge>
                            </>
                          ) : (
                            <>
                              <span className="text-base">📦</span>
                              <span>未分類</span>
                              <Badge variant="outline" className="ml-1 text-xs">
                                {group.items.length} 項
                              </Badge>
                            </>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  )}
                  {(!hasGroups || isExpanded) && group.items.map(renderTemplateRow)}
                </React.Fragment>
              )
            })
          ) : (
            <TableEmptyRow
              colSpan={7}
              icon={Droplets}
              title={search ? '找不到符合的檢查項目' : '尚無檢查項目資料'}
            />
          )}
        </TableBody>
      </Table>
    </div>
  )
}
