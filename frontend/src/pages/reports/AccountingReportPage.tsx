import { useState } from 'react'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Calculator, FileText, Receipt, CreditCard, TrendingUp } from 'lucide-react'

import { TrialBalanceTab } from './components/TrialBalanceTab'
import { JournalEntriesTab } from './components/JournalEntriesTab'
import { ApAgingTab } from './components/ApAgingTab'
import { ArAgingTab } from './components/ArAgingTab'
import { ProfitLossTab } from './components/ProfitLossTab'

export function AccountingReportPage() {
  const today = new Date().toISOString().slice(0, 10)
  const [asOfDate, setAsOfDate] = useState(today)
  const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter({
    initialFrom: today.slice(0, 7) + '-01',
    initialTo: today,
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">會計報表</h1>
          <p className="text-muted-foreground">試算表、傳票、應付／應收帳款</p>
        </div>
      </div>

      <Tabs defaultValue="trial-balance" className="space-y-4">
        <TabsList className="grid w-full grid-cols-5 lg:w-auto lg:inline-grid">
          <TabsTrigger value="trial-balance" className="gap-2">
            <Calculator className="h-4 w-4" />
            試算表
          </TabsTrigger>
          <TabsTrigger value="journal-entries" className="gap-2">
            <FileText className="h-4 w-4" />
            傳票查詢
          </TabsTrigger>
          <TabsTrigger value="ap-aging" className="gap-2">
            <CreditCard className="h-4 w-4" />
            應付帳款
          </TabsTrigger>
          <TabsTrigger value="ar-aging" className="gap-2">
            <Receipt className="h-4 w-4" />
            應收帳款
          </TabsTrigger>
          <TabsTrigger value="profit-loss" className="gap-2">
            <TrendingUp className="h-4 w-4" />
            損益表
          </TabsTrigger>
        </TabsList>

        <TabsContent value="trial-balance">
          <TrialBalanceTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </TabsContent>

        <TabsContent value="journal-entries">
          <JournalEntriesTab
            dateFrom={dateFrom}
            dateTo={dateTo}
            onDateFromChange={setDateFrom}
            onDateToChange={setDateTo}
          />
        </TabsContent>

        <TabsContent value="ap-aging">
          <ApAgingTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </TabsContent>

        <TabsContent value="ar-aging">
          <ArAgingTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </TabsContent>

        <TabsContent value="profit-loss">
          <ProfitLossTab
            dateFrom={dateFrom}
            dateTo={dateTo}
            onDateFromChange={setDateFrom}
            onDateToChange={setDateTo}
          />
        </TabsContent>
      </Tabs>
    </div>
  )
}
