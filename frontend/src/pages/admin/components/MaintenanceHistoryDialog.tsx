/**
 * 維修/保養紀錄變更歷史 Dialog — timeline 顯示每筆操作的 before/after 差異
 */
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, History, Clock, User } from 'lucide-react'
import type { UserActivityLog } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import { MAINTENANCE_TYPE_LABELS, MAINTENANCE_STATUS_LABELS } from '../types'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  recordId: string | null
}

const EVENT_TYPE_CONFIG: Record<string, { label: string; variant: StatusVariant }> = {
  MAINTENANCE_CREATE: { label: '新增', variant: 'success' },
  MAINTENANCE_UPDATE: { label: '編輯', variant: 'info' },
  MAINTENANCE_DELETE: { label: '刪除', variant: 'error' },
}

const FIELD_LABELS: Record<string, string> = {
  maintenance_type: '類型',
  status: '狀態',
  reported_at: '報修日期',
  completed_at: '完修日期',
  problem_description: '問題描述',
  repair_content: '維修內容',
  maintenance_items: '保養項目',
  performed_by: '執行人',
  notes: '備註',
  repair_partner_id: '維修廠商',
}

/** 將 enum 值轉為中文顯示 */
function formatFieldValue(key: string, value: unknown): string {
  if (value === null || value === undefined) return '—'
  const str = String(value)
  if (key === 'maintenance_type') return (MAINTENANCE_TYPE_LABELS as Record<string, string>)[str] ?? str
  if (key === 'status') return (MAINTENANCE_STATUS_LABELS as Record<string, string>)[str] ?? str
  return str
}

/** 比較 before/after，回傳有變更的欄位 */
function getChangedFields(
  before: Record<string, unknown> | null,
  after: Record<string, unknown> | null,
): { key: string; label: string; before: string; after: string }[] {
  if (!before || !after) return []
  const tracked = Object.keys(FIELD_LABELS)
  return tracked
    .filter((k) => String(before[k] ?? '') !== String(after[k] ?? ''))
    .map((k) => ({
      key: k,
      label: FIELD_LABELS[k],
      before: formatFieldValue(k, before[k]),
      after: formatFieldValue(k, after[k]),
    }))
}

function formatDateTime(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-TW', {
    timeZone: 'Asia/Taipei',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function MaintenanceHistoryDialog({ open, onOpenChange, recordId }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['maintenance-history', recordId],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<UserActivityLog>>(
        `/equipment-maintenance/${recordId}/history`,
      )
      return res.data.data
    },
    enabled: open && !!recordId,
    staleTime: 30_000,
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            變更歷史
          </DialogTitle>
          <DialogDescription>維修/保養紀錄的操作歷程</DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto py-4">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : !data || data.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <History className="h-12 w-12 mx-auto mb-4" />
              <p>尚無變更歷史</p>
            </div>
          ) : (
            <div className="relative">
              {data.map((log, index) => {
                const config = EVENT_TYPE_CONFIG[log.event_type] ?? {
                  label: log.event_type,
                  variant: 'neutral' as StatusVariant,
                }
                const changes = getChangedFields(log.before_data, log.after_data)
                const snapshot = log.after_data ?? log.before_data

                return (
                  <div key={log.id} className="relative flex gap-4 pb-6 last:pb-0">
                    {index < data.length - 1 && (
                      <div className="absolute left-[11px] top-6 bottom-0 w-0.5 bg-muted" />
                    )}
                    <div className="relative z-10 flex h-6 w-6 items-center justify-center rounded-full bg-muted ring-4 ring-background">
                      <span className="text-xs font-medium text-muted-foreground">
                        {data.length - index}
                      </span>
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2 text-sm">
                          <StatusBadge variant={config.variant}>{config.label}</StatusBadge>
                          <Clock className="h-3.5 w-3.5 text-muted-foreground" />
                          <span className="text-muted-foreground">
                            {formatDateTime(log.created_at)}
                          </span>
                        </div>
                        <div className="flex items-center gap-1 text-sm text-muted-foreground">
                          <User className="h-3.5 w-3.5" />
                          <span>{log.actor_display_name ?? '系統'}</span>
                        </div>
                      </div>

                      {log.event_type === 'MAINTENANCE_UPDATE' && changes.length > 0 ? (
                        <div className="p-3 bg-muted rounded-lg text-sm space-y-1.5">
                          {changes.map((c) => (
                            <div key={c.key} className="flex items-baseline gap-1">
                              <span className="text-muted-foreground shrink-0">{c.label}：</span>
                              <span className="line-through text-muted-foreground">{c.before}</span>
                              <span className="mx-1">→</span>
                              <span className="font-medium">{c.after}</span>
                            </div>
                          ))}
                        </div>
                      ) : snapshot ? (
                        <SnapshotPreview data={snapshot} />
                      ) : null}
                    </div>
                  </div>
                )
              })}
            </div>
          )}
        </div>

        <div className="flex justify-end pt-4 border-t">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            關閉
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}

/** 建立/刪除時的快照預覽 */
function SnapshotPreview({ data }: { data: Record<string, unknown> }) {
  const fields = Object.keys(FIELD_LABELS).filter((k) => data[k] != null)
  if (fields.length === 0) return null
  return (
    <div className="p-3 bg-muted rounded-lg text-sm">
      <div className="grid grid-cols-2 gap-1.5">
        {fields.map((k) => (
          <div key={k}>
            <span className="text-muted-foreground">{FIELD_LABELS[k]}：</span>
            <span className="ml-1">{formatFieldValue(k, data[k])}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
