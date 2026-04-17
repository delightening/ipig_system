/**
 * Dashboard Widget 設定對話框
 */
import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Slider } from '@/components/ui/slider'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Loader2, Settings2 } from 'lucide-react'
import {
  WidgetLayoutItem,
  widgetNames,
  widgetDescriptions,
  widgetPermissions,
  widgetCategories,
  widgetCategoryNames,
} from '@/components/dashboard'

interface DashboardSettingsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentLayout: WidgetLayoutItem[]
  availableWidgets: WidgetLayoutItem[]
  onSave: (layout: WidgetLayoutItem[]) => void
  onReset: () => void
  isSaving: boolean
}

export function DashboardSettingsDialog({
  open,
  onOpenChange,
  currentLayout,
  availableWidgets,
  onSave,
  onReset,
  isSaving,
}: DashboardSettingsDialogProps) {
  const { t } = useTranslation()
  const [tempLayout, setTempLayout] = useState<WidgetLayoutItem[]>([])

  useEffect(() => {
    if (open) {
      setTempLayout([...currentLayout])
    }
  }, [open, currentLayout])

  const toggleWidgetVisibility = (widgetId: string) => {
    setTempLayout((prev) =>
      prev.map((w) =>
        w.i === widgetId ? { ...w, visible: !w.visible } : w
      )
    )
  }

  const changeWidgetOption = (widgetId: string, key: string, value: number) => {
    setTempLayout((prev) =>
      prev.map((w) =>
        w.i === widgetId ? { ...w, options: { ...w.options, [key]: value } } : w
      )
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings2 className="h-5 w-5" />
            {t('dashboard.settings.title')}
          </DialogTitle>
          <DialogDescription>
            {t('dashboard.settings.description')}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          {Object.entries(widgetCategoryNames).map(([categoryId, categoryName]) => {
            const categoryWidgets = tempLayout.filter(
              (w) => widgetCategories[w.i] === categoryId && !widgetPermissions[w.i] ||
                widgetCategories[w.i] === categoryId && availableWidgets.some(aw => aw.i === w.i)
            )
            if (categoryWidgets.length === 0) return null
            return (
              <div key={categoryId}>
                <h4 className="text-sm font-medium text-muted-foreground mb-2">
                  {categoryName.translate ? t(categoryName.label) : categoryName.label}
                </h4>
                <div className="space-y-2">
                  {categoryWidgets.map((widget) => {
                    const nameCfg = widgetNames[widget.i]
                    const descCfg = widgetDescriptions[widget.i]
                    return (
                      <div
                        key={widget.i}
                        className="p-3 border rounded-lg hover:bg-muted/50 space-y-3"
                      >
                        <div className="flex items-start gap-3">
                          <Checkbox
                            id={widget.i}
                            checked={widget.visible !== false}
                            onCheckedChange={() => toggleWidgetVisibility(widget.i)}
                          />
                          <label htmlFor={widget.i} className="flex-1 cursor-pointer">
                            <p className="text-sm font-medium">
                              {nameCfg.translate ? t(nameCfg.label) : nameCfg.label}
                            </p>
                            <p className="text-xs text-muted-foreground">
                              {descCfg.translate ? t(descCfg.label) : descCfg.label}
                            </p>
                          </label>
                        </div>
                        {widget.visible !== false && widget.i === 'weekly_trend' && (
                          <div className="flex items-center gap-2 ml-6">
                            <span className="text-xs text-muted-foreground">{t('dashboard.settings.days')}</span>
                            <Slider
                              value={widget.options?.days || 7}
                              min={3}
                              max={7}
                              step={1}
                              quickValues={[3, 5, 7]}
                              onChange={(value: number) => changeWidgetOption(widget.i, 'days', value)}
                              className="w-48"
                            />
                          </div>
                        )}
                      </div>
                    )
                  })}
                </div>
              </div>
            )
          })}
        </div>
        <DialogFooter className="flex justify-between sm:justify-between">
          <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={onReset}>
            重設為預設佈局
          </Button>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={() => onSave(tempLayout)} disabled={isSaving}>
              {isSaving && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
