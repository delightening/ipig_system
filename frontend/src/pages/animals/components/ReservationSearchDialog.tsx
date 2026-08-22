import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import { reservationPlanningApi, type ReservableQuery } from '@/lib/api/reservationPlanning'
import { animalGenderNames } from '@/types/animal'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { useReservationMutations } from '../hooks/useReservationPlanning'

interface Props {
  open: boolean
  onOpenChange: (o: boolean) => void
  /** 預約目標：規劃試驗或已核准計畫 */
  target: { kind: 'planned' | 'protocol'; id: string; label: string }
}

/** 搜尋配對 modal：依體重/月齡/性別篩未分配未預約動物 → 多選 → 批次預約進此試驗（Phase 4）。 */
export function ReservationSearchDialog({ open, onOpenChange, target }: Props) {
  const [gender, setGender] = useState<'' | 'male' | 'female'>('')
  const [weightMin, setWeightMin] = useState('')
  const [weightMax, setWeightMax] = useState('')
  const [ageMin, setAgeMin] = useState('')
  const [ageMax, setAgeMax] = useState('')
  const [selected, setSelected] = useState<Set<string>>(new Set())

  const query: ReservableQuery = {
    gender: gender || null,
    weight_min: weightMin ? Number(weightMin) : null,
    weight_max: weightMax ? Number(weightMax) : null,
    age_months_min: ageMin ? Number(ageMin) : null,
    age_months_max: ageMax ? Number(ageMax) : null,
  }

  const { data: rows, isFetching } = useQuery({
    queryKey: ['reservable', query],
    queryFn: () => reservationPlanningApi.searchReservable(query).then((r) => r.data),
    enabled: open,
    staleTime: 10_000,
  })

  const { reserve } = useReservationMutations()

  const toggle = (id: string) =>
    setSelected((s) => {
      const n = new Set(s)
      if (n.has(id)) n.delete(id)
      else n.add(id)
      return n
    })

  const doReserve = () => {
    const body =
      target.kind === 'planned'
        ? { animal_ids: [...selected], planned_experiment_id: target.id }
        : { animal_ids: [...selected], protocol_id: target.id }
    reserve.mutate(body, {
      onSuccess: () => {
        setSelected(new Set())
        onOpenChange(false)
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>搜尋配對動物 — {target.label}</DialogTitle>
          <DialogDescription>
            依體重 / 月齡 / 性別篩選未分配未預約動物，勾選後批次預約進此試驗。
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-wrap items-end gap-2">
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            性別
            <select
              className="h-9 rounded-md border bg-background px-2 text-sm text-foreground"
              value={gender}
              onChange={(e) => setGender(e.target.value as '' | 'male' | 'female')}
            >
              <option value="">不限</option>
              <option value="male">公</option>
              <option value="female">母</option>
            </select>
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            體重下限(kg)
            <Input className="w-24" type="number" value={weightMin} onChange={(e) => setWeightMin(e.target.value)} />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            體重上限(kg)
            <Input className="w-24" type="number" value={weightMax} onChange={(e) => setWeightMax(e.target.value)} />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            月齡下限
            <Input className="w-20" type="number" value={ageMin} onChange={(e) => setAgeMin(e.target.value)} />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            月齡上限
            <Input className="w-20" type="number" value={ageMax} onChange={(e) => setAgeMax(e.target.value)} />
          </label>
        </div>

        <div className="max-h-[380px] overflow-y-auto rounded-md border">
          <table className="w-full text-sm">
            <thead className="sticky top-0 bg-muted">
              <tr>
                <th className="w-10 p-2"></th>
                <th className="p-2 text-left">耳號</th>
                <th className="p-2 text-left">性別</th>
                <th className="p-2 text-left">出生</th>
                <th className="p-2 text-left">最新體重</th>
                <th className="p-2 text-left">月齡</th>
              </tr>
            </thead>
            <tbody>
              {isFetching && (
                <tr>
                  <td colSpan={6} className="p-4 text-center text-muted-foreground">
                    搜尋中…
                  </td>
                </tr>
              )}
              {!isFetching && (rows?.length ?? 0) === 0 && (
                <tr>
                  <td colSpan={6} className="p-4 text-center text-muted-foreground">
                    查無符合條件的可預約動物
                  </td>
                </tr>
              )}
              {rows?.map((a) => (
                <tr key={a.id} className="cursor-pointer border-t hover:bg-muted/50" onClick={() => toggle(a.id)}>
                  <td className="p-2 text-center">
                    <input
                      type="checkbox"
                      checked={selected.has(a.id)}
                      onChange={() => toggle(a.id)}
                      onClick={(e) => e.stopPropagation()}
                    />
                  </td>
                  <td className="p-2 font-mono">{a.ear_tag}</td>
                  <td className="p-2">{animalGenderNames[a.gender]}</td>
                  <td className="p-2">{a.birth_date ?? '—'}</td>
                  <td className="p-2">{a.latest_weight_kg ? `${a.latest_weight_kg}kg` : '—'}</td>
                  <td className="p-2">{a.age_months != null ? `${a.age_months}月` : '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button disabled={selected.size === 0 || reserve.isPending} onClick={doReserve}>
            預約選取的 {selected.size} 隻
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
