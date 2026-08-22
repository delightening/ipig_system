import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import { reservationPlanningApi } from '@/lib/api/reservationPlanning'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

export function CreatePlannedExperimentDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (o: boolean) => void
}) {
  const [unit, setUnit] = useState('')
  const [description, setDescription] = useState('')
  const [demand, setDemand] = useState('')
  const qc = useQueryClient()

  const create = useMutation({
    mutationFn: () =>
      reservationPlanningApi.createPlannedExperiment({
        unit: unit.trim(),
        description: description.trim() || null,
        demand_count: Number(demand) || 0,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reservation-planning'] })
      toast({ title: '已新增', description: '預定試驗已建立' })
      setUnit('')
      setDescription('')
      setDemand('')
      onOpenChange(false)
    },
    onError: (e: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(e, '新增失敗'), variant: 'destructive' }),
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>新增預定試驗</DialogTitle>
          <DialogDescription>規劃中（未核准）試驗，供預約動物；核准後連結真計畫。</DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">
              委託單位 <span className="text-status-error-text">*</span>
            </span>
            <Input value={unit} onChange={(e) => setUnit(e.target.value)} placeholder="如：昱展新藥" />
          </label>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">試驗內容概述</span>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="如：心臟支架長期試驗"
            />
          </label>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">需求動物數</span>
            <Input
              type="number"
              min="0"
              value={demand}
              onChange={(e) => setDemand(e.target.value)}
              className="w-32"
            />
          </label>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button disabled={!unit.trim() || create.isPending} onClick={() => create.mutate()}>
            建立
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
