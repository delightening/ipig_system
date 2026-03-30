import { useTranslation } from 'react-i18next'
import { GripVertical, RotateCcw, X } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface SidebarNavEditControlsProps {
  isEditMode: boolean
  setIsEditMode: (editMode: boolean) => void
  onResetNavOrder: () => void
  isResetting: boolean
}

export function SidebarNavEditControls({
  isEditMode,
  setIsEditMode,
  onResetNavOrder,
  isResetting,
}: SidebarNavEditControlsProps) {
  const { t } = useTranslation()

  if (isEditMode) {
    return (
      <div className="mt-4 pt-4 border-t border-slate-700 space-y-2">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setIsEditMode(false)}
          className="w-full text-muted-foreground hover:text-white hover:bg-slate-800 text-xs"
        >
          <X className="h-4 w-4 mr-1" />
          {t('common.finishEdit')}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={onResetNavOrder}
          disabled={isResetting}
          className="w-full text-muted-foreground hover:text-white hover:bg-slate-800 text-xs"
        >
          <RotateCcw className="h-4 w-4 mr-1" />
          {t('common.resetToDefault')}
        </Button>
      </div>
    )
  }

  return (
    <div className="mt-4 pt-4 border-t border-slate-700 space-y-2">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsEditMode(true)}
        className="w-full text-muted-foreground hover:text-white hover:bg-slate-800 text-xs"
      >
        <GripVertical className="h-4 w-4 mr-1" />
        {t('common.editMenuOrder')}
      </Button>
    </div>
  )
}
