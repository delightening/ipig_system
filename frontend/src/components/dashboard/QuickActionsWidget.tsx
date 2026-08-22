import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Zap } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useAuthHasPermission } from '@/stores/auth'

import { QUICK_ACTION_CATALOG } from './widgetConfig'

/**
 * 快捷按鈕 widget：依使用者於設定中選用的 shortcuts（action id）渲染捷徑按鈕。
 * 依權限過濾（無權限的動作不顯示）；route 連到既有頁面。
 */
export function QuickActionsWidget({ shortcuts }: { shortcuts?: string[] }) {
  const { t } = useTranslation()
  const hasPermission = useAuthHasPermission()
  const ids = shortcuts ?? []
  const actions = QUICK_ACTION_CATALOG.filter(
    (a) => ids.includes(a.id) && (!a.permission || hasPermission(a.permission))
  )

  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader className="pt-3 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Zap className="h-4 w-4 text-primary" />
          {t('dashboard.quickActions.title')}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {actions.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {actions.map((a) => (
              <Button key={a.id} variant="outline" size="sm" asChild>
                <Link to={a.route}>{t(a.label)}</Link>
              </Button>
            ))}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">
            {t('dashboard.quickActions.empty')}
          </p>
        )}
      </CardContent>
    </Card>
  )
}
