import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { SmartInput, ProductSuggestion } from '@/components/product/SmartInput'
import { QuickSelectGrid, QuickSelectItem, SpecSelectionPanel, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { ArrowRight } from 'lucide-react'
import { QUICK_ITEMS, GLOVE_SPECS } from '@/components/product/createProductTypes'

interface ProductInputStepProps {
  rawInput: string
  name: string
  suggestions: ProductSuggestion[]
  isSuggestionsLoading: boolean
  selectedQuickItem: QuickSelectItem | null
  selectedSpec: QuickSelectSpec | null
  glovesMaterial: string
  onInputChange: (value: string) => void
  onSelectSuggestion: (suggestion: ProductSuggestion) => void
  onCreateNew: () => void
  onQuickItemSelect: (item: QuickSelectItem) => void
  onSpecSelect: (spec: QuickSelectSpec) => void
  onGlovesMaterialChange: (material: string) => void
  onNext: () => void
}

export function ProductInputStep({
  rawInput,
  name,
  suggestions,
  isSuggestionsLoading,
  selectedQuickItem,
  selectedSpec,
  glovesMaterial,
  onInputChange,
  onSelectSuggestion,
  onCreateNew,
  onQuickItemSelect,
  onSpecSelect,
  onGlovesMaterialChange,
  onNext,
}: ProductInputStepProps) {
  return (
    <div className="space-y-6 animate-fade-in">
      <Card>
        <CardContent className="pt-6">
          <div className="space-y-6">
            {/* Smart Input */}
            <div className="space-y-3">
              <Label className="text-base">輸入產品名稱和規格</Label>
              <SmartInput
                value={rawInput}
                onChange={onInputChange}
                onSelect={onSelectSuggestion}
                onCreateNew={() => onCreateNew()}
                suggestions={suggestions}
                isLoading={isSuggestionsLoading}
                placeholder="例如：Amoxicillin 500mg tablet"
              />
              <p className="text-xs text-slate-500">
                💡 直接輸入「名稱 規格」，例如：手套 L號 無粉、生理食鹽水 500ml
              </p>
            </div>

            <div className="border-t pt-6">
              <Label className="text-sm text-slate-600 dark:text-slate-400 mb-3 block">
                🏷️ 快速選擇常用品項
              </Label>
              <QuickSelectGrid
                items={QUICK_ITEMS}
                selectedId={selectedQuickItem?.id}
                onSelect={onQuickItemSelect}
                showMore
                onShowMore={() => { }}
              />
            </div>

            {/* Spec Selection for Quick Item */}
            {selectedQuickItem?.id === 'glove' && (
              <div className="border-t pt-6">
                <SpecSelectionPanel
                  title={selectedQuickItem.label}
                  specs={GLOVE_SPECS}
                  selectedId={selectedSpec?.id}
                  onSelect={onSpecSelect}
                  extraOptions={[
                    {
                      label: '材質',
                      options: [
                        { value: 'NBR', label: 'NBR丁腈' },
                        { value: 'LATEX', label: '乳膠' },
                        { value: 'PVC', label: 'PVC' },
                        { value: 'PE', label: 'PE' },
                      ],
                      value: glovesMaterial,
                      onChange: onGlovesMaterialChange,
                    },
                  ]}
                />
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Action Buttons */}
      <div className="flex justify-end">
        <Button
          onClick={onNext}
          disabled={!rawInput && !name}
          size="lg"
        >
          下一步
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
