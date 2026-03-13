import { Building, Database, Mail, Shield, Eye, EyeOff } from 'lucide-react'

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
import type { Warehouse } from '@/types/erp'

const SMTP_MASK = '********'

interface SettingsForm {
  form: Record<string, string>
  updateField: (key: string, value: string) => void
  passwordEdited: boolean
  setPasswordEdited: (v: boolean) => void
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
  return (
    <>
      {/* Basic settings */}
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
            <Select
              value={settingsForm.form.defaultWarehouseId || undefined}
              onValueChange={(v) => settingsForm.updateField('defaultWarehouseId', v)}
            >
              <SelectTrigger id="defaultWarehouse">
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

      {/* Inventory settings */}
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
            <Select
              value={settingsForm.form.costMethod}
              onValueChange={(v) => settingsForm.updateField('costMethod', v)}
            >
              <SelectTrigger id="costMethod">
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

      {/* Email settings */}
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

      {/* Security settings */}
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
            <Select
              value={settingsForm.form.sessionTimeout}
              onValueChange={(v) => settingsForm.updateField('sessionTimeout', v)}
            >
              <SelectTrigger id="sessionTimeout">
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
    </>
  )
}
