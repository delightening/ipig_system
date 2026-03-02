import { useState, useEffect, useRef } from 'react'
import { useToggle } from '@/hooks/useToggle'
import { useSettingsForm } from './hooks/useSettingsForm'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { Slider } from '@/components/ui/slider'
import {
  Save,
  Building,
  Mail,
  Database,
  Shield,
  Bell,
  Loader2,
  AlertCircle,
  CheckCircle2,
  Eye,
  EyeOff,
  Download,
  Upload,
} from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/types/error'
import { NotificationRoutingSection } from '@/components/admin/NotificationRoutingSection'
import type { Warehouse } from '@/types/erp'

interface NotificationSettings {
  user_id: string
  email_low_stock: boolean
  email_expiry_warning: boolean
  email_document_approval: boolean
  email_protocol_status: boolean
  email_monthly_report: boolean
  expiry_warning_days: number
  low_stock_notify_immediately: boolean
  updated_at: string
}

interface UpdateNotificationSettingsRequest {
  email_low_stock?: boolean
  email_expiry_warning?: boolean
  email_document_approval?: boolean
  email_protocol_status?: boolean
  email_monthly_report?: boolean
  expiry_warning_days?: number
  low_stock_notify_immediately?: boolean
}

type SystemSettings = Record<string, string>

const SMTP_MASK = '********'

