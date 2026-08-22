import { useState, useMemo, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { ArrowLeft, FileDown } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/ui/page-header'
import { toast } from '@/components/ui/use-toast'
import { useDebounce } from '@/hooks/useDebounce'

import {
  availablePigsApi,
  type AvailablePigQuery,
  type AvailablePigListResponse,
} from '@/lib/api/animalAvailable'

/**
 * R47 — 可用豬隻快速查詢（庫存盤點）
 *
 * 規劃新 protocol 時用：依月齡 / 體重 / 性別篩出符合條件的可用豬，
 * 即時顯示總數 / 公母 / 品種分佈，可匯出 Excel。
 *
 * 詳見 docs/plans/r47-available-pigs-query.md。
 */
export function AvailablePigsPage() {
  const [sex, setSex] = useState<'' | 'male' | 'female'>('')
  const [ageMin, setAgeMin] = useState<string>('')
  const [ageMax, setAgeMax] = useState<string>('')
  const [weightMin, setWeightMin] = useState<string>('')
  const [weightMax, setWeightMax] = useState<string>('')
  const [includeBreeding, setIncludeBreeding] = useState(false)
  const [page, setPage] = useState(1)
  const perPage = 20

  // debounce filter 變動，避免每次按鍵都打 API
  const debouncedAgeMin = useDebounce(ageMin, 300)
  const debouncedAgeMax = useDebounce(ageMax, 300)
  const debouncedWeightMin = useDebounce(weightMin, 300)
  const debouncedWeightMax = useDebounce(weightMax, 300)

  // filter 變動時回到第 1 頁
  useEffect(() => {
    setPage(1)
  }, [sex, debouncedAgeMin, debouncedAgeMax, debouncedWeightMin, debouncedWeightMax, includeBreeding])

  const queryParams: AvailablePigQuery = useMemo(
    () => ({
      sex: sex === '' ? null : sex,
      age_months_min: debouncedAgeMin === '' ? null : Number(debouncedAgeMin),
      age_months_max: debouncedAgeMax === '' ? null : Number(debouncedAgeMax),
      weight_min: debouncedWeightMin === '' ? null : Number(debouncedWeightMin),
      weight_max: debouncedWeightMax === '' ? null : Number(debouncedWeightMax),
      include_breeding: includeBreeding,
      page,
      per_page: perPage,
    }),
    [sex, debouncedAgeMin, debouncedAgeMax, debouncedWeightMin, debouncedWeightMax, includeBreeding, page]
  )

  const { data, isLoading, isFetching, error } = useQuery({
    queryKey: ['animals', 'available', queryParams],
    queryFn: async () => {
      const res = await availablePigsApi.list(queryParams)
      return res.data as AvailablePigListResponse
    },
    placeholderData: (prev) => prev,
  })

  const handleExport = async () => {
    try {
      await availablePigsApi.exportXlsx(queryParams)
      toast({ title: '已開始下載', description: '可用豬隻 Excel 已產生' })
    } catch (err) {
      toast({
        title: '匯出失敗',
        description: err instanceof Error ? err.message : '請稍後再試',
        variant: 'destructive',
      })
    }
  }

  const animals = data?.animals ?? []
  const summary = data?.summary ?? { total: 0, male: 0, female: 0, by_breed: {}, excluded_weight_expired: 0 }
  const totalPages = Math.max(1, Math.ceil(summary.total / perPage))

  return (
    <div className="container mx-auto p-6 space-y-4">
      <div className="flex items-center gap-2">
        <Link to="/animals">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="size-4 mr-1" />
            返回動物列表
          </Button>
        </Link>
      </div>

      <PageHeader title="可用豬隻快速查詢" description="規劃 protocol 前的庫存盤點 — 依月齡 / 體重 / 性別篩選，即時統計可用豬。" />

      {/* 進階篩選 */}
      <div className="border rounded-lg p-4 bg-muted/30 space-y-3">
        <div className="text-sm font-semibold">進階篩選</div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          <div className="space-y-1">
            <Label htmlFor="sex">性別</Label>
            <select
              id="sex"
              className="w-full border rounded-md px-2 py-1.5 text-sm bg-background"
              value={sex}
              onChange={(e) => setSex(e.target.value as '' | 'male' | 'female')}
            >
              <option value="">全部</option>
              <option value="male">公</option>
              <option value="female">母</option>
            </select>
          </div>
          <div className="space-y-1">
            <Label>月齡（最小）</Label>
            <Input
              type="number"
              placeholder="例：24"
              value={ageMin}
              onChange={(e) => setAgeMin(e.target.value)}
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>月齡（最大）</Label>
            <Input
              type="number"
              placeholder="例：30"
              value={ageMax}
              onChange={(e) => setAgeMax(e.target.value)}
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>體重最小 (kg)</Label>
            <Input
              type="number"
              placeholder="例：20"
              value={weightMin}
              onChange={(e) => setWeightMin(e.target.value)}
              step="0.1"
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>體重最大 (kg)</Label>
            <Input
              type="number"
              placeholder="例：40"
              value={weightMax}
              onChange={(e) => setWeightMax(e.target.value)}
              step="0.1"
              min={0}
            />
          </div>
          <div className="flex items-end gap-2">
            <label className="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={includeBreeding}
                onChange={(e) => setIncludeBreeding(e.target.checked)}
                className="size-4"
              />
              包含飼養計畫 000 中的豬
            </label>
          </div>
          <div className="flex items-end justify-end col-span-1 md:col-span-2 lg:col-span-1 ml-auto">
            <Button onClick={handleExport} disabled={summary.total === 0} variant="outline">
              <FileDown className="size-4 mr-1" />
              匯出 Excel
            </Button>
          </div>
        </div>
      </div>

      {/* 統計列 */}
      <div className="flex flex-wrap gap-2 items-center">
        <Badge variant="default">
          符合 {summary.total} 頭 / ♂ {summary.male} / ♀ {summary.female}
        </Badge>
        {Object.entries(summary.by_breed).map(([breed, count]) => (
          <Badge key={breed} variant="secondary">
            {breed} {count}
          </Badge>
        ))}
        {summary.excluded_weight_expired > 0 && (
          <span className="text-xs text-muted-foreground ml-2">
            另有 {summary.excluded_weight_expired} 頭因體重資料 &gt; 40 天未列入
          </span>
        )}
      </div>

      {/* 結果表格 */}
      <div className="border rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-muted/50">
            <tr className="text-left">
              <th className="px-3 py-2">品種</th>
              <th className="px-3 py-2">耳號</th>
              <th className="px-3 py-2">性別</th>
              <th className="px-3 py-2">出生日</th>
              <th className="px-3 py-2 text-right">月齡</th>
              <th className="px-3 py-2 text-right">最近體重(kg)</th>
              <th className="px-3 py-2">量測日</th>
              <th className="px-3 py-2">位置</th>
              <th className="px-3 py-2">備註</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-muted-foreground">
                  載入中…
                </td>
              </tr>
            ) : error ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-destructive">
                  載入失敗
                </td>
              </tr>
            ) : animals.length === 0 ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-muted-foreground">
                  無符合條件的可用豬隻
                </td>
              </tr>
            ) : (
              animals.map((a) => (
                <tr key={a.id} className="border-t hover:bg-muted/30">
                  <td className="px-3 py-2">{breedLabel(a.breed)}</td>
                  <td className="px-3 py-2 font-mono">{a.ear_tag}</td>
                  <td className="px-3 py-2">{a.gender === 'male' ? '公' : '母'}</td>
                  <td className="px-3 py-2">{a.birth_date}</td>
                  <td className="px-3 py-2 text-right">{a.age_months}</td>
                  <td className="px-3 py-2 text-right">{Number(a.latest_weight_kg).toFixed(1)}</td>
                  <td className="px-3 py-2">{a.weight_measured_at}</td>
                  <td className="px-3 py-2">{a.pen_location ?? '—'}</td>
                  <td className="px-3 py-2 whitespace-pre-wrap break-words">{a.remark ?? '—'}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {summary.total > perPage && (
        <div className="flex items-center justify-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.max(1, p - 1))}
            disabled={page <= 1}
          >
            上一頁
          </Button>
          <span className="text-sm text-muted-foreground">
            第 {page} / {totalPages} 頁
            {isFetching && <span className="ml-2">…</span>}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page >= totalPages}
          >
            下一頁
          </Button>
        </div>
      )}
    </div>
  )
}

function breedLabel(breed: 'minipig' | 'white' | 'lyd' | 'other'): string {
  switch (breed) {
    case 'minipig':
      return '迷你豬'
    case 'white':
      return '白豬'
    case 'lyd':
      return 'LYD'
    case 'other':
      return '其他'
  }
}
