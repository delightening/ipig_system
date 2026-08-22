import { Bell, AlertCircle, CheckCircle2, Loader2, Save } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Slider } from '@/components/ui/slider'
import type { NotificationSettings } from '@/types/notification'

interface NotificationPreferencesSectionProps {
  settings: NotificationSettings | null
  isLoading: boolean
  error: unknown
  isSaving: boolean
  onUpdate: <K extends keyof NotificationSettings>(key: K, value: NotificationSettings[K]) => void
  onSave: () => void
}

export function NotificationPreferencesSection({
  settings,
  isLoading,
  error,
  isSaving,
  onUpdate,
  onSave,
}: NotificationPreferencesSectionProps) {
  const { t } = useTranslation()
  return (
    <div className="border-t pt-6">
      <h2 className="text-2xl font-bold tracking-tight mb-4">{t('admin.notificationPreferencesSection.title')}</h2>
      <p className="text-muted-foreground mb-6">{t('admin.notificationPreferencesSection.subtitle')}</p>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : error ? (
        <Card className="border-destructive bg-status-error-bg">
          <CardContent className="flex items-center gap-3 py-6">
            <AlertCircle className="h-5 w-5 text-destructive" />
            <span className="text-status-error-text">{t('admin.notificationPreferencesSection.loadError')}</span>
          </CardContent>
        </Card>
      ) : settings ? (
        <div className="grid gap-6 md:grid-cols-2">
          <EmailNotificationsCard settings={settings} onUpdate={onUpdate} />
          <WarningParametersCard settings={settings} onUpdate={onUpdate} />
        </div>
      ) : null}

      {settings && (
        <div className="flex justify-end mt-6">
          <Button onClick={onSave} disabled={isSaving}>
            {isSaving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            {t('admin.notificationPreferencesSection.saveButton')}
          </Button>
        </div>
      )}
    </div>
  )
}

// --- Sub-components ---

interface NotificationCardProps {
  settings: NotificationSettings
  onUpdate: <K extends keyof NotificationSettings>(key: K, value: NotificationSettings[K]) => void
}

function EmailNotificationsCard({ settings, onUpdate }: NotificationCardProps) {
  const { t } = useTranslation()
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Bell className="h-5 w-5" />
          {t('admin.notificationPreferencesSection.emailCard.title')}
        </CardTitle>
        <CardDescription>{t('admin.notificationPreferencesSection.emailCard.description')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <NotificationGroup title={t('admin.notificationPreferencesSection.groups.inventory')}>
          <NotificationCheckbox
            id="email_low_stock"
            label={t('admin.notificationPreferencesSection.items.lowStock.label')}
            description={t('admin.notificationPreferencesSection.items.lowStock.description')}
            checked={settings.email_low_stock}
            onChange={(v) => onUpdate('email_low_stock', v)}
          />
          {settings.email_low_stock && (
            <div className="ml-6">
              <NotificationCheckbox
                id="low_stock_notify_immediately"
                label={t('admin.notificationPreferencesSection.items.notifyImmediately.label')}
                description={t('admin.notificationPreferencesSection.items.notifyImmediately.description')}
                checked={settings.low_stock_notify_immediately}
                onChange={(v) => onUpdate('low_stock_notify_immediately', v)}
                small
              />
            </div>
          )}
          <NotificationCheckbox
            id="email_expiry_warning"
            label={t('admin.notificationPreferencesSection.items.expiryWarning.label')}
            description={t('admin.notificationPreferencesSection.items.expiryWarning.description')}
            checked={settings.email_expiry_warning}
            onChange={(v) => onUpdate('email_expiry_warning', v)}
          />
        </NotificationGroup>

        <NotificationGroup title={t('admin.notificationPreferencesSection.groups.documentApproval')}>
          <NotificationCheckbox
            id="email_document_approval"
            label={t('admin.notificationPreferencesSection.items.documentApproval.label')}
            description={t('admin.notificationPreferencesSection.items.documentApproval.description')}
            checked={settings.email_document_approval}
            onChange={(v) => onUpdate('email_document_approval', v)}
          />
        </NotificationGroup>

        <NotificationGroup title={t('admin.notificationPreferencesSection.groups.aup')}>
          <NotificationCheckbox
            id="email_protocol_status"
            label={t('admin.notificationPreferencesSection.items.protocolStatus.label')}
            description={t('admin.notificationPreferencesSection.items.protocolStatus.description')}
            checked={settings.email_protocol_status}
            onChange={(v) => onUpdate('email_protocol_status', v)}
          />
        </NotificationGroup>

        <NotificationGroup title={t('admin.notificationPreferencesSection.groups.report')}>
          <NotificationCheckbox
            id="email_monthly_report"
            label={t('admin.notificationPreferencesSection.items.monthlyReport.label')}
            description={t('admin.notificationPreferencesSection.items.monthlyReport.description')}
            checked={settings.email_monthly_report}
            onChange={(v) => onUpdate('email_monthly_report', v)}
          />
        </NotificationGroup>
      </CardContent>
    </Card>
  )
}

