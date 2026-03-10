import { ArrowLeft, Sparkles } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { StepIndicator } from '@/components/product/StepIndicator'
import { SkuPreviewBlock } from '@/components/sku/SkuPreviewBlock'
import { cn } from '@/lib/utils'
import { STEPS } from './constants'
import { useCreateProductForm } from './hooks/useCreateProductForm'
import { StepQuickInput } from './components/StepQuickInput'
import { StepConfirmDetails } from './components/StepConfirmDetails'
import { StepSuccess } from './components/StepSuccess'

export function CreateProductPage() {
  const form = useCreateProductForm()

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-blue-50/50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-800">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => form.navigate('/products')}
            disabled={form.isCreating}
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div className="flex-1">
            <h1 className="text-2xl sm:text-3xl font-bold tracking-tight text-slate-900 dark:text-slate-100">
              新增產品
            </h1>
            <p className="text-slate-500 dark:text-slate-400 text-sm">
              SKU 由系統自動產生
            </p>
          </div>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={form.quickMode}
              onChange={(e) => form.setQuickMode(e.target.checked)}
              className="rounded"
            />
            <span className="text-slate-600 dark:text-slate-400 hidden sm:inline">快速模式</span>
            <Sparkles className={cn("w-4 h-4", form.quickMode ? "text-amber-500" : "text-slate-400")} />
          </label>
        </div>

        {/* Step Indicator */}
        <div className="mb-8">
          <StepIndicator
            steps={STEPS}
            currentStep={form.currentStep}
            completedSteps={form.isCreated ? [0, 1, 2] : form.currentStep > 0 ? [0] : []}
          />
        </div>

        {/* Main Content */}
        <div className="flex flex-col gap-8">
          <div className="w-full">
            {form.currentStep === 0 && <StepQuickInput form={form} />}
            {form.currentStep === 1 && <StepConfirmDetails form={form} />}
            {form.currentStep === 2 && <StepSuccess form={form} />}
          </div>

          {/* SKU Preview */}
          <div className="w-full">
            <SkuPreviewBlock
              status={form.skuStatus}
              previewResult={form.previewResult}
              error={form.previewError}
              missingFields={form.currentStep === 1 ? form.missingFields : []}
              finalSku={form.finalSku}
              isLoading={form.isPreviewLoading}
              onRefresh={form.generatePreview}
              compact={form.currentStep === 2}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
