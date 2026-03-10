import { Eye } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import { formatDateTime } from '@/lib/utils'

interface AuditLog {
  id: string
  actor_user_id: string
  actor_email: string
  actor_name: string
  action: string
  entity_type: string
  entity_id: string
  entity_email?: string
  entity_name?: string
  before_data?: Record<string, unknown>
  after_data?: Record<string, unknown>
  created_at: string
}

interface AuditLogDetailDialogProps {
  log: AuditLog | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AuditLogDetailDialog({ log, open, onOpenChange }: AuditLogDetailDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Eye className="h-5 w-5" />
            活動記錄詳情
          </DialogTitle>
        </DialogHeader>
        {log && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">操作時間</Label>
                <p className="font-medium">{formatDateTime(log.created_at)}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">操作者</Label>
                <p className="font-medium">{log.actor_name || '-'}</p>
                <p className="text-sm text-muted-foreground">{log.actor_email || ''}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">操作類型</Label>
                <div className="mt-1">
                  <Badge variant={{
                    'CREATE': 'default',
                    'UPDATE': 'default',
                    'DELETE': 'destructive',
                    'PASSWORD_RESET': 'secondary',
                    'IMPERSONATE': 'secondary',
                    'force_logout': 'destructive',
                  }[log.action] as 'default' | 'destructive' | 'secondary' || 'outline'}>
                    {{
                      'CREATE': '建立使用者',
                      'UPDATE': '更新使用者',
                      'DELETE': '刪除使用者',
                      'PASSWORD_RESET': '重設密碼',
                      'IMPERSONATE': '模擬登入',
                      'force_logout': '強制登出',
                    }[log.action] || log.action}
                  </Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">目標使用者</Label>
                {log.entity_name || log.entity_email ? (
                  <div>
                    <p className="font-medium">{log.entity_name || '-'}</p>
                    <p className="text-sm text-muted-foreground">{log.entity_email || ''}</p>
                  </div>
                ) : (
                  <p className="font-medium">
                    <Badge variant="outline">{log.entity_type}</Badge>
                  </p>
                )}
              </div>
              <div className="col-span-2">
                <Label className="text-muted-foreground">實體 ID</Label>
                <p className="font-mono text-sm">{log.entity_id || '-'}</p>
              </div>
            </div>

            {log.before_data && (
              <div>
                <Label className="text-muted-foreground">變更前資料</Label>
                <pre className="mt-1 p-3 bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800 rounded-md text-sm overflow-x-auto">
                  {JSON.stringify(log.before_data, null, 2)}
                </pre>
              </div>
            )}

            {log.after_data && (
              <div>
                <Label className="text-muted-foreground">變更後資料</Label>
                <pre className="mt-1 p-3 bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 rounded-md text-sm overflow-x-auto">
                  {JSON.stringify(log.after_data, null, 2)}
                </pre>
              </div>
            )}

            {!log.before_data && !log.after_data && (
              <p className="text-muted-foreground text-center py-4">無變更資料</p>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
