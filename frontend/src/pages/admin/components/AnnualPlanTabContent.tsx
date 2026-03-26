/**
 * 年度校正/確效/查核計畫矩陣視圖
 */
import { CheckCircle2, ChevronLeft, ChevronRight, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'

import type { AnnualPlanWithEquipment } from '../types'
import { CALIBRATION_TYPE_LABELS, CALIBRATION_CYCLE_LABELS } from '../types'
import { EmptyState } from '@/components/ui/empty-state'
import { FileText } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'

interface AnnualPlanTabContentProps {
  canManage: boolean
  plans: AnnualPlanWithEquipment[]
  year: number
  onYearChange: (year: number) => void
  onGenerate: () => void
  isGenerating: boolean
}

const MONTHS = Array.from({ length: 12 }, (_, i) => `${i + 1}月`)

function isMonthScheduled(plan: AnnualPlanWithEquipment, month: number): boolean {
  const key = `month_${month}` as keyof AnnualPlanWithEquipment
  return plan[key] as boolean
}

export default function AnnualPlanTabContent({
  canManage,
  plans,
  year,
  onYearChange,
  onGenerate,
  isGenerating,
}: AnnualPlanTabContentProps) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">年度計畫</CardTitle>
          <div className="flex items-center gap-4">
            <YearSelector year={year} onYearChange={onYearChange} />
            {canManage && (
              <Button onClick={onGenerate} disabled={isGenerating}>
                {isGenerating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                產生年度計畫
              </Button>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {plans.length === 0 ? (
          <EmptyState icon={FileText} title={`${year} 年尚無年度計畫資料`} />
        ) : (
          <PlanMatrix plans={plans} />
        )}
      </CardContent>
    </Card>
  )
}

function YearSelector({
  year,
  onYearChange,
}: {
  year: number
  onYearChange: (year: number) => void
}) {
  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size="icon"
        onClick={() => onYearChange(year - 1)}
        aria-label="前一年"
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>
      <span className="min-w-[4rem] text-center font-semibold">{year}</span>
      <Button
        variant="outline"
        size="icon"
        onClick={() => onYearChange(year + 1)}
        aria-label="後一年"
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    </div>
  )
}

function PlanMatrix({ plans }: { plans: AnnualPlanWithEquipment[] }) {
  const { sortedData, sort, toggleSort } = useTableSort(plans)

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <SortableTableHead sortKey="equipment_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="min-w-[160px]">設備名稱</SortableTableHead>
          <SortableTableHead sortKey="calibration_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="min-w-[100px]">類型</SortableTableHead>
          {MONTHS.map((m) => (
            <TableHead key={m} className="text-center">
              {m}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {(sortedData ?? plans).map((plan) => (
          <PlanRow key={plan.id} plan={plan} />
        ))}
      </TableBody>
    </Table>
  )
}

function PlanRow({ plan }: { plan: AnnualPlanWithEquipment }) {
  return (
    <TableRow>
      <TableCell>
        <div className="font-medium">{plan.equipment_name}</div>
        {plan.equipment_serial_number && (
          <div className="text-xs text-muted-foreground">
            {plan.equipment_serial_number}
          </div>
        )}
      </TableCell>
      <TableCell>
        <div>{CALIBRATION_TYPE_LABELS[plan.calibration_type]}</div>
        <div className="text-xs text-muted-foreground">
          {CALIBRATION_CYCLE_LABELS[plan.cycle]}
        </div>
      </TableCell>
      {Array.from({ length: 12 }, (_, i) => i + 1).map((month) => (
        <TableCell key={month} className="text-center">
          {isMonthScheduled(plan, month) && (
            <CheckCircle2 className="mx-auto h-5 w-5 text-status-success-text" />
          )}
        </TableCell>
      ))}
    </TableRow>
  )
}
