import { Pill, Syringe, Package, FlaskConical, Settings } from 'lucide-react'
import { createElement } from 'react'
import type { TFunction } from 'i18next'
import type { Step } from '@/components/product/StepIndicator'
import type { QuickSelectItem, QuickSelectSpec } from '@/components/product/QuickSelectCard'

// 表單資料型別
export interface ProductFormData {
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

export const initialFormData: ProductFormData = {
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

// 步驟定義（顯示文字在呼叫當下才翻譯，語系切換後重繪即更新）
export function getSteps(t: TFunction): Step[] {
  return [
    {
      id: 'input',
      label: t('erpMaster.createProduct.steps.inputLabel'),
      description: t('erpMaster.createProduct.steps.inputDescription'),
    },
    {
      id: 'confirm',
      label: t('erpMaster.createProduct.steps.confirmLabel'),
      description: t('erpMaster.createProduct.steps.confirmDescription'),
    },
    {
      id: 'complete',
      label: t('erpMaster.createProduct.steps.completeLabel'),
      description: t('erpMaster.createProduct.steps.completeDescription'),
    },
  ]
}

// 快速選擇品項
export function getQuickItems(t: TFunction): QuickSelectItem[] {
  return [
    { id: 'glove', icon: '🧤', label: t('erpMaster.createProduct.quickItems.glove') },
    { id: 'mask', icon: '😷', label: t('erpMaster.createProduct.quickItems.mask') },
    { id: 'syringe', icon: '💉', label: t('erpMaster.createProduct.quickItems.needleSyringe') },
    { id: 'gauze', icon: '🩹', label: t('erpMaster.createProduct.quickItems.gauze') },
    { id: 'cotton', icon: '⚪', label: t('erpMaster.createProduct.quickItems.cottonBall') },
    { id: 'alcohol', icon: '🧴', label: t('erpMaster.createProduct.quickItems.alcohol') },
    { id: 'saline', icon: '💧', label: t('erpMaster.createProduct.quickItems.saline') },
  ]
}

// 手套規格
export function getGloveSpecs(t: TFunction): QuickSelectSpec[] {
  const size = (s: string) => t('erpMaster.createProduct.gloveSpecs.size', { size: s })
  const powderFree = t('erpMaster.createProduct.gloveSpecs.powderFree')
  const powdered = t('erpMaster.createProduct.gloveSpecs.powdered')
  return [
    { id: 's-powder-free', primary: size('S'), secondary: powderFree },
    { id: 'm-powder-free', primary: size('M'), secondary: powderFree },
    { id: 'l-powder-free', primary: size('L'), secondary: powderFree },
    { id: 'xl-powder-free', primary: size('XL'), secondary: powderFree },
    { id: 's-powdered', primary: size('S'), secondary: powdered },
    { id: 'm-powdered', primary: size('M'), secondary: powdered },
    { id: 'l-powdered', primary: size('L'), secondary: powdered },
    { id: 'xl-powdered', primary: size('XL'), secondary: powdered },
  ]
}

// 品類圖示（顯示用，品類清單改由 API useSkuCategories 取得）
export const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  DRG: createElement(Pill, { className: 'w-4 h-4' }),
  MED: createElement(Syringe, { className: 'w-4 h-4' }),
  CON: createElement(Package, { className: 'w-4 h-4' }),
  CHM: createElement(FlaskConical, { className: 'w-4 h-4' }),
  EQP: createElement(Settings, { className: 'w-4 h-4' }),
  GEN: createElement(Package, { className: 'w-4 h-4' }),
}

// 單位定義
// ⚠️ `name` 是寫入表單並隨 API 送出的中文單位值（與 pages/master/constants.tsx 的 UNITS 相同），
// 也是比對依據，不是顯示文字，不可翻譯。
export const UNITS = {
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

// 取得可用的包裝單位選項（用於庫存相關 Select）
export function getAvailableUnits(
  formData: ProductFormData
): Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> {
  const units: Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> = []
  const allUnits = [...UNITS.outer, ...UNITS.inner, ...UNITS.base]

  // 外層包裝
  if (formData.outerUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.outerUnit || unit.code === formData.outerUnit
    )
    if (found) {
      units.push({ code: found.code, name: found.name, type: 'outer' })
    }
  }

  // 內層包裝
  if (formData.innerUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.innerUnit || unit.code === formData.innerUnit
    )
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'inner' })
    }
  }

  // 基礎單位（僅三層包裝時顯示）
  if (formData.packagingLayers === 3 && formData.baseUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.baseUnit || unit.code === formData.baseUnit
    )
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'base' })
    }
  }

  return units
}

// 取得補貨提醒點可用單位（包含基礎單位不限層數）
export function getReorderPointUnits(
  formData: ProductFormData
): Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> {
  const units: Array<{ code: string; name: string; type: 'outer' | 'inner' | 'base' }> = []
  const allUnits = [...UNITS.outer, ...UNITS.inner, ...UNITS.base]

  if (formData.outerUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.outerUnit || unit.code === formData.outerUnit
    )
    if (found) {
      units.push({ code: found.code, name: found.name, type: 'outer' })
    }
  }

  if (formData.innerUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.innerUnit || unit.code === formData.innerUnit
    )
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'inner' })
    }
  }

  if (formData.baseUnit) {
    const found = allUnits.find(
      unit => unit.name === formData.baseUnit || unit.code === formData.baseUnit
    )
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'base' })
    }
  }

  return units
}
