import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  TrendingUp,
  BarChart3,
  FlaskConical,
  AlertTriangle,
} from 'lucide-react'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts'
import { formatDate } from '@/lib/utils'
import type { BloodTestAnalysisRow } from '@/types'
import { BoxPlotChart } from './BoxPlotChart'
import type { BoxPlotData } from '../hooks/useBloodTestAnalysis'

const CHART_COLORS = [
  '#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#8b5cf6',
  '#ef4444', '#06b6d4', '#84cc16', '#f97316', '#a855f7',
]

interface AnalysisChartTabsProps {
  activeTab: 'trend' | 'boxplot' | 'table'
  setActiveTab: (tab: 'trend' | 'boxplot' | 'table') => void
  trendData: { chartData: Record<string, number | string>[]; animals: string[] }
  boxPlotData: BoxPlotData[]
  chartFilteredData: BloodTestAnalysisRow[]
  selectedItems: string[]
}

export function AnalysisChartTabs({
  activeTab,
  setActiveTab,
  trendData,
  boxPlotData,
  chartFilteredData,
  selectedItems,
}: AnalysisChartTabsProps) {
  return (
    <>
      {/* Tab buttons */}
      <div className="flex gap-2 border-b pb-2">
        <Button variant={activeTab === 'trend' ? 'default' : 'ghost'} size="sm" onClick={() => setActiveTab('trend')}>
          <TrendingUp className="mr-2 h-4 w-4" /> 趨勢圖
        </Button>
        <Button variant={activeTab === 'boxplot' ? 'default' : 'ghost'} size="sm" onClick={() => setActiveTab('boxplot')}>
          <BarChart3 className="mr-2 h-4 w-4" /> 盒鬚圖
        </Button>
        <Button variant={activeTab === 'table' ? 'default' : 'ghost'} size="sm" onClick={() => setActiveTab('table')}>
          <FlaskConical className="mr-2 h-4 w-4" /> 資料明細
        </Button>
      </div>

      {activeTab === 'trend' && <TrendChart data={trendData} selectedItems={selectedItems} />}
      {activeTab === 'boxplot' && <BoxPlotTab data={boxPlotData} />}
      {activeTab === 'table' && <DataTable data={chartFilteredData} />}
    </>
  )
}

function TrendChart({
  data,
  selectedItems,
}: {
  data: { chartData: Record<string, number | string>[]; animals: string[] }
  selectedItems: string[]
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">
          趨勢分析
          {selectedItems.length > 0 && (
            <span className="text-muted-foreground font-normal ml-2 text-sm">
              ({selectedItems.join(', ')})
            </span>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {data.chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={data.chartData}>
              <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
              <XAxis dataKey="date" fontSize={12} />
              <YAxis fontSize={12} />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                  fontSize: '12px',
                }}
              />
              <Legend />
              {data.animals.map((animal, idx) => (
                <Line
                  key={animal}
                  type="monotone"
                  dataKey={animal}
                  name={animal}
                  stroke={CHART_COLORS[idx % CHART_COLORS.length]}
                  strokeWidth={2}
                  dot={{ r: 4 }}
                  activeDot={{ r: 6 }}
                  connectNulls
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <TrendingUp className="h-12 w-12 mb-2" />
            <p>請選擇具有數值結果的檢查項目以顯示趨勢圖</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function BoxPlotTab({ data }: { data: BoxPlotData[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">數值分布（盒鬚圖）</CardTitle>
      </CardHeader>
      <CardContent>
        {data.length > 0 ? (
          <BoxPlotChart data={data} />
        ) : (
          <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
            <BarChart3 className="h-12 w-12 mb-2" />
            <p>需要至少 2 筆數值資料才能繪製盒鬚圖</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function DataTable({ data }: { data: BloodTestAnalysisRow[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">資料明細</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>耳號</TableHead>
                <TableHead>專案</TableHead>
                <TableHead>日期</TableHead>
                <TableHead>實驗室</TableHead>
                <TableHead>項目</TableHead>
                <TableHead className="text-right">結果值</TableHead>
                <TableHead>單位</TableHead>
                <TableHead>參考範圍</TableHead>
                <TableHead className="text-center">異常</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.slice(0, 200).map((r, idx) => (
                <TableRow key={idx} className={r.is_abnormal ? 'bg-red-50/50 dark:bg-red-950/20' : ''}>
                  <TableCell className="font-medium">{r.ear_tag}</TableCell>
                  <TableCell className="font-mono text-sm">{r.iacuc_no || '-'}</TableCell>
                  <TableCell>{formatDate(r.test_date)}</TableCell>
                  <TableCell>{r.lab_name || '-'}</TableCell>
                  <TableCell>{r.item_name}</TableCell>
                  <TableCell className={`text-right ${r.is_abnormal ? 'font-semibold text-red-600 dark:text-red-400' : ''}`}>
                    {r.result_value || '-'}
                  </TableCell>
                  <TableCell className="text-muted-foreground">{r.result_unit || '-'}</TableCell>
                  <TableCell className="text-muted-foreground">{r.reference_range || '-'}</TableCell>
                  <TableCell className="text-center">
                    {r.is_abnormal && <AlertTriangle className="h-4 w-4 text-red-500 inline" />}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
          {data.length > 200 && (
            <p className="text-sm text-muted-foreground text-center py-2">
              僅顯示前 200 筆，完整資料請使用匯出功能
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
