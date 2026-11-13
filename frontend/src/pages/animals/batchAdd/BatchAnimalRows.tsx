/**
 * 批次新增動物的預覽表格。
 *
 * 版型依「容器寬度」（Tailwind `@container`，不寫 JS 斷點邏輯），斷點經使用者核准：
 * - 容器 ≥ 960px：完整欄寬並顯示序號
 * - 容器 480–959px：緊縮欄寬、隱藏序號（全部欄位都「逐隻不同」時容器小於 754px 會橫向捲動）
 * - 容器 < 480px：卡片版型，無橫向捲動
 *
 * 所有輸入欄在表格模式都不隱藏（隱藏就看不到必填輸入框）；「逐隻不同」才會出現的欄位
 * （品種、性別、出生日期、欄位）由 `modes` 決定。
 */
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Trash2 } from 'lucide-react'

import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { sanitizeDecimalInput, taipeiTodayIso } from '@/lib/utils'

import type { BatchRow, FieldModes, Gender, RowIssue } from './batchAddLogic'

export interface SpeciesOption {
  id: string
  name: string
}

export interface PenOption {
  id: string
  code: string
  /** 下拉選單中顯示的完整說明（棟／區／欄位與容量） */
  label: string
}

interface Props {
  rows: BatchRow[]
  modes: FieldModes
  issuesByKey: Record<string, RowIssue[]>
  speciesOptions: SpeciesOption[]
  penOptions: PenOption[]
  onChange: (key: string, patch: Partial<BatchRow>) => void
  onToggleConfirm: (key: string, confirmed: boolean) => void
  onRemove: (key: string) => void
}

function issueText(t: TFunction, issue: RowIssue): string {
  return t(`animalPages.batchAdd.issue.${issue.code}`, issue.params)
}

/** 單列的狀態：本地檢查的問題優先，其次是後端回報，最後才是「可建立」。 */
function RowStatus({
  row,
  issues,
  onToggleConfirm,
}: {
  row: BatchRow
  issues: RowIssue[]
  onToggleConfirm: Props['onToggleConfirm']
}) {
  const { t } = useTranslation()

  if (issues.length > 0) {
    const text = issues.map((i) => issueText(t, i)).join('；')
    return (
      <span className="text-status-error-text" title={text}>
        ✕ {text}
      </span>
    )
  }
  if (row.serverOutcome === 'duplicate_warning') {
    return (
      <span className="flex flex-col gap-1 text-status-warning-text">
        <span title={row.serverMessage}>⚠ {row.serverMessage}</span>
        <label className="flex items-center gap-1 text-foreground">
          <Checkbox
            checked={row.confirmForce}
            onCheckedChange={(v) => onToggleConfirm(row.key, v === true)}
            aria-label={t('animalPages.batchAdd.confirmDuplicate')}
          />
          {t('animalPages.batchAdd.confirmDuplicate')}
        </label>
      </span>
    )
  }
  if (row.serverOutcome === 'failed') {
    return (
      <span className="text-status-error-text" title={row.serverMessage}>
        ✕ {row.serverMessage}
      </span>
    )
  }
  return <span className="text-status-success-text">✓ {t('animalPages.batchAdd.statusOk')}</span>
}

