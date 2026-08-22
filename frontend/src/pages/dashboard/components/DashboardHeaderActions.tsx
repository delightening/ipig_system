import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Loader2, Settings2, Unlock, Save } from 'lucide-react'

interface DashboardHeaderActionsProps {
  isEditMode: boolean
  isSaving: boolean
  onSaveLayout: () => void
  onEnterEditMode: () => void
  onResetLayout: () => void
  onOpenSettings: () => void
}

/**
 * 儀表板頁首的操作按鈕群：編輯模式顯示「儲存佈局」，非編輯模式顯示
 * 「編輯佈局 / 重設佈局 / 自訂儀表板」。
 */
export function DashboardHeaderActions({
  isEditMode,
  isSaving,
  onSaveLayout,
  onEnterEditMode,
  onResetLayout,
  onOpenSettings,
}: DashboardHeaderActionsProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-wrap gap-2">
      {isEditMode ? (
        <Button size="sm" onClick={onSaveLayout} disabled={isSaving}>
          {isSaving && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
          <Save className="h-4 w-4 mr-1" />
          {t('dashboard.editMode.lock')}
        </Button>
      ) : (
        <>
          <Button variant="outline" size="sm" onClick={onEnterEditMode}>
            <Unlock className="h-4 w-4 mr-1" />
            {t('dashboard.editMode.unlock')}
          </Button>
          <Button variant="outline" size="sm" onClick={onResetLayout}>
            {t('dashboard.editMode.resetLayout')}
          </Button>
          <Button variant="outline" size="sm" onClick={onOpenSettings}>
            <Settings2 className="h-4 w-4 mr-1" />
            {t('dashboard.settings.title')}
          </Button>
        </>
      )}
    </div>
  )
}
