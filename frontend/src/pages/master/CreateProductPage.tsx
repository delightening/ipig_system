import { useState, useEffect, useCallback, useMemo } from 'react'
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
import { Slider } from '@/components/ui/slider'
import { SmartInput, ProductSuggestion } from '@/components/product/SmartInput'
import { QuickSelectGrid, QuickSelectItem, SpecSelectionPanel, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { StepIndicator, Step } from '@/components/product/StepIndicator'
import { SkuPreviewBlock, SkuStatus, SkuPreviewResult, SkuPreviewError, MissingField } from '@/components/sku/SkuPreviewBlock'
import { 
  ArrowLeft, ArrowRight, Loader2, Check, Package, 
  Pill, Syringe, TestTube, FlaskConical, Settings,
  ListPlus, FileText, LayoutGrid, Sparkles
} from 'lucide-react'
import { cn } from '@/lib/utils'

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
  { id: 'saline', icon: '💧', label: '生理食鹽' },
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

// 分類定義
const CATEGORIES = [
  { code: 'DRG', name: '藥品', icon: <Pill className="w-4 h-4" />, subcategories: [
    { code: 'ABX', name: '抗生素' },
    { code: 'ANL', name: '止痛藥' },
    { code: 'VIT', name: '維生素' },
    { code: 'OTH', name: '其他藥品' },
  ]},
  { code: 'MED', name: '醫材', icon: <Syringe className="w-4 h-4" />, subcategories: [
    { code: 'SYR', name: '注射器材' },
    { code: 'BND', name: '敷料繃帶' },
    { code: 'GLV', name: '手套' },
    { code: 'OTH', name: '其他醫材' },
  ]},
  { code: 'LAB', name: '實驗耗材', icon: <TestTube className="w-4 h-4" />, subcategories: [
    { code: 'TUB', name: '試管' },
    { code: 'PIP', name: '吸管' },
    { code: 'PLT', name: '培養皿' },
    { code: 'OTH', name: '其他耗材' },
  ]},
  { code: 'CHM', name: '化學品', icon: <FlaskConical className="w-4 h-4" />, subcategories: [
    { code: 'RGT', name: '試劑' },
    { code: 'SOL', name: '溶劑' },
    { code: 'STD', name: '標準品' },
    { code: 'OTH', name: '其他化學品' },
  ]},
  { code: 'EQP', name: '設備', icon: <Settings className="w-4 h-4" />, subcategories: [
    { code: 'INS', name: '儀器' },
    { code: 'TOL', name: '工具' },
    { code: 'PRT', name: '零件' },
    { code: 'OTH', name: '其他設備' },
  ]},
]

// 單位定義
const UNITS = {
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

// 防抖 Hook
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value)

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => clearTimeout(handler)
  }, [value, delay])

  return debouncedValue
}

interface ProductFormData {
  rawInput: string
  name: string
  spec: string
  category: string
  subcategory: string
  unit: string
  packQty: number
  trackBatch: boolean
  trackExpiry: boolean
  safetyStock: number
  safetyStockUnit: string
  reorderPoint: number
}

const initialFormData: ProductFormData = {
  rawInput: '',
  name: '',
  spec: '',
  category: '',
  subcategory: '',
  unit: '',
  packQty: 1,
  trackBatch: false,
  trackExpiry: false,
  safetyStock: 100,
  safetyStockUnit: '',
  reorderPoint: 50,
}

