import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
  const today = new Date().toISOString().slice(0, 10)
  const [asOfDate, setAsOfDate] = useState(today)
  const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter({
    initialFrom: today.slice(0, 7) + '-01',
    initialTo: today,
  })

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('reportsPages.accounting.title')}
        description={t('reportsPages.accounting.description')}
      />

      <PageTabs
        tabs={[
          { value: 'trial-balance', label: t('reportsPages.accounting.tabs.trialBalance'), icon: Calculator },
          { value: 'journal-entries', label: t('reportsPages.accounting.tabs.journalEntries'), icon: FileText },
          { value: 'ap-aging', label: t('reportsPages.accounting.tabs.apAging'), icon: CreditCard },
          { value: 'ar-aging', label: t('reportsPages.accounting.tabs.arAging'), icon: Receipt },
          { value: 'profit-loss', label: t('reportsPages.accounting.tabs.profitLoss'), icon: TrendingUp },
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
