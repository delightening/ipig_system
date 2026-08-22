import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation } from '@tanstack/react-query'
import { Building, Database, Mail, Shield, Eye, EyeOff, Send, Loader2, Bell } from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import api from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import type { Warehouse } from '@/types/erp'
import type { SystemSettingsFormData } from '../hooks/useSettingsForm'

const SMTP_MASK = '********'

interface SettingsForm {
  form: SystemSettingsFormData
  updateField: <K extends keyof SystemSettingsFormData>(key: K, value: SystemSettingsFormData[K]) => void
  passwordEdited: boolean
  setPasswordEdited: (v: boolean) => void
  dirty: boolean
}

interface SystemSettingsCardsProps {
  settingsForm: SettingsForm
  warehouses: Warehouse[]
  showPassword: boolean
  togglePassword: () => void
}

export function SystemSettingsCards({
  settingsForm,
  warehouses,
  showPassword,
  togglePassword,
}: SystemSettingsCardsProps) {
  const { t } = useTranslation()
  return (
    <>
      {/* Basic settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Building className="h-5 w-5" />
            {t('admin.systemSettingsCards.basicTitle')}
          </CardTitle>
          <CardDescription>{t('admin.systemSettingsCards.basicDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="companyName">{t('admin.systemSettingsCards.systemName')}</Label>
            <Input
              id="companyName"
              value={settingsForm.form.companyName}
              onChange={(e) => settingsForm.updateField('companyName', e.target.value)}
              placeholder={t('admin.systemSettingsCards.systemNamePlaceholder')}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="defaultWarehouse">{t('admin.systemSettingsCards.defaultWarehouse')}</Label>
            <Select
              value={settingsForm.form.defaultWarehouseId || undefined}
              onValueChange={(v) => settingsForm.updateField('defaultWarehouseId', v)}
            >
              <SelectTrigger id="defaultWarehouse">
                <SelectValue placeholder={t('admin.systemSettingsCards.defaultWarehousePlaceholder')} />
              </SelectTrigger>
              <SelectContent>
                {warehouses.map(w => (
                  <SelectItem key={w.id} value={w.id}>{w.name} ({w.code})</SelectItem>
                ))}
                {warehouses.length === 0 && (
                  <SelectItem value="__none__" disabled>{t('admin.systemSettingsCards.noWarehouses')}</SelectItem>
                )}
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {/* Security settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            {t('admin.systemSettingsCards.securityTitle')}
          </CardTitle>
          <CardDescription>{t('admin.systemSettingsCards.securityDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="sessionTimeout">{t('admin.systemSettingsCards.sessionTimeout')}</Label>
            <Select
              value={settingsForm.form.sessionTimeout}
              onValueChange={(v) => settingsForm.updateField('sessionTimeout', v)}
            >
              <SelectTrigger id="sessionTimeout">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="15">{t('admin.systemSettingsCards.minutes', { count: 15 })}</SelectItem>
                <SelectItem value="30">{t('admin.systemSettingsCards.minutes', { count: 30 })}</SelectItem>
                <SelectItem value="60">{t('admin.systemSettingsCards.minutesHours', { count: 60, hours: 1 })}</SelectItem>
                <SelectItem value="120">{t('admin.systemSettingsCards.minutesHours', { count: 120, hours: 2 })}</SelectItem>
                <SelectItem value="360">{t('admin.systemSettingsCards.minutesHours', { count: 360, hours: 6 })}</SelectItem>
                <SelectItem value="480">{t('admin.systemSettingsCards.minutesHours', { count: 480, hours: 8 })}</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {t('admin.systemSettingsCards.sessionTimeoutHint')}
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Email settings */}
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Mail className="h-5 w-5" />
            {t('admin.systemSettingsCards.emailTitle')}
          </CardTitle>
          <CardDescription>{t('admin.systemSettingsCards.emailDescription')}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <div className="space-y-2">
              <Label htmlFor="emailHost">{t('admin.systemSettingsCards.smtpHost')}</Label>
              <Input
                id="emailHost"
                value={settingsForm.form.emailHost}
                onChange={(e) => settingsForm.updateField('emailHost', e.target.value)}
                placeholder="smtp.example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="emailPort">{t('admin.systemSettingsCards.smtpPort')}</Label>
              <Input
                id="emailPort"
                type="number"
                min={1}
                max={65535}
                value={settingsForm.form.emailPort}
                onChange={(e) => settingsForm.updateField('emailPort', e.target.value)}
                placeholder="587"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="emailUser">{t('admin.systemSettingsCards.smtpUser')}</Label>
              <Input
                id="emailUser"
                value={settingsForm.form.emailUser}
                onChange={(e) => settingsForm.updateField('emailUser', e.target.value)}
                placeholder="user@example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="emailPassword">{t('admin.systemSettingsCards.smtpPassword')}</Label>
              <form onSubmit={(e) => e.preventDefault()} className="relative" autoComplete="off">
                <Input
                  id="emailPassword"
                  type={showPassword ? 'text' : 'password'}
                  value={settingsForm.form.emailPassword}
                  onFocus={() => {
                    if (!settingsForm.passwordEdited && settingsForm.form.emailPassword === SMTP_MASK) {
                      settingsForm.updateField('emailPassword', '')
                      settingsForm.setPasswordEdited(true)
                    }
                  }}
                  onChange={(e) => {
                    settingsForm.updateField('emailPassword', e.target.value)
                    settingsForm.setPasswordEdited(true)
                  }}
                  placeholder={t('admin.systemSettingsCards.smtpPasswordPlaceholder')}
                  autoComplete="off"
                />
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                  onClick={togglePassword}
                  aria-label={
                    showPassword
                      ? t('admin.systemSettingsCards.hidePassword')
                      : t('admin.systemSettingsCards.showPassword')
                  }
                  aria-pressed={showPassword}
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </Button>
              </form>
            </div>
            <div className="space-y-2">
              <Label htmlFor="emailFromEmail">{t('admin.systemSettingsCards.fromEmail')}</Label>
              <Input
                id="emailFromEmail"
                value={settingsForm.form.emailFromEmail}
                onChange={(e) => settingsForm.updateField('emailFromEmail', e.target.value)}
                placeholder="noreply@example.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="emailFromName">{t('admin.systemSettingsCards.fromName')}</Label>
              <Input
                id="emailFromName"
                value={settingsForm.form.emailFromName}
                onChange={(e) => settingsForm.updateField('emailFromName', e.target.value)}
                placeholder="iPig System"
              />
            </div>
          </div>
          <TestEmailSection formDirty={settingsForm.dirty} />
        </CardContent>
      </Card>

      {/* IACUC notification settings */}
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="h-5 w-5" />
            {t('admin.systemSettingsCards.iacucTitle')}
          </CardTitle>
          <CardDescription>
            {t('admin.systemSettingsCards.iacucDescription')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="iacucNotifyEmails">{t('admin.systemSettingsCards.notifyEmails')}</Label>
            <Input
              id="iacucNotifyEmails"
              value={settingsForm.form.iacucNotifyEmails}
              onChange={(e) => settingsForm.updateField('iacucNotifyEmails', e.target.value)}
              placeholder="secretary@example.com,chair@example.com"
            />
            <p className="text-xs text-muted-foreground">
              {t('admin.systemSettingsCards.notifyEmailsHint')}
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Inventory settings */}
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="h-5 w-5" />
            {t('admin.systemSettingsCards.inventoryTitle')}
          </CardTitle>
          <CardDescription>{t('admin.systemSettingsCards.inventoryDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2 max-w-xs">
            <Label htmlFor="costMethod">{t('admin.systemSettingsCards.costMethod')}</Label>
            <Select
              value={settingsForm.form.costMethod}
              onValueChange={(v) => settingsForm.updateField('costMethod', v)}
            >
              <SelectTrigger id="costMethod">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="weighted_average">{t('admin.systemSettingsCards.costWeightedAverage')}</SelectItem>
                <SelectItem value="moving_average">{t('admin.systemSettingsCards.costMovingAverage')}</SelectItem>
                <SelectItem value="fifo" disabled>{t('admin.systemSettingsCards.costFifo')}</SelectItem>
                <SelectItem value="lifo" disabled>{t('admin.systemSettingsCards.costLifo')}</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {t('admin.systemSettingsCards.costMethodHint')}
            </p>
          </div>
        </CardContent>
      </Card>
    </>
  )
}

function TestEmailSection({ formDirty }: { formDirty: boolean }) {
  const { t } = useTranslation()
  const [testEmail, setTestEmail] = useState('')

  const testEmailMutation = useMutation({
    mutationFn: async (toEmail: string) => {
      const res = await api.post<{ message: string }>('/admin/system-settings/test-email', {
        to_email: toEmail,
      })
      return res.data
    },
    onSuccess: (data) => {
      toast({ title: t('common.success'), description: data.message })
    },
    onError: (error: unknown) => {
      toast({
        title: t('admin.systemSettingsCards.sendFailed'),
        description: getErrorMessage(error) || t('admin.systemSettingsCards.testEmailFailed'),
        variant: 'destructive',
      })
    },
  })

  const handleSend = () => {
    const email = testEmail.trim()
    if (!email || !email.includes('@')) {
      toast({ title: t('common.error'), description: t('admin.systemSettingsCards.invalidRecipientEmail'), variant: 'destructive' })
      return
    }
    testEmailMutation.mutate(email)
  }

  const isDisabled = testEmailMutation.isPending || !testEmail.trim() || formDirty

  return (
    <div className="mt-6 border-t pt-6">
      <Label className="text-sm font-medium mb-3 block">{t('admin.systemSettingsCards.sendTestEmail')}</Label>
      <div className="flex gap-2 items-end">
        <div className="flex-1">
          <Input
            type="email"
            value={testEmail}
            onChange={(e) => setTestEmail(e.target.value)}
            placeholder={t('admin.systemSettingsCards.recipientEmailPlaceholder')}
            disabled={formDirty}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !isDisabled) handleSend()
            }}
          />
        </div>
        <Button
          type="button"
          variant="outline"
          onClick={handleSend}
          disabled={isDisabled}
        >
          {testEmailMutation.isPending ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Send className="mr-2 h-4 w-4" />
          )}
          {t('admin.systemSettingsCards.sendTest')}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground mt-2">
        {formDirty
          ? t('admin.systemSettingsCards.saveBeforeTest')
          : t('admin.systemSettingsCards.testUsesSavedSettings')}
      </p>
    </div>
  )
}
