import { AlertTriangle, ExternalLink, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
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
import { formatDateTime } from '@/lib/utils'
import type { SecurityAlert } from '@/types/hr'
import { AlertLockPanel } from './AlertLockPanel'
import { alertTypeLabels, severityLabels } from '../constants/auditLogs'

// R46-6: refresh_token_reuse 處理 SOP — 對齊 backend SEC_EVENT_REFRESH_TOKEN_REUSE 常數
const ALERT_TYPE_REFRESH_TOKEN_REUSE = 'REFRESH_TOKEN_REUSE'

interface AuditAlertDetailDialogProps {
  alert: SecurityAlert | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onResolve: (alertId: string) => void
  isResolving: boolean
}

const readText = (data: Record<string, unknown> | null, key: string): string | null => {
  const value = data?.[key]
  return typeof value === 'string' && value.length > 0 ? value : null
}

/**
 * 警報來源摘要：把 context_data 裡最常被查的欄位提到前面，
 * 不必從下方那坨 JSON 裡自己撈。IP 是判斷「本人換網路 vs 他人盜用」的第一手線索。
 */
function AlertSourceSummary({ contextData }: { contextData: Record<string, unknown> | null }) {
  const { t } = useTranslation()

  const ip = readText(contextData, 'ip')
  const country = readText(contextData, 'geo_country')
  const city = readText(contextData, 'geo_city')
  const browser = readText(contextData, 'browser')
  const os = readText(contextData, 'os')
  const previousIp = readText(contextData, 'previous_login_ip')
  const previousAt = readText(contextData, 'previous_login_at')

  const location = [country, city].filter(Boolean).join(' ')
  const device = [browser, os].filter(Boolean).join(' / ')
  const previousLogin = previousIp
    ? [previousIp, previousAt ? `(${formatDateTime(previousAt)})` : null].filter(Boolean).join(' ')
    : null

  if (!ip && !location && !device && !previousLogin) return null

  const fields: { label: string; value: string; mono?: boolean }[] = [
    ip ? { label: t('admin.auditAlertDetailDialog.sourceIp'), value: ip, mono: true } : null,
    location ? { label: t('admin.auditAlertDetailDialog.sourceLocation'), value: location } : null,
    device ? { label: t('admin.auditAlertDetailDialog.sourceDevice'), value: device } : null,
    previousLogin
      ? { label: t('admin.auditAlertDetailDialog.previousLogin'), value: previousLogin, mono: true }
      : null,
  ].filter((field): field is { label: string; value: string; mono?: boolean } => field !== null)

  return (
    <div className="grid grid-cols-2 gap-4">
      {fields.map((field) => (
        <div key={field.label}>
          <Label className="text-muted-foreground">{field.label}</Label>
          <p className={`text-sm mt-1 break-all ${field.mono ? 'font-mono' : ''}`}>{field.value}</p>
        </div>
      ))}
    </div>
  )
}

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
  const { t } = useTranslation()
  const navigate = useNavigate()
  const isRefreshTokenReuse = alert?.alert_type === ALERT_TYPE_REFRESH_TOKEN_REUSE

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="2xl" className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            {t('admin.auditAlertDetailDialog.title')}
          </DialogTitle>
          <DialogDescription>
            {t('admin.auditAlertDetailDialog.description')}
          </DialogDescription>
        </DialogHeader>
        {alert && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.alertTime')}</Label>
                <p className="font-medium">{formatDateTime(alert.created_at)}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.alertType')}</Label>
                <div className="mt-1">
                  <Badge variant="outline">{alertTypeLabels[alert.alert_type] ?? alert.alert_type}</Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.severity')}</Label>
                <div className="mt-1">
                  <Badge variant={getSeverityColor(alert.severity)}>
                    {severityLabels[alert.severity] ?? alert.severity}
                  </Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.status')}</Label>
                <div className="mt-1">
                  <Badge variant={alert.status === 'resolved' ? 'secondary' : 'default'}>
                    {alert.status === 'open'
                      ? t('admin.auditAlertDetailDialog.statusOpen')
                      : t('admin.auditAlertDetailDialog.statusResolved')}
                  </Badge>
                </div>
              </div>
            </div>

            <hr className="border-border" />

            <div>
              <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.alertTitle')}</Label>
              <p className="font-medium text-base">{alert.title}</p>
            </div>
            {alert.description && (
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.descriptionLabel')}</Label>
                <p className="text-sm mt-1 whitespace-pre-wrap">{alert.description}</p>
              </div>
            )}

            {alert.user_id && (
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.relatedUserId')}</Label>
                <div className="flex items-center gap-2 mt-1">
                  <p className="font-mono text-sm flex-1">{alert.user_id}</p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      onOpenChange(false)
                      navigate('/admin/users')
                    }}
                  >
                    <ExternalLink className="h-3.5 w-3.5 mr-1" />
                    {t('admin.auditAlertDetailDialog.viewUser')}
                  </Button>
                </div>
              </div>
            )}

            {/* R46-6: refresh_token_reuse 處理 SOP — 降低秘書認知負擔 */}
            {isRefreshTokenReuse && (
              <div className="rounded-md border border-status-warning-border bg-status-warning-bg p-4">
                <div className="flex items-center gap-2 mb-2">
                  <ShieldAlert className="h-4 w-4 text-status-warning-text" />
                  <Label className="text-status-warning-text font-medium">{t('admin.auditAlertDetailDialog.sopTitle')}</Label>
                </div>
                <ol className="text-sm space-y-1 list-decimal list-inside text-status-warning-text">
                  <li>{t('admin.auditAlertDetailDialog.sopStep1')}</li>
                  <li>{t('admin.auditAlertDetailDialog.sopStep2')}</li>
                  <li>{t('admin.auditAlertDetailDialog.sopStep3')}</li>
                  <li>{t('admin.auditAlertDetailDialog.sopStep4')}</li>
                </ol>
              </div>
            )}

            {/* 先資訊後操作：來源摘要（這次登入從哪來）在前，解鎖面板（要不要放行）在後 */}
            <AlertSourceSummary contextData={alert.context_data} />
            <AlertLockPanel alertId={alert.id} />

            {alert.context_data && Object.keys(alert.context_data).length > 0 && (
              <div>
                <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.contextData')}</Label>
                <pre className="mt-1 p-3 bg-muted/50 border rounded-md text-sm whitespace-pre-wrap break-all">
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
                      <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.resolvedAt')}</Label>
                      <p className="font-medium">{formatDateTime(alert.resolved_at)}</p>
                    </div>
                  )}
                  {alert.resolved_by && (
                    <div>
                      <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.resolvedBy')}</Label>
                      <p className="font-medium">{alert.resolved_by}</p>
                    </div>
                  )}
                </div>
                {alert.resolution_notes && (
                  <div>
                    <Label className="text-muted-foreground">{t('admin.auditAlertDetailDialog.resolutionNotes')}</Label>
                    <p className="text-sm mt-1">{alert.resolution_notes}</p>
                  </div>
                )}
              </>
            )}

            {alert.status !== 'resolved' && (
              <DialogFooter>
                <Button variant="outline" onClick={() => onOpenChange(false)}>
                  {t('common.closeDialog')}
                </Button>
                <Button
                  onClick={() => {
                    onResolve(alert.id)
                    onOpenChange(false)
                  }}
                  disabled={isResolving}
                >
                  {t('admin.auditAlertDetailDialog.markResolved')}
                </Button>
              </DialogFooter>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
