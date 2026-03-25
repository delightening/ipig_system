import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useToggle } from '@/hooks/useToggle'

import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { toast } from '@/components/ui/use-toast'
import { Loader2, AlertCircle, Save } from 'lucide-react'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/types/error'
import type { Warehouse } from '@/types/erp'
import type { NotificationSettings, UpdateNotificationSettingsRequest } from '@/types/notification'
import { useSettingsForm } from './hooks/useSettingsForm'
import { SystemSettingsCards } from './components/SystemSettingsCards'
import { NotificationPreferencesSection } from './components/NotificationPreferencesSection'
import { DataExportImportCard } from './components/DataExportImportCard'
import { AiApiKeySection } from './components/AiApiKeySection'

type SystemSettings = Record<string, string>

const SMTP_MASK = '********'

export function SettingsPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const [showPassword, togglePassword] = useToggle()

  // --- System settings ---
  const { data: sysSettings, isLoading: isLoadingSys, error: sysError } = useQuery({
    queryKey: ['system-settings'],
    queryFn: async () => {
      const res = await api.get<SystemSettings>('/admin/system-settings')
      return res.data
    },
    staleTime: 1_800_000,
  })

  const { data: warehousesData } = useQuery({
    queryKey: ['warehouses-list'],
    queryFn: async () => {
      const res = await api.get<{ data: Warehouse[] } | Warehouse[]>('/warehouses')
      return Array.isArray(res.data) ? res.data : res.data.data
    },
    staleTime: 600_000,
  })
  const warehouses = warehousesData?.filter(w => w.is_active) ?? []

  const settingsForm = useSettingsForm(sysSettings)

  const saveSysMutation = useMutation({
    mutationFn: async (data: Record<string, string>) => {
      const res = await api.put<SystemSettings>('/admin/system-settings', data)
      return res.data
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['system-settings'] })
      settingsForm.clearDirty()
      toast({ title: '成功', description: '系統設定已儲存' })
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(error) || '儲存失敗', variant: 'destructive' })
    },
  })

  // --- Notification settings ---
  const [notificationSettings, setNotificationSettings] = useState<NotificationSettings | null>(null)

  const { data: fetchedSettings, isLoading: isLoadingSettings, error: settingsError } = useQuery({
    queryKey: ['notification-settings'],
    queryFn: async () => {
      const res = await api.get<NotificationSettings>('/notifications/settings')
      return res.data
    },
    staleTime: 1_800_000,
  })

  useEffect(() => {
    if (fetchedSettings) setNotificationSettings(fetchedSettings)
  }, [fetchedSettings])

  const updateSettingsMutation = useMutation({
    mutationFn: async (data: UpdateNotificationSettingsRequest) => {
      const res = await api.put<NotificationSettings>('/notifications/settings', data)
      return res.data
    },
    onSuccess: (data) => {
      setNotificationSettings(data)
      queryClient.invalidateQueries({ queryKey: ['notification-settings'] })
      toast({ title: '成功', description: '通知設定已儲存' })
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getErrorMessage(error) || '儲存失敗', variant: 'destructive' })
    },
  })

  const handleSaveNotificationSettings = () => {
    if (!notificationSettings) return
    updateSettingsMutation.mutate({
      email_low_stock: notificationSettings.email_low_stock,
      email_expiry_warning: notificationSettings.email_expiry_warning,
      email_document_approval: notificationSettings.email_document_approval,
      email_protocol_status: notificationSettings.email_protocol_status,
      email_monthly_report: notificationSettings.email_monthly_report,
      expiry_warning_days: notificationSettings.expiry_warning_days,
      low_stock_notify_immediately: notificationSettings.low_stock_notify_immediately,
    })
  }

  const updateNotificationSetting = <K extends keyof NotificationSettings>(
    key: K,
    value: NotificationSettings[K]
  ) => {
    if (!notificationSettings) return
    setNotificationSettings({ ...notificationSettings, [key]: value })
  }

  const canExport = hasPermission('admin.data.export')
  const canImport = hasPermission('admin.data.import')

  return (
    <div className="space-y-6">
      <PageHeader
        title="系統設定"
        description="管理系統的全域設定參數"
      />

      {isLoadingSys && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      )}

      {sysError && (
        <Card className="border-destructive bg-status-error-bg">
          <CardContent className="flex items-center gap-3 py-6">
            <AlertCircle className="h-5 w-5 text-destructive" />
            <span className="text-status-error-text">無法載入系統設定，請確認您有管理員權限</span>
          </CardContent>
        </Card>
      )}

      {!isLoadingSys && !sysError && (
        <>
          <div className="grid gap-6 md:grid-cols-2">
            <SystemSettingsCards
              settingsForm={settingsForm}
              warehouses={warehouses}
              showPassword={showPassword}
              togglePassword={togglePassword}
            />
            {(canExport || canImport) && (
              <DataExportImportCard canExport={canExport} canImport={canImport} />
            )}
          </div>

          <div className="flex justify-end">
            <Button
              onClick={() => saveSysMutation.mutate(settingsForm.buildPayload(SMTP_MASK))}
              disabled={saveSysMutation.isPending || !settingsForm.dirty}
            >
              {saveSysMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              儲存系統設定
            </Button>
          </div>
        </>
      )}

      <NotificationPreferencesSection
        settings={notificationSettings}
        isLoading={isLoadingSettings}
        error={settingsError}
        isSaving={updateSettingsMutation.isPending}
        onUpdate={updateNotificationSetting}
        onSave={handleSaveNotificationSettings}
      />

      <AiApiKeySection />
    </div>
  )
}
