export interface TrialBalanceRow {
  account_id: string
  account_code: string
  account_name: string
  account_type: string
  debit_balance: string
  credit_balance: string
}

export interface JournalEntryLine {
  line_id: string
  line_no: number
  account_code: string
  account_name: string
  debit_amount: string
  credit_amount: string
  description: string | null
}

export interface JournalEntry {
  id: string
  entry_no: string
  entry_date: string
  description: string | null
  source_entity_type: string | null
  source_entity_id: string | null
}

export interface JournalEntryResponse {
  entry: JournalEntry
  lines: JournalEntryLine[]
}

export interface ApAgingRow {
  partner_id: string
  partner_code: string
  partner_name: string
  total_payable: string
  total_paid: string
  balance: string
}

export interface ArAgingRow {
  partner_id: string
  partner_code: string
  partner_name: string
  total_receivable: string
  total_received: string
  balance: string
}

export interface Partner {
  id: string
  code: string
  name: string
  partner_type: 'supplier' | 'customer'
}