function WarningParametersCard({ settings, onUpdate }: NotificationCardProps) {
  const { t } = useTranslation()
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertCircle className="h-5 w-5" />
          {t('admin.notificationPreferencesSection.warningCard.title')}
        </CardTitle>
        <CardDescription>{t('admin.notificationPreferencesSection.warningCard.description')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <Slider
            label={t('admin.notificationPreferencesSection.expiryWarningDays.label')}
            value={settings.expiry_warning_days}
            onChange={(value) => onUpdate('expiry_warning_days', value)}
            min={1}
            max={90}
            step={1}
            quickValues={[7, 14, 30, 60]}
            unit={t('admin.notificationPreferencesSection.expiryWarningDays.unit')}
            className="w-full"
          />
          <p className="text-xs text-muted-foreground">
            {t('admin.notificationPreferencesSection.expiryWarningDays.hint', { days: settings.expiry_warning_days })}
          </p>
        </div>

        <div className="rounded-lg bg-muted p-4 space-y-3">
          <h4 className="text-sm font-medium">{t('admin.notificationPreferencesSection.schedule.title')}</h4>
          <ul className="text-xs text-muted-foreground space-y-2">
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>{t('admin.notificationPreferencesSection.schedule.lowStockCheck')}</span>
            </li>
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>{t('admin.notificationPreferencesSection.schedule.expiryCheck')}</span>
            </li>
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>{t('admin.notificationPreferencesSection.schedule.expiredCleanup')}</span>
            </li>
          </ul>
        </div>

        <ActiveNotificationsBadges settings={settings} />
      </CardContent>
    </Card>
  )
}

function ActiveNotificationsBadges({ settings }: { settings: NotificationSettings }) {
  const { t } = useTranslation()
  const badges = [
    { key: 'email_low_stock', label: t('admin.notificationPreferencesSection.badges.lowStock'), color: 'blue' },
    { key: 'email_expiry_warning', label: t('admin.notificationPreferencesSection.badges.expiryWarning'), color: 'orange' },
    { key: 'email_document_approval', label: t('admin.notificationPreferencesSection.badges.documentApproval'), color: 'green' },
    { key: 'email_protocol_status', label: t('admin.notificationPreferencesSection.badges.protocolStatus'), color: 'purple' },
    { key: 'email_monthly_report', label: t('admin.notificationPreferencesSection.badges.monthlyReport'), color: 'gray' },
  ] as const

  const activeBadges = badges.filter(b => settings[b.key])

  const colorMap: Record<string, string> = {
    blue: 'bg-status-info-bg text-status-info-text',
    orange: 'bg-status-warning-bg text-status-warning-text',
    green: 'bg-status-success-bg text-status-success-text',
    purple: 'bg-status-purple-bg text-status-purple-text',
    gray: 'bg-status-neutral-bg text-status-neutral-text',
  }

  return (
    <div className="rounded-lg border p-4">
      <h4 className="text-sm font-medium mb-3">{t('admin.notificationPreferencesSection.activeTitle')}</h4>
      <div className="flex flex-wrap gap-2">
        {activeBadges.map(b => (
          <span
            key={b.key}
            className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${colorMap[b.color]}`}
          >
            {b.label}
          </span>
        ))}
        {activeBadges.length === 0 && (
          <span className="text-xs text-muted-foreground">{t('admin.notificationPreferencesSection.noneActive')}</span>
        )}
      </div>
    </div>
  )
}

function NotificationGroup({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="space-y-4">
      <h4 className="text-sm font-medium text-muted-foreground">{title}</h4>
      {children}
    </div>
  )
}

function NotificationCheckbox({
  id, label, description, checked, onChange, small = false,
}: {
  id: string
  label: string
  description: string
  checked: boolean
  onChange: (v: boolean) => void
  small?: boolean
}) {
  return (
    <div className="flex items-start space-x-3">
      <Checkbox id={id} checked={checked} onCheckedChange={(v) => onChange(v as boolean)} />
      <div className="grid gap-1.5 leading-none">
        <Label htmlFor={id} className={`cursor-pointer${small ? ' text-sm' : ''}`}>
          {label}
        </Label>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
    </div>
  )
}
