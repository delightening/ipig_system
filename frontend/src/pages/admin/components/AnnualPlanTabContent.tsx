/**
 * 年度校正/確效/查核計畫矩陣視圖
 * 支援自動產生 + 手動新增/編輯/刪除 + 計畫 vs 實際執行對照
 */
import { useState } from 'react'
import {
  AlertCircle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Circle,
  FileText,
  Loader2,
  Pencil,
  Plus,
  Trash2,
} from 'lucide-react'

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
import { EmptyState } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'

import type {
  AnnualPlanExecutionSummary,
  AnnualPlanWithEquipment,
  CalibrationType,
  CalibrationCycle,
  Equipment,
  MonthExecutionStatus,
} from '../types'
import { CALIBRATION_TYPE_LABELS, CALIBRATION_CYCLE_LABELS } from '../types'

interface AnnualPlanTabContentProps {
  canManage: boolean
  plans: AnnualPlanWithEquipment[]
  year: number
  onYearChange: (year: number) => void
  onGenerate: () => void
  isGenerating: boolean
  equipmentList: Equipment[]
  executionSummary: AnnualPlanExecutionSummary | null
  onCreatePlan: (data: Record<string, unknown>) => void
  onEditPlan: (id: string, data: Record<string, unknown>) => void
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onDeletePlan: (id: string) => void
}

const MONTHS = Array.from({ length: 12 }, (_, i) => `${i + 1}月`)

type StatusCellConfig = {
  icon: React.ElementType
  className: string
  title: string
}

const STATUS_CELL_CONFIG: Record<MonthExecutionStatus, StatusCellConfig> = {
  unplanned: {
    icon: Circle,
    className: 'invisible',
    title: '未計畫',
  },
  planned_pending: {
    icon: Circle,
    className: 'text-muted-foreground/40',
    title: '計畫待執行',
  },
  completed: {
    icon: CheckCircle2,
    className: 'text-status-success-text',
    title: '已執行',
  },
  overdue: {
    icon: AlertCircle,
    className: 'text-status-error-text',
    title: '逾期未執行',
  },
}

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
  executionSummary,
  onCreatePlan,
  onEditPlan,
  onToggleMonth,
  onDeletePlan,
}: AnnualPlanTabContentProps) {
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [editingPlan, setEditingPlan] = useState<AnnualPlanWithEquipment | null>(null)

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
        {executionSummary && plans.length > 0 && (
          <ExecutionSummaryCards summary={executionSummary} />
        )}
        {plans.length === 0 ? (
          <EmptyState icon={FileText} title={`${year} 年尚無年度計畫資料`} />
        ) : (
          <PlanMatrix
            plans={plans}
            canManage={canManage}
            executionSummary={executionSummary}
            onToggleMonth={onToggleMonth}
            onEdit={setEditingPlan}
            onDelete={onDeletePlan}
          />
        )}
      </CardContent>

      {canManage && (
        <>
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
          <EditPlanDialog
            plan={editingPlan}
            onOpenChange={(open) => { if (!open) setEditingPlan(null) }}
            onSubmit={(data) => {
              if (editingPlan) {
                onEditPlan(editingPlan.id, data)
                setEditingPlan(null)
              }
            }}
          />
        </>
      )}
    </Card>
  )
}

