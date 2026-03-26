/**
 * 血液檢查結果分析頁面
 * 提供血液檢查數據的統計、趨勢分析、異常標記與視覺化圖表
 */
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
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
import { EmptyState } from '@/components/ui/empty-state'

export function BloodTestAnalysisPage() {
  const analysis = useBloodTestAnalysis()

  return (
    <div className="space-y-6">
      <PageHeader
        title="血液檢查結果分析"
        description="對血液檢查結果進行統計分析、趨勢追蹤與異常值偵測"
        actions={
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={analysis.exportToCSV} disabled={!analysis.filteredData.length}>
              <Download className="mr-2 h-4 w-4" /> CSV
            </Button>
            <Button size="sm" onClick={analysis.exportToExcel} disabled={!analysis.filteredData.length}>
              <FileSpreadsheet className="mr-2 h-4 w-4" /> Excel
            </Button>
          </div>
        }
      />

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
            <SummaryCard icon={<FlaskConical className="h-6 w-6 text-primary" />} bgClass="bg-primary/10" label="檢查項目數" value={analysis.summary.totalItems.toLocaleString()} />
            <SummaryCard
              icon={<AlertTriangle className="h-6 w-6 text-destructive" />}
              bgClass="bg-status-error-bg"
              label="異常比率"
              value={`${analysis.summary.abnormalRate.toFixed(1)}%`}
              suffix={`(${analysis.summary.abnormalCount})`}
            />
            <SummaryCard icon={<Users className="h-6 w-6 text-status-success-text" />} bgClass="bg-status-success-bg" label="涵蓋動物數" value={String(analysis.summary.animalCount)} />
            <SummaryCard icon={<Activity className="h-6 w-6 text-status-purple-text" />} bgClass="bg-status-purple-bg" label="檢查次數" value={String(analysis.summary.testDates)} />
          </div>

          {/* Abnormal records */}
          {analysis.abnormalRecords.length > 0 && (
            <Card className="border-destructive/30">
              <CardHeader>
                <CardTitle className="text-base flex items-center gap-2 text-destructive">
                  <AlertTriangle className="h-5 w-5" />
                  異常值警示（共 {analysis.abnormalRecords.length} 項）
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="rounded-md border border-destructive/30">
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
                        <TableRow key={idx} className="bg-status-error-bg/50">
                          <TableCell className="font-medium">{r.ear_tag}</TableCell>
                          <TableCell className="font-mono text-sm">{r.iacuc_no || '-'}</TableCell>
                          <TableCell>{formatDate(r.test_date)}</TableCell>
                          <TableCell>{r.item_name}</TableCell>
                          <TableCell className="text-right font-semibold text-destructive">
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
          <CardContent className="py-4">
            <EmptyState icon={FlaskConical} title="尚無血液檢查分析資料" description="請先在動物管理頁面建立血液檢查紀錄，或調整篩選條件" />
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
