import { useState, useMemo, useCallback } from 'react'
import { useTabState } from '@/hooks/useTabState'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useQuery } from '@tanstack/react-query'
import { bloodTestAnalysisApi, bloodTestPanelApi, bloodTestPresetApi } from '@/lib/api'
import type { BloodTestAnalysisRow, BloodTestPanel } from '@/types'
import { formatDate } from '@/lib/utils'

function calcBoxPlot(values: number[]) {
  if (values.length === 0) return null
  const sorted = [...values].sort((a, b) => a - b)
  const n = sorted.length
  return {
    min: sorted[0],
    q1: sorted[Math.floor(n * 0.25)],
    median: n % 2 === 0 ? (sorted[n / 2 - 1] + sorted[n / 2]) / 2 : sorted[Math.floor(n / 2)],
    q3: sorted[Math.floor(n * 0.75)],
    max: sorted[n - 1],
  }
}

export interface BoxPlotData {
  name: string
  min: number
  q1: number
  median: number
  q3: number
  max: number
  unit?: string
  count: number
}

export function useBloodTestAnalysis() {
  const [iacucNo, setIacucNo] = useState('')
  const [earTag, setEarTag] = useState('')
  const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter()
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const { activeTab, setActiveTab } = useTabState<'trend' | 'boxplot' | 'table'>('trend')

  const queryParams = useMemo(() => {
    const params = new URLSearchParams()
    if (iacucNo.trim()) params.set('iacuc_no', iacucNo.trim())
    if (dateFrom) params.set('date_from', dateFrom)
    if (dateTo) params.set('date_to', dateTo)
    return params.toString()
  }, [iacucNo, dateFrom, dateTo])

  const { data: rawData, isLoading } = useQuery<BloodTestAnalysisRow[]>({
    queryKey: ['blood-test-analysis', queryParams],
    queryFn: async () => {
      const response = await bloodTestAnalysisApi.query(queryParams)
      return response.data
    },
  })

  const { data: panelsData } = useQuery<BloodTestPanel[]>({
    queryKey: ['blood-test-panels-all'],
    queryFn: async () => {
      const response = await bloodTestPanelApi.listAll()
      return response.data
    },
  })

  const { data: presetsData } = useQuery({
    queryKey: ['blood-test-presets'],
    queryFn: async () => {
      const response = await bloodTestPresetApi.list()
      return response.data
    },
  })

  const filteredData = useMemo(() => {
    if (!rawData) return []
    if (!earTag.trim()) return rawData
    const search = earTag.trim().toLowerCase()
    return rawData.filter(r => r.ear_tag.toLowerCase().includes(search))
  }, [rawData, earTag])

  const availableItems = useMemo(() => {
    const items = new Set<string>()
    filteredData.forEach(r => items.add(r.item_name))
    return Array.from(items).sort()
  }, [filteredData])

  const groupedAnalysisOptions = useMemo(() => {
    const allTemplateNames = new Set<string>()
    const groups: { key: string; label: string; items: { name: string }[] }[] = []

    if (panelsData) {
      const activePanels = panelsData
        .filter(p => p.is_active && p.key !== 'TUBE')
        .sort((a, b) => (a.sort_order ?? 0) - (b.sort_order ?? 0))
      for (const panel of activePanels) {
        const items = (panel.items ?? []).filter(t => t.is_active !== false).map(t => ({ name: t.name }))
        if (items.length > 0) {
          items.forEach(t => allTemplateNames.add(t.name))
          groups.push({ key: panel.key, label: panel.name, items })
        }
      }
    }

    const otherItems = availableItems.filter(n => !allTemplateNames.has(n))
    if (otherItems.length > 0) {
      groups.push({ key: 'OTHER_DATA', label: '其他（本次資料）', items: otherItems.map(name => ({ name })) })
    }

    return groups
  }, [panelsData, availableItems])

  const presetItemNames = useMemo(() => {
    const map = new Map<string, string[]>()
    groupedAnalysisOptions.forEach(g => map.set(g.key, g.items.map(i => i.name)))
    return map
  }, [groupedAnalysisOptions])

  const applyPreset = useCallback((keys: string[]) => {
    const names = keys.flatMap(k => presetItemNames.get(k) ?? [])
    setSelectedItems(names)
  }, [presetItemNames])

  const chartFilteredData = useMemo(() => {
    if (selectedItems.length === 0) return filteredData
    return filteredData.filter(r => selectedItems.includes(r.item_name))
  }, [filteredData, selectedItems])

  const summary = useMemo(() => {
    if (filteredData.length === 0) return { totalItems: 0, abnormalCount: 0, abnormalRate: 0, animalCount: 0, testDates: 0 }
    const abnormal = filteredData.filter(r => r.is_abnormal)
    const animals = new Set(filteredData.map(r => r.animal_id))
    const dates = new Set(filteredData.map(r => `${r.animal_id}_${r.test_date}`))
    return {
      totalItems: filteredData.length,
      abnormalCount: abnormal.length,
      abnormalRate: (abnormal.length / filteredData.length) * 100,
      animalCount: animals.size,
      testDates: dates.size,
    }
  }, [filteredData])

  const abnormalRecords = useMemo(() => filteredData.filter(r => r.is_abnormal), [filteredData])

  const trendData = useMemo(() => {
    if (chartFilteredData.length === 0) return { chartData: [], animals: [] as string[] }
    const animalSet = new Set<string>()
    chartFilteredData.forEach(r => animalSet.add(r.ear_tag))
    const animals = Array.from(animalSet).sort()

    const dateMap = new Map<string, Record<string, number | string>>()
    chartFilteredData.forEach(r => {
      const val = r.result_value ? parseFloat(r.result_value) : NaN
      if (isNaN(val)) return
      if (!dateMap.has(r.test_date)) dateMap.set(r.test_date, { date: formatDate(r.test_date) })
      dateMap.get(r.test_date)![r.ear_tag] = val
    })

    const chartData = Array.from(dateMap.values()).sort((a, b) => String(a.date).localeCompare(String(b.date)))
    return { chartData, animals }
  }, [chartFilteredData])

  const boxPlotData = useMemo((): BoxPlotData[] => {
    if (filteredData.length === 0) return []
    const itemMap = new Map<string, { values: number[]; unit?: string }>()
    filteredData.forEach(r => {
      const val = r.result_value ? parseFloat(r.result_value) : NaN
      if (isNaN(val)) return
      if (!itemMap.has(r.item_name)) itemMap.set(r.item_name, { values: [], unit: r.result_unit || undefined })
      itemMap.get(r.item_name)!.values.push(val)
    })

    const targetItems = selectedItems.length > 0
      ? availableItems.filter(name => selectedItems.includes(name))
      : availableItems

    const result: BoxPlotData[] = []
    targetItems.forEach(name => {
      const info = itemMap.get(name)
      if (!info || info.values.length < 2) return
      const bp = calcBoxPlot(info.values)
      if (!bp) return
      result.push({ name, ...bp, unit: info.unit, count: info.values.length })
    })
    return result
  }, [filteredData, selectedItems, availableItems])

  const exportToCSV = useCallback(() => {
    if (!filteredData.length) return
    const headers = ['專案編號', '耳號', '檢查日期', '實驗室', '項目名稱', '項目代碼', '結果值', '單位', '參考範圍', '是否異常']
    const rows = filteredData.map(r => [
      r.iacuc_no || '', r.ear_tag, r.test_date, r.lab_name || '',
      r.item_name, r.template_code || '', r.result_value || '',
      r.result_unit || '', r.reference_range || '', r.is_abnormal ? '是' : '否',
    ])
    const csvContent = [headers, ...rows].map(row => row.map(cell => `"${cell}"`).join(',')).join('\n')
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `blood_test_analysis_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
  }, [filteredData])

  const exportToExcel = useCallback(() => {
    if (!filteredData.length) return
    const headers = ['專案編號', '耳號', '檢查日期', '實驗室', '項目名稱', '項目代碼', '結果值', '單位', '參考範圍', '是否異常']
    const escapeHtml = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    const headerRow = headers.map(h => `<th>${escapeHtml(h)}</th>`).join('')
    const dataRows = filteredData.map(r => {
      const cells = [
        r.iacuc_no || '', r.ear_tag, r.test_date, r.lab_name || '',
        r.item_name, r.template_code || '', r.result_value || '',
        r.result_unit || '', r.reference_range || '', r.is_abnormal ? '是' : '否',
      ]
      return `<tr>${cells.map(c => `<td>${escapeHtml(String(c))}</td>`).join('')}</tr>`
    }).join('')
    const html = [
      '<html xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:x="urn:schemas-microsoft-com:office:excel">',
      '<head><meta charset="utf-8"></head>',
      '<body><table border="1">',
      `<tr>${headerRow}</tr>`,
      dataRows,
      '</table></body></html>',
    ].join('')
    const blob = new Blob(['\ufeff' + html], { type: 'application/vnd.ms-excel' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `blood_test_analysis_${new Date().toISOString().split('T')[0]}.xls`
    link.click()
    URL.revokeObjectURL(link.href)
  }, [filteredData])

  const toggleItem = (item: string) => {
    setSelectedItems(prev => prev.includes(item) ? prev.filter(i => i !== item) : [...prev, item])
  }

  return {
    // Filters
    iacucNo, setIacucNo,
    earTag, setEarTag,
    dateFrom, dateTo, setDateFrom, setDateTo,
    selectedItems, setSelectedItems,
    activeTab, setActiveTab,
    // Data
    isLoading, filteredData, chartFilteredData,
    presetsData, groupedAnalysisOptions,
    // Computed
    summary, abnormalRecords, trendData, boxPlotData,
    // Actions
    applyPreset, toggleItem, exportToCSV, exportToExcel,
  }
}
