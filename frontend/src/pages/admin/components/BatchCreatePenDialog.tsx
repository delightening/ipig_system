import { useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FormField } from '@/components/ui/form-field'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Loader2 } from 'lucide-react'
import api from '@/lib/api'
import type { ZoneWithBuilding } from '@/types/facility'

interface BatchCreatePenDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  zones: ZoneWithBuilding[]
}

const batchPenSchema = z.object({
  zone_id: z.string().min(1, '請選擇區域'),
  prefix: z.string().min(1, '代碼前綴為必填'),
  count: z.coerce.number().int().min(1, '數量至少為 1').max(200, '數量不可超過 200'),
  layout: z.enum(['single', 'double']),
  capacity: z.coerce.number().int().min(1),
})

type BatchPenFormData = z.output<typeof batchPenSchema>

const INITIAL: BatchPenFormData = { zone_id: '', prefix: '', count: 20, layout: 'double', capacity: 1 }

export function BatchCreatePenDialog({ open, onOpenChange, zones }: BatchCreatePenDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const { register, handleSubmit, reset, setValue, watch, formState: { errors, isValid } } = useForm<z.input<typeof batchPenSchema>, unknown, BatchPenFormData>({
    resolver: zodResolver(batchPenSchema),
    defaultValues: INITIAL,
    mode: 'onChange',
  })

  const zoneId = watch('zone_id')
  const prefix = watch('prefix')
  const count = watch('count') as number
  const layout = watch('layout') as 'single' | 'double'
  const preview = useMemo(() => generatePreview({ prefix, count, layout } as BatchPenFormData), [prefix, count, layout])

  const mutation = useMutation({
    mutationFn: (data: BatchPenFormData) => api.post('/facilities/pens/batch', {
      zone_id: data.zone_id,
      prefix: data.prefix,
      count: data.count,
      layout: data.layout,
      capacity: data.capacity,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pens'] })
      onOpenChange(false)
      reset(INITIAL)
      toast({ title: '批次建立成功', description: `已建立 ${count} 個欄位` })
    },
    onError: (err: unknown) => {
      toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' })
    },
  })

  const onSubmit = handleSubmit(data => mutation.mutate(data))

  const selectedZone = zones.find(z => z.id === zoneId)

  return (
    <Dialog open={open} onOpenChange={o => { if (!o) reset(INITIAL); onOpenChange(o) }}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>批次建立欄位</DialogTitle>
          <DialogDescription>快速建立多個欄位，自動分配代碼與排列位置</DialogDescription>
        </DialogHeader>

        <form onSubmit={onSubmit} className="space-y-4">
          <FormField label="區域" required error={errors.zone_id?.message}>
            <Select value={zoneId} onValueChange={v => setValue('zone_id', v, { shouldValidate: true })}>
              <SelectTrigger><SelectValue placeholder="選擇區域" /></SelectTrigger>
              <SelectContent>
                {zones.map(z => (
                  <SelectItem key={z.id} value={z.id}>
                    {z.building_code}棟 {z.code}區 {z.name ? `(${z.name})` : ''}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </FormField>

          <div className="grid grid-cols-2 gap-4">
            <FormField label="代碼前綴" required error={errors.prefix?.message}>
              <Input
                {...register('prefix', { onChange: e => { e.target.value = e.target.value.toUpperCase() } })}
                placeholder="例如：Q"
                maxLength={10}
              />
            </FormField>
            <FormField label="數量" required error={errors.count?.message}>
              <Input
                type="number"
                min={1}
                max={200}
                {...register('count', { valueAsNumber: true })}
              />
            </FormField>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>排列模式</Label>
              <Select value={layout} onValueChange={v => setValue('layout', v as 'single' | 'double')}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="double">兩欄並排</SelectItem>
                  <SelectItem value="single">單欄</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>每欄容量</Label>
              <Input
                type="number"
                min={1}
                {...register('capacity', { valueAsNumber: true })}
              />
            </div>
          </div>

          {prefix && count > 0 && (
            <div>
              <Label className="text-sm">預覽排列</Label>
              <div className="mt-1 p-3 bg-muted rounded border font-mono text-xs max-h-48 overflow-y-auto">
                {selectedZone?.color && (
                  <div className="flex items-center gap-2 mb-2 text-muted-foreground">
                    <span className="w-3 h-3 rounded" style={{ backgroundColor: selectedZone.color }} />
                    {selectedZone.code} {selectedZone.name || ''}
                  </div>
                )}
                {layout === 'double' ? (
                  <div className="grid grid-cols-2 gap-x-8 gap-y-0.5">
                    {preview.map((row, i) => (
                      <div key={i}>{row}</div>
                    ))}
                  </div>
                ) : (
                  <div className="space-y-0.5">
                    {preview.map((code, i) => (
                      <div key={i}>{code}</div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
            <Button type="submit" disabled={!isValid || mutation.isPending}>
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
              建立 {count} 個欄位
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function generatePreview(form: BatchPenFormData): string[] {
  if (!form.prefix || form.count <= 0) return []
  const codes = Array.from({ length: Math.min(form.count, 200) }, (_, i) =>
    `${form.prefix}${String(i + 1).padStart(2, '0')}`
  )

  if (form.layout === 'single') return codes

  // 兩欄並排：左欄 1~half，右欄 half+1~count
  const half = Math.ceil(form.count / 2)
  const rows: string[] = []
  for (let i = 0; i < half; i++) {
    const left = codes[i]
    const right = i + half < codes.length ? codes[i + half] : ''
    rows.push(left)
    rows.push(right)
  }
  return rows
}
