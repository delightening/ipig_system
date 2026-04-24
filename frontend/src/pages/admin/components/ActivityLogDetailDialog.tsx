import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FileJson, ShieldCheck, UserCog } from 'lucide-react'
import type { UserActivityLog } from '@/types/hr'
import { formatDate, formatTime } from '@/lib/utils'

import { categoryLabels, eventTypeLabels, entityTypeLabels } from '../constants/auditLogs'

/** R26-6 HMAC 編碼版本標籤對照 */
const HMAC_VERSION_LABELS: Record<number, string> = {
  1: 'v1 (legacy string-concat)',
  2: 'v2 (length-prefix canonical)',
}

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
  const config = eventTypeLabels[eventType] || { label: eventType, color: 'bg-muted0' }
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
              {/* R26-1: SEC-11 impersonate 場景顯示真正執行的管理員 */}
              {selectedLog.impersonated_by_user_id && (
                <div className="col-span-2">
                  <Label className="text-muted-foreground flex items-center gap-1">
                    <UserCog className="h-4 w-4" />
                    模擬登入：真正執行管理員
                  </Label>
                  <p className="font-mono text-sm text-status-warning-text">
                    {selectedLog.impersonated_by_user_id}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    SEC-11：actor_user_id 為被模擬者，此欄位記錄真正操作的管理員
                  </p>
                </div>
              )}
            </div>

            {/* R26-3: changed_fields — 變動的欄位名稱清單（含 redact 後欄位名）*/}
            {selectedLog.changed_fields && selectedLog.changed_fields.length > 0 && (
              <div>
                <Label className="text-muted-foreground">變動欄位</Label>
                <div className="mt-1 flex flex-wrap gap-1">
                  {selectedLog.changed_fields.map((field) => (
                    <Badge key={field} variant="outline" className="font-mono text-xs">
                      {field}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

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

            {/* SEC-34 + R26-6: HMAC 雜湊鏈完整性資訊（GLP §11.10 audit trail 不可竄改） */}
            {(selectedLog.integrity_hash || selectedLog.hmac_version !== null) && (
              <details className="border rounded-md p-3 bg-muted/30">
                <summary className="cursor-pointer flex items-center gap-2 text-sm font-medium">
                  <ShieldCheck className="h-4 w-4 text-status-success-text" />
                  資料完整性 (HMAC chain)
                </summary>
                <div className="mt-3 space-y-2 text-xs">
                  {selectedLog.hmac_version !== null && (
                    <div>
                      <Label className="text-muted-foreground">HMAC 編碼版本</Label>
                      <p className="font-mono">
                        {HMAC_VERSION_LABELS[selectedLog.hmac_version] ??
                          `v${selectedLog.hmac_version}`}
                      </p>
                    </div>
                  )}
                  {selectedLog.integrity_hash && (
                    <div>
                      <Label className="text-muted-foreground">Integrity Hash</Label>
                      <p className="font-mono break-all">{selectedLog.integrity_hash}</p>
                    </div>
                  )}
                  {selectedLog.previous_hash && (
                    <div>
                      <Label className="text-muted-foreground">Previous Hash</Label>
                      <p className="font-mono break-all">{selectedLog.previous_hash}</p>
                    </div>
                  )}
                  <p className="text-muted-foreground italic mt-2">
                    每日 02:00 UTC 自動驗證；斷鏈時觸發 audit_chain_broken 安全告警
                  </p>
                </div>
              </details>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
