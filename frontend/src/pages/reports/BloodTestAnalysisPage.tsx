/**
 * 血液檢查結果分析頁面
 * 提供血液檢查數據的統計、趨勢分析、異常標記與視覺化圖表
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Loader2,
  Download,
  AlertTriangle,
  Activity,
  Users,
  FlaskConical,
  FileSpreadsheet,
} from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { useBloodTestAnalysis } from './hooks/useBloodTestAnalysis'
import { AnalysisItemSelector } from './components/AnalysisItemSelector'
import { AnalysisChartTabs } from './components/AnalysisChartTabs'

export function BloodTestAnalysisPage() {
  const analysis = useBloodTestAnalysis()

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">血液檢查結果分析</h1>
          <p className="text-muted-foreground">對血液檢查結果進行統計分析、趨勢追蹤與異常值偵測</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={analysis.exportToCSV} disabled={!analysis.filteredData.length}>
            <Download className="mr-2 h-4 w-4" /> CSV
          </Button>
          <Button onClick={analysis.exportToExcel} disabled={!analysis.filteredData.length}>
            <FileSpreadsheet className="mr-2 h-4 w-4" /> Excel
          </Button>
        </div>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">篩選條件</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <div className="space-y-2">
              <Label htmlFor="iacuc_no">專案編號 (IACUC No.)</Label>
              <Input id="iacuc_no" placeholder="例: PIG-115001" value={analysis.iacucNo} onChange={(e) => analysis.setIacucNo(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="ear_tag">動物耳號</Label>
              <Input id="ear_tag" placeholder="輸入耳號搜尋" value={analysis.earTag} onChange={(e) => analysis.setEarTag(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="date_from">開始日期</Label>
              <Input id="date_from" type="date" value={analysis.dateFrom} onChange={(e) => analysis.setDateFrom(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="date_to">結束日期</Label>
              <Input id="date_to" type="date" value={analysis.dateTo} onChange={(e) => analysis.setDateTo(e.target.value)} />
            </div>
          </div>
        </CardContent>
      </Card>

      {analysis.isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      )}

      {!analysis.isLoading && analysis.filteredData.length > 0 && (
        <>
          {/* Summary stats */}
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <SummaryCard icon={<FlaskConical className="h-6 w-6 text-blue-600 dark:text-blue-400" />} bgClass="bg-blue-100 dark:bg-blue-900/30" label="檢查項目數" value={analysis.summary.totalItems.toLocaleString()} />
            <SummaryCard
              icon={<AlertTriangle className="h-6 w-6 text-red-600 dark:text-red-400" />}
              bgClass="bg-red-100 dark:bg-red-900/30"
              label="異常比率"
              value={`${analysis.summary.abnormalRate.toFixed(1)}%`}
              suffix={`(${analysis.summary.abnormalCount})`}
            />
            <SummaryCard icon={<Users className="h-6 w-6 text-green-600 dark:text-green-400" />} bgClass="bg-green-100 dark:bg-green-900/30" label="涵蓋動物數" value={String(analysis.summary.animalCount)} />
            <SummaryCard icon={<Activity className="h-6 w-6 text-purple-600 dark:text-purple-400" />} bgClass="bg-purple-100 dark:bg-purple-900/30" label="檢查次數" value={String(analysis.summary.testDates)} />
          </div>

          {/* Abnormal records */}
          {analysis.abnormalRecords.length > 0 && (
            <Card className="border-red-200 dark:border-red-800">
              <CardHeader>
                <CardTitle className="text-base flex items-center gap-2 text-red-600 dark:text-red-400">
                  <AlertTriangle className="h-5 w-5" />
                  異常值警示（共 {analysis.abnormalRecords.length} 項）
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="rounded-md border border-red-200 dark:border-red-800">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>耳號</TableHead>
                        <TableHead>專案</TableHead>
                        <TableHead>日期</TableHead>
                        <TableHead>項目</TableHead>
                        <TableHead className="text-right">結果值</TableHead>
                        <TableHead>參考範圍</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {analysis.abnormalRecords.slice(0, 20).map((r, idx) => (
                        <TableRow key={idx} className="bg-red-50/50 dark:bg-red-950/20">
                          <TableCell className="font-medium">{r.ear_tag}</TableCell>
                          <TableCell className="font-mono text-sm">{r.iacuc_no || '-'}</TableCell>
                          <TableCell>{formatDate(r.test_date)}</TableCell>
                          <TableCell>{r.item_name}</TableCell>
                          <TableCell className="text-right font-semibold text-red-600 dark:text-red-400">
                            {r.result_value} {r.result_unit}
                          </TableCell>
                          <TableCell className="text-muted-foreground">{r.reference_range || '-'}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                  {analysis.abnormalRecords.length > 20 && (
                    <p className="text-sm text-muted-foreground text-center py-2">
                      僅顯示前 20 筆，匯出報表可查看完整異常清單
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>
          )}

          <AnalysisItemSelector
            groupedOptions={analysis.groupedAnalysisOptions}
            presetsData={analysis.presetsData}
            selectedItems={analysis.selectedItems}
            setSelectedItems={analysis.setSelectedItems}
            applyPreset={analysis.applyPreset}
            toggleItem={analysis.toggleItem}
          />

          <AnalysisChartTabs
            activeTab={analysis.activeTab}
            setActiveTab={analysis.setActiveTab}
            trendData={analysis.trendData}
            boxPlotData={analysis.boxPlotData}
            chartFilteredData={analysis.chartFilteredData}
            selectedItems={analysis.selectedItems}
          />
        </>
      )}

      {/* Empty state */}
      {!analysis.isLoading && analysis.filteredData.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <FlaskConical className="h-16 w-16 text-muted-foreground mb-4" />
            <p className="text-lg font-medium text-muted-foreground">尚無血液檢查分析資料</p>
            <p className="text-sm text-muted-foreground mt-1">
              請先在動物管理頁面建立血液檢查紀錄，或調整篩選條件
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}

function SummaryCard({
  icon, bgClass, label, value, suffix,
}: {
  icon: React.ReactNode
  bgClass: string
  label: string
  value: string
  suffix?: string
}) {
  return (
    <Card>
      <CardContent className="flex items-center gap-4 pt-6">
        <div className={`rounded-full p-3 ${bgClass}`}>{icon}</div>
        <div>
          <p className="text-sm text-muted-foreground">{label}</p>
          <p className="text-2xl font-bold">
            {value}
            {suffix && <span className="text-sm font-normal text-muted-foreground ml-1">{suffix}</span>}
          </p>
        </div>
      </CardContent>
    </Card>
  )
}
