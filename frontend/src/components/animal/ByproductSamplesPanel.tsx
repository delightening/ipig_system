// R53-5: 廢棄物再利用紀錄 panel — collapsible，掛在 AnimalDetailPage 底部。
//
// 顯示條件：
//   1. hasPermission('animal.byproduct_sample.view')
//
// 新增 / 編輯 dialog 展開條件（#445）：
//   - 新增需要 `animalSacrificed`（動物已犧牲：status=euthanized / 犧牲確認）。
//     byproduct 主要來自計劃內犧牲（SD 填犧牲單），不綁安樂死單；建立走 animal-path。
//   - 編輯一律可開（用 existing.id）。
//
// PI / GUEST 無 view 權限 → 整個 panel 不渲染（R53-6 audit blacklist 配套
//   保證 PI 也看不到 audit log 內的此 entity_type 事件）。
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { ChevronDown, ChevronRight, Plus, Recycle, Trash2 } from 'lucide-react'

import { byproductSampleApi, type ByproductSample } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { toast } from '@/components/ui/use-toast'
import { useAuthHasPermission } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { formatDateTime } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/apiError'

import { ByproductSampleDialog } from './ByproductSampleDialog'

interface Props {
  animalId: string
  earTag: string
  /** 動物是否已犧牲（status=euthanized / 犧牲確認）。false → 新增採樣鈕 disabled。 */
  animalSacrificed: boolean
}

export function ByproductSamplesPanel({
  animalId,
  earTag,
  animalSacrificed,
}: Props) {
  const { t } = useTranslation()
  const hasPermission = useAuthHasPermission()
  const canView = hasPermission('animal.byproduct_sample.view')
  const canWrite = hasPermission('animal.byproduct_sample.write')

  const [expanded, setExpanded] = useState(false)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editing, setEditing] = useState<ByproductSample | undefined>(undefined)
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()

  const { data: samples = [], isLoading } = useQuery({
    queryKey: ['byproduct-samples', 'animal', animalId],
    queryFn: () => byproductSampleApi.listByAnimal(animalId).then((r) => r.data),
    enabled: canView,
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => byproductSampleApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['byproduct-samples', 'animal', animalId],
      })
      toast({ title: t('animalActions.byproduct.panel.deletedToast') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalActions.byproduct.panel.deleteFailed')),
        variant: 'destructive',
      })
    },
  })

  if (!canView) return null

  const handleAdd = () => {
    if (!animalSacrificed) {
      toast({
        title: t('animalActions.byproduct.panel.cannotAddTitle'),
        description: t('animalActions.byproduct.panel.cannotAddDescription'),
        variant: 'destructive',
      })
      return
    }
    setEditing(undefined)
    setDialogOpen(true)
  }

  const handleEdit = (s: ByproductSample) => {
    setEditing(s)
    setDialogOpen(true)
  }

  const handleDelete = async (s: ByproductSample) => {
    const ok = await confirm({
      title: t('animalActions.byproduct.panel.deleteConfirmTitle'),
      description: t('animalActions.byproduct.panel.deleteConfirmDescription', { time: formatDateTime(s.sampled_at), content: s.sample_content }),
      confirmLabel: t('common.delete'),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(s.id)
  }

  return (
    <section className="border rounded-lg bg-card">
      <button
        type="button"
        onClick={() => setExpanded((v) => !v)}
        className="w-full flex items-center gap-2 px-4 py-3 text-left hover:bg-muted/40 transition"
      >
        {expanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        <Recycle className="h-4 w-4 text-muted-foreground" />
        <span className="font-medium">{t('animalActions.byproduct.panel.title')}</span>
        <span className="text-sm text-muted-foreground ml-2">({samples.length})</span>
      </button>

      {expanded && (
        <div className="border-t px-4 py-3 space-y-3">
          {canWrite && (
            <div className="flex flex-col items-end gap-1">
              <Button size="sm" onClick={handleAdd} disabled={!animalSacrificed}>
                <Plus className="h-4 w-4 mr-1" />
                {t('animalActions.byproduct.panel.addSample')}
              </Button>
              {!animalSacrificed && (
                <p className="text-xs text-muted-foreground">
                  {t('animalActions.byproduct.panel.addRequiresSacrificed')}
                </p>
              )}
            </div>
          )}

          {isLoading ? (
            <p className="text-sm text-muted-foreground">{t('animalActions.common.loadingEllipsis')}</p>
          ) : samples.length === 0 ? (
            <p className="text-sm text-muted-foreground italic">
              {t('animalActions.byproduct.panel.empty')}
            </p>
          ) : (
            <ul className="space-y-2">
              {samples.map((s) => (
                <li
                  key={s.id}
                  className="border rounded p-3 text-sm flex justify-between items-start gap-3"
                >
                  <div className="flex-1 space-y-1">
                    <div className="font-medium">{s.sample_content}</div>
                    <div className="text-muted-foreground text-xs">
                      {t('animalActions.byproduct.panel.sampledAt', { time: formatDateTime(s.sampled_at) })}
                      {' / '}
                      {t('animalActions.byproduct.panel.requester', {
                        requester: s.requester_user_id
                          ? t('animalActions.byproduct.panel.requesterInternal', { id: s.requester_user_id })
                          : t('animalActions.byproduct.panel.requesterExternal', { org: s.requester_org_name ?? '', contact: s.requester_contact_name ?? '' }),
                      })}
                    </div>
                    {s.notes && <div className="text-xs text-muted-foreground">{t('animalActions.byproduct.panel.notes', { notes: s.notes })}</div>}
                  </div>
                  {canWrite && (
                    <div className="flex gap-1 shrink-0">
                      <Button size="sm" variant="ghost" onClick={() => handleEdit(s)}>
                        {t('common.edit')}
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => handleDelete(s)}
                        className="text-destructive hover:text-destructive"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      {/* 新增需動物已犧牲；編輯一律可開（用 existing.id） */}
      {dialogOpen && (editing || animalSacrificed) && (
        <ByproductSampleDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          animalId={animalId}
          earTag={earTag}
          existing={editing}
        />
      )}

      <ConfirmDialog state={dialogState} />
    </section>
  )
}
