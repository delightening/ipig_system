import { useTranslation } from 'react-i18next'
import { ArrowRight } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { SmartInput } from '@/components/product/SmartInput'
import { QuickSelectGrid, SpecSelectionPanel } from '@/components/product/QuickSelectCard'
import { getQuickItems, getGloveSpecs } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface StepQuickInputProps {
  form: CreateProductFormReturn
}

export function StepQuickInput({ form }: StepQuickInputProps) {
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
                value={form.formData.rawInput}
                onChange={form.handleInputChange}
                onSelect={form.handleSelectSuggestion}
                onCreateNew={() => form.setCurrentStep(1)}
                suggestions={form.suggestions}
                isLoading={form.isSuggestionsLoading}
                placeholder={t('erpMaster.createProduct.inputPlaceholder')}
              />
              <p className="text-xs text-muted-foreground">
                {t('erpMaster.createProduct.inputHint')}
              </p>
            </div>

            <div className="border-t pt-6">
              <Label className="text-sm text-muted-foreground mb-3 block">
                {t('erpMaster.createProduct.quickSelectLabel')}
              </Label>
              <QuickSelectGrid
                items={getQuickItems(t)}
                selectedId={form.selectedQuickItem?.id}
                onSelect={form.handleQuickItemSelect}
                showMore
                onShowMore={() => { }}
              />
            </div>

            {/* Glove Spec Selection */}
            {form.selectedQuickItem?.id === 'glove' && (
              <div className="border-t pt-6">
                <SpecSelectionPanel
                  title={form.selectedQuickItem.label}
                  specs={getGloveSpecs(t)}
                  selectedId={form.selectedSpec?.id}
                  onSelect={form.handleSpecSelect}
                  extraOptions={[
                    {
                      label: t('erpMaster.createProduct.material.label'),
                      options: [
                        { value: 'NBR', label: t('erpMaster.createProduct.material.nbr') },
                        { value: 'LATEX', label: t('erpMaster.createProduct.material.latex') },
                        { value: 'PVC', label: 'PVC' },
                        { value: 'PE', label: 'PE' },
                      ],
                      value: form.glovesMaterial,
                      onChange: form.setGlovesMaterial,
                    },
                  ]}
                />
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-end">
        <Button
          onClick={form.handleNext}
          disabled={!form.formData.rawInput && !form.formData.name}
          size="lg"
        >
          {t('erpMaster.createProduct.next')}
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
