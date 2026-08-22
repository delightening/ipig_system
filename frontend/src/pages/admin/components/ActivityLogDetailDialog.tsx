import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FileJson, ShieldCheck, UserCog } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import type { UserActivityLog } from '@/types/hr'
import { formatDate, formatTime } from '@/lib/utils'

import { categoryLabels, eventTypeLabels, entityTypeLabels } from '../constants/auditLogs'

// R30-13: 逐 key diff 工具與 DiffView component
type JsonObj = Record<string, unknown>

function isPlainObject(v: unknown): v is JsonObj {
  return typeof v === 'object' && v !== null && !Array.isArray(v)
}

/** 將值轉為單行字串以便顯示。物件/陣列用 JSON.stringify。 */
function formatValue(v: unknown): string {
  if (v === null || v === undefined) return '∅'
  if (typeof v === 'string') return v
  if (typeof v === 'number' || typeof v === 'boolean') return String(v)
  try {
    return JSON.stringify(v)
  } catch {
    return String(v)
  }
}

interface DiffRow {
  key: string
  kind: 'removed' | 'added' | 'modified' | 'unchanged'
  before?: unknown
  after?: unknown
  /** 是否在 changed_fields 清單中（service 層回報） */
  highlighted: boolean
}

function computeDiffRows(
  before: unknown,
  after: unknown,
  changedFields: string[] | null | undefined,
): DiffRow[] {
  const beforeObj: JsonObj = isPlainObject(before) ? before : {}
  const afterObj: JsonObj = isPlainObject(after) ? after : {}
  const allKeys = Array.from(new Set([...Object.keys(beforeObj), ...Object.keys(afterObj)])).sort()
  const changedSet = new Set(changedFields ?? [])

  return allKeys.map<DiffRow>((key) => {
    const inBefore = key in beforeObj
    const inAfter = key in afterObj
    const bVal = beforeObj[key]
    const aVal = afterObj[key]

    let kind: DiffRow['kind']
    if (inBefore && !inAfter) kind = 'removed'
    else if (!inBefore && inAfter) kind = 'added'
    else if (formatValue(bVal) !== formatValue(aVal)) kind = 'modified'
    else kind = 'unchanged'

    return {
      key,
      kind,
      before: bVal,
      after: aVal,
      highlighted: changedSet.has(key),
    }
  })
}

interface DiffViewProps {
  before: unknown
  after: unknown
  changedFields: string[] | null | undefined
  t: TFunction
}

