import { useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FormField } from '@/components/ui/form-field'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Loader2 } from 'lucide-react'
import api from '@/lib/api'
import type { ZoneWithBuilding } from '@/types/facility'

interface BatchCreatePenDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  zones: ZoneWithBuilding[]
}

interface BatchPenFormData {
  zone_id: string
  prefix: string
  count: number
  layout: 'single' | 'double'
  capacity: number
}

const INITIAL: BatchPenFormData = { zone_id: '', prefix: '', count: 20, layout: 'double', capacity: 1 }

export function BatchCreatePenDialog({ open, onOpenChange, zones }: BatchCreatePenDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  const { register, handleSubmit, reset, setValue, watch, formState: { errors, isValid } } = useForm<BatchPenFormData>({
    defaultValues: INITIAL,
    mode: 'onChange',
  })

  const zoneId = watch('zone_id')
  const prefix = watch('prefix')
  const count = watch('count')
  const layout = watch('layout')
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
      queryClient.invalidateQueries({ queryKey: ['facility-pens'] })
      onOpenChange(false)
      reset(INITIAL)
      toast({ title: t('admin.batchCreatePenDialog.successTitle'), description: t('admin.batchCreatePenDialog.successDescription', { count }) })
    },
    onError: (err: unknown) => {
      toast({ title: t('admin.batchCreatePenDialog.errorTitle'), description: getApiErrorMessage(err), variant: 'destructive' })
    },
  })

  const onSubmit = handleSubmit(data => mutation.mutate(data))

  const selectedZone = zones.find(z => z.id === zoneId)

  return (
    <Dialog open={open} onOpenChange={o => { if (!o) reset(INITIAL); onOpenChange(o) }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.batchCreatePenDialog.title')}</DialogTitle>
          <DialogDescription>{t('admin.batchCreatePenDialog.description')}</DialogDescription>
        </DialogHeader>

        <form onSubmit={onSubmit} className="space-y-4">
          <FormField label={t('admin.batchCreatePenDialog.zoneLabel')} required error={errors.zone_id?.message}>
            <input type="hidden" {...register('zone_id', { required: t('admin.batchCreatePenDialog.zoneRequired') })} />
            <Select value={zoneId} onValueChange={v => setValue('zone_id', v, { shouldValidate: true })}>
              <SelectTrigger><SelectValue placeholder={t('admin.batchCreatePenDialog.zonePlaceholder')} /></SelectTrigger>
              <SelectContent>
                {zones.map(z => (
                  <SelectItem key={z.id} value={z.id}>
                    {t('admin.batchCreatePenDialog.zoneOption', { building: z.building_code, zone: z.code })} {z.name ? `(${z.name})` : ''}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </FormField>

          <div className="grid grid-cols-2 gap-4">
            <FormField label={t('admin.batchCreatePenDialog.prefixLabel')} required error={errors.prefix?.message}>
              <Input
                {...register('prefix', {
                  required: t('admin.batchCreatePenDialog.prefixRequired'),
                  onChange: e => { e.target.value = e.target.value.toUpperCase() },
                })}
                placeholder={t('admin.batchCreatePenDialog.prefixPlaceholder')}
                maxLength={10}
              />
            </FormField>
            <FormField label={t('admin.batchCreatePenDialog.countLabel')} required error={errors.count?.message}>
              <Input
                type="number"
                min={1}
                max={200}
                {...register('count', {
                  valueAsNumber: true,
                  min: { value: 1, message: t('admin.batchCreatePenDialog.countMin') },
                  max: { value: 200, message: t('admin.batchCreatePenDialog.countMax') },
                })}
              />
            </FormField>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <FormField label={t('admin.batchCreatePenDialog.layoutLabel')}>
              <Select value={layout} onValueChange={v => setValue('layout', v as 'single' | 'double')}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="double">{t('admin.batchCreatePenDialog.layoutDouble')}</SelectItem>
                  <SelectItem value="single">{t('admin.batchCreatePenDialog.layoutSingle')}</SelectItem>
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.batchCreatePenDialog.capacityLabel')}>
              <Input
                type="number"
                min={1}
                {...register('capacity', {
                  valueAsNumber: true,
                  min: { value: 1, message: t('admin.batchCreatePenDialog.capacityMin') },
                })}
              />
            </FormField>
          </div>

          {prefix && count > 0 && (
            <div>
              <Label className="text-sm">{t('admin.batchCreatePenDialog.previewLabel')}</Label>
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
              {t('admin.batchCreatePenDialog.submit', { count })}
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
