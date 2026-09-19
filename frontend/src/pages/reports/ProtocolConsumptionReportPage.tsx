import { useState, useMemo, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'

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
import { GuestDateNotice } from '@/components/ui/guest-date-notice'
import { AlertTriangle, Download, FlaskConical } from 'lucide-react'
import {
  MAX_CROSS_CELLS,
  ROW_LIMIT,
  aggregateByProduct,
  aggregateByProtocol,
  buildCrossTab,
  cellKey,
  crossTabCellCount,
  crossTabCsv,
  crossTabTooLarge,
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
  const { t } = useTranslation()
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
  //
  // 🔴 **同一條原則有第二個觸發點：換日期之後，已選的計畫可能不在新結果裡。**
  // （CodeRabbit 於 MR !15 指出，2026-09-11，成立。）
  //
  // 情境：選了計畫 A，把日期改成只有計畫 B 有領用的區間。此時
  //   - `rows` 被 `activeProtocolId` 篩成空陣列 → 畫面顯示「這段期間沒有案件領用紀錄」，
  //     而那句話是**錯的**：這段期間有紀錄，只是不屬於 A。
  //   - 更糟的是下拉：`protocolOptions` 由**新資料**長出來，A 已經不在選項裡，
  //     於是 `<Select>` 的 value 找不到對應項、顯示回 placeholder「全部計畫」——
  //     **畫面說「全部計畫」，實際仍在篩 A**。這正是上面那段註解要防的東西，
  //     只是觸發路徑不同（那裡是截斷，這裡是換日期）。
  //
  // ⚠️ 判斷必須等 `report` 真的有值。`queryKey` 含 from/to，日期一改就換 key，
  // 此時 `report` 會先變成 `undefined`、`protocolOptions` 暫時為空——在那個瞬間做
  // 判斷，會把一個其實仍然有效的選擇誤清掉。
  useEffect(() => {
    if (!protocolId) return
    if (truncated) {
      setProtocolId('')
      return
    }
    if (report !== undefined && !protocolOptions.some(p => p.protocol_id === protocolId)) {
      setProtocolId('')
    }
  }, [truncated, protocolId, report, protocolOptions])

  const rows = useMemo(
    () =>
      activeProtocolId ? allRows.filter(r => r.protocol_id === activeProtocolId) : allRows,
    [allRows, activeProtocolId]
  )

  const byProtocol = useMemo(() => aggregateByProtocol(rows), [rows])
  const byProduct = useMemo(() => aggregateByProduct(rows), [rows])
  const cross = useMemo(() => buildCrossTab(rows), [rows])

  // 交叉表的規模上限。`truncated`（ROW_LIMIT）擋不住這件事——它限的是扁平列數，
  // 而這裡的格數是「案件數 × 品項數」，最壞情況下 1000 列會攤成一百萬格。
  // 詳見 protocolConsumptionAggregate.ts 的 MAX_CROSS_CELLS。
  const crossTooLarge = useMemo(() => crossTabTooLarge(cross), [cross])
  const crossCells = useMemo(() => crossTabCellCount(cross), [cross])

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
    //
    // 這裡再擋一次雖然按鈕已經停用（見下方 disabled）：停用只防得了滑鼠，
    // 防不了鍵盤、擴充套件或未來改動把這條路重新接通。crossTabCsv 內部還有
    // 第三道，三層都是同一個判準。
    if (crossTooLarge) return
    download(exportFilename('cross', stamp, truncated), crossTabCsv(cross, formatUom))
  }

  const hasData = rows.length > 0
  const exportDisabled = isError || !hasData || (activeTab === 'cross' && crossTooLarge)

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('reportsPages.protocolConsumption.title')}
        description={t('reportsPages.protocolConsumption.description')}
        actions={
          <Button size="sm" onClick={exportCurrentTab} disabled={exportDisabled}>
            <Download className="mr-2 h-4 w-4" />
            {t('reportsPages.protocolConsumption.exportCurrentTab')}
          </Button>
        }
      />

      <div className="grid grid-cols-2 gap-4 md:grid-cols-3">
        <div className="space-y-1">
          <Label htmlFor="pc-date-from">{t('reportsPages.shared.startDate')}</Label>
          <Input
            id="pc-date-from"
            type="date"
            value={from}
            onChange={e => setFrom(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="pc-date-to">{t('reportsPages.shared.endDate')}</Label>
          <Input
            id="pc-date-to"
            type="date"
            value={to}
            onChange={e => setTo(e.target.value)}
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="pc-protocol">
            {t('reportsPages.protocolConsumption.protocol')}
            {truncated && (
              <span className="ml-1 font-normal text-muted-foreground">{t('reportsPages.protocolConsumption.protocolUnavailable')}</span>
            )}
          </Label>
          <Select
            value={activeProtocolId || ALL_VALUE}
            onValueChange={v => setProtocolId(v === ALL_VALUE ? '' : v)}
            disabled={isError || !filterEnabled}
          >
            <SelectTrigger id="pc-protocol">
              <SelectValue placeholder={t('reportsPages.protocolConsumption.allProtocols')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL_VALUE}>{t('reportsPages.protocolConsumption.allProtocols')}</SelectItem>
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

      {/* 訪客示範模式下日期欄位不會改變結果——把這件事講出來，而不是假裝能篩。
          `getGuestDemoData` 在比對路由前就剝掉 query string，所以十幾支報表的日期
          在示範模式下全部沒有作用。
          🔴 本報表**特別**不能靠篩夾具來假裝：每一列是後端 GROUP BY 之後的彙總
          （`qty_base` 已加總、`first/last_trx_date` 是 MIN/MAX），跨越邊界的列必須
          **重算**才對，而夾具裡只有總數、原始異動已經不在了——整列留著會顯示錯的
          數字，整列丟掉則是漏資料，兩個都錯。理由詳見該元件的註解。
          （2026-09-11：我一度改用 `queryAwareRoutes` 按日期篩，那正是該元件明文
          否決、且點名本報表的做法，已 revert。） */}
      <GuestDateNotice />

      {truncated && (
        <div className="flex items-start gap-2 rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm dark:border-amber-800 dark:bg-amber-950/30">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
          <span>
            <Trans
              i18nKey="reportsPages.protocolConsumption.truncatedWarning"
              values={{ limit: ROW_LIMIT }}
              components={{ strong: <strong /> }}
            />
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
          title={t('reportsPages.protocolConsumption.loadFailedTitle')}
          description={
            error instanceof Error
              ? t('reportsPages.protocolConsumption.loadFailedWithMessage', { message: error.message })
              : t('reportsPages.protocolConsumption.loadFailedDescription')
          }
          action={{
            label: isFetching
              ? t('reportsPages.protocolConsumption.retrying')
              : t('reportsPages.protocolConsumption.reload'),
            onClick: () => void refetch(),
          }}
        />
      ) : (
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="by-protocol">{t('reportsPages.protocolConsumption.tabs.byProtocol')}</TabsTrigger>
            <TabsTrigger value="by-product">{t('reportsPages.protocolConsumption.tabs.byProduct')}</TabsTrigger>
            <TabsTrigger value="cross">{t('reportsPages.protocolConsumption.tabs.cross')}</TabsTrigger>
          </TabsList>

          {/* ── 依案件 ───────────────────────────────────────────────── */}
          <TabsContent value="by-protocol">
            <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead>{t('reportsPages.protocolConsumption.protocolNo')}</TableHead>
                    <TableHead>{t('reportsPages.protocolConsumption.approvalNo')}</TableHead>
                    <TableHead>{t('reportsPages.protocolConsumption.protocolTitle')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.protocolConsumption.itemCount')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.protocolConsumption.docCount')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.shared.amount')}</TableHead>
                    <TableHead>{t('reportsPages.protocolConsumption.period')}</TableHead>
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
                      title={t('reportsPages.protocolConsumption.emptyTitle')}
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
              {t('reportsPages.protocolConsumption.byProtocolNote')}
            </p>
          </TabsContent>

          {/* ── 依品項 ───────────────────────────────────────────────── */}
          <TabsContent value="by-product">
            <div className="rounded-lg border bg-card overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead>{t('reportsPages.shared.productCode')}</TableHead>
                    <TableHead>{t('reportsPages.shared.productName')}</TableHead>
                    <TableHead>{t('reportsPages.protocolConsumption.categoryName')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.protocolConsumption.protocolCount')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.protocolConsumption.consumedQty')}</TableHead>
                    <TableHead>{t('reportsPages.shared.unit')}</TableHead>
                    <TableHead className="text-right">{t('reportsPages.shared.amount')}</TableHead>
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
                      title={t('reportsPages.protocolConsumption.emptyTitle')}
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
            {/* 🔴 超過格數上限就整個不渲染。這道檢查必須在 <Table> 之外——
                放進表格內部再判斷，那些巢狀 map 已經先跑過一輪了。
                匯出按鈕也用同一個 crossTooLarge 停用（見 exportDisabled）：
                虛擬捲動保護得了畫面，保護不了 CSV，兩邊得是同一個判準。 */}
            {crossTooLarge ? (
              <EmptyState
                icon={AlertTriangle}
                title={t('reportsPages.protocolConsumption.crossTooLargeTitle')}
                description={t(
                  filterEnabled
                    ? 'reportsPages.protocolConsumption.crossTooLargeDescriptionWithFilter'
                    : 'reportsPages.protocolConsumption.crossTooLargeDescription',
                  {
                    cells: crossCells.toLocaleString(),
                    protocols: cross.protocols.length,
                    products: cross.products.length,
                    limit: MAX_CROSS_CELLS.toLocaleString(),
                  }
                )}
              />
            ) : (
              <>
                {/* 品項多的時候會很寬，讓表格自己橫向捲，不要把整頁撐爆 */}
                <div className="rounded-lg border bg-card overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <TableHead className="sticky left-0 bg-muted/50">{t('reportsPages.protocolConsumption.protocolNo')}</TableHead>
                    {/* 🔴 表頭要帶 sku，理由與 crossTabCsv 同一條：products 只保證 sku
                        唯一，(product_name, base_uom) 不保證。同名同單位的兩個品項在畫面上
                        會變成兩個一模一樣的欄位標題，底下卻是不同的數字。
                        ⚠️ 而且畫面與 CSV 必須用同一組欄位識別——先前只在 CSV 加了 sku 卻
                        沒動這裡，結果是「匯出檔分得出來、螢幕分不出來」，那比兩邊都不加更糟：
                        使用者在螢幕上看到兩個相同標題，匯出後卻多出兩個不同欄名，
                        會以為是匯出把資料弄錯了。（CodeRabbit 於 MR !15 指出，2026-09-11。） */}
                    {cross.products.map(p => (
                      <TableHead key={p.product_id} className="text-right whitespace-nowrap">
                        <span className="block font-mono text-xs font-normal text-muted-foreground">
                          {p.product_sku}
                        </span>
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
                      title={t('reportsPages.protocolConsumption.emptyTitle')}
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
                  {t('reportsPages.protocolConsumption.crossNote')}
                </p>
              </>
            )}
          </TabsContent>
        </Tabs>
      )}

      {!isError && hasData && (
        <p className="text-xs text-muted-foreground">
          {t('reportsPages.protocolConsumption.summary', {
            rows: rows.length,
            protocols: byProtocol.length,
            products: byProduct.length,
          })}
        </p>
      )}
    </div>
  )
}
