import { useState, useMemo, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'

import api, { ProtocolConsumptionReport } from '@/lib/api'
import { formatNumber, formatDate, formatUom } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTabState } from '@/hooks/useTabState'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { EmptyState, TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { AlertTriangle, Download, FlaskConical } from 'lucide-react'
import {
  ROW_LIMIT,
  aggregateByProduct,
  aggregateByProtocol,
  buildCrossTab,
  cellKey,
  crossTabCsv,
  exportFilename,
  splitTruncationSignal,
  taipeiDateStamp,
  toCsv,
} from './protocolConsumptionAggregate'

const ALL_VALUE = '__all__'

type TabKey = 'by-protocol' | 'by-product' | 'cross'

function download(filename: string, csv: string) {
  // BOM 前綴讓 Excel 認得 UTF-8，否則中文品名開起來是亂碼
  const blob = new Blob(['﻿' + csv], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  // 🔴 用完要 revoke，否則每按一次匯出就把一整份 CSV 釘在記憶體裡到整頁卸載為止。
  // 交叉表在品項多的時候一份就不小，反覆匯出會累積。
  //
  // 不能同步 revoke：部分瀏覽器在 click() 回傳時還沒真的開始讀這個 URL，
  // 立刻撤銷會讓下載變成空檔。延到下一個 macrotask，讓下載先啟動。
  setTimeout(() => URL.revokeObjectURL(url), 0)
}

export function ProtocolConsumptionReportPage() {
  const { from, to, setFrom, setTo } = useDateRangeFilter()
  const [protocolId, setProtocolId] = useState('')
  const { activeTab, setActiveTab } = useTabState<TabKey>('by-protocol')

  // 🔴 計畫篩選刻意做在前端，且下拉選項由報表資料自己長出來，不打 `/protocols`。
  //
  // 打 `/protocols` 有兩個問題：非管理員只會拿到自己有份的計畫（`get_my_protocols`，
  // 不是 403），倉管拿著 `erp.report.view` 進來會看到一個只有「全部計畫」的空下拉；
  // 而訪客示範模式的 `getGuestDemoData` 會先剝掉 query string 再查路由，
  // 伺服器端篩選在那個模式下本來就不會生效。
  //
  // 改成前端篩之後兩個問題一起消失，還順帶少一次網路往返。選項一律取自
  // **未篩選**的資料，否則選了一個計畫之後下拉會塌成只剩那一個、換不回去。
  const {
    data: report,
    isLoading,
    isError,
    error,
    refetch,
    isFetching,
  } = useQuery<ProtocolConsumptionReport[]>({
    queryKey: ['report-protocol-consumption', from, to],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (from) params.set('date_from', from)
      if (to) params.set('date_to', to)
      const qs = params.toString()
      const response = await api.get<ProtocolConsumptionReport[]>(
        `/reports/protocol-consumption${qs ? '?' + qs : ''}`
      )
      return response.data
    },
  })

  const { rows: allRows, truncated } = useMemo(
    () => splitTruncationSignal(report ?? []),
    [report]
  )

  const protocolOptions = useMemo(() => aggregateByProtocol(allRows), [allRows])

  // 🔴 被截斷時計畫篩選必須停用，不能只是「照樣篩」。
  //
  // `allRows` 在截斷時只是全域排序（protocol_no, sku）的前 1000 組：排序在後面的
  // 計畫整個不在裡面，剛好卡在切點的那個計畫則會少掉部分品項。此時篩選只是在一份
  // **殘缺的陣列**上過濾，不會重新查後端——選了一個計畫看到的數字可能是不完整的，
  // 而畫面看起來跟正常結果一模一樣。
  //
  // 唯一有效的補救是縮小日期範圍讓資料回到上限內。上一版的警告文案寫「或指定計畫」，
  // 那是錯的：那個動作不會重查，等於叫使用者做一件沒有作用的事。
  const filterEnabled = !truncated
  const activeProtocolId = filterEnabled ? protocolId : ''

  // 截斷一發生就把選擇清掉，不只是「忽略它」。
  //
  // 只靠 `activeProtocolId` 遮蔽的話，`protocolId` 還留著：使用者把日期放寬到截斷、
  // 下拉顯示回「全部計畫」，之後再把日期縮回來——那個看不見的舊選擇會自己復活，
  // 資料無聲變窄，而使用者從沒再選過。畫面上看得到的狀態必須就是實際生效的狀態。
  useEffect(() => {
    if (truncated && protocolId) setProtocolId('')
  }, [truncated, protocolId])

  const rows = useMemo(
    () =>
      activeProtocolId ? allRows.filter(r => r.protocol_id === activeProtocolId) : allRows,
    [allRows, activeProtocolId]
  )

  const byProtocol = useMemo(() => aggregateByProtocol(rows), [rows])
  const byProduct = useMemo(() => aggregateByProduct(rows), [rows])
  const cross = useMemo(() => buildCrossTab(rows), [rows])

  const exportCurrentTab = () => {
    const stamp = taipeiDateStamp()

    if (activeTab === 'by-protocol') {
      download(
        exportFilename('by-protocol', stamp, truncated),
        toCsv(
          ['計畫編號', '核准編號', '計畫名稱', '品項數', '單據數(至少)', '金額', '最早', '最晚'],
          byProtocol.map(p => [
            p.protocol_no,
            p.iacuc_no ?? '',
            p.protocol_title ?? '',
            p.product_count,
            p.doc_count,
            p.total_cost,
            formatDate(p.first_trx_date),
            formatDate(p.last_trx_date),
          ])
        )
      )
      return
    }

    if (activeTab === 'by-product') {
      download(
        exportFilename('by-product', stamp, truncated),
        toCsv(
          ['產品代碼', '產品名稱', '分類', '案件數', '消耗量', '單位', '金額'],
          byProduct.map(p => [
            p.product_sku,
            p.product_name,
            p.category_name ?? '',
            p.protocol_count,
            p.qty_base,
            formatUom(p.base_uom),
            p.total_cost,
          ])
        )
      )
      return
    }

    // 交叉表：第一欄是案件，其餘每個品項一欄。
    // 沒有紀錄的格輸出空字串而非 0——理由見 crossTabCsv 的註解。
    download(exportFilename('cross', stamp, truncated), crossTabCsv(cross, formatUom))
  }

  const hasData = rows.length > 0

  return (
    <div className="space-y-6">
      <PageHeader
        title="案件消耗報表"
        description="依計畫統計內部領用的耗材消耗（已扣除沖銷，僅計已核准）"
        actions={
          <Button size="sm" onClick={exportCurrentTab} disabled={isError || !hasData}>
            <Download className="mr-2 h-4 w-4" />
            匯出目前分頁
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-3">
        <div className="space-y-1">
          <Label htmlFor="pc-date-from">起始日期</Label>
          <Input
            id="pc-date-from"
            type="date"
            value={from}
            onChange={e => setFrom(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="pc-date-to">結束日期</Label>
          <Input
            id="pc-date-to"
            type="date"
            value={to}
            onChange={e => setTo(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="pc-protocol">
            計畫
            {truncated && (
              <span className="ml-1 font-normal text-muted-foreground">（資料截斷中不可用）</span>
            )}
          </Label>
          <Select
            value={activeProtocolId || ALL_VALUE}
            onValueChange={v => setProtocolId(v === ALL_VALUE ? '' : v)}
            disabled={isError || !filterEnabled}
          >
            <SelectTrigger id="pc-protocol">
              <SelectValue placeholder="全部計畫" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>全部計畫</SelectItem>
              {protocolOptions.map(p => (
                <SelectItem key={p.protocol_id} value={p.protocol_id}>
                  {p.protocol_no}
                  {p.protocol_title ? ` - ${p.protocol_title}` : ''}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {truncated && (
        <div className="flex items-start gap-2 rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm dark:border-amber-800 dark:bg-amber-950/30">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
          <span>
            資料已達 {ROW_LIMIT} 組上限而被截斷，下面看到的<strong>不是全部</strong>
            ——排序在後面的計畫整個不在其中，卡在切點的計畫也可能少掉部分品項。
            <strong>請縮小日期範圍後重查</strong>；計畫篩選在這個狀態下已停用，
            因為它只會在這份殘缺資料上過濾，不會重新查詢。
          </span>
        </div>
      )}

      {/* 🔴 查詢失敗必須跟「查成功但沒資料」分開呈現。
          兩者在 react-query 底下都是 data === undefined，若只判斷 isLoading，
          載入失敗會顯示成「這段期間沒有案件領用紀錄」——那是在報告一個
          我們根本不知道的事實。 */}
      {isError ? (
        <EmptyState
          icon={AlertTriangle}
          title="報表載入失敗"
          description={
            error instanceof Error
              ? `無法取得案件消耗資料：${error.message}`
              : '無法取得案件消耗資料。這不代表這段期間沒有領用紀錄，只代表查詢沒有成功。'
          }
          action={{
            label: isFetching ? '重試中…' : '重新載入',
            onClick: () => void refetch(),
          }}
        />
      ) : (
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="by-protocol">依案件</TabsTrigger>
            <TabsTrigger value="by-product">依品項</TabsTrigger>
            <TabsTrigger value="cross">交叉表</TabsTrigger>
          </TabsList>

          {/* ── 依案件 ───────────────────────────────────────────────── */}
          <TabsContent value="by-protocol">
            <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead>計畫編號</TableHead>
                    <TableHead>核准編號</TableHead>
                    <TableHead>計畫名稱</TableHead>
                    <TableHead className="text-right">品項數</TableHead>
                    <TableHead className="text-right">單據數</TableHead>
                    <TableHead className="text-right">金額</TableHead>
                    <TableHead>期間</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={7}>
                        <TableSkeleton rows={6} cols={7} />
                      </TableCell>
                    </TableRow>
                  ) : byProtocol.length === 0 ? (
                    <TableEmptyRow
                      colSpan={7}
                      icon={FlaskConical}
                      title="這段期間沒有案件領用紀錄"
                    />
                  ) : (
                    byProtocol.map(p => (
                      <TableRow key={p.protocol_id}>
                        <TableCell className="font-mono text-xs">{p.protocol_no}</TableCell>
                        <TableCell className="font-mono text-xs">{p.iacuc_no ?? '—'}</TableCell>
                        <TableCell>{p.protocol_title ?? '—'}</TableCell>
                        <TableCell className="text-right">{p.product_count}</TableCell>
                        <TableCell className="text-right">{p.doc_count}</TableCell>
                        <TableCell className="text-right">{formatNumber(p.total_cost)}</TableCell>
                        <TableCell className="text-xs text-muted-foreground">
                          {formatDate(p.first_trx_date)} ~ {formatDate(p.last_trx_date)}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              這一層刻意不顯示數量合計——一個案件會同時用到手套（雙）、滴管（包）、紗布（片），
              把不同單位的數字加起來沒有意義。要看數量請切到「依品項」或「交叉表」。
              單據數為下界（同一張領用單常同時領多種品項）。
            </p>
          </TabsContent>

          {/* ── 依品項 ───────────────────────────────────────────────── */}
          <TabsContent value="by-product">
            <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead>產品代碼</TableHead>
                    <TableHead>產品名稱</TableHead>
                    <TableHead>分類</TableHead>
                    <TableHead className="text-right">案件數</TableHead>
                    <TableHead className="text-right">消耗量</TableHead>
                    <TableHead>單位</TableHead>
                    <TableHead className="text-right">金額</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={7}>
                        <TableSkeleton rows={6} cols={7} />
                      </TableCell>
                    </TableRow>
                  ) : byProduct.length === 0 ? (
                    <TableEmptyRow
                      colSpan={7}
                      icon={FlaskConical}
                      title="這段期間沒有案件領用紀錄"
                    />
                  ) : (
                    byProduct.map(p => (
                      <TableRow key={p.product_id}>
                        <TableCell className="font-mono text-xs">{p.product_sku}</TableCell>
                        <TableCell>{p.product_name}</TableCell>
                        <TableCell>{p.category_name ?? '—'}</TableCell>
                        <TableCell className="text-right">{p.protocol_count}</TableCell>
                        <TableCell className="text-right">{formatNumber(p.qty_base)}</TableCell>
                        <TableCell>{formatUom(p.base_uom)}</TableCell>
                        <TableCell className="text-right">{formatNumber(p.total_cost)}</TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          </TabsContent>

          {/* ── 交叉表 ───────────────────────────────────────────────── */}
          <TabsContent value="cross">
            {/* 品項多的時候會很寬，讓表格自己橫向捲，不要把整頁撐爆 */}
            <div className="rounded-lg border bg-card overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead className="sticky left-0 bg-muted/50">計畫編號</TableHead>
                    {cross.products.map(p => (
                      <TableHead key={p.product_id} className="text-right whitespace-nowrap">
                        {p.product_name}
                        <span className="block text-xs font-normal text-muted-foreground">
                          {formatUom(p.base_uom)}
                        </span>
                      </TableHead>
                    ))}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={cross.products.length + 1}>
                        <TableSkeleton rows={6} cols={4} />
                      </TableCell>
                    </TableRow>
                  ) : cross.protocols.length === 0 ? (
                    <TableEmptyRow
                      colSpan={cross.products.length + 1}
                      icon={FlaskConical}
                      title="這段期間沒有案件領用紀錄"
                    />
                  ) : (
                    cross.protocols.map(pr => (
                      <TableRow key={pr.protocol_id}>
                        <TableCell className="sticky left-0 bg-card font-mono text-xs whitespace-nowrap">
                          {pr.protocol_no}
                        </TableCell>
                        {cross.products.map(pd => {
                          const qty = cross.cells.get(cellKey(pr.protocol_id, pd.product_id))
                          return (
                            <TableCell
                              key={pd.product_id}
                              className={
                                qty === undefined
                                  ? 'text-right text-muted-foreground'
                                  : 'text-right'
                              }
                            >
                              {qty === undefined ? '—' : formatNumber(qty)}
                            </TableCell>
                          )
                        })}
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              「—」表示該案件沒有領用過這個品項，與領用後整筆沖銷（不會出現在報表裡）不同。
              每一行的單位標在表頭，欄與欄之間不可相加。
            </p>
          </TabsContent>
        </Tabs>
      )}

      {!isError && hasData && (
        <p className="text-xs text-muted-foreground">
          共 {rows.length} 組（案件 × 品項）；{byProtocol.length} 個案件、{byProduct.length} 個品項。
          數量已扣除沖銷，僅計已核准的領用。
        </p>
      )}
    </div>
  )
}
