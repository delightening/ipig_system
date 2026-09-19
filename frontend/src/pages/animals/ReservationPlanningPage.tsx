import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'

import { PageHeader } from '@/components/ui/page-header'
import { Button } from '@/components/ui/button'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useReservationPlanning } from './hooks/useReservationPlanning'
import { ReservationPlanningGroupCard } from './components/ReservationPlanningGroupCard'
import { CreatePlannedExperimentDialog } from './components/CreatePlannedExperimentDialog'
import { SparePoolPanel, type ReserveTarget } from './components/SparePoolPanel'

/** 動物預約與試驗規劃頁（Phase 5）：全場活豬按計劃分配清冊 — 置頂備用池 + 各計劃分組 + orphan。 */
export function ReservationPlanningPage() {
  const { t } = useTranslation()
  const [createOpen, setCreateOpen] = useState(false)
  const { data: groups, isLoading, isError } = useReservationPlanning()

  // 備用池「預約到計劃」下拉目標：所有非 orphan 計劃組
  const targets: ReserveTarget[] = useMemo(
    () =>
      (groups ?? [])
        .filter((g) => g.group_type !== 'orphan')
        .map((g): ReserveTarget => ({
          kind: g.group_type === 'approved' ? 'protocol' : 'planned',
          id: g.id,
          label:
            g.group_type === 'approved'
              ? [g.iacuc_no, g.unit].filter(Boolean).join(' · ') || t('animalPages.reservation.noCaseNo')
              : t('animalPages.reservation.planningLabel', { unit: g.unit }),
        })),
    [groups, t],
  )

  return (
    <div className="space-y-4">
      <PageHeader
        title={t('animalPages.reservation.title')}
        description={t('animalPages.reservation.description')}
        actions={
          <Can permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}>
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="mr-1 h-4 w-4" /> {t('animalPages.reservation.addPlanned')}
            </Button>
          </Can>
        }
      />

      {isLoading && <div className="py-12 text-center text-muted-foreground">{t('animalPages.shared.loadingEllipsis')}</div>}
      {isError && <div className="py-12 text-center text-status-error-text">{t('animalPages.reservation.loadError')}</div>}

      {!isLoading && !isError && (
        <>
          <SparePoolPanel targets={targets} />
          {(groups?.length ?? 0) === 0 && (
            <div className="rounded-xl border bg-card py-12 text-center text-muted-foreground">
              {/* 無操作權者看不到「新增預定試驗」鈕，空狀態文案不能叫他去點一顆不存在的按鈕 */}
              <Can
                permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}
                fallback={<>{t('animalPages.reservation.emptyNoPermission')}</>}
              >
                {t('animalPages.reservation.emptyCanManage')}
              </Can>
            </div>
          )}
          {groups?.map((g) => (
            <ReservationPlanningGroupCard key={`${g.group_type}:${g.id}`} group={g} />
          ))}
        </>
      )}

      <CreatePlannedExperimentDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}
