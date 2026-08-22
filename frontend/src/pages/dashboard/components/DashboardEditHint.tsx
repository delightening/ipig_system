import { useTranslation } from 'react-i18next'

/** 編輯模式下顯示的操作提示橫幅。 */
export function DashboardEditHint() {
  const { t } = useTranslation()
  return (
    <div className="p-3 bg-primary/5 border border-primary/20 rounded-lg text-sm text-primary">
      {t('dashboard.editMode.hint')}
    </div>
  )
}
