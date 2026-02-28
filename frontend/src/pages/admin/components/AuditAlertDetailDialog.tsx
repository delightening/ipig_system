import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import { AlertTriangle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import type { SecurityAlert } from '@/types/hr'

interface AuditAlertDetailDialogProps {
  alert: SecurityAlert | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onResolve: (alertId: string) => void
  isResolving: boolean
}

const formatDateTime = (dateStr: string) =>
  format(new Date(dateStr), 'yyyy/MM/dd HH:mm', { locale: zhTW })

const getSeverityColor = (severity: string) => {
  switch (severity) {
    case 'critical':
    case 'high':
      return 'destructive' as const
    case 'warning':
      return 'warning' as const
    case 'medium':
    case 'info':
      return 'default' as const
    default:
      return 'secondary' as const
  }
}

export function AuditAlertDetailDialog({
  alert,
  open,
  onOpenChange,
  onResolve,
  isResolving,
}: AuditAlertDetailDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            安全警報詳情
          </DialogTitle>
          <DialogDescription>
            查看警報的完整資訊與上下文
          </DialogDescription>
        </DialogHeader>
        {alert && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">警報時間</Label>
                <p className="font-medium">{formatDateTime(alert.created_at)}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">警報類型</Label>
                <div className="mt-1">
                  <Badge variant="outline">{alert.alert_type}</Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">嚴重程度</Label>
                <div className="mt-1">
                  <Badge variant={getSeverityColor(alert.severity)}>
                    {alert.severity}
                  </Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">狀態</Label>
                <div className="mt-1">
                  <Badge variant={alert.status === 'resolved' ? 'secondary' : 'default'}>
                    {alert.status === 'open' ? '待處理' : '已解決'}
                  </Badge>
                </div>
              </div>
            </div>

            <hr className="border-border" />

            <div>
              <Label className="text-muted-foreground">標題</Label>
              <p className="font-medium text-base">{alert.title}</p>
            </div>
            {alert.description && (
              <div>
                <Label className="text-muted-foreground">描述</Label>
                <p className="text-sm mt-1 whitespace-pre-wrap">{alert.description}</p>
              </div>
            )}

            {alert.user_id && (
              <div>
                <Label className="text-muted-foreground">相關使用者 ID</Label>
                <p className="font-mono text-sm">{alert.user_id}</p>
              </div>
            )}

            {alert.context_data && Object.keys(alert.context_data).length > 0 && (
              <div>
                <Label className="text-muted-foreground">詳細上下文資料</Label>
                <pre className="mt-1 p-3 bg-muted/50 border rounded-md text-sm overflow-x-auto">
                  {JSON.stringify(alert.context_data, null, 2)}
                </pre>
              </div>
            )}

            {alert.status === 'resolved' && (
              <>
                <hr className="border-border" />
                <div className="grid grid-cols-2 gap-4">
                  {alert.resolved_at && (
                    <div>
                      <Label className="text-muted-foreground">解決時間</Label>
                      <p className="font-medium">{formatDateTime(alert.resolved_at)}</p>
                    </div>
                  )}
                  {alert.resolved_by && (
                    <div>
                      <Label className="text-muted-foreground">解決者</Label>
                      <p className="font-medium">{alert.resolved_by}</p>
                    </div>
                  )}
                </div>
                {alert.resolution_notes && (
                  <div>
                    <Label className="text-muted-foreground">解決備註</Label>
                    <p className="text-sm mt-1">{alert.resolution_notes}</p>
                  </div>
                )}
              </>
            )}

            {alert.status !== 'resolved' && (
              <DialogFooter>
                <Button variant="outline" onClick={() => onOpenChange(false)}>
                  關閉
                </Button>
                <Button
                  onClick={() => {
                    onResolve(alert.id)
                    onOpenChange(false)
                  }}
                  disabled={isResolving}
                >
                  標記為已解決
                </Button>
              </DialogFooter>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
