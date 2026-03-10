import { Check, ListPlus, FileText, LayoutGrid } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { UNITS } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface StepSuccessProps {
  form: CreateProductFormReturn
}

export function StepSuccess({ form }: StepSuccessProps) {
  return (
    <div className="animate-fade-in">
      <Card className="overflow-hidden">
        <div className="bg-gradient-to-r from-success/10 via-success/5 to-transparent p-8 text-center">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-success/20 flex items-center justify-center animate-success-bounce">
            <Check className="w-8 h-8 text-success" />
          </div>
          <h2 className="text-2xl font-bold text-slate-900 dark:text-slate-100 mb-2">
            產品建立成功！
          </h2>
          <p className="text-slate-500">
            {form.formData.name} {form.formData.spec}
          </p>
        </div>

        <CardContent className="pt-6">
          <div className="space-y-4">
            <div className="p-4 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700">
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-slate-500">SKU</span>
                <span className="font-mono font-bold text-lg text-primary">{form.finalSku}</span>
              </div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-slate-500">分類</span>
                <span>{form.skuCategories.find(c => c.code === form.formData.category)?.name ?? '—'}</span>
              </div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-slate-500">單位</span>
                <span>{UNITS.base.find(u => u.code === form.formData.baseUnit)?.name || '—'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-slate-500">追蹤</span>
                <span>
                  {form.formData.trackBatch && '批號'} {form.formData.trackBatch && form.formData.trackExpiry && '/'} {form.formData.trackExpiry && '效期'}
                  {!form.formData.trackBatch && !form.formData.trackExpiry && '無'}
                </span>
              </div>
            </div>

            <p className="text-sm text-slate-600 dark:text-slate-400">
              接下來您可以：
            </p>

            <div className="grid grid-cols-3 gap-3">
              <Button variant="outline" className="flex-col h-auto py-4" onClick={form.handleReset}>
                <ListPlus className="h-5 w-5 mb-1" />
                <span className="text-xs">繼續新增</span>
              </Button>
              <Button variant="outline" className="flex-col h-auto py-4" onClick={() => form.navigate('/documents?type=PO')}>
                <FileText className="h-5 w-5 mb-1" />
                <span className="text-xs">建立採購單</span>
              </Button>
              <Button variant="outline" className="flex-col h-auto py-4" onClick={() => form.navigate('/products')}>
                <LayoutGrid className="h-5 w-5 mb-1" />
                <span className="text-xs">產品列表</span>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
