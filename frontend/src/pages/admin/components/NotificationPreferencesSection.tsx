import { Bell, AlertCircle, CheckCircle2, Loader2, Save } from 'lucide-react'

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
  return (
    <div className="border-t pt-6">
      <h2 className="text-2xl font-bold tracking-tight mb-4">通知偏好設定</h2>
      <p className="text-muted-foreground mb-6">設定您希望接收的通知類型</p>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : error ? (
        <Card className="border-destructive bg-status-error-bg">
          <CardContent className="flex items-center gap-3 py-6">
            <AlertCircle className="h-5 w-5 text-destructive" />
            <span className="text-status-error-text">無法載入通知設定，請重新整理頁面</span>
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
            儲存通知設定
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
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Bell className="h-5 w-5" />
          Email 通知
        </CardTitle>
        <CardDescription>設定哪些事件要發送 Email 通知</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <NotificationGroup title="庫存管理">
          <NotificationCheckbox
            id="email_low_stock"
            label="低庫存預警"
            description="當產品庫存低於安全存量時發送通知"
            checked={settings.email_low_stock}
            onChange={(v) => onUpdate('email_low_stock', v)}
          />
          {settings.email_low_stock && (
            <div className="ml-6">
              <NotificationCheckbox
                id="low_stock_notify_immediately"
                label="即時通知"
                description="立即發送通知而非每日彙整"
                checked={settings.low_stock_notify_immediately}
                onChange={(v) => onUpdate('low_stock_notify_immediately', v)}
                small
              />
            </div>
          )}
          <NotificationCheckbox
            id="email_expiry_warning"
            label="效期預警"
            description="當產品即將到期時發送通知"
            checked={settings.email_expiry_warning}
            onChange={(v) => onUpdate('email_expiry_warning', v)}
          />
        </NotificationGroup>

        <NotificationGroup title="單據審核">
          <NotificationCheckbox
            id="email_document_approval"
            label="單據審核通知"
            description="當單據需要審核或審核完成時通知"
            checked={settings.email_document_approval}
            onChange={(v) => onUpdate('email_document_approval', v)}
          />
        </NotificationGroup>

        <NotificationGroup title="AUP">
          <NotificationCheckbox
            id="email_protocol_status"
            label="計畫狀態變更"
            description="當計畫審查狀態變更時發送通知"
            checked={settings.email_protocol_status}
            onChange={(v) => onUpdate('email_protocol_status', v)}
          />
        </NotificationGroup>

        <NotificationGroup title="報表">
          <NotificationCheckbox
            id="email_monthly_report"
            label="月報通知"
            description="每月自動發送庫存與成本彙整報表"
            checked={settings.email_monthly_report}
            onChange={(v) => onUpdate('email_monthly_report', v)}
          />
        </NotificationGroup>
      </CardContent>
    </Card>
  )
}

function WarningParametersCard({ settings, onUpdate }: NotificationCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertCircle className="h-5 w-5" />
          預警參數
        </CardTitle>
        <CardDescription>設定預警通知的觸發條件</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <Slider
            label="效期預警天數"
            value={settings.expiry_warning_days}
            onChange={(value) => onUpdate('expiry_warning_days', value)}
            min={1}
            max={90}
            step={1}
            quickValues={[7, 14, 30, 60]}
            unit="天"
            className="w-full"
          />
          <p className="text-xs text-muted-foreground">
            產品在到期前 {settings.expiry_warning_days} 天會發送預警通知
          </p>
        </div>

        <div className="rounded-lg bg-muted p-4 space-y-3">
          <h4 className="text-sm font-medium">通知時間說明</h4>
          <ul className="text-xs text-muted-foreground space-y-2">
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>低庫存檢查：每日上午 8:00 執行</span>
            </li>
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>效期檢查：每日上午 8:00 執行</span>
            </li>
            <li className="flex items-start gap-2">
              <CheckCircle2 className="h-4 w-4 text-status-success-text shrink-0 mt-0.5" />
              <span>過期通知清理：每週日凌晨 3:00 執行</span>
            </li>
          </ul>
        </div>

        <ActiveNotificationsBadges settings={settings} />
      </CardContent>
    </Card>
  )
}

function ActiveNotificationsBadges({ settings }: { settings: NotificationSettings }) {
  const badges = [
    { key: 'email_low_stock', label: '低庫存', color: 'blue' },
    { key: 'email_expiry_warning', label: '效期預警', color: 'orange' },
    { key: 'email_document_approval', label: '單據審核', color: 'green' },
    { key: 'email_protocol_status', label: '計畫狀態', color: 'purple' },
    { key: 'email_monthly_report', label: '月報', color: 'gray' },
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
      <h4 className="text-sm font-medium mb-3">目前啟用的通知</h4>
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
          <span className="text-xs text-muted-foreground">尚未啟用任何通知</span>
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
