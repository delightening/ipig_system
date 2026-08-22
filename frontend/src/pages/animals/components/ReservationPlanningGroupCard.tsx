import { useState } from 'react'
import { Link } from 'react-router-dom'
import { ExternalLink, MoreHorizontal, Search } from 'lucide-react'

import type { PlanningAnimalRow, ReservationPlanningGroup } from '@/lib/api/reservationPlanning'
import { animalGenderNames } from '@/types/animal'
import { Button } from '@/components/ui/button'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useReservationMutations } from '../hooks/useReservationPlanning'
import { ReservationSearchDialog } from './ReservationSearchDialog'
import { EditableRemarkCell } from './EditableRemarkCell'
import { fmtDate, fmtMd } from './planningFormat'

/** relation → 狀態標籤（Phase 5 三態）。 */
function relationBadge(relation: string): { text: string; cls: string } {
  switch (relation) {
    case 'reserved':
      return { text: '已預約', cls: 'bg-status-warning-bg text-status-warning-text' }
    case 'completed':
      return { text: '實驗完成', cls: 'bg-status-purple-bg text-status-purple-text' }
    case 'in_experiment':
    default:
      return { text: '實驗中', cls: 'bg-status-success-bg text-status-success-text' }
  }
}

export function ReservationPlanningGroupCard({ group }: { group: ReservationPlanningGroup }) {
  const [searchOpen, setSearchOpen] = useState(false)
  const { unreserve, assign } = useReservationMutations()

  const approved = group.group_type === 'approved'
  const orphan = group.group_type === 'orphan'
  const shortfall = Math.max(
    0,
    group.demand - group.reserved_count - group.in_experiment_count - group.completed_count,
  )
  const title = orphan ? group.unit : approved ? (group.iacuc_no ?? '（無案號）') : group.unit
  const sub = approved
    ? [group.unit, group.pi_name].filter(Boolean).join(' · ')
    : group.description

  const target = {
    kind: approved ? ('protocol' as const) : ('planned' as const),
    id: group.id,
    label: title ?? group.unit ?? '試驗',
  }

  return (
    <div className="mb-3.5 overflow-hidden rounded-xl border bg-card">
      {/* header */}
      <div className="flex flex-wrap items-center gap-2.5 border-b px-4 py-3">
        <span
          className={`rounded-full px-2.5 py-0.5 text-[11px] font-bold ${
            orphan
              ? 'bg-muted text-muted-foreground'
              : approved
                ? 'bg-status-success-bg text-status-success-text'
                : 'bg-status-warning-bg text-status-warning-text'
          }`}
        >
          {orphan ? '待處理' : approved ? '已核准' : '規劃中'}
        </span>
        <span className="font-bold">{title}</span>
        {sub && <span className="text-xs text-muted-foreground">{sub}</span>}
        <div className="ml-auto flex flex-wrap items-center gap-1.5 text-xs tabular-nums">
          {!orphan && (
            <span className="rounded-full bg-status-info-bg px-2 py-0.5 font-bold text-status-info-text">
              需求 {group.demand}
            </span>
          )}
          <span className="rounded-full bg-status-success-bg px-2 py-0.5 font-bold text-status-success-text">
            實驗中 {group.in_experiment_count}
          </span>
          <span className="rounded-full bg-status-purple-bg px-2 py-0.5 font-bold text-status-purple-text">
            已完成 {group.completed_count}
          </span>
          <span className="rounded-full bg-status-warning-bg px-2 py-0.5 font-bold text-status-warning-text">
            已預約 {group.reserved_count}
          </span>
          {!orphan && shortfall > 0 && (
            <span className="rounded-full bg-status-error-bg px-2 py-0.5 font-bold text-status-error-text">
              缺 {shortfall}
            </span>
          )}
          {!orphan && (
            <Can permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}>
              <Button size="sm" variant="outline" className="ml-1 h-7" onClick={() => setSearchOpen(true)}>
                <Search className="mr-1 h-3.5 w-3.5" /> 搜尋配對
              </Button>
            </Can>
          )}
        </div>
      </div>

      {/* animals: @container 表格↔卡片 @600px */}
      <div className="@container">
        {group.animals.length === 0 ? (
          <div className="px-4 py-6 text-center text-sm text-muted-foreground">
            {orphan ? '無待處理動物' : '尚無預約 / 分配動物'}
          </div>
        ) : (
          <>
            {/* 桌面表頭 */}
            <div className="hidden border-b bg-muted/40 px-4 py-1.5 text-xs font-medium text-muted-foreground @[600px]:flex">
              <div className="w-16">耳號</div>
              <div className="w-12">性別</div>
              <div className="w-24">出生日期</div>
              <div className="w-24">最新體重</div>
              <div className="flex-1">備註</div>
              <div className="w-20">狀態</div>
              <div className="w-11 text-center">操作</div>
            </div>
            {group.animals.map((a) => (
              <AnimalRow
                key={a.id}
                animal={a}
                approved={approved}
                iacucNo={group.iacuc_no}
                onUnreserve={() => unreserve.mutate([a.id])}
                onAssign={() => group.iacuc_no && assign.mutate({ animalIds: [a.id], iacucNo: group.iacuc_no })}
              />
            ))}
          </>
        )}
      </div>

      {!orphan && (
        <ReservationSearchDialog open={searchOpen} onOpenChange={setSearchOpen} target={target} />
      )}
    </div>
  )
}

