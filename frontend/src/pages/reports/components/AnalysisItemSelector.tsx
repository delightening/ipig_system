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
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">選擇分析項目</CardTitle>
        <p className="text-sm text-muted-foreground font-normal mt-1">
          常用組合一鍵選取，或展開分類勾選；未選擇時顯示全部
        </p>
      </CardHeader>
      <CardContent className="space-y-4">
        {groupedOptions.length === 0 ? (
          <p className="text-sm text-muted-foreground py-2">
            尚無分類資料，請先設定血液檢查組合或執行篩選以產生項目
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
                全部清除
              </Button>
            </div>

            {/* Selected tags */}
            {selectedItems.length > 0 && (
              <div className="rounded-lg border bg-muted/50 p-3">
                <div className="text-xs font-medium text-muted-foreground mb-2">
                  已選 {selectedItems.length} 項
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
                        aria-label={`移除 ${item}`}
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
                    清除選擇
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