export function CreateProductPage() {
  const navigate = useNavigate()
  const [currentStep, setCurrentStep] = useState(0)
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

  // 防抖輸入
  const debouncedInput = useDebounce(formData.rawInput, 400)

  // 計算缺失欄位
  const missingFields: MissingField[] = useMemo(() => {
    const fields: MissingField[] = []
    if (!formData.name && !formData.rawInput) fields.push({ field: 'name', label: '產品名稱' })
    if (!formData.unit) fields.push({ field: 'unit', label: '單位' })
    return fields
  }, [formData])

  const canPreview = missingFields.length === 0

  // 取得當前分類的單位選項
  const currentUnits = useMemo(() => {
    if (formData.category === 'DRG') return UNITS.drug
    if (formData.category === 'MED') return UNITS.medical
    return UNITS.all
  }, [formData.category])

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

    setSkuStatus('S2')
    setIsPreviewLoading(true)
    setPreviewError(null)

    try {
      // 模擬 API（實際應呼叫後端）
      await new Promise(resolve => setTimeout(resolve, 400))

      const parsed = parseInput(formData.rawInput)
      const name = formData.name || parsed.name
      const spec = formData.spec || parsed.spec

      // 生成名稱縮寫
      const nameAbbr = name
        .split(/\s+/)
        .map(w => w.charAt(0).toUpperCase())
        .join('')
        .slice(0, 4) || 'PRD'

      // 生成規格碼
      const specMatch = spec.match(/(\d+)\s*(mg|ml|g|l|號)?/i)
      const specCode = specMatch 
        ? `${specMatch[1]}${(specMatch[2] || '').toUpperCase()}`
        : spec.slice(0, 6).toUpperCase().replace(/\s+/g, '')

      // 生成日期
      const now = new Date()
      const yymmdd = `${String(now.getFullYear()).slice(-2)}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}`

      const previewSku = `${nameAbbr}-${specCode || 'STD'}-${formData.unit || 'EA'}-${yymmdd}-XXX-X`

      const result: SkuPreviewResult = {
        preview_sku: previewSku,
        rule_version: 'v1.3',
        rule_updated_at: '2026-01-07',
        rule_change_summary: '新版 SKU 結構：名稱-規格-單位-日期-序號-檢查碼',
        segments: [
          { code: 'NAME_ABBR', label: '名稱', value: nameAbbr, source: name || '產品名稱' },
          { code: 'SPEC_CODE', label: '規格', value: specCode || 'STD', source: spec || '標準規格' },
          { code: 'UNIT', label: '單位', value: formData.unit || 'EA', source: '庫存單位' },
          { code: 'YYMMDD', label: '日期', value: yymmdd, source: '系統日期' },
          { code: 'SEQ', label: '序號', value: 'XXX', source: '建立時分配' },
          { code: 'CHK', label: '檢查碼', value: 'X', source: 'Luhn 檢查碼' },
        ],
      }

      setPreviewResult(result)
      setSkuStatus('S3')
    } catch (error: any) {
      setSkuStatus('S4')
      setPreviewError({
        code: 'E5',
        message: error?.response?.data?.error?.message || '預覽失敗，請稍後再試',
        suggestion: '請確認網路連線正常',
      })
    } finally {
      setIsPreviewLoading(false)
    }
  }, [canPreview, formData, parseInput])

  // 監聽輸入變化，自動預覽
  useEffect(() => {
    if (currentStep === 1 && skuStatus !== 'S5' && skuStatus !== 'S6') {
      generatePreview()
    }
  }, [debouncedInput, formData.unit, currentStep, generatePreview, skuStatus])

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
    setFormData(prev => ({
      ...prev,
      rawInput: item.label,
      name: item.label,
      spec: '',
      category: item.id === 'glove' || item.id === 'mask' ? 'MED' : '',
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
      
      const response = await api.post('/products', {
        name: formData.name || formData.rawInput.split(' ')[0],
        spec: formData.spec,
        base_uom: formData.unit || 'EA',
        track_batch: formData.trackBatch,
        track_expiry: formData.trackExpiry,
        safety_stock: formData.safetyStock || null,
        reorder_point: formData.reorderPoint || null,
        category_code: formData.category,
        subcategory_code: formData.subcategory,
        pack_unit: formData.unit,
        pack_qty: formData.packQty,
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
    onError: (error: any) => {
      setSkuStatus('S3')
      toast({
        title: '建立失敗',
        description: error?.response?.data?.error?.message || '建立產品時發生錯誤',
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
      if (!formData.unit) {
        toast({
          title: '請選擇單位',
          variant: 'destructive',
        })
        return
      }
      createMutation.mutate()
    }
  }

  // 上一步
  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1)
    }
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
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-6">
          {/* Left: Form Steps */}
          <div className="lg:col-span-3">
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
                          onShowMore={() => {}}
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
                          {CATEGORIES.slice(0, 4).map((cat) => (
                            <button
                              key={cat.code}
                              type="button"
                              onClick={() => setFormData(prev => ({ 
                                ...prev, 
                                category: cat.code,
                                subcategory: ''
                              }))}
                              disabled={isCreated}
                              className={cn(
                                "flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all",
                                formData.category === cat.code
                                  ? "border-primary bg-primary/5"
                                  : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                              )}
                            >
                              <div className={cn(
                                "p-2 rounded-md",
                                formData.category === cat.code ? "bg-primary/10 text-primary" : "bg-slate-100 dark:bg-slate-800"
                              )}>
                                {cat.icon}
                              </div>
                              <span className="font-medium">{cat.name}</span>
                              {cat.code === 'DRG' && formData.name?.toLowerCase().match(/cillin|mycin|oxacin/) && (
                                <span className="ml-auto text-xs bg-success/10 text-success px-2 py-0.5 rounded-full">
                                  ✨ 推薦
                                </span>
                              )}
                            </button>
                          ))}
                        </div>
                      </div>

                      {/* Subcategory */}
                      {formData.category && (
                        <div className="space-y-2">
                          <Label>子分類</Label>
                          <Select
                            value={formData.subcategory}
                            onValueChange={(v) => setFormData(prev => ({ ...prev, subcategory: v }))}
                            disabled={isCreated}
                          >
                            <SelectTrigger>
                              <SelectValue placeholder="選擇子分類" />
                            </SelectTrigger>
                            <SelectContent>
                              {CATEGORIES.find(c => c.code === formData.category)?.subcategories.map((sub) => (
                                <SelectItem key={sub.code} value={sub.code}>
                                  {sub.name}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                      )}
                    </div>
                  </CardContent>
                </Card>

                {/* Unit Selection */}
                <Card>
                  <CardContent className="pt-6">
                    <h3 className="text-lg font-semibold mb-4">包裝單位</h3>
                    <div className="space-y-4">
                      <div className="flex flex-wrap gap-2">
                        {currentUnits.map((unit) => (
                          <button
                            key={unit.code}
                            type="button"
                            onClick={() => setFormData(prev => ({ 
                              ...prev, 
                              unit: unit.code,
                              safetyStockUnit: unit.name
                            }))}
                            disabled={isCreated}
                            className={cn(
                              "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all",
                              formData.unit === unit.code
                                ? "border-primary bg-primary/10"
                                : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                            )}
                          >
                            <span className="font-mono font-semibold text-sm">{unit.code}</span>
                            <span className="text-xs text-slate-500">{unit.name}</span>
                          </button>
                        ))}
                      </div>
                      
                      <div className="flex items-center gap-4">
                        <Label className="shrink-0">每盒包裝量：</Label>
                        <Input
                          type="number"
                          min={1}
                          value={formData.packQty}
                          onChange={(e) => setFormData(prev => ({ ...prev, packQty: parseInt(e.target.value) || 1 }))}
                          className="w-24"
                          disabled={isCreated}
                        />
                        <span className="text-slate-500 text-sm">{formData.unit ? currentUnits.find(u => u.code === formData.unit)?.name + '/盒' : ''}</span>
                      </div>
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

                      {/* Safety Stock Slider */}
                      <Slider
                        value={formData.safetyStock}
                        onChange={(v) => setFormData(prev => ({ ...prev, safetyStock: v }))}
                        min={0}
                        max={500}
                        step={10}
                        quickValues={[50, 100, 200, 500]}
                        label="安全庫存"
                        unitOptions={currentUnits.map(u => ({ value: u.code, label: u.name }))}
                        selectedUnit={formData.unit || currentUnits[0]?.code}
                        onUnitChange={(u) => setFormData(prev => ({ ...prev, unit: u }))}
                        disabled={isCreated}
                      />

                      {/* Reorder Point */}
                      <div className="space-y-2">
                        <Label>補貨提醒點</Label>
                        <div className="flex items-center gap-3">
                          <span className="text-sm text-slate-500">當庫存低於</span>
                          <Input
                            type="number"
                            min={0}
                            value={formData.reorderPoint}
                            onChange={(e) => setFormData(prev => ({ ...prev, reorderPoint: parseInt(e.target.value) || 0 }))}
                            className="w-24"
                            disabled={isCreated}
                          />
                          <span className="text-sm text-slate-500">
                            {formData.unit ? currentUnits.find(u => u.code === formData.unit)?.name : '單位'}
                            時，發送補貨提醒
                          </span>
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
                    disabled={!formData.unit || isCreating || skuStatus !== 'S3'}
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
                          <span>{CATEGORIES.find(c => c.code === formData.category)?.name || '—'}</span>
                        </div>
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-slate-500">單位</span>
                          <span>{currentUnits.find(u => u.code === formData.unit)?.name || '—'}</span>
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

          {/* Right: SKU Preview */}
          <div className="lg:col-span-2">
            <div className="sticky top-6">
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
    </div>
  )
}
