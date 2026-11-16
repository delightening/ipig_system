import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { facilityApi } from '@/lib/api/facility'
import type { PenDetails, ZoneWithBuilding } from '@/types/facility'
import { PenSizeFields } from './PenSizeFields'
import { validatePenSize } from '../utils/penSize'

interface ApplyPenSizeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  zone: ZoneWithBuilding
  /** 該區目前列出的欄位，用來告知「套用到幾個、其中幾個已有尺寸會被覆蓋」 */
  pens: PenDetails[]
}

/** 把同一個長寬一次套用到整個區（單一交易、單一稽核事件，套用前的值會被記錄） */
export function ApplyPenSizeDialog({ open, onOpenChange, zone, pens }: ApplyPenSizeDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [length, setLength] = useState('')
  const [width, setWidth] = useState('')

  // 套用時兩格都必填；validatePenSize 對「兩格皆空」回合法，這裡另外擋
  const bothEmpty = length.trim() === '' && width.trim() === ''
  const sizeError = validatePenSize(length, width)
  const overwriteCount = pens.filter(p => p.length_m != null).length

  const close = () => {
    setLength('')
    setWidth('')
    onOpenChange(false)
  }

  const mutation = useMutation({
    mutationFn: () => facilityApi.applyPenSize(zone.id, { length_m: length.trim(), width_m: width.trim() }),
    onSuccess: res => {
      queryClient.invalidateQueries({ queryKey: ['pens'] })
      queryClient.invalidateQueries({ queryKey: ['facility-pens'] })
      toast({ title: t('admin.penLayoutPreview.applySizeSuccess', { count: res.data.updated_count }) })
      close()
    },
    onError: (err: unknown) =>
      toast({ title: t('admin.penLayoutPreview.applySizeFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const zoneLabel = t('admin.penLayoutPreview.zoneLabel', { building: zone.building_code, zone: zone.code })

  return (
    <Dialog open={open} onOpenChange={o => { if (!o) close(); else onOpenChange(true) }}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.penLayoutPreview.applySizeTitle')}</DialogTitle>
          <DialogDescription>{t('admin.penLayoutPreview.applySizeDescription', { zone: zoneLabel })}</DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <PenSizeFields
            length={length}
            width={width}
            onLengthChange={setLength}
            onWidthChange={setWidth}
            error={sizeError}
            required
          />
          <p className="text-sm">{t('admin.penLayoutPreview.applySizeAffects', { count: pens.length })}</p>
          {overwriteCount > 0 && (
            <p className="text-sm text-status-warning-solid" role="status">
              {t('admin.penLayoutPreview.applySizeOverwrite', { count: overwriteCount })}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={close}>{t('common.cancel')}</Button>
          <Button
            type="button"
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending || bothEmpty || sizeError !== null || pens.length === 0}
          >
            {mutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
            {t('admin.penLayoutPreview.applySizeSubmit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
