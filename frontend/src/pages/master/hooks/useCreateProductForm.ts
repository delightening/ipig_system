import { useState, useCallback, useMemo, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { useSteps } from '@/hooks/useSteps'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useDebounce } from '@/hooks/useDebounce'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { getApiErrorMessage } from '@/lib/apiError'
import type { SkuStatus, SkuPreviewResult, SkuPreviewError, MissingField } from '@/components/sku/SkuPreviewBlock'
import type { ProductSuggestion } from '@/components/product/SmartInput'
import type { QuickSelectItem, QuickSelectSpec } from '@/components/product/QuickSelectCard'
import { initialFormData } from '../constants'
import type { ProductFormData } from '../constants'

/**
 * R84-3：依 SKU 類別預填批號/效期追蹤旗標的「起點預設值」。
 * 對齊 migration 138 與 `docs/spec/modules/ERP流程.md` §6.2.1：
 * DRG/MED/CON/CHM 天生有批號效期 → 預設開；EQP/GEN 無此概念 → 預設關。
 * 僅為起點，使用者在庫存設定卡仍可逐項再調整。
 */
const TRACK_DEFAULT_BY_CATEGORY: Record<string, boolean> = {
  DRG: true,
  MED: true,
  CON: true,
  CHM: true,
  EQP: false,
  GEN: false,
}

export function useCreateProductForm() {
  const { t } = useTranslation()
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
    if (!formData.name && !formData.rawInput) fields.push({ field: 'name', label: t('erpMaster.productDetail.productName') })
    if (!formData.category) fields.push({ field: 'category', label: t('erpMaster.createProduct.category') })
    const categoryHasSubcategories = formData.category && hasSubcategories(formData.category)
    if (!formData.subcategory && formData.category && categoryHasSubcategories) {
      fields.push({ field: 'subcategory', label: t('erpMaster.createProduct.subcategory') })
    }
    if (!formData.baseUnit) fields.push({ field: 'baseUnit', label: t('erpMaster.createProduct.baseUnit') })
    return fields
  }, [formData, hasSubcategories, t])

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
        rule_change_summary: t('erpMaster.skuPreview.ruleChangeSummary'),
        segments: [
          { code: 'CATEGORY', label: t('erpMaster.skuPreview.segment.CATEGORY'), value: category, source: catOption?.name ?? formData.category },
          { code: 'ITEM', label: t('erpMaster.skuPreview.segment.ITEM'), value: subcategory, source: subOption?.name ?? formData.subcategory ?? category },
          { code: 'SERIAL', label: t('erpMaster.skuPreview.segment.SERIAL'), value: 'XXX', source: t('erpMaster.skuPreview.serialSource') },
        ],
      }

      setPreviewResult(result)
      setSkuStatus('S3')
    } catch (error: unknown) {
      logger.error('SKU preview error:', error)
      setSkuStatus('S4')
      setPreviewError({
        code: 'E5',
        message: getApiErrorMessage(error, t('erpMaster.skuPreview.previewFailedMessage')),
        suggestion: t('erpMaster.skuPreview.previewFailedSuggestion'),
      })
    } finally {
      setIsPreviewLoading(false)
    }
  }, [canPreview, formData.category, formData.subcategory, formData.baseUnit, formData.name, hasSubcategories, getSubcategories, skuCategories, t])

  // Auto-clear subcategory when switching to a category without subcategories
  useEffect(() => {
    if (formData.category && !hasSubcategories(formData.category) && formData.subcategory) {
      setFormData(prev => ({ ...prev, subcategory: '' }))
    }
  }, [formData.category, formData.subcategory, hasSubcategories])

  // R84-3: 選定 SKU 類別時，把批號/效期追蹤旗標預填為該類別的起點預設值。
  // 僅在類別變動時套用（換類別＝重設為該類別預設）；使用者之後手動調整的值會保留，
  // 直到再次切換類別。對齊 migration 138 與 ERP流程.md §6.2.1。
  useEffect(() => {
    if (!formData.category) return
    const trackDefault = TRACK_DEFAULT_BY_CATEGORY[formData.category]
    if (trackDefault === undefined) return
    setFormData(prev => ({ ...prev, trackBatch: trackDefault, trackExpiry: trackDefault }))
  }, [formData.category])

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
    // 寫進資料的名稱用固定中文 value；label 只是按鈕顯示文字（使用者裁定 2026-09-19）
    const itemName = item.value ?? item.label
    setFormData(prev => ({
      ...prev,
      rawInput: itemName,
      name: itemName,
      spec: '',
      category: category,
      subcategory: item.id === 'glove' ? 'GLV' : '',
    }))
  }

  const handleSpecSelect = (spec: QuickSelectSpec) => {
    setSelectedSpec(spec)
    if (selectedQuickItem) {
      // 寫進資料的規格用固定中文 value；primary/secondary 只是按鈕顯示文字（使用者裁定 2026-09-19）
      const specValue = spec.value ?? { primary: spec.primary, secondary: spec.secondary }
      const fullSpec = selectedQuickItem.id === 'glove'
        ? `${specValue.primary} ${specValue.secondary} ${glovesMaterial}`
        : `${specValue.primary}${specValue.secondary ? ' ' + specValue.secondary : ''}`
      setFormData(prev => ({
        ...prev,
        rawInput: `${selectedQuickItem.value ?? selectedQuickItem.label} ${fullSpec}`,
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
      toast({ title: t('erpMaster.createProduct.created'), description: `SKU: ${data.sku}` })
    },
    onError: (error: unknown) => {
      setSkuStatus('S3')
      toast({
        title: t('erpMaster.createProduct.toast.createFailed'),
        description: getApiErrorMessage(error, t('erpMaster.createProduct.toast.createError')),
        variant: 'destructive',
      })
    },
  })

  const handleNext = () => {
    if (currentStep === 0) {
      if (!formData.rawInput && !formData.name) {
        toast({ title: t('erpMaster.createProduct.toast.enterName'), variant: 'destructive' })
        return
      }
      setCurrentStep(1)
      generatePreview()
    } else if (currentStep === 1) {
      if (!formData.baseUnit) {
        toast({ title: t('erpMaster.createProduct.toast.selectBaseUnit'), variant: 'destructive' })
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
