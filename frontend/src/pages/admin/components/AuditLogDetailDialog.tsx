import { useTranslation } from 'react-i18next'
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
import { entityTypeLabels } from '../constants/auditLogs'
import type { AuditLog } from '../types/audit'

interface AuditLogDetailDialogProps {
  log: AuditLog | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function AuditLogDetailDialog({ log, open, onOpenChange }: AuditLogDetailDialogProps) {
  const { t } = useTranslation()

  const actionLabels: Record<string, string> = {
    'CREATE': t('admin.auditLogDetailDialog.actionCreateUser'),
    'UPDATE': t('admin.auditLogDetailDialog.actionUpdateUser'),
    'DELETE': t('admin.auditLogDetailDialog.actionDeleteUser'),
    'PASSWORD_RESET': t('admin.auditLogDetailDialog.actionPasswordReset'),
    'IMPERSONATE': t('admin.auditLogDetailDialog.actionImpersonate'),
    'force_logout': t('admin.auditLogDetailDialog.actionForceLogout'),
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="2xl" className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Eye className="h-5 w-5" />
            {t('admin.auditLogDetailDialog.title')}
          </DialogTitle>
        </DialogHeader>
        {log && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.operationTime')}</Label>
                <p className="font-medium">{formatDateTime(log.created_at)}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.actor')}</Label>
                <p className="font-medium">{log.actor_name || '-'}</p>
                <p className="text-sm text-muted-foreground">{log.actor_email || ''}</p>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.actionType')}</Label>
                <div className="mt-1">
                  <Badge variant={{
                    'CREATE': 'default',
                    'UPDATE': 'default',
                    'DELETE': 'destructive',
                    'PASSWORD_RESET': 'secondary',
                    'IMPERSONATE': 'secondary',
                    'force_logout': 'destructive',
                  }[log.action] as 'default' | 'destructive' | 'secondary' || 'outline'}>
                    {actionLabels[log.action] || log.action}
                  </Badge>
                </div>
              </div>
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.targetUser')}</Label>
                {log.entity_name || log.entity_email ? (
                  <div>
                    <p className="font-medium">{log.entity_name || '-'}</p>
                    <p className="text-sm text-muted-foreground">{log.entity_email || ''}</p>
                  </div>
                ) : (
                  <p className="font-medium">
                    <Badge variant="outline">
                      {entityTypeLabels[log.entity_type || ''] || log.entity_type || '-'}
                    </Badge>
                  </p>
                )}
              </div>
              <div className="col-span-2">
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.dataId')}</Label>
                <p className="font-mono text-sm">{log.entity_id || '-'}</p>
              </div>
            </div>

            {log.before_data && (
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.beforeData')}</Label>
                <pre className="mt-1 p-3 bg-status-error-bg dark:bg-destructive/10 border border-destructive/20 dark:border-destructive/30 rounded-md text-sm whitespace-pre-wrap break-all">
                  {JSON.stringify(log.before_data, null, 2)}
                </pre>
              </div>
            )}

            {log.after_data && (
              <div>
                <Label className="text-muted-foreground">{t('admin.auditLogDetailDialog.afterData')}</Label>
                <pre className="mt-1 p-3 bg-status-success-bg dark:bg-status-success-text/10 border border-status-success-text/20 dark:border-status-success-text/30 rounded-md text-sm whitespace-pre-wrap break-all">
                  {JSON.stringify(log.after_data, null, 2)}
                </pre>
              </div>
            )}

            {!log.before_data && !log.after_data && (
              <p className="text-muted-foreground text-center py-4">{t('admin.auditLogDetailDialog.noChangeData')}</p>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
