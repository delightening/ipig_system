import React, { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
          <Badge variant="success">{t('erpMaster.common.active')}</Badge>
        ) : (
          <Badge variant="secondary">{t('erpMaster.common.inactive')}</Badge>
        )}
      </TableCell>
      <TableCell className="text-right">
        <div className="flex items-center justify-end gap-1">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onEdit(template)}
            title={t('common.edit')}
            aria-label={t('common.edit')}
          >
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onToggle(template)}
            title={template.is_active ? t('erpMaster.products.actions.deactivate') : t('erpMaster.bloodTest.table.restore')}
            aria-label={template.is_active ? t('erpMaster.products.actions.deactivate') : t('erpMaster.bloodTest.table.restore')}
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
              {t('erpMaster.common.code')} <SortIndicator field="code" />
            </TableHead>
            <TableHead className="cursor-pointer" onClick={() => onSort('name')}>
              {t('erpMaster.common.name')} <SortIndicator field="name" />
            </TableHead>
            <TableHead className="w-[100px] cursor-pointer" onClick={() => onSort('default_unit')}>
              {t('erpMaster.common.unit')} <SortIndicator field="default_unit" />
            </TableHead>
            <TableHead className="w-[140px]">{t('erpMaster.bloodTest.referenceRange')}</TableHead>
            <TableHead
              className="w-[100px] cursor-pointer text-right"
              onClick={() => onSort('default_price')}
            >
              {t('erpMaster.bloodTest.table.price')} <SortIndicator field="default_price" />
            </TableHead>
            <TableHead className="w-[80px] text-center">{t('erpMaster.common.status')}</TableHead>
            <TableHead className="w-[120px] text-right">{t('common.actions')}</TableHead>
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
                                {t('erpMaster.bloodTest.table.itemCount', { count: group.items.length })}
                              </Badge>
                            </>
                          ) : (
                            <>
                              <span className="text-base">📦</span>
                              <span>{t('erpMaster.bloodTest.uncategorized')}</span>
                              <Badge variant="outline" className="ml-1 text-xs">
                                {t('erpMaster.bloodTest.table.itemCount', { count: group.items.length })}
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
              title={search ? t('erpMaster.bloodTest.table.noMatch') : t('erpMaster.bloodTest.table.empty')}
            />
          )}
        </TableBody>
      </Table>
    </div>
  )
}