function AnimalRow({
  animal: a,
  approved,
  iacucNo,
  onUnreserve,
  onAssign,
}: {
  animal: PlanningAnimalRow
  approved: boolean
  iacucNo: string | null
  onUnreserve: () => void
  onAssign: () => void
}) {
  const reserved = a.relation === 'reserved'
  const badge = relationBadge(a.relation)
  const statusBadge = (
    <span className={`inline-block rounded-full px-2 py-0.5 text-[11px] ${badge.cls}`}>{badge.text}</span>
  )

  const menu = (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="h-7 w-7" aria-label="操作">
          <MoreHorizontal className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {/* 分配 / 解除預約限執秘；無權者選單只留「查看動物詳情」（唯讀動作） */}
        <Can permission={PERMISSIONS.ANIMAL_PLANNING_MANAGE}>
          {reserved && (
            <>
              {approved && iacucNo ? (
                <DropdownMenuItem onSelect={onAssign}>正式分配進實驗</DropdownMenuItem>
              ) : (
                <DropdownMenuItem disabled>正式分配（試驗核准後）</DropdownMenuItem>
              )}
              <DropdownMenuItem onSelect={onUnreserve}>解除預約</DropdownMenuItem>
              <DropdownMenuSeparator />
            </>
          )}
        </Can>
        <DropdownMenuItem asChild>
          <Link to={`/animals/${a.id}`}>
            <ExternalLink className="h-3.5 w-3.5" /> 查看動物詳情
          </Link>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )

  return (
    <div className="relative flex flex-col gap-1.5 border-b px-4 py-2.5 last:border-b-0 @[600px]:flex-row @[600px]:items-center @[600px]:gap-0 @[600px]:py-1.5">
      {/* 卡片頂列：耳號 + 狀態 */}
      <div className="flex items-center gap-2 @[600px]:contents">
        <span className="w-16 font-mono font-semibold @[600px]:font-normal">{a.ear_tag}</span>
        <span className="@[600px]:hidden">{statusBadge}</span>
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm @[600px]:contents">
        <span className="@[600px]:w-12">
          <span className="mr-1 text-[10px] text-muted-foreground @[600px]:hidden">性別</span>
          {animalGenderNames[a.gender]}
        </span>
        <span className="@[600px]:w-24">
          <span className="mr-1 text-[10px] text-muted-foreground @[600px]:hidden">出生</span>
          {fmtDate(a.birth_date)}
        </span>
        <span className="@[600px]:w-24">
          <span className="mr-1 text-[10px] text-muted-foreground @[600px]:hidden">最新體重</span>
          {a.latest_weight_kg ? (
            <>
              <b className="font-semibold">{a.latest_weight_kg}kg</b>{' '}
              <span className="text-[10px] text-muted-foreground">{fmtMd(a.weight_measured_at)}</span>
            </>
          ) : (
            '—'
          )}
        </span>
      </div>
      {/* 備註（Phase 5：整格 inline 編輯） */}
      <div className="text-sm text-muted-foreground @[600px]:flex-1">
        <span className="mr-1 text-[10px] @[600px]:hidden">備註</span>
        <EditableRemarkCell animalId={a.id} remark={a.remark} />
      </div>
      {/* 桌面狀態欄 */}
      <div className="hidden @[600px]:block @[600px]:w-20">{statusBadge}</div>
      {/* 操作 ⋯（卡片右上絕對定位、桌面列內） */}
      <div className="absolute right-2 top-2 @[600px]:static @[600px]:w-11 @[600px]:text-center">{menu}</div>
    </div>
  )
}
