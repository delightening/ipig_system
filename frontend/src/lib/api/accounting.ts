import api from './client'

import type { TrialBalanceRow, JournalEntryResponse, ApAgingRow, ArAgingRow } from '@/types/accounting'
import type { ProfitLossSummary } from '@/types/report'

// --- Query Keys ---

export const accountingKeys = {
  chartOfAccounts: ['accounting-chart-of-accounts'] as const,
  trialBalance: (asOfDate: string) => ['accounting-trial-balance', asOfDate] as const,
  journalEntries: (dateFrom: string, dateTo: string) => ['accounting-journal-entries', dateFrom, dateTo] as const,
  apAging: (asOfDate: string) => ['accounting-ap-aging', asOfDate] as const,
  arAging: (asOfDate: string) => ['accounting-ar-aging', asOfDate] as const,
  profitLoss: (dateFrom: string, dateTo: string) => ['accounting-profit-loss', dateFrom, dateTo] as const,
}

// --- API Functions ---

export const accountingApi = {
  /** 取得會計科目表 */
  async listChartOfAccounts() {
    const { data } = await api.get('/accounting/chart-of-accounts')
    return data
  },

  /** 取得試算表 */
  async getTrialBalance(asOfDate: string) {
    const { data } = await api.get<TrialBalanceRow[]>('/accounting/trial-balance', {
      params: { as_of_date: asOfDate },
    })
    return data
  },

  /** 取得傳票清單 */
  async listJournalEntries(params: { dateFrom?: string; dateTo?: string; limit?: number }) {
    const { data } = await api.get<JournalEntryResponse[]>('/accounting/journal-entries', {
      params: { date_from: params.dateFrom, date_to: params.dateTo, limit: params.limit ?? 100 },
    })
    return data
  },

  /** 取得應付帳款帳齡 */
  async getApAging(asOfDate: string) {
    const { data } = await api.get<ApAgingRow[]>('/accounting/ap-aging', {
      params: { as_of_date: asOfDate },
    })
    return data
  },

  /** 取得應收帳款帳齡 */
  async getArAging(asOfDate: string) {
    const { data } = await api.get<ArAgingRow[]>('/accounting/ar-aging', {
      params: { as_of_date: asOfDate },
    })
    return data
  },

  /** 建立 AP 付款 */
  async createApPayment(payload: {
    partner_id: string
    payment_date: string
    amount: number
    reference?: string
  }) {
    const { data } = await api.post<{ id: string }>('/accounting/ap-payments', payload)
    return data
  },

  /** 建立 AR 收款 */
  async createArReceipt(payload: {
    partner_id: string
    receipt_date: string
    amount: number
    reference?: string
  }) {
    const { data } = await api.post<{ id: string }>('/accounting/ar-receipts', payload)
    return data
  },

  /** 取得損益表 */
  async getProfitLoss(params: { dateFrom?: string; dateTo?: string }) {
    const { data } = await api.get<ProfitLossSummary>('/accounting/profit-loss', {
      params: { date_from: params.dateFrom, date_to: params.dateTo },
    })
    return data
  },
}
