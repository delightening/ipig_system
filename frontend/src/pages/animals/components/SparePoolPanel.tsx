import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { ArrowDown, ArrowUp, ChevronDown, ChevronUp, ChevronsUpDown, ExternalLink } from 'lucide-react'

import type { ReservableAnimalRow, ReservableQuery } from '@/lib/api/reservationPlanning'
import { animalGenderNames } from '@/types/animal'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useAuthHasPermission } from '@/stores/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useReservable, useReservationMutations } from '../hooks/useReservationPlanning'
import { EditableRemarkCell } from './EditableRemarkCell'
import { fmtDate, fmtMd } from './planningFormat'

/** 預約目標（計劃組），供「預約到計劃」下拉。 */
export interface ReserveTarget {
  kind: 'planned' | 'protocol'
  id: string
  label: string
}

/** 備用池可排序欄位（client 端排序）。 */
type SortKey = 'ear_tag' | 'gender' | 'birth_date' | 'weight'

/**
 * 未分配（備用池）置頂 sticky 面板（Phase 5）：全場未分配未預約活豬。
 * 條件篩選（性別/月齡/體重）+ 內部捲動 + 多選 → 批次「預約到計劃」。可收合。
 */
export function SparePoolPanel({ targets }: { targets: ReserveTarget[] }) {
  const [collapsed, setCollapsed] = useState(false)
  const [gender, setGender] = useState<'' | 'male' | 'female'>('')
  const [weightMin, setWeightMin] = useState('')
  const [weightMax, setWeightMax] = useState('')
  const [ageMin, setAgeMin] = useState('')
  const [ageMax, setAgeMax] = useState('')
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [targetKey, setTargetKey] = useState('')
  const hasPermission = useAuthHasPermission()
  // 預約屬「操作」，限執行秘書。無權者：不渲染勾選框與批次列 ——
  // 只留下可讀的備用池清單。若只藏批次列而留勾選框，使用者會選了半天發現沒有下一步。
  const canReserve = hasPermission(PERMISSIONS.ANIMAL_PLANNING_MANAGE)

  const query: ReservableQuery = useMemo(
    () => ({
      gender: gender || null,
      weight_min: weightMin ? Number(weightMin) : null,
      weight_max: weightMax ? Number(weightMax) : null,
      age_months_min: ageMin ? Number(ageMin) : null,
      age_months_max: ageMax ? Number(ageMax) : null,
    }),
    [gender, weightMin, weightMax, ageMin, ageMax],
  )

  const { data: rows, isFetching } = useReservable(query)
  const { reserve } = useReservationMutations()

  // 欄位排序（點表頭切 asc/desc；未選則維持後端預設耳號序）
  const [sortKey, setSortKey] = useState<SortKey | null>(null)
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc')
  const toggleSort = (k: SortKey) => {
    if (sortKey === k) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortKey(k)
      setSortDir('asc')
    }
  }

  const sortedRows = useMemo(() => {
    const list = rows ? [...rows] : []
    const key = sortKey
    if (!key) return list
    const val = (a: ReservableAnimalRow): string | number => {
      switch (key) {
        case 'ear_tag': {
          const n = Number(a.ear_tag)
          return Number.isNaN(n) ? a.ear_tag : n
        }
        case 'gender':
          return a.gender
        case 'birth_date':
          return a.birth_date ?? ''
        case 'weight':
          return a.latest_weight_kg == null ? -Infinity : Number(a.latest_weight_kg)
      }
    }
    list.sort((a, b) => {
      const va = val(a)
      const vb = val(b)
      const c =
        typeof va === 'number' && typeof vb === 'number'
          ? va - vb
          : String(va).localeCompare(String(vb))
      return sortDir === 'asc' ? c : -c
    })
    return list
  }, [rows, sortKey, sortDir])

  /** 可排序表頭儲存格（回傳 JSX，非巢狀元件，避開 react/no-unstable-nested-components）。 */
  const sortHead = (k: SortKey, label: string, w: string) => (
    <button
      type="button"
      onClick={() => toggleSort(k)}
      className={`flex items-center gap-0.5 text-left hover:text-foreground ${w}`}
      aria-label={`依${label}排序`}
    >
      {label}
      {sortKey === k ? (
        sortDir === 'asc' ? <ArrowUp className="h-3 w-3" /> : <ArrowDown className="h-3 w-3" />
      ) : (
        <ChevronsUpDown className="h-3 w-3 opacity-30" />
      )}
    </button>
  )

  const toggle = (id: string) =>
    setSelected((s) => {
      const n = new Set(s)
      if (n.has(id)) n.delete(id)
      else n.add(id)
      return n
    })

  const doReserve = () => {
    const target = targets.find((t) => `${t.kind}:${t.id}` === targetKey)
    if (!target || selected.size === 0) return
    const body =
      target.kind === 'planned'
        ? { animal_ids: [...selected], planned_experiment_id: target.id }
        : { animal_ids: [...selected], protocol_id: target.id }
    reserve.mutate(body, { onSuccess: () => setSelected(new Set()) })
  }

  const count = rows?.length ?? 0

  return (
    <section className="sticky top-14 z-20 mb-4 overflow-hidden rounded-xl border bg-card shadow-md md:top-16">
      {/* header */}
      <div className="flex flex-wrap items-center gap-2.5 border-b bg-card px-4 py-2.5">
        <span className="h-2.5 w-2.5 rounded-full bg-muted-foreground" />
        <span className="font-bold">未分配（備用池）</span>
        <span className="text-xs text-muted-foreground">共 {count} 隻未分配活豬</span>
        <Button
          variant="outline"
          size="sm"
          className="ml-auto h-7"
          onClick={() => setCollapsed((c) => !c)}
        >
          {collapsed ? <ChevronDown className="mr-1 h-3.5 w-3.5" /> : <ChevronUp className="mr-1 h-3.5 w-3.5" />}
          {collapsed ? '展開' : '收合'}
        </Button>
      </div>

      {!collapsed && (
        <>
          {/* 篩選列 */}
          <div className="flex flex-wrap items-end gap-2 border-b bg-muted/30 px-4 py-2">
            <label className="flex flex-col gap-0.5 text-[11px] text-muted-foreground">
              性別
              <select
                className="h-8 rounded-md border bg-background px-2 text-sm text-foreground"
                value={gender}
                onChange={(e) => setGender(e.target.value as '' | 'male' | 'female')}
              >
                <option value="">不限</option>
                <option value="male">公</option>
                <option value="female">母</option>
              </select>
            </label>
            <label className="flex flex-col gap-0.5 text-[11px] text-muted-foreground">
              月齡
              <span className="flex items-center gap-1">
                <Input className="h-8 w-16" type="number" value={ageMin} onChange={(e) => setAgeMin(e.target.value)} />
                <span>–</span>
                <Input className="h-8 w-16" type="number" value={ageMax} onChange={(e) => setAgeMax(e.target.value)} />
              </span>
            </label>
            <label className="flex flex-col gap-0.5 text-[11px] text-muted-foreground">
              體重(kg)
              <span className="flex items-center gap-1">
                <Input className="h-8 w-16" type="number" value={weightMin} onChange={(e) => setWeightMin(e.target.value)} />
                <span>–</span>
                <Input className="h-8 w-16" type="number" value={weightMax} onChange={(e) => setWeightMax(e.target.value)} />
              </span>
            </label>
          </div>

          {/* 清單（@container 表格↔卡片 @600px；內部捲動 max-h，含 header/篩選列後整個面板約 1/3–1/2 頁高） */}
          <div className="@container">
            <div className="max-h-[33vh] overflow-y-auto">
              {/* 桌面表頭 */}
              <div className="hidden border-b bg-muted/40 px-4 py-1.5 text-xs font-medium text-muted-foreground @[600px]:flex">
                <div className="w-8" />
                {sortHead('ear_tag', '耳號', 'w-16')}
                {sortHead('gender', '性別', 'w-12')}
                {sortHead('birth_date', '出生日期', 'w-24')}
                {sortHead('weight', '最新體重', 'w-24')}
                <div className="flex-1">備註</div>
                <div className="w-11 text-center">操作</div>
              </div>

              {isFetching && count === 0 && (
                <div className="px-4 py-6 text-center text-sm text-muted-foreground">載入中…</div>
              )}
              {!isFetching && count === 0 && (
                <div className="px-4 py-6 text-center text-sm text-muted-foreground">
                  無符合條件的未分配動物
                </div>
              )}

              {sortedRows.map((a) => (
                <div
                  key={a.id}
                  className="relative flex flex-col gap-1.5 border-b px-4 py-2.5 last:border-b-0 @[600px]:flex-row @[600px]:items-center @[600px]:gap-0 @[600px]:py-1.5"
                >
                  <div className="flex items-center gap-2 @[600px]:contents">
                    {canReserve && (
                      <input
                        type="checkbox"
                        className="@[600px]:w-8"
                        checked={selected.has(a.id)}
                        onChange={() => toggle(a.id)}
                        aria-label={`選取 ${a.ear_tag}`}
                      />
                    )}
                    <span className="w-16 font-mono font-semibold @[600px]:font-normal">{a.ear_tag}</span>
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
                  <div className="text-sm text-muted-foreground @[600px]:flex-1">
                    <span className="mr-1 text-[10px] @[600px]:hidden">備註</span>
                    <EditableRemarkCell animalId={a.id} remark={a.remark} />
                  </div>
                  <div className="absolute right-2 top-2 @[600px]:static @[600px]:w-11 @[600px]:text-center">
                    <Link
                      to={`/animals/${a.id}`}
                      className="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
                      aria-label="查看動物詳情"
                    >
                      <ExternalLink className="h-3.5 w-3.5" />
                    </Link>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 批次列（有操作權且選 ≥1 隻才顯示） */}
          {canReserve && selected.size > 0 && (
            <div className="flex flex-wrap items-center gap-2 border-t bg-accent/10 px-4 py-2 text-sm">
              <span className="font-semibold text-accent">已選 {selected.size} 隻 →</span>
              <span>預約到計劃</span>
              <select
                className="h-8 rounded-md border bg-background px-2 text-sm text-foreground"
                value={targetKey}
                onChange={(e) => setTargetKey(e.target.value)}
              >
                <option value="">選擇計劃…</option>
                {targets.map((t) => (
                  <option key={`${t.kind}:${t.id}`} value={`${t.kind}:${t.id}`}>
                    {t.label}
                  </option>
                ))}
              </select>
              <Button size="sm" className="h-8" disabled={!targetKey || reserve.isPending} onClick={doReserve}>
                預約
              </Button>
              <Button size="sm" variant="ghost" className="h-8" onClick={() => setSelected(new Set())}>
                取消選取
              </Button>
            </div>
          )}
        </>
      )}
    </section>
  )
}
