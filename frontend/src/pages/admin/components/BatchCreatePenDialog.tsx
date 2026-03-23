import { useState, useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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

interface BatchForm {
  zone_id: string
  prefix: string
  count: number
  layout: 'single' | 'double'
  capacity: number
}

const INITIAL: BatchForm = { zone_id: '', prefix: '', count: 20, layout: 'double', capacity: 1 }

export function BatchCreatePenDialog({ open, onOpenChange, zones }: BatchCreatePenDialogProps) {
  const queryClient = useQueryClient()
  const [form, setForm] = useState<BatchForm>(INITIAL)

  const preview = useMemo(() => generatePreview(form), [form.prefix, form.count, form.layout])

  const mutation = useMutation({
    mutationFn: () => api.post('/facilities/pens/batch', {
      zone_id: form.zone_id,
      prefix: form.prefix,
      count: form.count,
      layout: form.layout,
      capacity: form.capacity,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pens'] })
      onOpenChange(false)
      setForm(INITIAL)
      toast({ title: '批次建立成功', description: `已建立 ${form.count} 個欄位` })
    },
    onError: (err: unknown) => {
      toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' })
    },
  })

  const set = <K extends keyof BatchForm>(k: K, v: BatchForm[K]) =>
    setForm(prev => ({ ...prev, [k]: v }))

  const selectedZone = zones.find(z => z.id === form.zone_id)
  const canSubmit = form.zone_id && form.prefix && form.count > 0 && form.count <= 200

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>批次建立欄位</DialogTitle>
          <DialogDescription>快速建立多個欄位，自動分配代碼與排列位置</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div>
            <Label>區域 *</Label>
            <Select value={form.zone_id} onValueChange={v => set('zone_id', v)}>
              <SelectTrigger><SelectValue placeholder="選擇區域" /></SelectTrigger>
              <SelectContent>
                {zones.map(z => (
                  <SelectItem key={z.id} value={z.id}>
                    {z.building_code} / {z.code} {z.name ? `(${z.name})` : ''}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>代碼前綴 *</Label>
              <Input
                value={form.prefix}
                onChange={e => set('prefix', e.target.value.toUpperCase())}
                placeholder="例如：Q"
                maxLength={10}
              />
            </div>
            <div>
              <Label>數量 *</Label>
              <Input
                type="number"
                min={1}
                max={200}
                value={form.count}
                onChange={e => set('count', parseInt(e.target.value) || 0)}
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>排列模式</Label>
              <Select value={form.layout} onValueChange={v => set('layout', v as 'single' | 'double')}>
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
                value={form.capacity}
                onChange={e => set('capacity', parseInt(e.target.value) || 1)}
              />
            </div>
          </div>

          {form.prefix && form.count > 0 && (
            <div>
              <Label className="text-sm">預覽排列</Label>
              <div className="mt-1 p-3 bg-slate-50 rounded border font-mono text-xs max-h-48 overflow-y-auto">
                {selectedZone?.color && (
                  <div className="flex items-center gap-2 mb-2 text-muted-foreground">
                    <span className="w-3 h-3 rounded" style={{ backgroundColor: selectedZone.color }} />
                    {selectedZone.code} {selectedZone.name || ''}
                  </div>
                )}
                {form.layout === 'double' ? (
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
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button onClick={() => mutation.mutate()} disabled={!canSubmit || mutation.isPending}>
            {mutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
            建立 {form.count} 個欄位
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function generatePreview(form: BatchForm): string[] {
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
