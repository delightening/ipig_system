import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { CollapsibleSection } from '@/components/animal/SurgeryFormComponents'
import { PanelIcon } from '@/components/ui/panel-icon'

interface AnalysisItemSelectorProps {
  groupedOptions: { key: string; label: string; items: { name: string }[] }[]
  presetsData: { id: string; name: string; icon?: string; panel_keys?: string[] }[] | undefined
  selectedItems: string[]
  setSelectedItems: (items: string[]) => void
  applyPreset: (keys: string[]) => void
  toggleItem: (item: string) => void
}

export function AnalysisItemSelector({
  groupedOptions,
  presetsData,
  selectedItems,
  setSelectedItems,
  applyPreset,
  toggleItem,
}: AnalysisItemSelectorProps) {
  const { t } = useTranslation()
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t('reportsPages.bloodTestAnalysis.selector.title')}</CardTitle>
        <p className="text-sm text-muted-foreground font-normal mt-1">
          {t('reportsPages.bloodTestAnalysis.selector.hint')}
        </p>
      </CardHeader>
      <CardContent className="space-y-4">
        {groupedOptions.length === 0 ? (
          <p className="text-sm text-muted-foreground py-2">
            {t('reportsPages.bloodTestAnalysis.selector.noCategories')}
          </p>
        ) : (
          <>
            {/* Presets */}
            <div className="flex flex-wrap gap-2">
              {(presetsData ?? []).map((preset) => (
                <Button
                  key={preset.id}
                  variant="outline"
                  size="sm"
                  onClick={() => applyPreset(preset.panel_keys ?? [])}
                  className="text-xs"
                >
                  <PanelIcon icon={preset.icon} className="mr-1 text-xs" />
                  {preset.name}
                </Button>
              ))}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSelectedItems([])}
                className="text-xs text-muted-foreground"
              >
                {t('reportsPages.bloodTestAnalysis.selector.clearAll')}
              </Button>
            </div>

            {/* Selected tags */}
            {selectedItems.length > 0 && (
              <div className="rounded-lg border bg-muted/50 p-3">
                <div className="text-xs font-medium text-muted-foreground mb-2">
                  {t('reportsPages.bloodTestAnalysis.selector.selectedCount', { count: selectedItems.length })}
                </div>
                <div className="flex flex-wrap gap-2">
                  {selectedItems.map(item => (
                    <span
                      key={item}
                      className="inline-flex items-center gap-1 rounded-md bg-primary/15 px-2.5 py-1 text-sm"
                    >
                      {item}
                      <button
                        type="button"
                        className="ml-0.5 rounded hover:bg-primary/30 hover:text-destructive"
                        onClick={() => toggleItem(item)}
                        aria-label={t('common.removeItem', { name: item })}
                      >
                        ×
                      </button>
                    </span>
                  ))}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setSelectedItems([])}
                    className="text-xs h-7"
                  >
                    {t('reportsPages.bloodTestAnalysis.selector.clearSelection')}
                  </Button>
                </div>
              </div>
            )}

            {/* Category accordion */}
            <div className="space-y-2 max-h-[280px] overflow-y-auto pr-1">
              {groupedOptions.map(group => (
                <CollapsibleSection key={group.key} title={group.label} defaultOpen={false}>
                  <div className="flex flex-wrap gap-x-4 gap-y-2">
                    {group.items.map(({ name }) => (
                      <label key={name} className="flex items-center gap-2 cursor-pointer text-sm">
                        <Checkbox
                          checked={selectedItems.includes(name)}
                          onCheckedChange={checked =>
                            checked
                              ? setSelectedItems([...selectedItems, name])
                              : setSelectedItems(selectedItems.filter(i => i !== name))
                          }
                        />
                        <span>{name}</span>
                      </label>
                    ))}
                  </div>
                </CollapsibleSection>
              ))}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
