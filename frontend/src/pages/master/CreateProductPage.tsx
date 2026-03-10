import { useState, useCallback, useMemo } from 'react'
import { useSteps } from '@/hooks/useSteps'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
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
import { toast } from '@/components/ui/use-toast'
import { SmartInput, ProductSuggestion } from '@/components/product/SmartInput'
import { QuickSelectGrid, QuickSelectItem, SpecSelectionPanel, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { StepIndicator, Step } from '@/components/product/StepIndicator'
import { SkuPreviewBlock, SkuStatus, SkuPreviewResult, SkuPreviewError, MissingField } from '@/components/sku/SkuPreviewBlock'
import {
  ArrowLeft, ArrowRight, Loader2, Check, Package,
  Pill, Syringe, FlaskConical, Settings,
  ListPlus, FileText, LayoutGrid, Sparkles, Plus, X
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { logger } from '@/lib/logger'
import { getApiErrorMessage } from '@/lib/validation'
import { useDebounce } from '@/hooks/useDebounce'

// 步驟定義
const STEPS: Step[] = [
  { id: 'input', label: '輸入名稱', description: '名稱 + 規格' },
  { id: 'confirm', label: '確認規格', description: '分類 + 單位' },
  { id: 'complete', label: '完成建立', description: '檢視結果' },
]

// 快速選擇品項
const QUICK_ITEMS: QuickSelectItem[] = [
  { id: 'glove', icon: '🧤', label: '手套' },
  { id: 'mask', icon: '😷', label: '口罩' },
  { id: 'cotton', icon: '🏥', label: '棉棒' },
  { id: 'gauze', icon: '🩹', label: '紗布' },
  { id: 'syringe', icon: '💉', label: '注射器' },
  { id: 'alcohol', icon: '🧪', label: '酒精' },
  { id: 'saline', icon: '💧', label: '生理食鹽水', displayLabel: (<>生理<br />食鹽水</>) },
]

// 手套規格
const GLOVE_SPECS: QuickSelectSpec[] = [
  { id: 's-powder-free', primary: 'S號', secondary: '無粉' },
  { id: 'm-powder-free', primary: 'M號', secondary: '無粉' },
  { id: 'l-powder-free', primary: 'L號', secondary: '無粉' },
  { id: 'xl-powder-free', primary: 'XL號', secondary: '無粉' },
  { id: 's-powdered', primary: 'S號', secondary: '有粉' },
  { id: 'm-powdered', primary: 'M號', secondary: '有粉' },
  { id: 'l-powdered', primary: 'L號', secondary: '有粉' },
  { id: 'xl-powdered', primary: 'XL號', secondary: '有粉' },
]

// 品類圖示（顯示用，品類清單改由 API useSkuCategories 取得）
const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  DRG: <Pill className="w-4 h-4" />,
  MED: <Syringe className="w-4 h-4" />,
  CON: <Package className="w-4 h-4" />,
  CHM: <FlaskConical className="w-4 h-4" />,
  EQP: <Settings className="w-4 h-4" />,
  GEN: <Package className="w-4 h-4" />,
}

// 單位定義
const UNITS = {
  outer: [  // 外層單位
    { code: 'CTN', name: '箱' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'CASE', name: '件' },
  ],
  inner: [  // 內層單位
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'EA', name: '個' },
    { code: 'PC', name: '支' },
    { code: 'PR', name: '雙' },
    { code: 'BT', name: '瓶' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
  ],
  base: [  // 基礎單位（庫存管理用）
    { code: 'EA', name: '個' },
    { code: 'PC', name: '支' },
    { code: 'PR', name: '雙' },
    { code: 'BT', name: '瓶' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
  ],
  // 保留舊的定義以向後兼容
  drug: [
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
    { code: 'BT', name: '瓶' },
    { code: 'AMP', name: '安瓿' },
    { code: 'VIA', name: '小瓶' },
  ],
  medical: [
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'EA', name: '個' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
  ],
  all: [
    { code: 'EA', name: '個/支' },
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
    { code: 'BT', name: '瓶' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
  ],
}

interface ProductFormData {
  rawInput: string
  name: string
  spec: string
  category: string
  subcategory: string
  // 包裝層數：2層或3層
  packagingLayers: 2 | 3
  // 雙層或三層包裝單位
  outerUnit: string  // 外層單位（箱、盒等）
  outerQty: number   // 外層數量 (通常為1)
  innerUnit: string  // 內層單位（盒、支、個、雙等）
  innerQty: number   // 1 外層 = n 內層
  baseUnit: string   // 基礎單位（用於庫存管理）
  baseQty: number    // 1 內層 = n 基礎
  trackBatch: boolean
  trackExpiry: boolean
  currentStock: number  // 當前庫存值
  currentStockUnit: string  // 當前庫存單位
  safetyStock: number
  safetyStockUnit: string
  reorderPoint: number
  reorderPointUnit: string  // 補貨提醒點單位
}

const initialFormData: ProductFormData = {
  rawInput: '',
  name: '',
  spec: '',
  category: '',
  subcategory: '',
  packagingLayers: 2,
  outerUnit: '',
  outerQty: 1,
  innerUnit: '',
  innerQty: 1,
  baseUnit: '',
  baseQty: 1,
  trackBatch: true,
  trackExpiry: true,
  currentStock: 0,
  currentStockUnit: '',
  safetyStock: 100,
  safetyStockUnit: '',
  reorderPoint: 50,
  reorderPointUnit: '',
}

export function CreateProductPage() {
  const navigate = useNavigate()
  const { step: currentStep, setStep: setCurrentStep, prev } = useSteps(3)
  const { categories: skuCategories, subcategoriesByCategory, isLoading: skuCategoriesLoading } = useSkuCategories()
  const [formData, setFormData] = useState<ProductFormData>(initialFormData)
  const [skuStatus, setSkuStatus] = useState<SkuStatus>('S0')
  const [previewResult, setPreviewResult] = useState<SkuPreviewResult | null>(null)
  const [previewError, setPreviewError] = useState<SkuPreviewError | null>(null)
  const [finalSku, setFinalSku] = useState<string>('')
  const [isPreviewLoading, setIsPreviewLoading] = useState(false)
  const [suggestions, setSuggestions] = useState<ProductSuggestion[]>([])
  const [isSuggestionsLoading, setIsSuggestionsLoading] = useState(false)
  const [selectedQuickItem, setSelectedQuickItem] = useState<QuickSelectItem | null>(null)
  const [selectedSpec, setSelectedSpec] = useState<QuickSelectSpec | null>(null)
  const [glovesMaterial, setGlovesMaterial] = useState<string>('NBR')
  const [quickMode, setQuickMode] = useState(false)

  // 自定義單位狀態
  const [isOuterCustom, setIsOuterCustom] = useState(false)
  const [isInnerCustom, setIsInnerCustom] = useState(false)
  const [isBaseCustom, setIsBaseCustom] = useState(false)
  const [customOuter, setCustomOuter] = useState('')
  const [customInner, setCustomInner] = useState('')
  const [customBase, setCustomBase] = useState('')

  // 品類顯示用：排除 GEN，取前 5 個（與原 slice(0,4) 改為 API 前幾筆，保留 EQP）
  const displayCategories = useMemo(
    () => skuCategories.filter((c) => c.code !== 'GEN').slice(0, 5),
    [skuCategories]
  )
  const hasSubcategories = useCallback(
    (catCode: string) => {
      const subList = subcategoriesByCategory[catCode] ?? []
      if (subList.length === 0) return false
      if (subList.length === 1 && subList[0].code === catCode) return false
      return true
    },
    [subcategoriesByCategory]
  )
  const getSubcategories = useCallback(
    (catCode: string) => {
      const subList = subcategoriesByCategory[catCode] ?? []
      if (subList.length === 1 && subList[0].code === catCode) return []
      return subList
    },
    [subcategoriesByCategory]
  )

  // 防抖輸入
  const debouncedInput = useDebounce(formData.rawInput, 400)

  // 計算缺失欄位
  const missingFields: MissingField[] = useMemo(() => {
    const fields: MissingField[] = []
    if (!formData.name && !formData.rawInput) fields.push({ field: 'name', label: '產品名稱' })
    if (!formData.category) fields.push({ field: 'category', label: '分類' })
    const categoryHasSubcategories = formData.category && hasSubcategories(formData.category)
    if (!formData.subcategory && formData.category && categoryHasSubcategories) {
      fields.push({ field: 'subcategory', label: '子分類' })
    }
    if (!formData.baseUnit) fields.push({ field: 'baseUnit', label: '基礎單位' })
    return fields
  }, [formData, hasSubcategories])

  const canPreview = missingFields.length === 0

  // 取得當前分類的單位選項（用於基礎單位選擇）
  const currentUnits = useMemo(() => {
    return UNITS.base
  }, [])

  // 智能解析輸入
  const parseInput = useCallback((input: string) => {
    // 簡單解析邏輯：第一個空格前為名稱，之後為規格
    const parts = input.trim().split(/\s+/)
    if (parts.length === 0) return { name: '', spec: '' }

    const name = parts[0]
    const spec = parts.slice(1).join(' ')

    return { name, spec }
  }, [])

  // 生成 SKU 預覽
  const generatePreview = useCallback(async () => {
    if (!canPreview) {
      setSkuStatus('S0')
      setPreviewResult(null)
      return
    }

    // 檢查必要欄位
    const requiresSubcategory = formData.category ? hasSubcategories(formData.category) : false
    if (!formData.category || 
        (requiresSubcategory && !formData.subcategory) || 
        !formData.baseUnit || 
        !formData.name) {
      setSkuStatus('S0')
      setPreviewResult(null)
      return
    }

    setSkuStatus('S2')
    setIsPreviewLoading(true)
    setPreviewError(null)

    try {
      const category = formData.category || 'CAT'
      const subs = getSubcategories(category)
      const useSub = subs.length > 0
      const subcategory = useSub ? (formData.subcategory || 'SUB') : category

      const previewSku = `${category}-${subcategory}-XXX`
      const catOption = skuCategories.find((c) => c.code === formData.category)
      const subOption = useSub
        ? subs.find((s) => s.code === formData.subcategory)
        : catOption

      const result: SkuPreviewResult = {
        preview_sku: previewSku,
        rule_version: 'v3.0',
        rule_updated_at: new Date().toISOString().split('T')[0],
        rule_change_summary: '簡化 SKU 結構：種類-品項-流水號',
        segments: [
          {
            code: 'CATEGORY',
            label: '種類',
            value: category,
            source: catOption?.name ?? formData.category,
          },
          {
            code: 'ITEM',
            label: '品項',
            value: subcategory,
            source: subOption?.name ?? formData.subcategory ?? category,
          },
          {
            code: 'SERIAL',
            label: '流水號',
            value: 'XXX',
            source: '自動遞增序號',
          },
        ],
      }

      setPreviewResult(result)
      setSkuStatus('S3')
    } catch (error: unknown) {
      logger.error('SKU preview error:', error)
      setSkuStatus('S4')
      setPreviewError({
        code: 'E5',
        message: getApiErrorMessage(error, '預覽失敗，請稍後再試'),
        suggestion: '請確認網路連線正常，並檢查分類和單位是否已選擇',
      })
    } finally {
      setIsPreviewLoading(false)
    }
  }, [canPreview, formData.category, formData.subcategory, formData.baseUnit, formData.name, hasSubcategories, getSubcategories, skuCategories])

  // 當選擇沒有子分類的類別時，自動清空 subcategory
  useEffect(() => {
    if (formData.category && !hasSubcategories(formData.category) && formData.subcategory) {
      setFormData(prev => ({ ...prev, subcategory: '' }))
    }
  }, [formData.category, formData.subcategory, hasSubcategories])

  // 監聽輸入變化，自動預覽
  useEffect(() => {
    if (currentStep === 1 && skuStatus !== 'S5' && skuStatus !== 'S6') {
      generatePreview()
    }
  }, [debouncedInput, formData.baseUnit, formData.category, formData.subcategory, formData.name, currentStep, generatePreview, skuStatus])

  // 處理智能輸入變化
  const handleInputChange = (value: string) => {
    setFormData(prev => ({ ...prev, rawInput: value }))
    const parsed = parseInput(value)
    setFormData(prev => ({
      ...prev,
      name: parsed.name,
      spec: parsed.spec,
    }))

    // 模擬搜尋建議
    if (value.length > 2) {
      setIsSuggestionsLoading(true)
      setTimeout(() => {
        setSuggestions([
          { name: 'Amoxicillin', spec: '500mg tablet', category: '藥品/抗生素', similarity: 0.95 },
          { name: 'Amoxicillin', spec: '250mg capsule', category: '藥品/抗生素', similarity: 0.88 },
        ].filter(s => s.name.toLowerCase().includes(value.toLowerCase())))
        setIsSuggestionsLoading(false)
      }, 300)
    } else {
      setSuggestions([])
    }
  }

  // 選擇建議
  const handleSelectSuggestion = (suggestion: ProductSuggestion) => {
    setFormData(prev => ({
      ...prev,
      rawInput: `${suggestion.name} ${suggestion.spec}`,
      name: suggestion.name,
      spec: suggestion.spec,
    }))
    setSuggestions([])
  }

  // 選擇快速品項
  const handleQuickItemSelect = (item: QuickSelectItem) => {
    setSelectedQuickItem(item)
    setSelectedSpec(null)
    let category = item.id === 'glove' || item.id === 'mask' ? 'MED' : ''
    if (item.id === 'cotton' || item.id === 'gauze' || item.id === 'syringe' || item.id === 'alcohol' || item.id === 'saline') {
      category = 'CON'
    }
    setFormData(prev => ({
      ...prev,
      rawInput: item.label,
      name: item.label,
      spec: '',
      category: category,
      subcategory: item.id === 'glove' ? 'GLV' : '',
    }))
  }

  // 選擇規格
  const handleSpecSelect = (spec: QuickSelectSpec) => {
    setSelectedSpec(spec)
    if (selectedQuickItem) {
      const fullSpec = selectedQuickItem.id === 'glove'
        ? `${spec.primary} ${spec.secondary} ${glovesMaterial}`
        : `${spec.primary}${spec.secondary ? ' ' + spec.secondary : ''}`
      setFormData(prev => ({
        ...prev,
        rawInput: `${selectedQuickItem.label} ${fullSpec}`,
        spec: fullSpec,
      }))
    }
  }

  // 建立產品
  const createMutation = useMutation({
    mutationFn: async () => {
      setSkuStatus('S5')

      // 根據包裝層數確定消耗單位和包裝單位
      // 兩層：消耗每內層，baseUnit = innerUnit
      // 三層：消耗每基礎單位，baseUnit 是基礎單位
      const consumptionUnit = formData.packagingLayers === 2 
        ? (formData.innerUnit || formData.baseUnit || 'EA')
        : (formData.baseUnit || 'EA')
      
      const packUnit = formData.packagingLayers === 2
        ? (formData.outerUnit || formData.innerUnit || consumptionUnit)  // 兩層：外層或內層作為包裝單位
        : (formData.innerUnit || formData.baseUnit || consumptionUnit)  // 三層：內層作為包裝單位
      
      const packQty = formData.packagingLayers === 2
        ? (formData.outerUnit ? formData.innerQty : 1)  // 兩層：1外層 = n內層
        : (formData.innerQty * formData.baseQty)  // 三層：1內層 = n基礎單位

      const subcategoryCode = hasSubcategories(formData.category)
        ? formData.subcategory
        : formData.category
      
      const response = await api.post('/products', {
        name: formData.name || formData.rawInput.split(' ')[0],
        spec: formData.spec,
        base_uom: consumptionUnit,  // 消耗單位
        track_batch: formData.trackBatch,
        track_expiry: formData.trackExpiry,
        safety_stock: formData.safetyStock || null,
        safety_stock_uom: formData.safetyStockUnit || null,
        reorder_point: formData.reorderPoint || null,
        reorder_point_uom: formData.reorderPointUnit || null,
        category_code: formData.category,
        subcategory_code: subcategoryCode,
        pack_unit: packUnit,
        pack_qty: packQty,
      })

      return response.data
    },
    onSuccess: (data) => {
      setFinalSku(data.sku)
      setSkuStatus('S6')
      setCurrentStep(2)
      toast({
        title: '產品建立成功！',
        description: `SKU: ${data.sku}`,
      })
    },
    onError: (error: unknown) => {
      setSkuStatus('S3')
      toast({
        title: '建立失敗',
        description: getApiErrorMessage(error, '建立產品時發生錯誤'),
        variant: 'destructive',
      })
    },
  })

  // 下一步
  const handleNext = () => {
    if (currentStep === 0) {
      if (!formData.rawInput && !formData.name) {
        toast({
          title: '請輸入產品名稱',
          variant: 'destructive',
        })
        return
      }
      setCurrentStep(1)
      generatePreview()
    } else if (currentStep === 1) {
      if (!formData.baseUnit) {
        toast({
          title: '請選擇基礎單位',
          variant: 'destructive',
        })
        return
      }
      createMutation.mutate()
    }
  }

  // 上一步
  const handleBack = () => {
    prev()
  }

  // 重新開始
  const handleReset = () => {
    setFormData(initialFormData)
    setCurrentStep(0)
    setSkuStatus('S0')
    setPreviewResult(null)
    setFinalSku('')
    setSelectedQuickItem(null)
    setSelectedSpec(null)
  }

  const isCreating = skuStatus === 'S5'
  const isCreated = skuStatus === 'S6'

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-blue-50/50 dark:from-slate-950 dark:via-slate-900 dark:to-slate-800">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 py-8">
        {/* Header */}
        <div className="flex items-center gap-4 mb-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/products')}
            disabled={isCreating}
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
          {/* Quick Mode Toggle */}
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={quickMode}
              onChange={(e) => setQuickMode(e.target.checked)}
              className="rounded"
            />
            <span className="text-slate-600 dark:text-slate-400 hidden sm:inline">快速模式</span>
            <Sparkles className={cn("w-4 h-4", quickMode ? "text-amber-500" : "text-slate-400")} />
          </label>
        </div>

        {/* Step Indicator */}
        <div className="mb-8">
          <StepIndicator
            steps={STEPS}
            currentStep={currentStep}
            completedSteps={isCreated ? [0, 1, 2] : currentStep > 0 ? [0] : []}
          />
        </div>

        {/* Main Content */}
        <div className="flex flex-col gap-8">
          {/* Form Steps */}
          <div className="w-full">
            {/* Step 1: Quick Input */}
            {currentStep === 0 && (
              <div className="space-y-6 animate-fade-in">
                <Card>
                  <CardContent className="pt-6">
                    <div className="space-y-6">
                      {/* Smart Input */}
                      <div className="space-y-3">
                        <Label className="text-base">輸入產品名稱和規格</Label>
                        <SmartInput
                          value={formData.rawInput}
                          onChange={handleInputChange}
                          onSelect={handleSelectSuggestion}
                          onCreateNew={() => setCurrentStep(1)}
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
                          onSelect={handleQuickItemSelect}
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
                            onSelect={handleSpecSelect}
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
                                onChange: setGlovesMaterial,
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
                    onClick={handleNext}
                    disabled={!formData.rawInput && !formData.name}
                    size="lg"
                  >
                    下一步
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                </div>
              </div>
            )}

            {/* Step 2: Confirm Details */}
            {currentStep === 1 && (
              <div className="space-y-6 animate-slide-in-right">
                {/* Basic Info */}
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

                      {/* Category Recommendation */}
                      <div className="space-y-2">
                        <Label>分類（系統推薦）</Label>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                          {displayCategories.slice(0, 4).map((cat) => {
                            return (
                              <button
                                key={cat.code}
                                type="button"
                                onClick={() => {
                                  const defaultSubcategory = hasSubcategories(cat.code) ? '' : ''
                                  setFormData(prev => ({
                                    ...prev,
                                    category: cat.code,
                                    subcategory: defaultSubcategory
                                  }))
                                }}
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
                                    ✨ 推薦
                                  </span>
                                )}
                              </button>
                            )
                          })}
                        </div>
                      </div>

                      {/* Subcategory - 僅在有子分類的類別時顯示 */}
                      {formData.category && (() => {
                        const displayCategory = formData.category === 'CON' ? 'CON' : formData.category
                        const subs = getSubcategories(displayCategory)
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

                {/* Unit Selection - 雙層或三層包裝單位 */}
                <Card>
                  <CardContent className="pt-6">
                    <h3 className="text-lg font-semibold mb-4">包裝單位</h3>
                    <div className="space-y-6">
                      {/* 包裝層數選擇 */}
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">包裝層數</Label>
                        <div className="flex gap-3">
                          <button
                            type="button"
                            onClick={() => {
                              // 切換到兩層時，清除外層單位和基礎單位，內層成為消耗單位
                              setFormData(prev => ({ 
                                ...prev, 
                                packagingLayers: 2, 
                                outerUnit: '',
                                baseUnit: prev.innerUnit || prev.baseUnit, // 內層成為基礎單位
                                baseQty: 1
                              }))
                            }}
                            disabled={isCreated}
                            className={cn(
                              "flex-1 p-3 rounded-lg border-2 transition-all",
                              formData.packagingLayers === 2
                                ? "border-primary bg-primary/10 text-primary font-medium"
                                : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                            )}
                          >
                            兩層包裝
                            <span className="block text-xs mt-1 text-slate-500">外層 → 內層（消耗每內層）</span>
                          </button>
                          <button
                            type="button"
                            onClick={() => setFormData(prev => ({ ...prev, packagingLayers: 3 }))}
                            disabled={isCreated}
                            className={cn(
                              "flex-1 p-3 rounded-lg border-2 transition-all",
                              formData.packagingLayers === 3
                                ? "border-primary bg-primary/10 text-primary font-medium"
                                : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                            )}
                          >
                            三層包裝
                            <span className="block text-xs mt-1 text-slate-500">外層 → 內層 → 基礎（消耗每基礎）</span>
                          </button>
                        </div>
                      </div>
                      {/* 外層包裝和內層包裝（兩層和三層都顯示） */}
                      {(formData.packagingLayers === 2 || formData.packagingLayers === 3) && (
                        <div className="space-y-6">
                          {/* 外層包裝（兩層和三層都顯示） */}
                          <div className="space-y-3">
                            <Label className="text-sm font-medium">外層包裝</Label>
                            <div className="flex items-center gap-3">
                              <div className="flex flex-wrap gap-2 flex-1">
                            {UNITS.outer.map((unit) => (
                              <button
                                key={unit.code}
                                type="button"
                                onClick={() => {
                                  setIsOuterCustom(false);
                                  setFormData(prev => ({
                                    ...prev,
                                    outerUnit: prev.outerUnit === unit.name ? '' : unit.name
                                  }));
                                }}
                                disabled={isCreated}
                                className={cn(
                                  "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all",
                                  formData.outerUnit === unit.name && !isOuterCustom
                                    ? "border-primary bg-primary/10"
                                    : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                                )}
                              >
                                <span className="font-mono font-semibold text-sm">{unit.code}</span>
                                <span className="text-xs text-slate-500">{unit.name}</span>
                              </button>
                            ))}
                            <button
                              type="button"
                              onClick={() => {
                                setIsOuterCustom(true);
                                setFormData(prev => ({ ...prev, outerUnit: customOuter }));
                              }}
                              disabled={isCreated}
                              className={cn(
                                "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 border-dashed transition-all",
                                isOuterCustom
                                  ? "border-primary bg-primary/10"
                                  : "border-slate-300 dark:border-slate-600 hover:border-primary/50"
                              )}
                            >
                              <Plus className="w-5 h-5 text-slate-400" />
                              <span className="text-[10px] text-slate-500 mt-1">自填量詞</span>
                            </button>
                          </div>
                          {isOuterCustom && (
                            <div className="flex items-center gap-2">
                              <Input
                                placeholder="輸入量詞"
                                value={customOuter}
                                onChange={(e) => {
                                  setCustomOuter(e.target.value);
                                  setFormData(prev => ({ ...prev, outerUnit: e.target.value }));
                                }}
                                className="w-24"
                                disabled={isCreated}
                              />
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8"
                                onClick={() => {
                                  setIsOuterCustom(false);
                                  setCustomOuter('');
                                  if (formData.outerUnit === customOuter) {
                                    setFormData(prev => ({ ...prev, outerUnit: '' }));
                                  }
                                }}
                              >
                                <X className="h-4 w-4" />
                              </Button>
                            </div>
                          )}
                          {formData.outerUnit && (
                            <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
                              <span className="text-sm text-slate-500">1</span>
                              <span className="text-sm text-slate-700 dark:text-slate-200">{formData.outerUnit}</span>
                            </div>
                          )}
                        </div>
                      </div>

                          {/* 內層包裝 */}
                          <div className="space-y-3">
                        <Label className="text-sm font-medium">
                          內層包裝
                          {formData.packagingLayers === 2 && (
                            <span className="text-xs text-slate-400 ml-2">（消耗單位）</span>
                          )}
                        </Label>
                            <div className="flex items-center gap-3">
                              <div className="flex flex-wrap gap-2 flex-1">
                                {UNITS.inner.map((unit) => (
                                  <button
                                    key={unit.code}
                                    type="button"
                                    onClick={() => {
                                      setIsInnerCustom(false);
                                      const updates: Partial<ProductFormData> = {
                                        innerUnit: unit.name
                                      }
                                      // 兩層時，內層就是基礎單位（消耗單位）
                                      if (formData.packagingLayers === 2) {
                                        updates.baseUnit = unit.name
                                        updates.baseQty = 1
                                      }
                                      setFormData(prev => ({ ...prev, ...updates }))
                                    }}
                                    disabled={isCreated}
                                    className={cn(
                                      "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all",
                                      formData.innerUnit === unit.name && !isInnerCustom
                                        ? "border-primary bg-primary/10"
                                        : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                                    )}
                                  >
                                    <span className="font-mono font-semibold text-sm">{unit.code}</span>
                                    <span className="text-xs text-slate-500">{unit.name}</span>
                                  </button>
                                ))}
                                <button
                                  type="button"
                                  onClick={() => {
                                    setIsInnerCustom(true);
                                    const updates: Partial<ProductFormData> = {
                                      innerUnit: customInner
                                    }
                                    if (formData.packagingLayers === 2) {
                                      updates.baseUnit = customInner
                                      updates.baseQty = 1
                                    }
                                    setFormData(prev => ({ ...prev, ...updates }));
                                  }}
                                  disabled={isCreated}
                                  className={cn(
                                    "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 border-dashed transition-all",
                                    isInnerCustom
                                      ? "border-primary bg-primary/10"
                                      : "border-slate-300 dark:border-slate-600 hover:border-primary/50"
                                  )}
                                >
                                  <Plus className="w-5 h-5 text-slate-400" />
                                  <span className="text-[10px] text-slate-500 mt-1">自填量詞</span>
                                </button>
                              </div>
                              {isInnerCustom && (
                                <div className="flex items-center gap-2">
                                  <Input
                                    placeholder="輸入量詞"
                                    value={customInner}
                                    onChange={(e) => {
                                      setCustomInner(e.target.value);
                                      const updates: Partial<ProductFormData> = {
                                        innerUnit: e.target.value
                                      }
                                      if (formData.packagingLayers === 2) {
                                        updates.baseUnit = e.target.value
                                        updates.baseQty = 1
                                      }
                                      setFormData(prev => ({ ...prev, ...updates }));
                                    }}
                                    className="w-24"
                                    disabled={isCreated}
                                  />
                                </div>
                              )}
                              {formData.innerUnit && (
                                <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
                                  <span className="text-sm text-slate-500">
                                    {formData.outerUnit ? `一${formData.outerUnit}` : formData.packagingLayers === 2 ? '一' : '一'}
                                  </span>
                                  <Input
                                    type="number"
                                    min={1}
                                    value={formData.innerQty}
                                    onChange={(e) => setFormData(prev => ({ ...prev, innerQty: parseInt(e.target.value) || 1 }))}
                                    className="w-16 h-8 text-center"
                                    disabled={isCreated}
                                  />
                                  <span className="text-sm text-slate-500">
                                    {formData.innerUnit}
                                  </span>
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      )}

                      {/* 基礎單位（僅三層時顯示，為消耗單位） */}
                      {formData.packagingLayers === 3 && (
                        <div className="space-y-6">
                          <div className="space-y-3">
                            <Label className="text-sm font-medium">
                              基礎單位（消耗單位）
                              <span className="text-xs text-slate-400 ml-2">(庫存管理)</span>
                            </Label>
                            <div className="flex items-center gap-3">
                              <div className="flex flex-wrap gap-2 flex-1">
                              {/* 自動填充：優先顯示已選擇的包裝單位 */}
                              {(() => {
                                // 收集已選擇的包裝單位
                                const selectedUnits = [
                                  formData.outerUnit,
                                  formData.innerUnit,
                                ].filter(Boolean).map(u => {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === u || unit.code === u
                                  )
                                  return found ? { code: found.code, name: found.name } : null
                                }).filter(Boolean) as Array<{ code: string; name: string }>

                                // 去重並保持順序
                                const uniqueSelectedUnits = Array.from(
                                  new Map(selectedUnits.map(u => [u.code, u])).values()
                                )

                                // 合併：先顯示已選擇的單位，再顯示其他基礎單位
                                const otherBaseUnits = UNITS.base.filter(
                                  u => !uniqueSelectedUnits.some(su => su.code === u.code)
                                )
                                const displayUnits = [...uniqueSelectedUnits, ...otherBaseUnits]

                                return displayUnits.map((unit) => (
                                  <button
                                    key={unit.code}
                                    type="button"
                                    onClick={() => {
                                      setIsBaseCustom(false);
                                      setFormData(prev => ({
                                        ...prev,
                                        baseUnit: unit.name,
                                        safetyStockUnit: unit.name,
                                        currentStockUnit: unit.name,
                                        reorderPointUnit: unit.name
                                      }))
                                    }}
                                    disabled={isCreated}
                                    className={cn(
                                      "relative flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all",
                                      formData.baseUnit === unit.name && !isBaseCustom
                                        ? "border-primary bg-primary/10"
                                        : "border-slate-200 dark:border-slate-700 hover:border-primary/50",
                                      uniqueSelectedUnits.some(su => su.code === unit.code) && "ring-2 ring-blue-300 dark:ring-blue-700"
                                    )}
                                  >
                                    <span className="font-mono font-semibold text-sm">{unit.code}</span>
                                    <span className="text-xs text-slate-500">{unit.name}</span>
                                    {uniqueSelectedUnits.some(su => su.code === unit.code) && (
                                      <span className="absolute -top-1 -right-1 w-3 h-3 bg-blue-500 rounded-full border-2 border-white dark:border-slate-800"></span>
                                    )}
                                  </button>
                                ))
                              })()}
                              <button
                                type="button"
                                onClick={() => {
                                  setIsBaseCustom(true);
                                  setFormData(prev => ({ ...prev, baseUnit: customBase, safetyStockUnit: customBase, currentStockUnit: customBase, reorderPointUnit: customBase }));
                                }}
                                disabled={isCreated}
                                className={cn(
                                  "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 border-dashed transition-all",
                                  isBaseCustom
                                    ? "border-primary bg-primary/10"
                                    : "border-slate-300 dark:border-slate-600 hover:border-primary/50"
                                )}
                              >
                                <Plus className="w-5 h-5 text-slate-400" />
                                <span className="text-[10px] text-slate-500 mt-1">自填量詞</span>
                              </button>
                            </div>
                            {isBaseCustom && (
                              <div className="flex items-center gap-2">
                                <Input
                                  placeholder="輸入量詞"
                                  value={customBase}
                                  onChange={(e) => {
                                    setCustomBase(e.target.value);
                                    setFormData(prev => ({ ...prev, baseUnit: e.target.value, safetyStockUnit: e.target.value, currentStockUnit: e.target.value, reorderPointUnit: e.target.value }));
                                  }}
                                  className="w-24"
                                  disabled={isCreated}
                                />
                              </div>
                            )}

                            {/* 基礎單位換算 - 移至右側（僅三層時顯示） */}
                            {formData.packagingLayers === 3 && formData.innerUnit && formData.baseUnit && (
                              <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
                                <span className="text-sm text-slate-500">
                                  一{formData.innerUnit}
                                </span>
                                <Input
                                  type="number"
                                  min={1}
                                  value={formData.baseQty}
                                  onChange={(e) => setFormData(prev => ({ ...prev, baseQty: parseInt(e.target.value) || 1 }))}
                                  className="w-16 h-8 text-center"
                                  disabled={isCreated}
                                />
                                <span className="text-sm text-slate-500">
                                  {formData.baseUnit}
                                </span>
                              </div>
                            )}
                            </div>
                          </div>
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>

                {/* Inventory Settings */}
                <Card>
                  <CardContent className="pt-6">
                    <h3 className="text-lg font-semibold mb-4">庫存設定</h3>
                    <div className="space-y-6">
                      {/* Tracking Options */}
                      <div className="flex gap-6">
                        <label className="flex items-center gap-2">
                          <input
                            type="checkbox"
                            checked={formData.trackBatch}
                            onChange={(e) => setFormData(prev => ({ ...prev, trackBatch: e.target.checked }))}
                            disabled={isCreated}
                            className="rounded"
                          />
                          <span className="text-sm">追蹤批號</span>
                        </label>
                        <label className="flex items-center gap-2">
                          <input
                            type="checkbox"
                            checked={formData.trackExpiry}
                            onChange={(e) => setFormData(prev => ({ ...prev, trackExpiry: e.target.checked }))}
                            disabled={isCreated}
                            className="rounded"
                          />
                          <span className="text-sm">追蹤效期</span>
                        </label>
                      </div>
                      {formData.category === 'DRG' && (
                        <p className="text-xs text-amber-600 dark:text-amber-400">
                          💡 藥品建議開啟批號和效期追蹤
                        </p>
                      )}

                      {/* Current Stock */}
                      <div className="space-y-2">
                        <Label>當前值（單位）</Label>
                        <div className="flex items-center gap-3">
                          <Input
                            type="number"
                            min={0}
                            value={formData.currentStock}
                            onChange={(e) => setFormData(prev => ({ ...prev, currentStock: parseFloat(e.target.value) || 0 }))}
                            className="w-32"
                            disabled={isCreated}
                            placeholder="0"
                          />
                          <Select
                            value={formData.currentStockUnit}
                            onValueChange={(v) => setFormData(prev => ({ ...prev, currentStockUnit: v }))}
                            disabled={isCreated}
                          >
                            <SelectTrigger className="w-32">
                              <SelectValue placeholder="選擇單位" />
                            </SelectTrigger>
                            <SelectContent>
                              {/* 根據包裝層數顯示選項：兩層顯示內/外，三層顯示內/外/基礎 */}
                              {(() => {
                                const units: Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> = []
                                
                                // 外層包裝
                                if (formData.outerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.outerUnit || unit.code === formData.outerUnit
                                  )
                                  if (found) {
                                    units.push({ code: found.code, name: found.name, type: 'outer' })
                                  }
                                }
                                
                                // 內層包裝
                                if (formData.innerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.innerUnit || unit.code === formData.innerUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'inner' })
                                  }
                                }
                                
                                // 基礎單位（僅三層包裝時顯示）
                                if (formData.packagingLayers === 3 && formData.baseUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.baseUnit || unit.code === formData.baseUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'base' })
                                  }
                                }
                                
                                // 如果沒有選擇任何包裝單位，顯示基礎單位選項
                                if (units.length === 0) {
                                  return UNITS.base.map((unit) => (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code})
                                    </SelectItem>
                                  ))
                                }
                                
                                return units.map((unit) => {
                                  const typeLabel = unit.type === 'outer' ? '外' : unit.type === 'inner' ? '內' : '基礎'
                                  return (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code}) - {typeLabel}
                                    </SelectItem>
                                  )
                                })
                              })()}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>

                      {/* Safety Stock */}
                      <div className="space-y-2">
                        <Label>安全庫存</Label>
                        <div className="flex items-center gap-3">
                          <Input
                            type="number"
                            min={0}
                            value={formData.safetyStock}
                            onChange={(e) => setFormData(prev => ({ ...prev, safetyStock: parseFloat(e.target.value) || 0 }))}
                            className="w-32"
                            disabled={isCreated}
                            placeholder="0"
                          />
                          <Select
                            value={formData.safetyStockUnit}
                            onValueChange={(v) => setFormData(prev => ({ ...prev, safetyStockUnit: v }))}
                            disabled={isCreated}
                          >
                            <SelectTrigger className="w-32">
                              <SelectValue placeholder="選擇單位" />
                            </SelectTrigger>
                            <SelectContent>
                              {/* 根據包裝層數顯示選項：兩層顯示內/外，三層顯示內/外/基礎 */}
                              {(() => {
                                const units: Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> = []
                                
                                // 外層包裝
                                if (formData.outerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.outerUnit || unit.code === formData.outerUnit
                                  )
                                  if (found) {
                                    units.push({ code: found.code, name: found.name, type: 'outer' })
                                  }
                                }
                                
                                // 內層包裝
                                if (formData.innerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.innerUnit || unit.code === formData.innerUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'inner' })
                                  }
                                }
                                
                                // 基礎單位（僅三層包裝時顯示）
                                if (formData.packagingLayers === 3 && formData.baseUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.baseUnit || unit.code === formData.baseUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'base' })
                                  }
                                }
                                
                                // 如果沒有選擇任何包裝單位，顯示基礎單位選項
                                if (units.length === 0) {
                                  return UNITS.base.map((unit) => (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code})
                                    </SelectItem>
                                  ))
                                }
                                
                                return units.map((unit) => {
                                  const typeLabel = unit.type === 'outer' ? '外' : unit.type === 'inner' ? '內' : '基礎'
                                  return (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code}) - {typeLabel}
                                    </SelectItem>
                                  )
                                })
                              })()}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>

                      {/* Reorder Point */}
                      <div className="space-y-2">
                        <Label>補貨提醒點</Label>
                        <div className="flex items-center gap-3 flex-wrap">
                          <span className="text-sm text-slate-500">當庫存低於</span>
                          <Input
                            type="number"
                            min={0}
                            value={formData.reorderPoint}
                            onChange={(e) => setFormData(prev => ({ ...prev, reorderPoint: parseFloat(e.target.value) || 0 }))}
                            className="w-24"
                            disabled={isCreated}
                          />
                          <Select
                            value={formData.reorderPointUnit}
                            onValueChange={(v) => setFormData(prev => ({ ...prev, reorderPointUnit: v }))}
                            disabled={isCreated}
                          >
                            <SelectTrigger className="w-40">
                              <SelectValue placeholder="選擇單位" />
                            </SelectTrigger>
                            <SelectContent>
                              {/* 顯示：內層包裝或外層包裝或基礎單位 */}
                              {(() => {
                                const units: Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> = []
                                
                                // 外層包裝
                                if (formData.outerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.outerUnit || unit.code === formData.outerUnit
                                  )
                                  if (found) {
                                    units.push({ code: found.code, name: found.name, type: 'outer' })
                                  }
                                }
                                
                                // 內層包裝
                                if (formData.innerUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.innerUnit || unit.code === formData.innerUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'inner' })
                                  }
                                }
                                
                                // 基礎單位
                                if (formData.baseUnit) {
                                  const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
                                    unit => unit.name === formData.baseUnit || unit.code === formData.baseUnit
                                  )
                                  if (found && !units.some(u => u.code === found.code)) {
                                    units.push({ code: found.code, name: found.name, type: 'base' })
                                  }
                                }
                                
                                // 如果沒有選擇任何包裝單位，顯示基礎單位選項
                                if (units.length === 0) {
                                  return UNITS.base.map((unit) => (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code})
                                    </SelectItem>
                                  ))
                                }
                                
                                return units.map((unit) => {
                                  const typeLabel = unit.type === 'outer' ? '外層' : unit.type === 'inner' ? '內層' : '基礎'
                                  return (
                                    <SelectItem key={unit.code} value={unit.name}>
                                      {unit.name} ({unit.code}) - {typeLabel}包裝
                                    </SelectItem>
                                  )
                                })
                              })()}
                            </SelectContent>
                          </Select>
                          <span className="text-sm text-slate-500">時，發送補貨提醒</span>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Action Buttons */}
                <div className="flex justify-between">
                  <Button
                    variant="outline"
                    onClick={handleBack}
                    disabled={isCreating}
                  >
                    <ArrowLeft className="mr-2 h-4 w-4" />
                    上一步
                  </Button>
                  <Button
                    onClick={handleNext}
                    disabled={!formData.baseUnit || isCreating || skuStatus !== 'S3'}
                    size="lg"
                  >
                    {isCreating ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        建立中...
                      </>
                    ) : (
                      <>
                        建立產品
                        <Check className="ml-2 h-4 w-4" />
                      </>
                    )}
                  </Button>
                </div>
              </div>
            )}

            {/* Step 3: Success */}
            {currentStep === 2 && (
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
                      {formData.name} {formData.spec}
                    </p>
                  </div>

                  <CardContent className="pt-6">
                    <div className="space-y-4">
                      <div className="p-4 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700">
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-slate-500">SKU</span>
                          <span className="font-mono font-bold text-lg text-primary">{finalSku}</span>
                        </div>
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-slate-500">分類</span>
                          <span>{skuCategories.find(c => c.code === formData.category)?.name ?? '—'}</span>
                        </div>
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-slate-500">單位</span>
                          <span>{currentUnits.find(u => u.code === formData.baseUnit)?.name || '—'}</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-sm text-slate-500">追蹤</span>
                          <span>
                            {formData.trackBatch && '批號'} {formData.trackBatch && formData.trackExpiry && '/'} {formData.trackExpiry && '效期'}
                            {!formData.trackBatch && !formData.trackExpiry && '無'}
                          </span>
                        </div>
                      </div>

                      <p className="text-sm text-slate-600 dark:text-slate-400">
                        接下來您可以：
                      </p>

                      <div className="grid grid-cols-3 gap-3">
                        <Button
                          variant="outline"
                          className="flex-col h-auto py-4"
                          onClick={handleReset}
                        >
                          <ListPlus className="h-5 w-5 mb-1" />
                          <span className="text-xs">繼續新增</span>
                        </Button>
                        <Button
                          variant="outline"
                          className="flex-col h-auto py-4"
                          onClick={() => navigate('/documents?type=PO')}
                        >
                          <FileText className="h-5 w-5 mb-1" />
                          <span className="text-xs">建立採購單</span>
                        </Button>
                        <Button
                          variant="outline"
                          className="flex-col h-auto py-4"
                          onClick={() => navigate('/products')}
                        >
                          <LayoutGrid className="h-5 w-5 mb-1" />
                          <span className="text-xs">產品列表</span>
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            )}
          </div>


          {/* Bottom: SKU Preview */}
          <div className="w-full">
            <SkuPreviewBlock
              status={skuStatus}
              previewResult={previewResult}
              error={previewError}
              missingFields={currentStep === 1 ? missingFields : []}
              finalSku={finalSku}
              isLoading={isPreviewLoading}
              onRefresh={generatePreview}
              compact={currentStep === 2}
            />
          </div>
        </div>
      </div>
    </div>
  )
}
