import { useState, useCallback, useMemo, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'

import api from '@/lib/api'
import { useSteps } from '@/hooks/useSteps'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useDebounce } from '@/hooks/useDebounce'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { getApiErrorMessage } from '@/lib/validation'
import type { SkuStatus, SkuPreviewResult, SkuPreviewError, MissingField } from '@/components/sku/SkuPreviewBlock'
import type { ProductSuggestion } from '@/components/product/SmartInput'
import type { QuickSelectItem, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { UNITS, initialFormData } from '../constants'
import type { ProductFormData } from '../constants'

export function useCreateProductForm() {
  const navigate = useNavigate()
  const { step: currentStep, setStep: setCurrentStep, prev } = useSteps(3)
  const { categories: skuCategories, subcategoriesByCategory, isLoading: skuCategoriesLoading } = useSkuCategories()

  // Form state
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

  // Custom unit state
  const [isOuterCustom, setIsOuterCustom] = useState(false)
  const [isInnerCustom, setIsInnerCustom] = useState(false)
  const [isBaseCustom, setIsBaseCustom] = useState(false)
  const [customOuter, setCustomOuter] = useState('')
  const [customInner, setCustomInner] = useState('')
  const [customBase, setCustomBase] = useState('')

  // Derived: display categories (exclude GEN, take first 5)
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

  const debouncedInput = useDebounce(formData.rawInput, 400)

  // Missing fields for SKU preview
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

  // Parse raw input into name + spec
  const parseInput = useCallback((input: string) => {
    const parts = input.trim().split(/\s+/)
    if (parts.length === 0) return { name: '', spec: '' }
    return { name: parts[0], spec: parts.slice(1).join(' ') }
  }, [])

  // Generate SKU preview
  const generatePreview = useCallback(async () => {
    if (!canPreview) {
      setSkuStatus('S0')
      setPreviewResult(null)
      return
    }

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
          { code: 'CATEGORY', label: '種類', value: category, source: catOption?.name ?? formData.category },
          { code: 'ITEM', label: '品項', value: subcategory, source: subOption?.name ?? formData.subcategory ?? category },
          { code: 'SERIAL', label: '流水號', value: 'XXX', source: '自動遞增序號' },
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

  // Auto-clear subcategory when switching to a category without subcategories
  useEffect(() => {
    if (formData.category && !hasSubcategories(formData.category) && formData.subcategory) {
      setFormData(prev => ({ ...prev, subcategory: '' }))
    }
  }, [formData.category, formData.subcategory, hasSubcategories])

  // Auto-preview on input changes
  useEffect(() => {
    if (currentStep === 1 && skuStatus !== 'S5' && skuStatus !== 'S6') {
      generatePreview()
    }
  }, [debouncedInput, formData.baseUnit, formData.category, formData.subcategory, formData.name, currentStep, generatePreview, skuStatus])

  // Handlers
  const handleInputChange = (value: string) => {
    setFormData(prev => ({ ...prev, rawInput: value }))
    const parsed = parseInput(value)
    setFormData(prev => ({ ...prev, name: parsed.name, spec: parsed.spec }))

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

  const handleSelectSuggestion = (suggestion: ProductSuggestion) => {
    setFormData(prev => ({
      ...prev,
      rawInput: `${suggestion.name} ${suggestion.spec}`,
      name: suggestion.name,
      spec: suggestion.spec,
    }))
    setSuggestions([])
  }

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

  // Create product mutation
  const createMutation = useMutation({
    mutationFn: async () => {
      setSkuStatus('S5')

      const consumptionUnit = formData.packagingLayers === 2
        ? (formData.innerUnit || formData.baseUnit || 'EA')
        : (formData.baseUnit || 'EA')

      const packUnit = formData.packagingLayers === 2
        ? (formData.outerUnit || formData.innerUnit || consumptionUnit)
        : (formData.innerUnit || formData.baseUnit || consumptionUnit)

      const packQty = formData.packagingLayers === 2
        ? (formData.outerUnit ? formData.innerQty : 1)
        : (formData.innerQty * formData.baseQty)

      const subcategoryCode = hasSubcategories(formData.category)
        ? formData.subcategory
        : formData.category

      const response = await api.post('/products', {
        name: formData.name || formData.rawInput.split(' ')[0],
        spec: formData.spec,
        base_uom: consumptionUnit,
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
      toast({ title: '產品建立成功！', description: `SKU: ${data.sku}` })
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

  const handleNext = () => {
    if (currentStep === 0) {
      if (!formData.rawInput && !formData.name) {
        toast({ title: '請輸入產品名稱', variant: 'destructive' })
        return
      }
      setCurrentStep(1)
      generatePreview()
    } else if (currentStep === 1) {
      if (!formData.baseUnit) {
        toast({ title: '請選擇基礎單位', variant: 'destructive' })
        return
      }
      createMutation.mutate()
    }
  }

  const handleBack = () => { prev() }

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

  return {
    // Navigation
    navigate,
    currentStep,
    setCurrentStep,

    // SKU categories
    skuCategories,
    skuCategoriesLoading,
    displayCategories,
    hasSubcategories,
    getSubcategories,

    // Form
    formData,
    setFormData,

    // SKU preview
    skuStatus,
    previewResult,
    previewError,
    finalSku,
    isPreviewLoading,
    missingFields,
    generatePreview,

    // Suggestions
    suggestions,
    isSuggestionsLoading,

    // Quick select
    selectedQuickItem,
    selectedSpec,
    glovesMaterial,
    setGlovesMaterial,
    quickMode,
    setQuickMode,

    // Custom units
    isOuterCustom, setIsOuterCustom,
    isInnerCustom, setIsInnerCustom,
    isBaseCustom, setIsBaseCustom,
    customOuter, setCustomOuter,
    customInner, setCustomInner,
    customBase, setCustomBase,

    // Handlers
    handleInputChange,
    handleSelectSuggestion,
    handleQuickItemSelect,
    handleSpecSelect,
    handleNext,
    handleBack,
    handleReset,

    // Status
    isCreating,
    isCreated,
  }
}

export type CreateProductFormReturn = ReturnType<typeof useCreateProductForm>
