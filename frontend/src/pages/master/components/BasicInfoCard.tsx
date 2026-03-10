import { Package } from 'lucide-react'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { CATEGORY_ICONS } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface BasicInfoCardProps {
  form: CreateProductFormReturn
}

export function BasicInfoCard({ form }: BasicInfoCardProps) {
  const { formData, setFormData, isCreated, displayCategories, getSubcategories, skuCategoriesLoading } = form

  return (
    <Card>
      <CardContent className="pt-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Package className="h-5 w-5 text-primary" />
          基本資訊
        </h3>
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>產品名稱</Label>
              <Input
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                placeholder="產品名稱"
                disabled={isCreated}
              />
            </div>
            <div className="space-y-2">
              <Label>規格描述</Label>
              <Input
                value={formData.spec}
                onChange={(e) => setFormData(prev => ({ ...prev, spec: e.target.value }))}
                placeholder="規格"
                disabled={isCreated}
              />
            </div>
          </div>

          {/* Category buttons */}
          <div className="space-y-2">
            <Label>分類（系統推薦）</Label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {displayCategories.slice(0, 4).map((cat) => (
                <button
                  key={cat.code}
                  type="button"
                  onClick={() => setFormData(prev => ({ ...prev, category: cat.code, subcategory: '' }))}
                  disabled={isCreated || skuCategoriesLoading}
                  className={cn(
                    "flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all",
                    formData.category === cat.code
                      ? "border-primary bg-primary/5"
                      : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                  )}
                >
                  <div className={cn(
                    "p-2 rounded-md",
                    formData.category === cat.code
                      ? "bg-primary/10 text-primary"
                      : "bg-slate-100 dark:bg-slate-800"
                  )}>
                    {CATEGORY_ICONS[cat.code]}
                  </div>
                  <span className="font-medium">{cat.name}</span>
                  {cat.code === 'DRG' && formData.name?.toLowerCase().match(/cillin|mycin|oxacin/) && (
                    <span className="ml-auto text-xs bg-success/10 text-success px-2 py-0.5 rounded-full">
                      推薦
                    </span>
                  )}
                </button>
              ))}
            </div>
          </div>

          {/* Subcategory dropdown */}
          {formData.category && (() => {
            const subs = getSubcategories(formData.category)
            if (subs.length === 0) return null
            return (
              <div className="space-y-2">
                <Label>子分類</Label>
                <Select
                  value={formData.subcategory}
                  onValueChange={(v) => setFormData(prev => ({ ...prev, subcategory: v }))}
                  disabled={isCreated || skuCategoriesLoading}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="選擇子分類" />
                  </SelectTrigger>
                  <SelectContent>
                    {subs.map((sub) => (
                      <SelectItem key={sub.code} value={sub.code}>
                        {sub.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )
          })()}
        </div>
      </CardContent>
    </Card>
  )
}
