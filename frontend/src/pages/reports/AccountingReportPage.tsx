import { useState } from 'react'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
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
      <PageHeader
        title="會計報表"
        description="試算表、傳票、應付／應收帳款"
      />

      <PageTabs
        tabs={[
          { value: 'trial-balance', label: '試算表', icon: Calculator },
          { value: 'journal-entries', label: '傳票查詢', icon: FileText },
          { value: 'ap-aging', label: '應付帳款', icon: CreditCard },
          { value: 'ar-aging', label: '應收帳款', icon: Receipt },
          { value: 'profit-loss', label: '損益表', icon: TrendingUp },
        ]}
        defaultTab="trial-balance"
        className="space-y-4"
      >
        <PageTabContent value="trial-balance">
          <TrialBalanceTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </PageTabContent>

        <PageTabContent value="journal-entries">
          <JournalEntriesTab
            dateFrom={dateFrom}
            dateTo={dateTo}
            onDateFromChange={setDateFrom}
            onDateToChange={setDateTo}
          />
        </PageTabContent>

        <PageTabContent value="ap-aging">
          <ApAgingTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </PageTabContent>

        <PageTabContent value="ar-aging">
          <ArAgingTab asOfDate={asOfDate} onAsOfDateChange={setAsOfDate} />
        </PageTabContent>

        <PageTabContent value="profit-loss">
          <ProfitLossTab
            dateFrom={dateFrom}
            dateTo={dateTo}
            onDateFromChange={setDateFrom}
            onDateToChange={setDateTo}
          />
        </PageTabContent>
      </PageTabs>
    </div>
  )
}
