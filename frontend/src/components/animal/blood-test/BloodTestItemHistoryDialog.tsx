/**
 * R30-16: 血檢 item 修正歷史時間軸
 *
 * 顯示某筆血檢的所有 items（含 superseded rows），按項目分組 + 修正鏈時間序排列。
 */
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { bloodTestApi } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import type { AnimalBloodTestItem } from '@/types'

interface Props {
  open: boolean
  testId: string | null
  onOpenChange: (open: boolean) => void
}

interface Chain {
  /** 此 chain 的最新版（current row） */
  current: AnimalBloodTestItem
  /** 由舊到新的歷史；最後一筆即 current */
  history: AnimalBloodTestItem[]
}

/** 把 items[] 重組為 chain[]：按 superseded_by_id 串成鏈。
 * O(n) 預建 predecessor map（誰的 superseded_by_id 指向 X），避免每筆都掃全 array。 */
function buildChains(items: AnimalBloodTestItem[]): Chain[] {
  const predecessorOf = new Map<string, AnimalBloodTestItem>()
  for (const it of items) {
    if (it.superseded_by_id) predecessorOf.set(it.superseded_by_id, it)
  }
  const currents = items.filter((i) => !i.superseded_by_id)
  return currents.map((cur) => {
    const history: AnimalBloodTestItem[] = [cur]
    let pointer: AnimalBloodTestItem = cur
    while (true) {
      const predecessor = predecessorOf.get(pointer.id)
      if (!predecessor) break
      history.unshift(predecessor)
      pointer = predecessor
    }
    return { current: cur, history }
  })
}

export function BloodTestItemHistoryDialog({ open, testId, onOpenChange }: Props) {
  const { t } = useTranslation()
  const { data, isLoading, isError } = useQuery({
    queryKey: ['blood-test-item-history', testId],
    queryFn: () => bloodTestApi.itemHistory(testId!).then((r) => r.data),
    enabled: !!testId && open,
  })

  const chains: Chain[] = data ? buildChains(data) : []

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg" className="max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('animalRecords.bloodTest.historyTitle')}</DialogTitle>
          <DialogDescription>
            {t('animalRecords.bloodTest.historyDescription')}
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : isError ? (
          <div className="py-8 text-center text-status-error-text">
            {t('animalRecords.bloodTest.historyLoadFailed')}
          </div>
        ) : chains.length === 0 ? (
          <div className="py-8 text-center text-muted-foreground">{t('animalRecords.bloodTest.historyEmpty')}</div>
        ) : (
          <div className="space-y-4">
            {chains.map((chain) => (
              <div key={chain.current.id} className="border rounded-lg p-3 space-y-2">
                <div className="font-medium">{chain.current.item_name}</div>
                {chain.history.length === 1 ? (
                  <div className="text-xs text-muted-foreground">
                    {t('animalRecords.bloodTest.noCorrections')}
                  </div>
                ) : (
                  <ol className="space-y-2 text-sm">
                    {chain.history.map((h, idx) => {
                      const isLast = idx === chain.history.length - 1
                      return (
                        <li
                          key={h.id}
                          className={`pl-3 border-l-2 ${
                            isLast ? 'border-primary' : 'border-muted'
                          }`}
                        >
                          <div className="flex items-center gap-2">
                            <span
                              className={`text-xs px-2 py-0.5 rounded ${
                                isLast
                                  ? 'bg-primary text-primary-foreground'
                                  : 'bg-muted text-muted-foreground'
                              }`}
                            >
                              {isLast ? t('animalRecords.bloodTest.versionCurrent') : t('animalRecords.bloodTest.versionN', { n: idx + 1 })}
                            </span>
                            <span className="text-xs text-muted-foreground">
                              {new Date(h.created_at).toLocaleString(uiLocale(), {
                                timeZone: 'Asia/Taipei',
                              })}
                            </span>
                          </div>
                          <div className="mt-1 grid grid-cols-2 gap-x-4 gap-y-1">
                            <div>
                              <span className="text-muted-foreground">{t('animalRecords.bloodTest.resultColon')}</span>
                              {h.result_value || '—'} {h.result_unit || ''}
                            </div>
                            <div>
                              <span className="text-muted-foreground">{t('animalRecords.bloodTest.referenceColon')}</span>
                              {h.reference_range || '—'}
                            </div>
                            <div>
                              <span className="text-muted-foreground">{t('animalRecords.bloodTest.abnormalColon')}</span>
                              {h.is_abnormal ? t('common.yes') : t('common.no')}
                            </div>
                            <div>
                              <span className="text-muted-foreground">{t('animalRecords.bloodTest.remarkColon')}</span>
                              {h.remark || '—'}
                            </div>
                          </div>
                          {h.superseded_at && h.correction_reason && (
                            <div className="mt-1 text-xs text-status-warning-text">
                              <span className="font-medium">{t('animalRecords.bloodTest.correctionReasonColon')}</span>
                              {h.correction_reason}
                              <span className="ml-2 text-muted-foreground">
                                {t('animalRecords.bloodTest.supersededAt', {
                                  time: new Date(h.superseded_at).toLocaleString(uiLocale(), {
                                    timeZone: 'Asia/Taipei',
                                  }),
                                })}
                              </span>
                            </div>
                          )}
                        </li>
                      )
                    })}
                  </ol>
                )}
              </div>
            ))}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
