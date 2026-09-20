import { useState, useMemo, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
      toast({
        title: t('animalPages.availablePigs.toast.downloadStarted'),
        description: t('animalPages.availablePigs.toast.excelGenerated'),
      })
    } catch (err) {
      toast({
        title: t('common.exportFailed'),
        description: err instanceof Error ? err.message : t('animalPages.availablePigs.toast.retryLater'),
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
            {t('animalPages.availablePigs.backToList')}
          </Button>
        </Link>
      </div>

      <PageHeader
        title={t('animalPages.availablePigs.title')}
        description={t('animalPages.availablePigs.description')}
      />

      {/* 進階篩選 */}
      <div className="border rounded-lg p-4 bg-muted/30 space-y-3">
        <div className="text-sm font-semibold">{t('animalPages.availablePigs.advancedFilters')}</div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          <div className="space-y-1">
            <Label htmlFor="sex">{t('animals.gender')}</Label>
            <select
              id="sex"
              className="w-full border rounded-md px-2 py-1.5 text-sm bg-background"
              value={sex}
              onChange={(e) => setSex(e.target.value as '' | 'male' | 'female')}
            >
              <option value="">{t('animalPages.shared.all')}</option>
              <option value="male">{t('animals.genderLabels.male')}</option>
              <option value="female">{t('animals.genderLabels.female')}</option>
            </select>
          </div>
          <div className="space-y-1">
            <Label>{t('animalPages.availablePigs.ageMin')}</Label>
            <Input
              type="number"
              placeholder={t('animalPages.shared.example', { value: 24 })}
              value={ageMin}
              onChange={(e) => setAgeMin(e.target.value)}
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>{t('animalPages.availablePigs.ageMax')}</Label>
            <Input
              type="number"
              placeholder={t('animalPages.shared.example', { value: 30 })}
              value={ageMax}
              onChange={(e) => setAgeMax(e.target.value)}
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>{t('animalPages.availablePigs.weightMin')}</Label>
            <Input
              type="number"
              placeholder={t('animalPages.shared.example', { value: 20 })}
              value={weightMin}
              onChange={(e) => setWeightMin(e.target.value)}
              step="0.1"
              min={0}
            />
          </div>
          <div className="space-y-1">
            <Label>{t('animalPages.availablePigs.weightMax')}</Label>
            <Input
              type="number"
              placeholder={t('animalPages.shared.example', { value: 40 })}
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
              {t('animalPages.availablePigs.includeBreeding')}
            </label>
          </div>
          <div className="flex items-end justify-end col-span-1 md:col-span-2 lg:col-span-1 ml-auto">
            <Button onClick={handleExport} disabled={summary.total === 0} variant="outline">
              <FileDown className="size-4 mr-1" />
              {t('animalPages.availablePigs.exportExcel')}
            </Button>
          </div>
        </div>
      </div>

      {/* 統計列 */}
      <div className="flex flex-wrap gap-2 items-center">
        <Badge variant="default">
          {t('animalPages.availablePigs.summaryMatched', { total: summary.total, male: summary.male, female: summary.female })}
        </Badge>
        {Object.entries(summary.by_breed).map(([breed, count]) => (
          <Badge key={breed} variant="secondary">
            {breed} {count}
          </Badge>
        ))}
        {summary.excluded_weight_expired > 0 && (
          <span className="text-xs text-muted-foreground ml-2">
            {t('animalPages.availablePigs.summaryExcluded', { count: summary.excluded_weight_expired })}
          </span>
        )}
      </div>

      {/* 結果表格 */}
      <div className="border rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead className="bg-muted/50">
            <tr className="text-left">
              <th className="px-3 py-2">{t('animals.breed')}</th>
              <th className="px-3 py-2">{t('animals.earTag')}</th>
              <th className="px-3 py-2">{t('animals.gender')}</th>
              <th className="px-3 py-2">{t('animalPages.availablePigs.columns.birthDay')}</th>
              <th className="px-3 py-2 text-right">{t('animalPages.shared.ageMonths')}</th>
              <th className="px-3 py-2 text-right">{t('animalPages.availablePigs.columns.latestWeight')}</th>
              <th className="px-3 py-2">{t('animalPages.availablePigs.columns.measuredOn')}</th>
              <th className="px-3 py-2">{t('animalPages.availablePigs.columns.location')}</th>
              <th className="px-3 py-2">{t('animalPages.shared.remark')}</th>
            </tr>
          </thead>
          <tbody>
            {isLoading ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-muted-foreground">
                  {t('animalPages.shared.loadingEllipsis')}
                </td>
              </tr>
            ) : error ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-destructive">
                  {t('animalPages.availablePigs.loadFailed')}
                </td>
              </tr>
            ) : animals.length === 0 ? (
              <tr>
                <td colSpan={9} className="px-3 py-8 text-center text-muted-foreground">
                  {t('animalPages.availablePigs.empty')}
                </td>
              </tr>
            ) : (
              animals.map((a) => (
                <tr key={a.id} className="border-t hover:bg-muted/30">
                  <td className="px-3 py-2">{t(`animals.breedLabels.${a.breed}`)}</td>
                  <td className="px-3 py-2 font-mono">{a.ear_tag}</td>
                  <td className="px-3 py-2">{a.gender === 'male' ? t('animals.genderLabels.male') : t('animals.genderLabels.female')}</td>
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
            {t('common.previous')}
          </Button>
          <span className="text-sm text-muted-foreground">
            {t('animalPages.availablePigs.pagination.pageOf', { page, total: totalPages })}
            {isFetching && <span className="ml-2">…</span>}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
            disabled={page >= totalPages}
          >
            {t('common.next')}
          </Button>
        </div>
      )}
    </div>
  )
}
