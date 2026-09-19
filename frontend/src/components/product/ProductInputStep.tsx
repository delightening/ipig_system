import { useTranslation } from 'react-i18next'

import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { SmartInput, ProductSuggestion } from '@/components/product/SmartInput'
import { QuickSelectGrid, QuickSelectItem, SpecSelectionPanel, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { ArrowRight } from 'lucide-react'
import { getQuickItems, getGloveSpecs } from '@/components/product/createProductTypes'

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
  const { t } = useTranslation()

  return (
    <div className="space-y-6 animate-fade-in">
      <Card>
        <CardContent className="pt-6">
          <div className="space-y-6">
            {/* Smart Input */}
            <div className="space-y-3">
              <Label className="text-base">{t('erpMaster.createProduct.inputLabel')}</Label>
              <SmartInput
                value={rawInput}
                onChange={onInputChange}
                onSelect={onSelectSuggestion}
                onCreateNew={() => onCreateNew()}
                suggestions={suggestions}
                isLoading={isSuggestionsLoading}
                placeholder={t('erpMaster.createProduct.inputPlaceholder')}
              />
              <p className="text-xs text-muted-foreground">
                💡 {t('erpMaster.createProduct.inputHint')}
              </p>
            </div>

            <div className="border-t pt-6">
              <Label className="text-sm text-muted-foreground dark:text-muted-foreground mb-3 block">
                🏷️ {t('erpMaster.createProduct.quickSelectLabel')}
              </Label>
              <QuickSelectGrid
                items={getQuickItems(t)}
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
                  specs={getGloveSpecs(t)}
                  selectedId={selectedSpec?.id}
                  onSelect={onSpecSelect}
                  extraOptions={[
                    {
                      label: t('erpMaster.createProduct.material.label'),
                      options: [
                        { value: 'NBR', label: t('erpMaster.createProduct.material.nbr') },
                        { value: 'LATEX', label: t('erpMaster.createProduct.material.latex') },
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
          {t('erpMaster.createProduct.next')}
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