function ExecutionSummaryCards({ summary }: { summary: AnnualPlanExecutionSummary }) {
  const completionPct = summary.total_planned > 0
    ? Math.round(summary.completion_rate * 100)
    : 0
  const rateColor =
    completionPct >= 80
      ? 'text-status-success-text'
      : completionPct >= 50
        ? 'text-status-warning-text'
        : summary.total_planned > 0
          ? 'text-status-error-text'
          : 'text-muted-foreground'

  return (
    <div className="mb-4 grid grid-cols-4 gap-3">
      <div className="rounded-lg border bg-card p-3">
        <div className="text-xs text-muted-foreground">已計畫月份</div>
        <div className="mt-1 text-2xl font-semibold">{summary.total_planned}</div>
      </div>
      <div className="rounded-lg border bg-card p-3">
        <div className="flex items-center gap-1 text-xs text-muted-foreground">
          <CheckCircle2 className="h-3 w-3 text-status-success-text" />
          已完成
        </div>
        <div className="mt-1 text-2xl font-semibold text-status-success-text">
          {summary.total_completed}
        </div>
      </div>
      <div className="rounded-lg border bg-card p-3">
        <div className="flex items-center gap-1 text-xs text-muted-foreground">
          <AlertCircle className="h-3 w-3 text-status-error-text" />
          逾期未執行
        </div>
        <div className={`mt-1 text-2xl font-semibold ${summary.total_overdue > 0 ? 'text-status-error-text' : ''}`}>
          {summary.total_overdue}
        </div>
      </div>
      <div className="rounded-lg border bg-card p-3">
        <div className="text-xs text-muted-foreground">完成率</div>
        <div className={`mt-1 text-2xl font-semibold ${rateColor}`}>
          {summary.total_planned > 0 ? `${completionPct}%` : '—'}
        </div>
      </div>
    </div>
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
  executionSummary,
  onToggleMonth,
  onEdit,
  onDelete,
}: {
  plans: AnnualPlanWithEquipment[]
  canManage: boolean
  executionSummary: AnnualPlanExecutionSummary | null
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onEdit: (plan: AnnualPlanWithEquipment) => void
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
          {canManage && <TableHead className="w-[80px]" />}
        </TableRow>
      </TableHeader>
      <TableBody>
        {(sortedData ?? plans).map((plan) => {
          const execRow = executionSummary?.rows.find((r) => r.plan_id === plan.id) ?? null
          return (
            <PlanRow
              key={plan.id}
              plan={plan}
              canManage={canManage}
              execRow={execRow}
              onToggleMonth={onToggleMonth}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          )
        })}
      </TableBody>
    </Table>
  )
}

function PlanRow({
  plan,
  canManage,
  execRow,
  onToggleMonth,
  onEdit,
  onDelete,
}: {
  plan: AnnualPlanWithEquipment
  canManage: boolean
  execRow: AnnualPlanExecutionSummary['rows'][number] | null
  onToggleMonth: (plan: AnnualPlanWithEquipment, month: number) => void
  onEdit: (plan: AnnualPlanWithEquipment) => void
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
      {Array.from({ length: 12 }, (_, i) => i + 1).map((month) => {
        const monthDetail = execRow?.months[month - 1]
        const status: MonthExecutionStatus = monthDetail?.status
          ?? (isMonthScheduled(plan, month) ? 'planned_pending' : 'unplanned')
        const config = STATUS_CELL_CONFIG[status]
        const Icon = config.icon
        const isVisible = status !== 'unplanned'

        return (
          <TableCell key={month} className="text-center">
            {canManage ? (
              <button
                type="button"
                className="mx-auto flex h-7 w-7 items-center justify-center rounded hover:bg-muted transition-colors"
                onClick={() => onToggleMonth(plan, month)}
                title={config.title}
              >
                <Icon className={`h-5 w-5 ${config.className}`} />
              </button>
            ) : (
              isVisible && (
                <Icon className={`mx-auto h-5 w-5 ${config.className}`} />
              )
            )}
          </TableCell>
        )
      })}
      {canManage && (
        <TableCell>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => onEdit(plan)}
              title="編輯"
            >
              <Pencil className="h-4 w-4" />
            </Button>
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
          </div>
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

function EditPlanDialog({
  plan,
  onOpenChange,
  onSubmit,
}: {
  plan: AnnualPlanWithEquipment | null
  onOpenChange: (open: boolean) => void
  onSubmit: (data: Record<string, unknown>) => void
}) {
  const [calibrationType, setCalibrationType] = useState<CalibrationType>('calibration')
  const [cycle, setCycle] = useState<CalibrationCycle>('quarterly')
  const [months, setMonths] = useState<boolean[]>(Array(12).fill(false))

  const [prevPlanId, setPrevPlanId] = useState<string | null>(null)
  if (plan && plan.id !== prevPlanId) {
    setPrevPlanId(plan.id)
    setCalibrationType(plan.calibration_type)
    setCycle(plan.cycle)
    setMonths(Array.from({ length: 12 }, (_, i) => isMonthScheduled(plan, i + 1)))
  } else if (!plan && prevPlanId) {
    setPrevPlanId(null)
  }

  const handleSubmit = () => {
    const data: Record<string, unknown> = {
      calibration_type: calibrationType,
      cycle,
    }
    months.forEach((v, i) => {
      data[`month_${i + 1}`] = v
    })
    onSubmit(data)
  }

  const toggleMonth = (idx: number) => {
    setMonths((prev) => prev.map((v, i) => (i === idx ? !v : v)))
  }

  return (
    <Dialog open={!!plan} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            編輯計畫項目 — {plan?.equipment_name}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
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
          <Button onClick={handleSubmit}>儲存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
