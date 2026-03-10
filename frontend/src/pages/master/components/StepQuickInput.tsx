import { ArrowRight } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { SmartInput } from '@/components/product/SmartInput'
import { QuickSelectGrid, SpecSelectionPanel } from '@/components/product/QuickSelectCard'
import { QUICK_ITEMS, GLOVE_SPECS } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface StepQuickInputProps {
  form: CreateProductFormReturn
}

export function StepQuickInput({ form }: StepQuickInputProps) {
  return (
    <div className="space-y-6 animate-fade-in">
      <Card>
        <CardContent className="pt-6">
          <div className="space-y-6">
            {/* Smart Input */}
            <div className="space-y-3">
              <Label className="text-base">輸入產品名稱和規格</Label>
              <SmartInput
                value={form.formData.rawInput}
                onChange={form.handleInputChange}
                onSelect={form.handleSelectSuggestion}
                onCreateNew={() => form.setCurrentStep(1)}
                suggestions={form.suggestions}
                isLoading={form.isSuggestionsLoading}
                placeholder="例如：Amoxicillin 500mg tablet"
              />
              <p className="text-xs text-slate-500">
                直接輸入「名稱 規格」，例如：手套 L號 無粉、生理食鹽水 500ml
              </p>
            </div>

            <div className="border-t pt-6">
              <Label className="text-sm text-slate-600 dark:text-slate-400 mb-3 block">
                快速選擇常用品項
              </Label>
              <QuickSelectGrid
                items={QUICK_ITEMS}
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
                  specs={GLOVE_SPECS}
                  selectedId={form.selectedSpec?.id}
                  onSelect={form.handleSpecSelect}
                  extraOptions={[
                    {
                      label: '材質',
                      options: [
                        { value: 'NBR', label: 'NBR丁腈' },
                        { value: 'LATEX', label: '乳膠' },
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
          下一步
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