function DiffView({ before, after, changedFields, t }: DiffViewProps) {
  const rows = computeDiffRows(before, after, changedFields)
  // 若無法呈現為物件 diff（兩邊都不是 plain object），fallback 為 raw JSON
  const fallback = !isPlainObject(before) && !isPlainObject(after)
  if (fallback) {
    return (
      <pre className="mt-1 p-3 bg-muted/50 border rounded-md text-sm overflow-x-auto">
        {JSON.stringify({ before, after }, null, 2)}
      </pre>
    )
  }

  return (
    <div className="mt-1 border rounded-md divide-y text-sm overflow-hidden">
      {rows.length === 0 ? (
        <div className="p-3 text-muted-foreground italic">{t('admin.activityLogDetailDialog.noFieldChanges')}</div>
      ) : (
        rows.map((row) => {
          const baseRow = 'grid grid-cols-[auto_1fr_auto_1fr] gap-2 px-3 py-1.5 items-start'
          const ringCls = row.highlighted ? 'ring-1 ring-status-warning-text/40' : ''
          if (row.kind === 'removed') {
            return (
              <div key={row.key} className={`${baseRow} bg-status-error-bg ${ringCls}`}>
                <span className="font-mono text-xs font-semibold">{row.key}</span>
                <span className="line-through break-all">{formatValue(row.before)}</span>
                <span className="text-muted-foreground">→</span>
                <span className="text-muted-foreground italic">{t('admin.activityLogDetailDialog.diffRemoved')}</span>
              </div>
            )
          }
          if (row.kind === 'added') {
            return (
              <div key={row.key} className={`${baseRow} bg-status-success-bg ${ringCls}`}>
                <span className="font-mono text-xs font-semibold">{row.key}</span>
                <span className="text-muted-foreground italic">{t('admin.activityLogDetailDialog.diffAbsent')}</span>
                <span className="text-muted-foreground">→</span>
                <span className="break-all">{formatValue(row.after)}</span>
              </div>
            )
          }
          if (row.kind === 'modified') {
            return (
              <div key={row.key} className={`${baseRow} bg-status-warning-bg ${ringCls}`}>
                <span className="font-mono text-xs font-semibold">{row.key}</span>
                <span className="line-through break-all opacity-70">{formatValue(row.before)}</span>
                <span className="text-muted-foreground">→</span>
                <span className="break-all">{formatValue(row.after)}</span>
              </div>
            )
          }
          // unchanged
          return (
            <div key={row.key} className={`${baseRow} text-muted-foreground/80`}>
              <span className="font-mono text-xs">{row.key}</span>
              <span className="break-all col-span-3">{formatValue(row.after)}</span>
            </div>
          )
        })
      )}
    </div>
  )
}

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
  const { t } = useTranslation()
  return (
    <Dialog open={!!selectedLog} onOpenChange={() => onClose()}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileJson className="h-5 w-5" />
            {t('admin.activityLogDetailDialog.title')}
          </DialogTitle>
        </DialogHeader>
        {selectedLog && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.operationTime')}</Label>
                <div className="font-medium">{formatDateTimeDisplay(selectedLog.created_at)}</div>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.operator')}</Label>
                <p className="font-medium">{selectedLog.actor_display_name || '-'}</p>
                <p className="text-sm text-muted-foreground">{selectedLog.actor_email || ''}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.eventCategory')}</Label>
                <p className="font-medium">{categoryLabels[selectedLog.event_category] || selectedLog.event_category}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.eventType')}</Label>
                <div className="mt-1">{getEventBadge(selectedLog.event_type)}</div>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.entityType')}</Label>
                <p className="font-medium">{entityTypeLabels[selectedLog.entity_type || ''] || selectedLog.entity_type || '-'}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.entityName')}</Label>
                <p className="font-medium">{selectedLog.entity_display_name || '-'}</p>
              </div>
              {selectedLog.entity_id && (
                <div className="col-span-2">
                  <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.entityId')}</Label>
                  <p className="font-mono text-sm">{selectedLog.entity_id}</p>
                </div>
              )}
              {selectedLog.ip_address && (
                <div>
                  <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.ipAddress')}</Label>
                  <p className="text-sm">{selectedLog.ip_address}</p>
                </div>
              )}
              {/* R26-1: SEC-11 impersonate 場景顯示真正執行的管理員 */}
              {selectedLog.impersonated_by_user_id && (
                <div className="col-span-2">
                  <Label className="text-muted-foreground flex items-center gap-1">
                    <UserCog className="h-4 w-4" />
                    {t('admin.activityLogDetailDialog.impersonationRealAdmin')}
                  </Label>
                  <p className="font-mono text-sm text-status-warning-text">
                    {selectedLog.impersonated_by_user_id}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {t('admin.activityLogDetailDialog.impersonationHint')}
                  </p>
                </div>
              )}
            </div>

            {/* R26-3: changed_fields — 變動的欄位名稱清單（含 redact 後欄位名）*/}
            {selectedLog.changed_fields && selectedLog.changed_fields.length > 0 && (
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.changedFields')}</Label>
                <div className="mt-1 flex flex-wrap gap-1">
                  {selectedLog.changed_fields.map((field) => (
                    <Badge key={field} variant="outline" className="font-mono text-xs">
                      {field}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {/* R30-13: 逐 key diff，三色標示新增 / 移除 / 修改；
                changed_fields 在右側加 ring 強調 */}
            {(selectedLog.before_data || selectedLog.after_data) && (
              <div>
                <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.dataDiff')}</Label>
                <DiffView
                  before={selectedLog.before_data}
                  after={selectedLog.after_data}
                  changedFields={selectedLog.changed_fields}
                  t={t}
                />
                <p className="mt-2 text-xs text-muted-foreground">
                  {t('admin.activityLogDetailDialog.legendIntro')}
                  <span className="ml-1 px-1 bg-status-error-bg">{t('admin.activityLogDetailDialog.legendRed')}</span> {t('admin.activityLogDetailDialog.legendRemoved')}
                  <span className="ml-1 px-1 bg-status-success-bg">{t('admin.activityLogDetailDialog.legendGreen')}</span> {t('admin.activityLogDetailDialog.legendAdded')}
                  <span className="ml-1 px-1 bg-status-warning-bg">{t('admin.activityLogDetailDialog.legendYellow')}</span> {t('admin.activityLogDetailDialog.legendModified')}
                  {t('admin.activityLogDetailDialog.legendOutro')}
                </p>
              </div>
            )}

            {/* SEC-34 + R26-6: HMAC 雜湊鏈完整性資訊（GLP §11.10 audit trail 不可竄改） */}
            {(selectedLog.integrity_hash || selectedLog.hmac_version !== null) && (
              <details className="border rounded-md p-3 bg-muted/30">
                <summary className="cursor-pointer flex items-center gap-2 text-sm font-medium">
                  <ShieldCheck className="h-4 w-4 text-status-success-text" />
                  {t('admin.activityLogDetailDialog.integrityTitle')}
                </summary>
                <div className="mt-3 space-y-2 text-xs">
                  {selectedLog.hmac_version !== null && (
                    <div>
                      <Label className="text-muted-foreground">{t('admin.activityLogDetailDialog.hmacVersion')}</Label>
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
                    {t('admin.activityLogDetailDialog.integrityHint')}
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
