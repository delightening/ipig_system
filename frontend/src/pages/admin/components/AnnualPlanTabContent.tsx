/**
 * 年度校正/確效/查核計畫矩陣視圖
 * 支援自動產生 + 手動新增/編輯/刪除
 */
import { useState } from 'react'
import { CheckCircle2, ChevronLeft, ChevronRight, Loader2, Plus, Trash2, Circle } from 'lucide-react'

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
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { SortableTableHead } from '@/components/ui/sortable-table-head'

import type { AnnualPlanWithEquipment, Equipment, CalibrationType, CalibrationCycle } from '../types'
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
  equipmentList: Equipment[]
  onCreatePlan: (data: Record<string, unknown>) => void
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onDeletePlan: (id: string) => void
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
  equipmentList,
  onCreatePlan,
  onToggleMonth,
  onDeletePlan,
}: AnnualPlanTabContentProps) {
  const [showAddDialog, setShowAddDialog] = useState(false)

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">年度計畫</CardTitle>
          <div className="flex items-center gap-4">
            <YearSelector year={year} onYearChange={onYearChange} />
            {canManage && (
              <div className="flex items-center gap-2">
                <Button variant="outline" size="sm" onClick={() => setShowAddDialog(true)}>
                  <Plus className="mr-2 h-4 w-4" />
                  新增項目
                </Button>
                <Button onClick={onGenerate} disabled={isGenerating}>
                  {isGenerating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  產生年度計畫
                </Button>
              </div>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {plans.length === 0 ? (
          <EmptyState icon={FileText} title={`${year} 年尚無年度計畫資料`} />
        ) : (
          <PlanMatrix plans={plans} canManage={canManage} onToggleMonth={onToggleMonth} onDelete={onDeletePlan} />
        )}
      </CardContent>

      {canManage && (
        <AddPlanDialog
          open={showAddDialog}
          onOpenChange={setShowAddDialog}
          equipmentList={equipmentList}
          year={year}
          onSubmit={(data) => {
            onCreatePlan(data)
            setShowAddDialog(false)
          }}
        />
      )}
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

function PlanMatrix({
  plans,
  canManage,
  onToggleMonth,
  onDelete,
}: {
  plans: AnnualPlanWithEquipment[]
  canManage: boolean
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onDelete: (id: string) => void
}) {
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
          {canManage && <TableHead className="w-[60px]" />}
        </TableRow>
      </TableHeader>
      <TableBody>
        {(sortedData ?? plans).map((plan) => (
          <PlanRow key={plan.id} plan={plan} canManage={canManage} onToggleMonth={onToggleMonth} onDelete={onDelete} />
        ))}
      </TableBody>
    </Table>
  )
}

function PlanRow({
  plan,
  canManage,
  onToggleMonth,
  onDelete,
}: {
  plan: AnnualPlanWithEquipment
  canManage: boolean
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onDelete: (id: string) => void
}) {
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
          {canManage ? (
            <button
              type="button"
              className="mx-auto flex h-7 w-7 items-center justify-center rounded hover:bg-muted transition-colors"
              onClick={() => onToggleMonth(plan, month)}
              title={isMonthScheduled(plan, month) ? '點擊取消' : '點擊排程'}
            >
              {isMonthScheduled(plan, month) ? (
                <CheckCircle2 className="h-5 w-5 text-status-success-text" />
              ) : (
                <Circle className="h-5 w-5 text-muted-foreground/30" />
              )}
            </button>
          ) : (
            isMonthScheduled(plan, month) && (
              <CheckCircle2 className="mx-auto h-5 w-5 text-status-success-text" />
            )
          )}
        </TableCell>
      ))}
      {canManage && (
        <TableCell>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-destructive hover:text-destructive"
            onClick={() => {
              if (window.confirm(`確定要刪除「${plan.equipment_name}」的計畫項目嗎？`)) {
                onDelete(plan.id)
              }
            }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </TableCell>
      )}
    </TableRow>
  )
}

function AddPlanDialog({
  open,
  onOpenChange,
  equipmentList,
  year,
  onSubmit,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  equipmentList: Equipment[]
  year: number
  onSubmit: (data: Record<string, unknown>) => void
}) {
  const [equipmentId, setEquipmentId] = useState('')
  const [calibrationType, setCalibrationType] = useState<CalibrationType>('calibration')
  const [cycle, setCycle] = useState<CalibrationCycle>('quarterly')
  const [months, setMonths] = useState<boolean[]>(Array(12).fill(false))

  const activeEquipment = equipmentList.filter((e) => e.status === 'active')

  const handleSubmit = () => {
    if (!equipmentId) return
    const data: Record<string, unknown> = {
      year,
      equipment_id: equipmentId,
      calibration_type: calibrationType,
      cycle,
    }
    months.forEach((v, i) => {
      data[`month_${i + 1}`] = v
    })
    onSubmit(data)
    // reset
    setEquipmentId('')
    setMonths(Array(12).fill(false))
  }

  const toggleMonth = (idx: number) => {
    setMonths((prev) => prev.map((v, i) => (i === idx ? !v : v)))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>新增年度計畫項目</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label>設備 *</Label>
            <Select value={equipmentId} onValueChange={setEquipmentId}>
              <SelectTrigger>
                <SelectValue placeholder="選擇設備" />
              </SelectTrigger>
              <SelectContent>
                {activeEquipment.map((eq) => (
                  <SelectItem key={eq.id} value={eq.id}>
                    {eq.name} {eq.serial_number ? `(${eq.serial_number})` : ''}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>類型 *</Label>
              <Select value={calibrationType} onValueChange={(v) => setCalibrationType(v as CalibrationType)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(CALIBRATION_TYPE_LABELS).map(([k, v]) => (
                    <SelectItem key={k} value={k}>{v}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>週期 *</Label>
              <Select value={cycle} onValueChange={(v) => setCycle(v as CalibrationCycle)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(CALIBRATION_CYCLE_LABELS).map(([k, v]) => (
                    <SelectItem key={k} value={k}>{v}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="space-y-2">
            <Label>排程月份</Label>
            <div className="grid grid-cols-6 gap-2">
              {months.map((checked, idx) => (
                <button
                  key={idx}
                  type="button"
                  className={`flex items-center justify-center gap-1 rounded border px-2 py-1.5 text-sm transition-colors ${
                    checked
                      ? 'border-primary bg-primary/10 text-primary'
                      : 'border-border text-muted-foreground hover:bg-muted'
                  }`}
                  onClick={() => toggleMonth(idx)}
                >
                  {checked ? <CheckCircle2 className="h-3.5 w-3.5" /> : <Circle className="h-3.5 w-3.5" />}
                  {idx + 1}月
                </button>
              ))}
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button onClick={handleSubmit} disabled={!equipmentId}>新增</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
