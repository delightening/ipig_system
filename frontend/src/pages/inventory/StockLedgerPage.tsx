import { useQuery } from '@tanstack/react-query'
import api, { StockLedgerDetail } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2, FileText } from 'lucide-react'
import { formatDateTime, formatNumber, formatCurrency } from '@/lib/utils'

const directionNames: Record<string, string> = {
  in: '入庫',
  out: '出庫',
  transfer_in: '調入',
  transfer_out: '調出',
  adjust_in: '調增',
  adjust_out: '調減',
}

export function StockLedgerPage() {
  const { data: ledger, isLoading } = useQuery({
    queryKey: ['stock-ledger'],
    queryFn: async () => {
      const response = await api.get<StockLedgerDetail[]>('/inventory/ledger')
      return response.data
    },
  })

  const getDirectionBadge = (direction: string) => {
    const isInbound = ['in', 'transfer_in', 'adjust_in'].includes(direction)
    return (
      <Badge variant={isInbound ? 'success' : 'destructive'}>
        {directionNames[direction] || direction}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">庫存流水</h1>
        <p className="text-muted-foreground">查看所有庫存異動記錄</p>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>時間</TableHead>
              <TableHead>倉庫</TableHead>
              <TableHead>品項</TableHead>
              <TableHead>單據</TableHead>
              <TableHead>方向</TableHead>
              <TableHead className="text-right">數量</TableHead>
              <TableHead className="text-right">單位成本</TableHead>
              <TableHead>批號</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : ledger && ledger.length > 0 ? (
              ledger.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="text-sm">{formatDateTime(item.trx_date)}</TableCell>
                  <TableCell>{item.warehouse_name}</TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{item.product_name}</div>
                      <div className="text-xs text-muted-foreground">{item.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">{item.doc_no}</TableCell>
                  <TableCell>{getDirectionBadge(item.direction)}</TableCell>
                  <TableCell className="text-right font-medium">
                    {formatNumber(item.qty_base, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {item.unit_cost ? formatCurrency(item.unit_cost) : '-'}
                  </TableCell>
                  <TableCell>{item.batch_no || '-'}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <FileText className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無庫存流水資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
