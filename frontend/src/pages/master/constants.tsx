import { Pill, Syringe, Package, FlaskConical, Settings } from 'lucide-react'
import type { TFunction } from 'i18next'

import { formatUom } from '@/lib/utils'
import type { Step } from '@/components/product/StepIndicator'
import type { QuickSelectItem, QuickSelectSpec } from '@/components/product/QuickSelectCard'

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
// label = 按鈕顯示文字（隨 UI 語系）；value = 點按後寫入名稱欄的資料值（固定中文，使用者裁定 2026-09-19）
export function getQuickItems(t: TFunction): QuickSelectItem[] {
  const zh = { lng: 'zh-TW' }
  return [
    { id: 'glove', icon: '🧤', label: t('erpMaster.createProduct.quickItems.glove'), value: t('erpMaster.createProduct.quickItems.glove', zh) },
    { id: 'mask', icon: '😷', label: t('erpMaster.createProduct.quickItems.mask'), value: t('erpMaster.createProduct.quickItems.mask', zh) },
    { id: 'cotton', icon: '🏥', label: t('erpMaster.createProduct.quickItems.cotton'), value: t('erpMaster.createProduct.quickItems.cotton', zh) },
    { id: 'gauze', icon: '🩹', label: t('erpMaster.createProduct.quickItems.gauze'), value: t('erpMaster.createProduct.quickItems.gauze', zh) },
    { id: 'syringe', icon: '💉', label: t('erpMaster.createProduct.quickItems.syringe'), value: t('erpMaster.createProduct.quickItems.syringe', zh) },
    { id: 'alcohol', icon: '🧪', label: t('erpMaster.createProduct.quickItems.alcohol'), value: t('erpMaster.createProduct.quickItems.alcohol', zh) },
    {
      id: 'saline',
      icon: '💧',
      label: t('erpMaster.createProduct.quickItems.saline'),
      value: t('erpMaster.createProduct.quickItems.saline', zh),
      displayLabel: (<>{t('erpMaster.createProduct.quickItems.salineLine1')}<br />{t('erpMaster.createProduct.quickItems.salineLine2')}</>),
    },
  ]
}

// 手套規格
// primary/secondary = 按鈕顯示文字（隨 UI 語系）；value = 點按後寫入規格欄的資料值（固定中文，使用者裁定 2026-09-19）
export function getGloveSpecs(t: TFunction): QuickSelectSpec[] {
  const zh = { lng: 'zh-TW' }
  const size = (s: string) => t('erpMaster.createProduct.gloveSpecs.size', { size: s })
  const sizeZh = (s: string) => t('erpMaster.createProduct.gloveSpecs.size', { size: s, ...zh })
  const powderFree = t('erpMaster.createProduct.gloveSpecs.powderFree')
  const powdered = t('erpMaster.createProduct.gloveSpecs.powdered')
  const powderFreeZh = t('erpMaster.createProduct.gloveSpecs.powderFree', zh)
  const powderedZh = t('erpMaster.createProduct.gloveSpecs.powdered', zh)
  return [
    { id: 's-powder-free', primary: size('S'), secondary: powderFree, value: { primary: sizeZh('S'), secondary: powderFreeZh } },
    { id: 'm-powder-free', primary: size('M'), secondary: powderFree, value: { primary: sizeZh('M'), secondary: powderFreeZh } },
    { id: 'l-powder-free', primary: size('L'), secondary: powderFree, value: { primary: sizeZh('L'), secondary: powderFreeZh } },
    { id: 'xl-powder-free', primary: size('XL'), secondary: powderFree, value: { primary: sizeZh('XL'), secondary: powderFreeZh } },
    { id: 's-powdered', primary: size('S'), secondary: powdered, value: { primary: sizeZh('S'), secondary: powderedZh } },
    { id: 'm-powdered', primary: size('M'), secondary: powdered, value: { primary: sizeZh('M'), secondary: powderedZh } },
    { id: 'l-powdered', primary: size('L'), secondary: powdered, value: { primary: sizeZh('L'), secondary: powderedZh } },
    { id: 'xl-powdered', primary: size('XL'), secondary: powdered, value: { primary: sizeZh('XL'), secondary: powderedZh } },
  ]
}

// 品類圖示（顯示用，品類清單改由 API useSkuCategories 取得）
export const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  DRG: <Pill className="w-4 h-4" />,
  MED: <Syringe className="w-4 h-4" />,
  CON: <Package className="w-4 h-4" />,
  CHM: <FlaskConical className="w-4 h-4" />,
  EQP: <Settings className="w-4 h-4" />,
  GEN: <Package className="w-4 h-4" />,
}

// 單位定義
// ⚠️ `name` 是**寫入表單並隨 API 送出的中文單位值**（例：base_uom / pack_unit 存「個」「箱」），
// 也是 `selectedUnit === unit.name` 的比對依據，不是顯示文字，不可翻譯、不可隨語系變動。
// 畫面上要顯示單位名稱一律走 {@link unitDisplayName}。
export const UNITS = {
  outer: [
    { code: 'CTN', name: '箱' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'CASE', name: '件' },
  ],
  inner: [
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
  base: [
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
} as const

const ALL_UNITS = [...UNITS.outer, ...UNITS.inner, ...UNITS.base]

/**
 * 單位顯示名稱：`value` 可為單位代碼或 UNITS 的中文值；找得到就走 i18n（`uom.<code>`），
 * 找不到（自填量詞）原樣回傳。
 */
export function unitDisplayName(value: string): string {
  const found = ALL_UNITS.find(u => u.code === value || u.name === value)
  return found ? formatUom(found.code) : value
}

export interface ProductFormData {
  rawInput: string
  name: string
  spec: string
  category: string
  subcategory: string
  packagingLayers: 2 | 3
  outerUnit: string
  outerQty: number
  innerUnit: string
  innerQty: number
  baseUnit: string
  baseQty: number
  trackBatch: boolean
  trackExpiry: boolean
  currentStock: number
  currentStockUnit: string
  safetyStock: number
  safetyStockUnit: string
  reorderPoint: number
  reorderPointUnit: string
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
