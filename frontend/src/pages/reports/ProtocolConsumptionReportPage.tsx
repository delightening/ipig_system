import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import api, { ProtocolConsumptionReport } from '@/lib/api'
import { formatNumber, formatDate, formatUom } from '@/lib/utils'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTabState } from '@/hooks/useTabState'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { TableEmptyRow } from '@/components/ui/empty-state'
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
import { Download, FlaskConical } from 'lucide-react'
import {
  aggregateByProduct,
  aggregateByProtocol,
  buildCrossTab,
  cellKey,
  toCsv,
  toNum,
} from './protocolConsumptionAggregate'

const ALL_VALUE = '__all__'

type TabKey = 'by-protocol' | 'by-product' | 'cross'

interface ProtocolOption {
  id: string
  protocol_no: string
  iacuc_no: string | null
  title: string
}

function download(filename: string, csv: string) {
  const blob = new Blob(['﻿' + csv], { type: 'text/csv;charset=utf-8;' })
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = filename
  link.click()
}

export function ProtocolConsumptionReportPage() {
  const { from, to, setFrom, setTo } = useDateRangeFilter()
  const [protocolId, setProtocolId] = useState('')
  const { activeTab, setActiveTab } = useTabState<TabKey>('by-protocol')

  const { data: protocols } = useQuery<ProtocolOption[]>({
    queryKey: ['protocols-for-consumption-report'],
    queryFn: async () => {
      const res = await api.get<ProtocolOption[]>('/protocols')
      return res.data
    },
  })

  const { data: report, isLoading } = useQuery<ProtocolConsumptionReport[]>({
    queryKey: ['report-protocol-consumption', from, to, protocolId],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (from) params.set('date_from', from)
      if (to) params.set('date_to', to)
      if (protocolId) params.set('protocol_id', protocolId)
      const qs = params.toString()
      const response = await api.get<ProtocolConsumptionReport[]>(
        `/reports/protocol-consumption${qs ? '?' + qs : ''}`
      )
      return response.data
    },
  })

  const rows = useMemo(() => report ?? [], [report])
  const byProtocol = useMemo(() => aggregateByProtocol(rows), [rows])
  const byProduct = useMemo(() => aggregateByProduct(rows), [rows])
  const cross = useMemo(() => buildCrossTab(rows), [rows])

  const stamp = new Date().toISOString().split('T')[0]

  const exportCurrentTab = () => {
    if (activeTab === 'by-protocol') {
      download(
        `protocol_consumption_by_protocol_${stamp}.csv`,
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
        `protocol_consumption_by_product_${stamp}.csv`,
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

    // 交叉表：第一欄是案件，其餘每個品項一欄
    download(
      `protocol_consumption_cross_${stamp}.csv`,
      toCsv(
        ['計畫編號', ...cross.products.map(p => `${p.product_name}(${formatUom(p.base_uom)})`)],
        cross.protocols.map(pr => [
          pr.protocol_no,
          ...cross.products.map(pd => cross.cells.get(cellKey(pr.protocol_id, pd.product_id)) ?? 0),
        ])
      )
    )
  }

  const hasData = rows.length > 0

  return (
    <div className="space-y-6">
      <PageHeader
        title="案件消耗報表"
        description="依計畫統計內部領用的耗材消耗（已扣除沖銷，僅計已核准）"
        actions={
          <Button size="sm" onClick={exportCurrentTab} disabled={!hasData}>
            <Download className="mr-2 h-4 w-4" />
            匯出目前分頁
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-3">
        <div className="space-y-1">
          <Label>起始日期</Label>
          <Input type="date" value={from} onChange={e => setFrom(e.target.value)} />
        </div>
        <div className="space-y-1">
          <Label>結束日期</Label>
          <Input type="date" value={to} onChange={e => setTo(e.target.value)} />
        </div>
        <div className="space-y-1">
          <Label>計畫</Label>
          <Select
            value={protocolId || ALL_VALUE}
            onValueChange={v => setProtocolId(v === ALL_VALUE ? '' : v)}
          >
            <SelectTrigger>
              <SelectValue placeholder="全部計畫" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>全部計畫</SelectItem>
              {protocols?.map(p => (
                <SelectItem key={p.id} value={p.id}>
                  {p.protocol_no} - {p.title}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

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
                  <TableEmptyRow colSpan={7} icon={FlaskConical} title="這段期間沒有案件領用紀錄" />
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
                  <TableEmptyRow colSpan={7} icon={FlaskConical} title="這段期間沒有案件領用紀錄" />
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
                  <TableEmptyRow colSpan={2} icon={FlaskConical} title="這段期間沒有案件領用紀錄" />
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

      {hasData && (
        <p className="text-xs text-muted-foreground">
          共 {rows.length} 組（案件 × 品項）；{byProtocol.length} 個案件、{byProduct.length} 個品項。
          數量已扣除沖銷，僅計已核准的領用。{toNum(String(rows.length)) >= 1000 && '⚠️ 已達 1000 筆上限，請縮小日期範圍。'}
        </p>
      )}
    </div>
  )
}