function SpeciesSelect({
  value,
  options,
  onChange,
}: {
  value: string
  options: SpeciesOption[]
  onChange: (v: string) => void
}) {
  const { t } = useTranslation()
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="h-8 px-2">
        <SelectValue placeholder={t('animalPages.batchAdd.select')} />
      </SelectTrigger>
      <SelectContent>
        {options.map((sp) => (
          <SelectItem key={sp.id} value={sp.id}>
            {sp.name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

function GenderSelect({ value, onChange }: { value: Gender; onChange: (v: Gender) => void }) {
  const { t } = useTranslation()
  return (
    <Select value={value} onValueChange={(v) => onChange(v as Gender)}>
      <SelectTrigger className="h-8 px-2">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="male">{t('animals.genderLabels.male')}</SelectItem>
        <SelectItem value="female">{t('animals.genderLabels.female')}</SelectItem>
      </SelectContent>
    </Select>
  )
}

function PenSelect({
  value,
  options,
  onChange,
}: {
  value: string
  options: PenOption[]
  onChange: (v: string) => void
}) {
  const { t } = useTranslation()
  const selected = options.find((p) => p.id === value)
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="h-8 px-2">
        {/* 欄位下拉選項帶完整說明，但選定後只顯示欄位代碼，避免撐爆窄欄 */}
        <SelectValue placeholder={t('animalPages.batchAdd.select')}>{selected?.code}</SelectValue>
      </SelectTrigger>
      <SelectContent>
        {options.map((p) => (
          <SelectItem key={p.id} value={p.id}>
            {p.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

export function BatchAnimalRows({
  rows,
  modes,
  issuesByKey,
  speciesOptions,
  penOptions,
  onChange,
  onToggleConfirm,
  onRemove,
}: Props) {
  const { t } = useTranslation()
  const todayIso = taipeiTodayIso()
  const weightHeader = t('animalPages.shared.entryWeightKg')

  const removeButton = (row: BatchRow) => (
    <button
      type="button"
      onClick={() => onRemove(row.key)}
      aria-label={t('animalPages.batchAdd.removeRow', { earTag: row.earTag })}
      className="inline-flex h-7 w-7 items-center justify-center rounded border text-muted-foreground hover:bg-muted hover:text-status-error-text"
    >
      <Trash2 className="h-4 w-4" />
    </button>
  )

  return (
    <div className="@container">
      {/* 表格：容器 ≥ 480px */}
      <div className="hidden overflow-x-auto @[480px]:block">
        <table className="w-max min-w-full table-fixed text-sm">
          <thead>
            <tr className="bg-muted/50 text-left text-xs font-semibold">
              <th className="hidden w-[40px] px-1.5 py-1.5 @[960px]:table-cell">#</th>
              <th className="w-[80px] px-1.5 py-1.5 @[960px]:w-[90px]">{t('animals.earTag')}</th>
              {modes.species === 'each' && (
                <th className="w-[120px] px-1.5 py-1.5 @[960px]:w-[150px]">{t('animals.breed')}</th>
              )}
              {modes.gender === 'each' && (
                <th className="w-[80px] px-1.5 py-1.5 @[960px]:w-[90px]">{t('animals.gender')}</th>
              )}
              {modes.birthDate === 'each' && (
                <th className="w-[130px] px-1.5 py-1.5 @[960px]:w-[140px]">{t('animals.birthDate')}</th>
              )}
              <th className="w-[90px] px-1.5 py-1.5 @[960px]:w-[100px]">{weightHeader}</th>
              {modes.pen === 'each' && (
                <th className="w-[100px] px-1.5 py-1.5 @[960px]:w-[120px]">
                  {t('animalPages.addDialog.penNumber')}
                </th>
              )}
              <th className="w-[110px] px-1.5 py-1.5 @[960px]:w-[150px]">{t('animalPages.batchAdd.col.status')}</th>
              <th className="w-[44px] px-1.5 py-1.5">
                <span className="sr-only">{t('animalPages.batchAdd.col.remove')}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, index) => {
              const issues = issuesByKey[row.key] ?? []
              return (
                <tr key={row.key} className="border-t align-top">
                  <td className="hidden px-1.5 py-1.5 text-muted-foreground @[960px]:table-cell">{index + 1}</td>
                  <td className="px-1.5 py-1.5 font-medium">{row.earTag}</td>
                  {modes.species === 'each' && (
                    <td className="px-1.5 py-1.5">
                      <SpeciesSelect
                        value={row.speciesId}
                        options={speciesOptions}
                        onChange={(v) => onChange(row.key, { speciesId: v })}
                      />
                    </td>
                  )}
                  {modes.gender === 'each' && (
                    <td className="px-1.5 py-1.5">
                      <GenderSelect value={row.gender} onChange={(v) => onChange(row.key, { gender: v })} />
                    </td>
                  )}
                  {modes.birthDate === 'each' && (
                    <td className="px-1.5 py-1.5">
                      <Input
                        type="date"
                        className="h-8 px-2"
                        value={row.birthDate}
                        max={todayIso}
                        aria-label={`${t('animals.birthDate')} ${row.earTag}`}
                        onChange={(e) => onChange(row.key, { birthDate: e.target.value })}
                      />
                    </td>
                  )}
                  <td className="px-1.5 py-1.5">
                    <Input
                      type="text"
                      inputMode="decimal"
                      className="h-8 px-2"
                      value={row.weight}
                      aria-label={`${weightHeader} ${row.earTag}`}
                      onChange={(e) => onChange(row.key, { weight: sanitizeDecimalInput(e.target.value) })}
                    />
                  </td>
                  {modes.pen === 'each' && (
                    <td className="px-1.5 py-1.5">
                      <PenSelect
                        value={row.penId}
                        options={penOptions}
                        onChange={(v) => onChange(row.key, { penId: v })}
                      />
                    </td>
                  )}
                  <td className="whitespace-normal px-1.5 py-1.5 text-xs leading-tight">
                    <RowStatus row={row} issues={issues} onToggleConfirm={onToggleConfirm} />
                  </td>
                  <td className="px-1.5 py-1.5 text-center">{removeButton(row)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      {/* 卡片：容器 < 480px，每隻一張卡，所有輸入欄由上到下排，無橫向捲動 */}
      <div className="space-y-2 @[480px]:hidden">
        {rows.map((row, index) => {
          const issues = issuesByKey[row.key] ?? []
          return (
            <div key={row.key} className="rounded-lg border bg-card p-3 text-sm">
              <div className="mb-2 flex items-center justify-between font-semibold">
                <span>
                  #{index + 1} · {t('animals.earTag')} {row.earTag}
                </span>
                {removeButton(row)}
              </div>
              <div className="space-y-2">
                {modes.species === 'each' && (
                  <div className="space-y-1">
                    <div className="text-xs text-muted-foreground">{t('animals.breed')}</div>
                    <SpeciesSelect
                      value={row.speciesId}
                      options={speciesOptions}
                      onChange={(v) => onChange(row.key, { speciesId: v })}
                    />
                  </div>
                )}
                {modes.gender === 'each' && (
                  <div className="space-y-1">
                    <div className="text-xs text-muted-foreground">{t('animals.gender')}</div>
                    <GenderSelect value={row.gender} onChange={(v) => onChange(row.key, { gender: v })} />
                  </div>
                )}
                {modes.birthDate === 'each' && (
                  <div className="space-y-1">
                    <div className="text-xs text-muted-foreground">{t('animals.birthDate')}</div>
                    <Input
                      type="date"
                      className="h-8 px-2"
                      value={row.birthDate}
                      max={todayIso}
                      onChange={(e) => onChange(row.key, { birthDate: e.target.value })}
                    />
                  </div>
                )}
                <div className="space-y-1">
                  <div className="text-xs text-muted-foreground">{weightHeader}</div>
                  <Input
                    type="text"
                    inputMode="decimal"
                    className="h-8 px-2"
                    value={row.weight}
                    onChange={(e) => onChange(row.key, { weight: sanitizeDecimalInput(e.target.value) })}
                  />
                </div>
                {modes.pen === 'each' && (
                  <div className="space-y-1">
                    <div className="text-xs text-muted-foreground">{t('animalPages.addDialog.penNumber')}</div>
                    <PenSelect
                      value={row.penId}
                      options={penOptions}
                      onChange={(v) => onChange(row.key, { penId: v })}
                    />
                  </div>
                )}
                <div className="text-xs leading-tight">
                  <RowStatus row={row} issues={issues} onToggleConfirm={onToggleConfirm} />
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
