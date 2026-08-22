import { useMemo, useState } from 'react'
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
              ? [g.iacuc_no, g.unit].filter(Boolean).join(' · ') || '（無案號）'
              : `${g.unit}（規劃中）`,
        })),
    [groups],
  )

  return (
    <div className="space-y-4">
      <PageHeader
        title="動物預約與試驗規劃"
        description="全場活豬按計劃分配：置頂備用池快速配對，各計劃顯示需求 vs 已預約 / 實驗中 / 已完成缺口。"
        actions={
          <Can permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}>
            <Button onClick={() => setCreateOpen(true)}>
              <Plus className="mr-1 h-4 w-4" /> 新增預定試驗
            </Button>
          </Can>
        }
      />

      {isLoading && <div className="py-12 text-center text-muted-foreground">載入中…</div>}
      {isError && <div className="py-12 text-center text-status-error-text">載入失敗，請重新整理</div>}

      {!isLoading && !isError && (
        <>
          <SparePoolPanel targets={targets} />
          {(groups?.length ?? 0) === 0 && (
            <div className="rounded-xl border bg-card py-12 text-center text-muted-foreground">
              {/* 無操作權者看不到「新增預定試驗」鈕，空狀態文案不能叫他去點一顆不存在的按鈕 */}
              <Can
                permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}
                fallback={<>尚無試驗群組。待執行秘書建立預定試驗，或已核准計畫有動物預約 / 分配後顯示。</>}
              >
                尚無試驗群組。點右上「新增預定試驗」開始規劃，或待已核准計畫有動物預約 / 分配後顯示。
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
