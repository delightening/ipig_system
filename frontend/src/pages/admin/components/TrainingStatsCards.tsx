import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { GraduationCap, AlertTriangle } from 'lucide-react'

interface TrainingStatsCardsProps {
  totalRecords: number
  expiringSoonCount: number
}

export function TrainingStatsCards({ totalRecords, expiringSoonCount }: TrainingStatsCardsProps) {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">訓練紀錄數</CardTitle>
          <GraduationCap className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{totalRecords}</div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <CardTitle className="text-sm font-medium">證照即將到期</CardTitle>
          <AlertTriangle className="h-4 w-4 text-orange-500" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-orange-500">{expiringSoonCount}</div>
        </CardContent>
      </Card>
    </div>
  )
}
