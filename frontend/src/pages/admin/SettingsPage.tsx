import { useState, useEffect } from 'react'
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
} from 'lucide-react'
import api from '@/lib/api'

// 通知設定型別
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

export function SettingsPage() {
  const queryClient = useQueryClient()
  
  // 系統設定
  const [companyName, setCompanyName] = useState('進銷存管理系統')
  const [defaultWarehouse, setDefaultWarehouse] = useState('')
  const [emailHost, setEmailHost] = useState('')
  const [emailPort, setEmailPort] = useState('587')
  const [emailUser, setEmailUser] = useState('')
  const [sessionTimeout, setSessionTimeout] = useState('30')
  const [costMethod, setCostMethod] = useState('weighted_average')

  // 通知設定
  const [notificationSettings, setNotificationSettings] = useState<NotificationSettings | null>(null)

  // 取得通知設定
  const { data: fetchedSettings, isLoading: isLoadingSettings, error: settingsError } = useQuery({
    queryKey: ['notification-settings'],
    queryFn: async () => {
      const res = await api.get<NotificationSettings>('/notifications/settings')
      return res.data
    },
  })

  // 當設定載入後更新本地狀態
  useEffect(() => {
    if (fetchedSettings) {
      setNotificationSettings(fetchedSettings)
    }
  }, [fetchedSettings])

  // 更新通知設定
  const updateSettingsMutation = useMutation({
    mutationFn: async (data: UpdateNotificationSettingsRequest) => {
      const res = await api.put<NotificationSettings>('/notifications/settings', data)
      return res.data
    },
    onSuccess: (data) => {
      setNotificationSettings(data)
      queryClient.invalidateQueries({ queryKey: ['notification-settings'] })
      toast({
        title: '成功',
        description: '通知設定已儲存',
      })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '儲存通知設定失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSave = () => {
    toast({
      title: '成功',
      description: '系統設定已儲存',
    })
  }

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
    setNotificationSettings({
      ...notificationSettings,
      [key]: value,
    })
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">系統設定</h1>
        <p className="text-muted-foreground">管理系統的全域設定參數</p>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        {/* 基本設定 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Building className="h-5 w-5" />
              基本設定
            </CardTitle>
            <CardDescription>設定公司基本資訊</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="companyName">公司名稱</Label>
              <Input
                id="companyName"
                value={companyName}
                onChange={(e) => setCompanyName(e.target.value)}
                placeholder="輸入公司名稱"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="defaultWarehouse">預設倉庫</Label>
              <Select value={defaultWarehouse} onValueChange={setDefaultWarehouse}>
                <SelectTrigger>
                  <SelectValue placeholder="選擇預設倉庫" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="main">主倉庫</SelectItem>
                  <SelectItem value="secondary">副倉庫</SelectItem>
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
              <Select value={costMethod} onValueChange={setCostMethod}>
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
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Mail className="h-5 w-5" />
              郵件設定
            </CardTitle>
            <CardDescription>設定 SMTP 郵件伺服器</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="emailHost">SMTP 伺服器</Label>
              <Input
                id="emailHost"
                value={emailHost}
                onChange={(e) => setEmailHost(e.target.value)}
                placeholder="smtp.example.com"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="emailPort">連接埠</Label>
                <Input
                  id="emailPort"
                  value={emailPort}
                  onChange={(e) => setEmailPort(e.target.value)}
                  placeholder="587"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="emailUser">帳號</Label>
                <Input
                  id="emailUser"
                  value={emailUser}
                  onChange={(e) => setEmailUser(e.target.value)}
                  placeholder="user@example.com"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* 安全設定 */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              安全設定
            </CardTitle>
            <CardDescription>設定系統安全相關參數</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="sessionTimeout">Session 逾時（分鐘）</Label>
              <Select value={sessionTimeout} onValueChange={setSessionTimeout}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="15">15 分鐘</SelectItem>
                  <SelectItem value="30">30 分鐘</SelectItem>
                  <SelectItem value="60">60 分鐘</SelectItem>
                  <SelectItem value="120">120 分鐘</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* 儲存系統設定按鈕 */}
      <div className="flex justify-end">
        <Button onClick={handleSave}>
          <Save className="mr-2 h-4 w-4" />
          儲存系統設定
        </Button>
      </div>

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
                  <h4 className="text-sm font-medium text-muted-foreground">AUP 審查系統</h4>
                  
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
                {/* 效期預警天數 */}
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

                {/* 預警說明 */}
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

                {/* 目前設定摘要 */}
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

        {/* 儲存通知設定按鈕 */}
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
    </div>
  )
}
