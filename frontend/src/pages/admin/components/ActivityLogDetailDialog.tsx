import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FileJson } from 'lucide-react'
import type { UserActivityLog } from '@/types/hr'
import { formatDate, formatTime } from '@/lib/utils'

import { categoryLabels, eventTypeLabels, entityTypeLabels } from '../constants/auditLogs'

interface ActivityLogDetailDialogProps {
  selectedLog: UserActivityLog | null
  onClose: () => void
}

function formatDateTimeDisplay(dateStr: string) {
  return (
    <div className="block leading-tight">
      <div>{formatDate(dateStr)}</div>
      <div>{formatTime(dateStr)}</div>
    </div>
  )
}

function getEventBadge(eventType: string) {
  const config = eventTypeLabels[eventType] || { label: eventType, color: 'bg-gray-500' }
  return (
    <Badge className={`${config.color} text-white`}>
      {config.label}
    </Badge>
  )
}

export function ActivityLogDetailDialog({ selectedLog, onClose }: ActivityLogDetailDialogProps) {
  return (
    <Dialog open={!!selectedLog} onOpenChange={() => onClose()}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileJson className="h-5 w-5" />
            操作日誌詳情
          </DialogTitle>
        </DialogHeader>
        {selectedLog && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">操作時間</Label>
                <div className="font-medium">{formatDateTimeDisplay(selectedLog.created_at)}</div>
              </div>
              <div>
                <Label className="text-muted-foreground">操作者</Label>
                <p className="font-medium">{selectedLog.actor_display_name || '-'}</p>
                <p className="text-sm text-muted-foreground">{selectedLog.actor_email || ''}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">事件類別</Label>
                <p className="font-medium">{categoryLabels[selectedLog.event_category] || selectedLog.event_category}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">事件類型</Label>
                <div className="mt-1">{getEventBadge(selectedLog.event_type)}</div>
              </div>
              <div>
                <Label className="text-muted-foreground">實體類型</Label>
                <p className="font-medium">{entityTypeLabels[selectedLog.entity_type || ''] || selectedLog.entity_type || '-'}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">實體名稱</Label>
                <p className="font-medium">{selectedLog.entity_display_name || '-'}</p>
              </div>
              {selectedLog.entity_id && (
                <div className="col-span-2">
                  <Label className="text-muted-foreground">實體 ID</Label>
                  <p className="font-mono text-sm">{selectedLog.entity_id}</p>
                </div>
              )}
              {selectedLog.ip_address && (
                <div>
                  <Label className="text-muted-foreground">IP 位址</Label>
                  <p className="text-sm">{selectedLog.ip_address}</p>
                </div>
              )}
            </div>

            {selectedLog.before_data && (
              <div>
                <Label className="text-muted-foreground">變更前資料</Label>
                <pre className="mt-1 p-3 bg-status-error-bg border border-destructive/20 rounded-md text-sm overflow-x-auto">
                  {JSON.stringify(selectedLog.before_data, null, 2)}
                </pre>
              </div>
            )}

            {selectedLog.after_data && (
              <div>
                <Label className="text-muted-foreground">變更後資料</Label>
                <pre className="mt-1 p-3 bg-status-success-bg border border-status-success-text/20 rounded-md text-sm overflow-x-auto">
                  {JSON.stringify(selectedLog.after_data, null, 2)}
                </pre>
              </div>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
