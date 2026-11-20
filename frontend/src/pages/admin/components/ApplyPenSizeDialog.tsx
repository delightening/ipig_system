import { useState, useSyncExternalStore, type RefObject } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { AxiosError } from 'axios'
import { Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { facilityApi } from '@/lib/api/facility'
import type { PenDetails, ZoneWithBuilding } from '@/types/facility'
import { PenSizeFields } from './PenSizeFields'
import { validatePenSize } from '../utils/penSize'

function subscribeOnline(onChange: () => void) {
  window.addEventListener('online', onChange)
  window.addEventListener('offline', onChange)
  return () => {
    window.removeEventListener('online', onChange)
    window.removeEventListener('offline', onChange)
  }
}

interface ApplyPenSizeDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  zone: ZoneWithBuilding
  /** 該區目前列出的欄位，用來告知「套用到幾個、其中幾個已有尺寸會被覆蓋」 */
  pens: PenDetails[]
  /** 關閉時把焦點還給開啟它的按鈕（本對話框由 state 開啟、沒有 DialogTrigger，Radix 不知道要回哪裡） */
  returnFocusRef?: RefObject<HTMLElement | null>
}

/** 把同一個長寬一次套用到整個區（單一交易、單一稽核事件，套用前的值會被記錄） */
export function ApplyPenSizeDialog({ open, onOpenChange, zone, pens, returnFocusRef }: ApplyPenSizeDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [length, setLength] = useState('')
  const [width, setWidth] = useState('')
  // 離線時 TanStack Query 會把寫入暫停排隊，對話框又在送出中不可關閉，使用者會被鎖住；
  // 所以離線就直接停用「套用」，不讓請求進入暫停狀態
  const online = useSyncExternalStore(subscribeOnline, () => navigator.onLine, () => true)

  // 套用時兩格都必填；validatePenSize 對「兩格皆空」回合法，這裡另外擋
  const bothEmpty = length.trim() === '' && width.trim() === ''
  const sizeError = validatePenSize(length, width)
  const overwriteCount = pens.filter(p => p.length_m != null).length

  const close = () => {
    setLength('')
    setWidth('')
    onOpenChange(false)
  }

  const refreshPens = () => {
    queryClient.invalidateQueries({ queryKey: ['pens'] })
    queryClient.invalidateQueries({ queryKey: ['facility-pens'] })
  }

  // 送出的長寬在點擊當下就固定成變數，不在 mutation 真正執行時才讀 state
  const mutation = useMutation({
    mutationFn: (size: { length_m: string; width_m: string }) => facilityApi.applyPenSize(zone.id, size),
    onSuccess: res => {
      refreshPens()
      toast({ title: t('admin.penLayoutPreview.applySizeSuccess', { count: res.data.updated_count }) })
      close()
    },
    onError: (err: unknown) => {
      // 沒收到回應或伺服器端錯誤時，不能確定寫入有沒有成功：重新載入欄位資料並如實告知，
      // 讓使用者確認後再決定要不要重試；明確的 4xx 拒絕則維持原本「修正後再送」流程
      const unknownOutcome = err instanceof AxiosError && (!err.response || err.response.status >= 500)
      if (unknownOutcome) refreshPens()
      toast({
        title: t(unknownOutcome ? 'admin.penLayoutPreview.applySizeUnknown' : 'admin.penLayoutPreview.applySizeFailed'),
        description: getApiErrorMessage(err),
        variant: 'destructive',
      })
    },
  })

  const zoneLabel = t('admin.penLayoutPreview.zoneLabel', { building: zone.building_code, zone: zone.code })

  return (
    // 送出中不可關閉：關掉對話框並不會取消已送出的寫入，使用者會誤以為沒套用
    <Dialog open={open} onOpenChange={o => { if (mutation.isPending) return; if (!o) close(); else onOpenChange(true) }}>
      <DialogContent
        onCloseAutoFocus={e => {
          // 沒有 DialogTrigger，Radix 無處可回；有給目標才接手，沒給就維持預設
          if (!returnFocusRef?.current) return
          e.preventDefault()
          returnFocusRef.current.focus()
        }}
      >
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
          {!online && (
            <p className="text-sm text-destructive" role="alert">
              {t('admin.penLayoutPreview.applySizeOffline')}
            </p>
          )}
          {overwriteCount > 0 && (
            <p className="text-sm text-status-warning-solid" role="status">
              {t('admin.penLayoutPreview.applySizeOverwrite', { count: overwriteCount })}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button type="button" variant="outline" onClick={close} disabled={mutation.isPending}>{t('common.cancel')}</Button>
          <Button
            type="button"
            onClick={() => mutation.mutate({ length_m: length.trim(), width_m: width.trim() })}
            disabled={mutation.isPending || !online || bothEmpty || sizeError !== null || pens.length === 0}
          >
            {mutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
            {t('admin.penLayoutPreview.applySizeSubmit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
