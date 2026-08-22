import { Pill, Syringe, Package, FlaskConical, Settings } from 'lucide-react'

import type { Step } from '@/components/product/StepIndicator'
import type { QuickSelectItem, QuickSelectSpec } from '@/components/product/QuickSelectCard'

// 步驟定義
export const STEPS: Step[] = [
  { id: 'input', label: '輸入名稱', description: '名稱 + 規格' },
  { id: 'confirm', label: '確認規格', description: '分類 + 單位' },
  { id: 'complete', label: '完成建立', description: '檢視結果' },
]

// 快速選擇品項
export const QUICK_ITEMS: QuickSelectItem[] = [
  { id: 'glove', icon: '🧤', label: '手套' },
  { id: 'mask', icon: '😷', label: '口罩' },
  { id: 'cotton', icon: '🏥', label: '棉棒' },
  { id: 'gauze', icon: '🩹', label: '紗布' },
  { id: 'syringe', icon: '💉', label: '注射器' },
  { id: 'alcohol', icon: '🧪', label: '酒精' },
  { id: 'saline', icon: '💧', label: '生理食鹽水', displayLabel: (<>生理<br />食鹽水</>) },
]

// 手套規格
export const GLOVE_SPECS: QuickSelectSpec[] = [
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
export const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  DRG: <Pill className="w-4 h-4" />,
  MED: <Syringe className="w-4 h-4" />,
  CON: <Package className="w-4 h-4" />,
  CHM: <FlaskConical className="w-4 h-4" />,
  EQP: <Settings className="w-4 h-4" />,
  GEN: <Package className="w-4 h-4" />,
}

// 單位定義
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