export function SettingsPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()

  const [notificationSettings, setNotificationSettings] = useState<NotificationSettings | null>(null)
  const [showPassword, togglePassword] = useToggle()

  // --- 系統設定 API ---
  const { data: sysSettings, isLoading: isLoadingSys, error: sysError } = useQuery({
    queryKey: ['system-settings'],
    queryFn: async () => {
      const res = await api.get<SystemSettings>('/admin/system-settings')
      return res.data
    },
    staleTime: 1_800_000,
  })

  // 倉庫列表
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

  // 全庫 IDXF 匯出/匯入
  const [exporting, setExporting] = useState(false)
  const [importing, setImporting] = useState(false)
  const [includeAudit, setIncludeAudit] = useState(false)
  const [exportAsZip, setExportAsZip] = useState(false)
  const canExport = hasPermission('admin.data.export')
  const canImport = hasPermission('admin.data.import')
  const handleFullExport = async () => {
    if (!canExport) return
    setExporting(true)
    try {
      const res = await api.get<Blob>('/admin/data-export', {
        params: { include_audit: includeAudit, format: exportAsZip ? 'zip' : 'json' },
        responseType: 'blob',
      })
      const blob = new Blob([res.data], {
        type: exportAsZip ? 'application/zip' : 'application/json;charset=utf-8',
      })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      const cd = res.headers['content-disposition']
      const match = cd && typeof cd === 'string' && cd.match(/filename="?([^"]+)"?/)
      a.download = match ? match[1] : `ipig_export_${new Date().toISOString().slice(0, 19).replace(/[-:]/g, '').replace('T', '_')}.${exportAsZip ? 'zip' : 'json'}`
      a.click()
      URL.revokeObjectURL(url)
      toast({ title: '匯出成功', description: '全庫資料已下載' })
    } catch (err) {
      toast({
        title: '匯出失敗',
        description: getErrorMessage(err) || '請稍後再試',
        variant: 'destructive',
      })
    } finally {
      setExporting(false)
    }
  }

  const fileInputRef = useRef<HTMLInputElement>(null)
  const [importResultOpen, setImportResultOpen] = useState(false)
  const [lastImportResult, setLastImportResult] = useState<{
    errors: string[]
    skipped_details: { table: string; reason: string; count?: number }[]
  } | null>(null)

  const handleFullImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (!canImport || !e.target.files?.[0]) return
    const file = e.target.files[0]
    if (!file.name.endsWith('.json') && !file.name.endsWith('.zip')) {
      toast({ title: '請選擇 .json 或 .zip 檔', variant: 'destructive' })
      e.target.value = ''
      return
    }
    setImporting(true)
    try {
      const fd = new FormData()
      fd.append('file', file)
      const res = await api.post<{
        tables_processed: number
        rows_inserted: number
        rows_skipped: number
        errors: string[]
        skipped_details: { table: string; reason: string; count?: number }[]
      }>('/admin/data-import', fd)
      const d = res.data
      const msg =
        d.errors.length > 0
          ? `${d.tables_processed} 表處理，${d.rows_inserted} 筆新增，${d.rows_skipped} 筆略過；${d.errors.length} 個錯誤`
          : `${d.tables_processed} 表處理，${d.rows_inserted} 筆新增，${d.rows_skipped} 筆略過`
      toast({ title: '匯入完成', description: msg })
      const hasDetails = d.errors.length > 0 || (d.skipped_details?.length ?? 0) > 0
      if (hasDetails) {
        setLastImportResult({
          errors: d.errors,
          skipped_details: d.skipped_details ?? [],
        })
        setImportResultOpen(true)
      }
      queryClient.invalidateQueries()
      // 強制重新載入設定相關資料（匯入可能更新 system_settings、notification_settings 等）
      await queryClient.refetchQueries({ queryKey: ['system-settings'] })
      await queryClient.refetchQueries({ queryKey: ['notification-settings'] })
      await queryClient.refetchQueries({ queryKey: ['warehouses-list'] })
      if (fileInputRef.current) fileInputRef.current.value = ''
    } catch (err) {
      toast({
        title: '匯入失敗',
        description: getErrorMessage(err) || '請稍後再試',
        variant: 'destructive',
      })
    } finally {
      setImporting(false)
      e.target.value = ''
    }
  }

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
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '儲存失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSaveSysSettings = () => {
    saveSysMutation.mutate(settingsForm.buildPayload(SMTP_MASK))
  }

  // --- 通知設定 API ---
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
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '儲存失敗',
        variant: 'destructive',
      })
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

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">系統設定</h1>
        <p className="text-muted-foreground">管理系統的全域設定參數</p>
      </div>

      {/* Loading / Error for system settings */}
      {isLoadingSys && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      )}

      {sysError && (
        <Card className="border-red-200 bg-red-50">
          <CardContent className="flex items-center gap-3 py-6">
            <AlertCircle className="h-5 w-5 text-red-500" />
            <span className="text-red-700">無法載入系統設定，請確認您有管理員權限</span>
          </CardContent>
        </Card>
      )}

      {!isLoadingSys && !sysError && (
        <>
          <div className="grid gap-6 md:grid-cols-2">
            {/* 基本設定 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Building className="h-5 w-5" />
                  基本設定
                </CardTitle>
                <CardDescription>設定系統基本資訊</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="companyName">系統名稱</Label>
                  <Input
                    id="companyName"
                    value={settingsForm.form.companyName}
                    onChange={(e) => settingsForm.updateField('companyName', e.target.value)}
                    placeholder="輸入系統名稱"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="defaultWarehouse">預設倉庫</Label>
                  <Select value={settingsForm.form.defaultWarehouseId || undefined} onValueChange={(v) => settingsForm.updateField('defaultWarehouseId', v)}>
                    <SelectTrigger>
                      <SelectValue placeholder="選擇預設倉庫" />
                    </SelectTrigger>
                    <SelectContent>
                      {warehouses.map(w => (
                        <SelectItem key={w.id} value={w.id}>{w.name} ({w.code})</SelectItem>
                      ))}
                      {warehouses.length === 0 && (
                        <SelectItem value="__none__" disabled>尚無可用倉庫</SelectItem>
                      )}
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* 庫存設定 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Database className="h-5 w-5" />
                  庫存設定
                </CardTitle>
                <CardDescription>設定庫存計算方式</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="costMethod">成本計算方式</Label>
                  <Select value={settingsForm.form.costMethod} onValueChange={(v) => settingsForm.updateField('costMethod', v)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="weighted_average">加權平均法</SelectItem>
                      <SelectItem value="moving_average">移動平均法</SelectItem>
                      <SelectItem value="fifo" disabled>先進先出 (v0.2)</SelectItem>
                      <SelectItem value="lifo" disabled>後進先出 (v0.2)</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    目前版本支援加權平均法和移動平均法
                  </p>
                </div>
              </CardContent>
            </Card>

            {/* 郵件設定 */}
            <Card className="md:col-span-2">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Mail className="h-5 w-5" />
                  郵件設定
                </CardTitle>
                <CardDescription>設定 SMTP 郵件伺服器，修改後即時生效</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                  <div className="space-y-2">
                    <Label htmlFor="emailHost">SMTP 伺服器</Label>
                    <Input
                      id="emailHost"
                      value={settingsForm.form.emailHost}
                      onChange={(e) => settingsForm.updateField('emailHost', e.target.value)}
                      placeholder="smtp.example.com"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emailPort">連接埠</Label>
                    <Input
                      id="emailPort"
                      value={settingsForm.form.emailPort}
                      onChange={(e) => settingsForm.updateField('emailPort', e.target.value)}
                      placeholder="587"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emailUser">SMTP 帳號</Label>
                    <Input
                      id="emailUser"
                      value={settingsForm.form.emailUser}
                      onChange={(e) => settingsForm.updateField('emailUser', e.target.value)}
                      placeholder="user@example.com"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emailPassword">SMTP 密碼</Label>
                    <form
                      onSubmit={(e) => e.preventDefault()}
                      className="relative"
                      autoComplete="off"
                    >
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
                        placeholder="輸入 SMTP 密碼"
                        autoComplete="off"
                      />
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
                        onClick={togglePassword}
                      >
                        {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                      </Button>
                    </form>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emailFromEmail">寄件人 Email</Label>
                    <Input
                      id="emailFromEmail"
                      value={settingsForm.form.emailFromEmail}
                      onChange={(e) => settingsForm.updateField('emailFromEmail', e.target.value)}
                      placeholder="noreply@example.com"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="emailFromName">寄件人名稱</Label>
                    <Input
                      id="emailFromName"
                      value={settingsForm.form.emailFromName}
                      onChange={(e) => settingsForm.updateField('emailFromName', e.target.value)}
                      placeholder="iPig System"
                    />
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 安全設定 */}
            <Card className="md:col-span-2">
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  安全設定
                </CardTitle>
                <CardDescription>設定系統安全相關參數</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2 max-w-xs">
                  <Label htmlFor="sessionTimeout">Session 逾時</Label>
                  <Select value={settingsForm.form.sessionTimeout} onValueChange={(v) => settingsForm.updateField('sessionTimeout', v)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="15">15 分鐘</SelectItem>
                      <SelectItem value="30">30 分鐘</SelectItem>
                      <SelectItem value="60">60 分鐘（1 小時）</SelectItem>
                      <SelectItem value="120">120 分鐘（2 小時）</SelectItem>
                      <SelectItem value="360">360 分鐘（6 小時）</SelectItem>
                      <SelectItem value="480">480 分鐘（8 小時）</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    使用者閒置超過此時間後需重新登入（需重啟後端服務才生效）
                  </p>
                </div>
              </CardContent>
            </Card>

            {/* 全庫匯出/匯入（僅具權限者可見） */}
            {(canExport || canImport) && (
              <Card className="md:col-span-2">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Download className="h-5 w-5" />
                    全庫資料匯出 / 匯入
                  </CardTitle>
                  <CardDescription>
                    IDXF JSON 格式，含 AUP、動物、倉庫進銷存、使用者、訓練紀錄等，可在不同 migration 版本間讀取
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {canExport && (
                    <>
                      <div className="space-y-2">
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id="includeAudit"
                            checked={includeAudit}
                            onCheckedChange={(v) => setIncludeAudit(!!v)}
                          />
                          <Label htmlFor="includeAudit" className="cursor-pointer text-sm">
                            包含稽核大表（user_activity_logs、login_events）
                          </Label>
                        </div>
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id="exportAsZip"
                            checked={exportAsZip}
                            onCheckedChange={(v) => setExportAsZip(!!v)}
                          />
                          <Label htmlFor="exportAsZip" className="cursor-pointer text-sm">
                            輸出為 Zip 分包（大表以 NDJSON 儲存，建議資料量大時使用）
                          </Label>
                        </div>
                      </div>
                      <Button
                        variant="outline"
                        onClick={handleFullExport}
                        disabled={exporting}
                      >
                        {exporting ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <Download className="mr-2 h-4 w-4" />
                        )}
                        一鍵匯出全庫
                      </Button>
                    </>
                  )}
                  {canImport && (
                    <div className="flex items-center gap-2 pt-2 border-t">
                        <input
                          ref={fileInputRef}
                          type="file"
                          accept=".json,.zip"
                          className="hidden"
                          aria-label="選擇 IDXF JSON 檔案"
                          onChange={handleFullImport}
                          disabled={importing}
                        />
                        <Button
                        variant="outline"
                        onClick={() => fileInputRef.current?.click()}
                        disabled={importing}
                      >
                        {importing ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : (
                          <Upload className="mr-2 h-4 w-4" />
                        )}
                        上傳 IDXF 匯入
                      </Button>
                      <span className="text-xs text-muted-foreground">
                        遇重複則取代；支援 JSON 或 Zip，最大 100 MB
                      </span>
                    </div>
                  )}
                </CardContent>
              </Card>
            )}
          </div>

          {/* 儲存系統設定按鈕 */}
          <div className="flex justify-end">
            <Button
              onClick={handleSaveSysSettings}
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

      {/* 通知偏好設定 */}
      <div className="border-t pt-6">
        <h2 className="text-2xl font-bold tracking-tight mb-4">通知偏好設定</h2>
        <p className="text-muted-foreground mb-6">設定您希望接收的通知類型</p>

        {isLoadingSettings ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : settingsError ? (
          <Card className="border-red-200 bg-red-50">
            <CardContent className="flex items-center gap-3 py-6">
              <AlertCircle className="h-5 w-5 text-red-500" />
              <span className="text-red-700">無法載入通知設定，請重新整理頁面</span>
            </CardContent>
          </Card>
        ) : notificationSettings ? (
          <div className="grid gap-6 md:grid-cols-2">
            {/* Email 通知設定 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Bell className="h-5 w-5" />
                  Email 通知
                </CardTitle>
                <CardDescription>設定哪些事件要發送 Email 通知</CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* 庫存相關 */}
                <div className="space-y-4">
                  <h4 className="text-sm font-medium text-muted-foreground">庫存管理</h4>

                  <div className="flex items-start space-x-3">
                    <Checkbox
                      id="email_low_stock"
                      checked={notificationSettings.email_low_stock}
                      onCheckedChange={(checked) =>
                        updateNotificationSetting('email_low_stock', checked as boolean)
                      }
                    />
                    <div className="grid gap-1.5 leading-none">
                      <Label htmlFor="email_low_stock" className="cursor-pointer">
                        低庫存預警
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        當產品庫存低於安全存量時發送通知
                      </p>
                    </div>
                  </div>

                  {notificationSettings.email_low_stock && (
                    <div className="ml-6 flex items-start space-x-3">
                      <Checkbox
                        id="low_stock_notify_immediately"
                        checked={notificationSettings.low_stock_notify_immediately}
                        onCheckedChange={(checked) =>
                          updateNotificationSetting('low_stock_notify_immediately', checked as boolean)
                        }
                      />
                      <div className="grid gap-1.5 leading-none">
                        <Label htmlFor="low_stock_notify_immediately" className="cursor-pointer text-sm">
                          即時通知
                        </Label>
                        <p className="text-xs text-muted-foreground">
                          立即發送通知而非每日彙整
                        </p>
                      </div>
                    </div>
                  )}

                  <div className="flex items-start space-x-3">
                    <Checkbox
                      id="email_expiry_warning"
                      checked={notificationSettings.email_expiry_warning}
                      onCheckedChange={(checked) =>
                        updateNotificationSetting('email_expiry_warning', checked as boolean)
                      }
                    />
                    <div className="grid gap-1.5 leading-none">
                      <Label htmlFor="email_expiry_warning" className="cursor-pointer">
                        效期預警
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        當產品即將到期時發送通知
                      </p>
                    </div>
                  </div>
                </div>

                {/* 單據相關 */}
                <div className="space-y-4">
                  <h4 className="text-sm font-medium text-muted-foreground">單據審核</h4>
                  <div className="flex items-start space-x-3">
                    <Checkbox
                      id="email_document_approval"
                      checked={notificationSettings.email_document_approval}
                      onCheckedChange={(checked) =>
                        updateNotificationSetting('email_document_approval', checked as boolean)
                      }
                    />
                    <div className="grid gap-1.5 leading-none">
                      <Label htmlFor="email_document_approval" className="cursor-pointer">
                        單據審核通知
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        當單據需要審核或審核完成時通知
                      </p>
                    </div>
                  </div>
                </div>

                {/* AUP 計畫相關 */}
                <div className="space-y-4">
                  <h4 className="text-sm font-medium text-muted-foreground">AUP</h4>
                  <div className="flex items-start space-x-3">
                    <Checkbox
                      id="email_protocol_status"
                      checked={notificationSettings.email_protocol_status}
                      onCheckedChange={(checked) =>
                        updateNotificationSetting('email_protocol_status', checked as boolean)
                      }
                    />
                    <div className="grid gap-1.5 leading-none">
                      <Label htmlFor="email_protocol_status" className="cursor-pointer">
                        計畫狀態變更
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        當計畫審查狀態變更時發送通知
                      </p>
                    </div>
                  </div>
                </div>

                {/* 報表相關 */}
                <div className="space-y-4">
                  <h4 className="text-sm font-medium text-muted-foreground">報表</h4>
                  <div className="flex items-start space-x-3">
                    <Checkbox
                      id="email_monthly_report"
                      checked={notificationSettings.email_monthly_report}
                      onCheckedChange={(checked) =>
                        updateNotificationSetting('email_monthly_report', checked as boolean)
                      }
                    />
                    <div className="grid gap-1.5 leading-none">
                      <Label htmlFor="email_monthly_report" className="cursor-pointer">
                        月報通知
                      </Label>
                      <p className="text-xs text-muted-foreground">
                        每月自動發送庫存與成本彙整報表
                      </p>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 通知參數設定 */}
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
                    value={notificationSettings.expiry_warning_days}
                    onChange={(value) =>
                      updateNotificationSetting('expiry_warning_days', value)
                    }
                    min={1}
                    max={90}
                    step={1}
                    quickValues={[7, 14, 30, 60]}
                    unit="天"
                    className="w-full"
                  />
                  <p className="text-xs text-muted-foreground">
                    產品在到期前 {notificationSettings.expiry_warning_days} 天會發送預警通知
                  </p>
                </div>

                <div className="rounded-lg bg-slate-50 p-4 space-y-3">
                  <h4 className="text-sm font-medium">通知時間說明</h4>
                  <ul className="text-xs text-muted-foreground space-y-2">
                    <li className="flex items-start gap-2">
                      <CheckCircle2 className="h-4 w-4 text-green-500 shrink-0 mt-0.5" />
                      <span>低庫存檢查：每日上午 8:00 執行</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <CheckCircle2 className="h-4 w-4 text-green-500 shrink-0 mt-0.5" />
                      <span>效期檢查：每日上午 8:00 執行</span>
                    </li>
                    <li className="flex items-start gap-2">
                      <CheckCircle2 className="h-4 w-4 text-green-500 shrink-0 mt-0.5" />
                      <span>過期通知清理：每週日凌晨 3:00 執行</span>
                    </li>
                  </ul>
                </div>

                <div className="rounded-lg border p-4">
                  <h4 className="text-sm font-medium mb-3">目前啟用的通知</h4>
                  <div className="flex flex-wrap gap-2">
                    {notificationSettings.email_low_stock && (
                      <span className="inline-flex items-center rounded-full bg-blue-50 px-2.5 py-0.5 text-xs font-medium text-blue-700">
                        低庫存
                      </span>
                    )}
                    {notificationSettings.email_expiry_warning && (
                      <span className="inline-flex items-center rounded-full bg-orange-50 px-2.5 py-0.5 text-xs font-medium text-orange-700">
                        效期預警
                      </span>
                    )}
                    {notificationSettings.email_document_approval && (
                      <span className="inline-flex items-center rounded-full bg-green-50 px-2.5 py-0.5 text-xs font-medium text-green-700">
                        單據審核
                      </span>
                    )}
                    {notificationSettings.email_protocol_status && (
                      <span className="inline-flex items-center rounded-full bg-purple-50 px-2.5 py-0.5 text-xs font-medium text-purple-700">
                        計畫狀態
                      </span>
                    )}
                    {notificationSettings.email_monthly_report && (
                      <span className="inline-flex items-center rounded-full bg-gray-100 px-2.5 py-0.5 text-xs font-medium text-gray-700">
                        月報
                      </span>
                    )}
                    {!notificationSettings.email_low_stock &&
                      !notificationSettings.email_expiry_warning &&
                      !notificationSettings.email_document_approval &&
                      !notificationSettings.email_protocol_status &&
                      !notificationSettings.email_monthly_report && (
                        <span className="text-xs text-muted-foreground">
                          尚未啟用任何通知
                        </span>
                      )}
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        ) : null}

        {notificationSettings && (
          <div className="flex justify-end mt-6">
            <Button
              onClick={handleSaveNotificationSettings}
              disabled={updateSettingsMutation.isPending}
            >
              {updateSettingsMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Save className="mr-2 h-4 w-4" />
              )}
              儲存通知設定
            </Button>
          </div>
        )}
      </div>

      {/* 通知路由管理 */}
      <NotificationRoutingSection />

      {/* 匯入結果詳情 Dialog（錯誤與略過項目） */}
      <Dialog open={importResultOpen} onOpenChange={setImportResultOpen}>
        <DialogContent className="sm:max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-amber-600">
              <AlertCircle className="h-5 w-5" />
              匯入結果詳情
            </DialogTitle>
            <DialogDescription>
              {lastImportResult?.errors.length
                ? `以下 ${lastImportResult.errors.length} 個資料表匯入時發生錯誤。`
                : ''}
              {lastImportResult?.skipped_details?.length
                ? `略過項目：${lastImportResult.skipped_details.length} 項。`
                : ''}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-y-auto flex-1 min-h-0 space-y-4">
            {lastImportResult?.errors.length ? (
              <div>
                <h4 className="font-medium text-destructive mb-2">錯誤</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.errors.map((err, i) => (
                    <li key={i} className="text-destructive/90 break-words">
                      {i + 1}. {err}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
            {lastImportResult?.skipped_details?.length ? (
              <div>
                <h4 className="font-medium text-amber-600 mb-2">略過項目</h4>
                <ul className="space-y-1.5 rounded border bg-muted/30 p-3 font-mono text-sm">
                  {lastImportResult.skipped_details.map((s, i) => (
                    <li key={i} className="break-words">
                      <span className="font-medium">{s.table}</span>：{s.reason}
                      {s.count != null ? `（${s.count} 筆）` : ''}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}

/** JSON value may come as `"hello"` (already a string) or as a raw string in JSONB */
function unwrap(val: unknown): string {
  if (val === null || val === undefined) return ''
  if (typeof val === 'string') return val
  if (typeof val === 'number' || typeof val === 'boolean') return String(val)
  return ''
}
