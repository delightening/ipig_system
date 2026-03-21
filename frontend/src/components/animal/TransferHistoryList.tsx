import { useState } from 'react'

import type { AnimalTransfer } from '@/lib/api'
import { transferStatusNames, transferTypeNames } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { CheckCircle2, XCircle, ChevronDown, ChevronUp } from 'lucide-react'

interface TransferHistoryListProps {
    transfers: AnimalTransfer[]
}

export function TransferHistoryList({ transfers }: TransferHistoryListProps) {
    const [showHistory, setShowHistory] = useState(false)

    if (transfers.length === 0) return null

    return (
        <div>
            <button
                className="flex items-center gap-2 text-sm font-medium text-slate-600 hover:text-slate-800 mb-3"
                onClick={() => setShowHistory(!showHistory)}
            >
                {showHistory ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                歷史轉讓紀錄 ({transfers.length})
            </button>
            {showHistory && (
                <div className="space-y-3">
                    {transfers.map(record => (
                        <Card key={record.id} className={`border ${record.status === 'completed' ? 'border-green-200' : 'border-red-200'}`}>
                            <CardContent className="pt-4">
                                <div className="flex items-center justify-between mb-2">
                                    <div className="flex items-center gap-2">
                                        {record.status === 'completed' ? (
                                            <CheckCircle2 className="h-4 w-4 text-green-600" />
                                        ) : (
                                            <XCircle className="h-4 w-4 text-red-600" />
                                        )}
                                        <span className="text-sm font-medium">
                                            {record.from_iacuc_no} → {record.to_iacuc_no || '—'}
                                        </span>
                                    </div>
                                    <Badge className={record.status === 'completed' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}>
                                        {transferStatusNames[record.status]}
                                    </Badge>
                                </div>
                                <p className="text-sm text-slate-500">
                                    類型：{transferTypeNames[record.transfer_type === 'external' ? 'external' : 'internal']}
                                </p>
                                <p className="text-sm text-slate-600">{record.reason}</p>
                                {record.rejected_reason && (
                                    <p className="text-sm text-red-600 mt-1">拒絕原因：{record.rejected_reason}</p>
                                )}
                                <p className="text-xs text-slate-400 mt-2">
                                    {new Date(record.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}
                                    {record.completed_at && ` → ${new Date(record.completed_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}`}
                                </p>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}
        </div>
    )
}
