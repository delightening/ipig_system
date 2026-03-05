import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Product } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { ArrowLeft, Loader2, Package, Pill, Syringe, FlaskConical, Settings } from 'lucide-react'
import { UOM_MAP } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { cn } from '@/lib/utils'

interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  pack_unit?: string
  pack_qty?: number
  default_expiry_days?: number
  safety_stock_uom?: string
  reorder_point_uom?: string
  barcode?: string
  storage_condition?: string
  license_no?: string
  remark?: string
  tags?: string[]
}

// 品類定義（與 CreateProductPage 一致，不另設「耗材(LAB)」主分類；實驗耗材為耗材子類）
const CATEGORIES = [
  { code: 'DRG', name: '藥品', icon: <Pill className="w-4 h-4" />, subcategories: [{ code: 'ABX', name: '抗生素' }, { code: 'ANL', name: '止痛藥' }, { code: 'VIT', name: '維生素' }, { code: 'OTH', name: '其他藥品' }] },
  { code: 'MED', name: '醫材', icon: <Syringe className="w-4 h-4" />, subcategories: [] },
  { code: 'CON', name: '耗材', icon: <Package className="w-4 h-4" />, subcategories: [{ code: 'GLV', name: '手套' }, { code: 'GAU', name: '紗布敷料' }, { code: 'CLN', name: '清潔消毒' }, { code: 'TAG', name: '標示耗材' }, { code: 'LAB', name: '實驗耗材' }, { code: 'OTH', name: '其他耗材' }] },
  { code: 'CHM', name: '化學品', icon: <FlaskConical className="w-4 h-4" />, subcategories: [{ code: 'RGT', name: '試劑' }, { code: 'SOL', name: '溶劑' }, { code: 'STD', name: '標準品' }, { code: 'OTH', name: '其他化學品' }] },
  { code: 'EQP', name: '設備', icon: <Settings className="w-4 h-4" />, subcategories: [{ code: 'INS', name: '儀器' }, { code: 'TOL', name: '工具' }, { code: 'PRT', name: '零件' }, { code: 'OTH', name: '其他設備' }] },
]

// 包裝單位選項（與 CreateProductPage 一致，用於編輯頁包裝結構）
const PACKAGING_UNITS = {
  outer: [{ code: 'CTN', name: '箱' }, { code: 'BX', name: '盒' }, { code: 'PK', name: '包' }, { code: 'CASE', name: '件' }],
  inner: [{ code: 'BX', name: '盒' }, { code: 'PK', name: '包' }, { code: 'EA', name: '個' }, { code: 'PC', name: '支' }, { code: 'PR', name: '雙' }, { code: 'BT', name: '瓶' }, { code: 'RL', name: '卷' }, { code: 'SET', name: '組' }, { code: 'TB', name: '錠' }, { code: 'CP', name: '膠囊' }],
  base: [{ code: 'EA', name: '個' }, { code: 'PC', name: '支' }, { code: 'PR', name: '雙' }, { code: 'BT', name: '瓶' }, { code: 'BX', name: '盒' }, { code: 'PK', name: '包' }, { code: 'RL', name: '卷' }, { code: 'SET', name: '組' }, { code: 'TB', name: '錠' }, { code: 'CP', name: '膠囊' }],
}

const STORAGE_CONDITIONS: Record<string, string> = {
  'RT': '常溫 (15-25°C)',
  'RF': '冷藏 (2-8°C)',
  'FZ': '冷凍 (-20°C 以下)',
  'DK': '避光',
  'DY': '乾燥',
}

const UOM_OPTIONS = Object.entries(UOM_MAP).map(([code, name]) => ({ code, name }))

// 由顯示名稱或代碼反查單位代碼（用於從 API 載入既有值）
function unitToCode(value: string | undefined): string {
  if (!value?.trim()) return ''
  if (UOM_MAP[value]) return value
  const entry = Object.entries(UOM_MAP).find(([, name]) => name === value)
  return entry ? entry[0] : value
}

export function ProductEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [name, setName] = useState('')
  const [spec, setSpec] = useState('')
  const [categoryCode, setCategoryCode] = useState('')
  const [subcategoryCode, setSubcategoryCode] = useState('')
  const [packUnit, setPackUnit] = useState('')
  const [packQty, setPackQty] = useState<number | ''>('')
  // 包裝結構（與新增產品一致：2 層或 3 層）
  const [packagingLayers, setPackagingLayers] = useState<2 | 3>(2)
  const [outerUnitCode, setOuterUnitCode] = useState('')
  const [outerQty, setOuterQty] = useState(1)
  const [innerUnitCode, setInnerUnitCode] = useState('')
  const [innerQty, setInnerQty] = useState(1)
  const [baseUnitCode, setBaseUnitCode] = useState('')
  const [baseQty, setBaseQty] = useState(1)
  const [trackBatch, setTrackBatch] = useState(false)
  const [trackExpiry, setTrackExpiry] = useState(false)
  const [defaultExpiryDays, setDefaultExpiryDays] = useState<number | ''>('')
  const [safetyStock, setSafetyStock] = useState<number | ''>('')
  const [safetyStockUom, setSafetyStockUom] = useState('')
  const [reorderPoint, setReorderPoint] = useState<number | ''>('')
  const [reorderPointUom, setReorderPointUom] = useState('')
  const [barcode, setBarcode] = useState('')
  const [storageCondition, setStorageCondition] = useState('')
  const [licenseNo, setLicenseNo] = useState('')
  const [remark, setRemark] = useState('')
  const [tagsInput, setTagsInput] = useState('')

  const { data: product, isLoading, error } = useQuery({
    queryKey: ['product', id],
    queryFn: async () => {
      const response = await api.get<ExtendedProduct>(`/products/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  useEffect(() => {
    if (product) {
      setName(product.name)
      setSpec(product.spec ?? '')
      // 舊資料可能為「耗材(LAB)」主分類，統一視為 耗材 + 子類 實驗耗材
      const catCode = product.category_code ?? ''
      const subCode = product.subcategory_code ?? ''
      if (catCode === 'LAB') {
        setCategoryCode('CON')
        setSubcategoryCode(subCode || 'LAB')
      } else {
        setCategoryCode(catCode)
        setSubcategoryCode(subCode)
      }
      setPackUnit(product.pack_unit ?? product.base_uom ?? '')
      setPackQty(product.pack_qty ?? '')
      // 包裝結構：由單一 pack_unit / pack_qty / base_uom 還原為 2 層（外層→內層=消耗單位）
      const pu = unitToCode(product.pack_unit || product.base_uom)
      const bu = unitToCode(product.base_uom)
      const pq = product.pack_qty ?? 1
      setPackagingLayers(2)
      setOuterUnitCode(pu)
      setOuterQty(1)
      setInnerUnitCode(bu)
      setInnerQty(pq)
      setBaseUnitCode(bu)
      setBaseQty(1)
      setTrackBatch(product.track_batch)
      setTrackExpiry(product.track_expiry)
      setDefaultExpiryDays(product.default_expiry_days ?? '')
      setSafetyStock(product.safety_stock != null ? Number(product.safety_stock) : '')
      setSafetyStockUom(product.safety_stock_uom ?? product.base_uom ?? '')
      setReorderPoint(product.reorder_point != null ? Number(product.reorder_point) : '')
      setReorderPointUom(product.reorder_point_uom ?? product.base_uom ?? '')
      setBarcode(product.barcode ?? '')
      setStorageCondition(product.storage_condition ?? '')
      setLicenseNo(product.license_no ?? '')
      setRemark(product.remark ?? '')
      setTagsInput(product.tags?.join(', ') ?? '')
    }
  }, [product])

  const updateMutation = useMutation({
    mutationFn: async () => {
      const categoryData = CATEGORIES.find(c => c.code === categoryCode)
      const hasSubcategories = (categoryData?.subcategories?.length ?? 0) > 0
      const subCode = hasSubcategories ? subcategoryCode : categoryCode
      // 依包裝層數計算 pack_unit / pack_qty（與新增產品邏輯一致）
      const computedPackUnit = packagingLayers === 2
        ? (outerUnitCode || innerUnitCode)
        : innerUnitCode
      const computedPackQty = packagingLayers === 2
        ? (outerUnitCode ? innerQty : 1)
        : (innerQty * baseQty)
      return api.put(`/products/${id}`, {
        name: name.trim() || undefined,
        spec: spec.trim() || undefined,
        category_code: categoryCode || undefined,
        subcategory_code: subCode || undefined,
        pack_unit: computedPackUnit || undefined,
        pack_qty: computedPackQty || undefined,
        track_batch: trackBatch,
        track_expiry: trackExpiry,
        default_expiry_days: defaultExpiryDays === '' ? undefined : Number(defaultExpiryDays),
        safety_stock: safetyStock === '' ? undefined : Number(safetyStock),
        safety_stock_uom: safetyStockUom || undefined,
        reorder_point: reorderPoint === '' ? undefined : Number(reorderPoint),
        reorder_point_uom: reorderPointUom || undefined,
        barcode: barcode.trim() || undefined,
        storage_condition: storageCondition || undefined,
        license_no: licenseNo.trim() || undefined,
        remark: remark.trim() || undefined,
        tags: tagsInput.trim() ? tagsInput.split(/,\s*/).map(t => t.trim()).filter(Boolean) : undefined,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['product', id] })
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '產品已更新', description: '變更已儲存' })
      navigate(`/products/${id}`)
    },
    onError: (error: unknown) => {
      toast({
        title: '更新失敗',
        description: getApiErrorMessage(error, '儲存時發生錯誤'),
        variant: 'destructive',
      })
    },
  })

  const currentCategory = CATEGORIES.find(c => c.code === categoryCode)
  const subcategories = currentCategory?.subcategories ?? []

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) {
      toast({ title: '請輸入產品名稱', variant: 'destructive' })
      return
    }
    updateMutation.mutate()
  }

  if (isLoading) {
    return (
      <div className="flex justify-center items-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error || !product) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <Package className="h-12 w-12 text-muted-foreground mb-4" />
        <h2 className="text-xl font-semibold mb-2">找不到產品</h2>
        <p className="text-muted-foreground mb-4">該產品可能已被刪除或不存在</p>
        <Button variant="outline" onClick={() => navigate('/products')}>返回產品列表</Button>
      </div>
    )
  }

  return (
    <div className="container max-w-3xl py-6">
      <div className="flex items-center gap-4 mb-6">
        <Button variant="ghost" size="icon" onClick={() => navigate(`/products/${id}`)}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">編輯產品</h1>
          <p className="text-muted-foreground text-sm">
            SKU: {product.sku}（唯讀，不可修改）
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <div className="grid gap-6">
          {/* 基本資訊 */}
          <Card>
            <CardHeader>
              <CardTitle>基本資訊</CardTitle>
              <CardDescription>產品名稱、規格、分類等</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="name">產品名稱 *</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="例：紗布"
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="spec">規格描述</Label>
                <Input
                  id="spec"
                  value={spec}
                  onChange={(e) => setSpec(e.target.value)}
                  placeholder="例：4x4"
                />
              </div>
              <div className="grid gap-2">
                <Label>分類（與新增產品一致）</Label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  {CATEGORIES.slice(0, 4).map((cat) => (
                    <button
                      key={cat.code}
                      type="button"
                      onClick={() => {
                        setCategoryCode(cat.code)
                        setSubcategoryCode((cat as { subcategories?: { code: string }[] }).subcategories?.length ? '' : cat.code)
                      }}
                      className={cn(
                        'flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all',
                        categoryCode === cat.code
                          ? 'border-primary bg-primary/5'
                          : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                      )}
                    >
                      <div className={cn(
                        'p-2 rounded-md',
                        categoryCode === cat.code ? 'bg-primary/10 text-primary' : 'bg-slate-100 dark:bg-slate-800'
                      )}>
                        {(cat as { icon?: React.ReactNode }).icon}
                      </div>
                      <span className="font-medium">{cat.name}</span>
                    </button>
                  ))}
                </div>
              </div>
              {subcategories.length > 0 && (
                <div className="grid gap-2">
                  <Label>子分類</Label>
                  <Select value={subcategoryCode} onValueChange={setSubcategoryCode}>
                    <SelectTrigger>
                      <SelectValue placeholder="選擇子分類" />
                    </SelectTrigger>
                    <SelectContent>
                      {subcategories.map(s => (
                        <SelectItem key={s.code} value={s.code}>{s.name}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}
              <div className="grid gap-2">
                <Label>庫存單位（消耗單位，唯讀）</Label>
                <Input value={`${product.base_uom} (${UOM_MAP[product.base_uom] || product.base_uom})`} disabled className="bg-muted" />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="barcode">原廠條碼</Label>
                <Input id="barcode" value={barcode} onChange={(e) => setBarcode(e.target.value)} placeholder="選填" />
              </div>
              <div className="grid gap-2">
                <Label>保存條件</Label>
                <Select
                  value={storageCondition || '__none__'}
                  onValueChange={(v) => setStorageCondition(v === '__none__' ? '' : v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="選填" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">不設定</SelectItem>
                    {Object.entries(STORAGE_CONDITIONS).map(([code, label]) => (
                      <SelectItem key={code} value={code}>{label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="licenseNo">許可證號</Label>
                <Input id="licenseNo" value={licenseNo} onChange={(e) => setLicenseNo(e.target.value)} placeholder="選填" />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="tags">搜尋標籤（逗號分隔）</Label>
                <Input id="tags" value={tagsInput} onChange={(e) => setTagsInput(e.target.value)} placeholder="例：敷料, 急救" />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="remark">備註</Label>
                <Input id="remark" value={remark} onChange={(e) => setRemark(e.target.value)} placeholder="選填" />
              </div>
            </CardContent>
          </Card>

          {/* 包裝結構（一箱幾盒、一盒幾個，與新增產品一致） */}
          <Card>
            <CardHeader>
              <CardTitle>包裝結構</CardTitle>
              <CardDescription>外層→內層→基礎單位（消耗單位），編輯時可檢視與修改</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                <Label className="text-sm font-medium">包裝層數</Label>
                <div className="flex gap-3">
                  <button
                    type="button"
                    onClick={() => setPackagingLayers(2)}
                    className={cn(
                      'flex-1 p-3 rounded-lg border-2 transition-all text-left',
                      packagingLayers === 2 ? 'border-primary bg-primary/10 text-primary font-medium' : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                    )}
                  >
                    兩層包裝
                    <span className="block text-xs mt-1 text-slate-500">外層 → 內層（消耗每內層）</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => setPackagingLayers(3)}
                    className={cn(
                      'flex-1 p-3 rounded-lg border-2 transition-all text-left',
                      packagingLayers === 3 ? 'border-primary bg-primary/10 text-primary font-medium' : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                    )}
                  >
                    三層包裝
                    <span className="block text-xs mt-1 text-slate-500">外層 → 內層 → 基礎（消耗每基礎）</span>
                  </button>
                </div>
              </div>

              {(packagingLayers === 2 || packagingLayers === 3) && (
                <div className="space-y-4">
                  {/* 外層包裝 */}
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">外層包裝</Label>
                    <div className="flex flex-wrap gap-2">
                      {PACKAGING_UNITS.outer.map((u) => (
                        <button
                          key={u.code}
                          type="button"
                          onClick={() => setOuterUnitCode(outerUnitCode === u.code ? '' : u.code)}
                          className={cn(
                            'flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all',
                            outerUnitCode === u.code ? 'border-primary bg-primary/10' : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                          )}
                        >
                          <span className="font-mono text-sm font-semibold">{u.code}</span>
                          <span className="text-xs text-slate-500">{u.name}</span>
                        </button>
                      ))}
                    </div>
                    {outerUnitCode && (
                      <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 w-fit">
                        <span className="text-sm text-slate-500">1</span>
                        <span className="text-sm">{UOM_MAP[outerUnitCode] || outerUnitCode}</span>
                      </div>
                    )}
                  </div>

                  {/* 內層包裝 */}
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">
                      內層包裝
                      {packagingLayers === 2 && <span className="text-xs text-slate-400 ml-2">（消耗單位）</span>}
                    </Label>
                    <div className="flex flex-wrap gap-2">
                      {PACKAGING_UNITS.inner.map((u) => (
                        <button
                          key={u.code}
                          type="button"
                          onClick={() => {
                            setInnerUnitCode(u.code)
                            if (packagingLayers === 2) setBaseUnitCode(u.code)
                          }}
                          className={cn(
                            'flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all',
                            innerUnitCode === u.code ? 'border-primary bg-primary/10' : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                          )}
                        >
                          <span className="font-mono text-sm font-semibold">{u.code}</span>
                          <span className="text-xs text-slate-500">{u.name}</span>
                        </button>
                      ))}
                    </div>
                    {innerUnitCode && (
                      <div className="flex items-center gap-2 flex-wrap">
                        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 w-fit">
                          <span className="text-sm text-slate-500">{outerUnitCode ? `一${UOM_MAP[outerUnitCode] || outerUnitCode}` : '一'}</span>
                          <Input
                            type="number"
                            min={1}
                            className="w-16 h-8 text-center"
                            value={innerQty}
                            onChange={(e) => setInnerQty(Math.max(1, parseInt(e.target.value, 10) || 1))}
                          />
                          <span className="text-sm">{UOM_MAP[innerUnitCode] || innerUnitCode}</span>
                        </div>
                      </div>
                    )}
                  </div>

                  {/* 基礎單位（僅三層） */}
                  {packagingLayers === 3 && (
                    <div className="space-y-2">
                      <Label className="text-sm font-medium">基礎單位（消耗單位，庫存管理）</Label>
                      <div className="flex flex-wrap gap-2">
                        {PACKAGING_UNITS.base.map((u) => (
                          <button
                            key={u.code}
                            type="button"
                            onClick={() => setBaseUnitCode(u.code)}
                            className={cn(
                              'flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all',
                              baseUnitCode === u.code ? 'border-primary bg-primary/10' : 'border-slate-200 dark:border-slate-700 hover:border-primary/50'
                            )}
                          >
                            <span className="font-mono text-sm font-semibold">{u.code}</span>
                            <span className="text-xs text-slate-500">{u.name}</span>
                          </button>
                        ))}
                      </div>
                      {innerUnitCode && baseUnitCode && (
                        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 w-fit">
                          <span className="text-sm text-slate-500">一{UOM_MAP[innerUnitCode] || innerUnitCode}</span>
                          <Input
                            type="number"
                            min={1}
                            className="w-16 h-8 text-center"
                            value={baseQty}
                            onChange={(e) => setBaseQty(Math.max(1, parseInt(e.target.value, 10) || 1))}
                          />
                          <span className="text-sm">{UOM_MAP[baseUnitCode] || baseUnitCode}</span>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )}
            </CardContent>
          </Card>

          {/* 追蹤設定 */}
          <Card>
            <CardHeader>
              <CardTitle>追蹤設定</CardTitle>
              <CardDescription>批號、效期追蹤</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="trackBatch"
                  checked={trackBatch}
                  onChange={(e) => setTrackBatch(e.target.checked)}
                  className="rounded"
                />
                <Label htmlFor="trackBatch">追蹤批號</Label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="trackExpiry"
                  checked={trackExpiry}
                  onChange={(e) => setTrackExpiry(e.target.checked)}
                  className="rounded"
                />
                <Label htmlFor="trackExpiry">追蹤效期</Label>
              </div>
              {trackExpiry && (
                <div className="grid gap-2">
                  <Label htmlFor="defaultExpiryDays">預設有效天數</Label>
                  <Input
                    id="defaultExpiryDays"
                    type="number"
                    min={1}
                    value={defaultExpiryDays}
                    onChange={(e) => setDefaultExpiryDays(e.target.value === '' ? '' : parseInt(e.target.value, 10))}
                    placeholder="例：365"
                  />
                </div>
              )}
            </CardContent>
          </Card>

          {/* 庫存設定 */}
          <Card>
            <CardHeader>
              <CardTitle>庫存管理設定</CardTitle>
              <CardDescription>安全庫存與補貨點</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="safetyStock">安全庫存</Label>
                  <Input
                    id="safetyStock"
                    type="number"
                    min={0}
                    value={safetyStock}
                    onChange={(e) => setSafetyStock(e.target.value === '' ? '' : parseFloat(e.target.value))}
                    placeholder="選填"
                  />
                </div>
                <div className="grid gap-2">
                  <Label>安全庫存單位</Label>
                  <Select value={safetyStockUom} onValueChange={setSafetyStockUom}>
                    <SelectTrigger>
                      <SelectValue placeholder="選擇單位" />
                    </SelectTrigger>
                    <SelectContent>
                      {UOM_OPTIONS.map(u => (
                        <SelectItem key={u.code} value={u.code}>{u.name}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="reorderPoint">補貨點</Label>
                  <Input
                    id="reorderPoint"
                    type="number"
                    min={0}
                    value={reorderPoint}
                    onChange={(e) => setReorderPoint(e.target.value === '' ? '' : parseFloat(e.target.value))}
                    placeholder="選填"
                  />
                </div>
                <div className="grid gap-2">
                  <Label>補貨點單位</Label>
                  <Select value={reorderPointUom} onValueChange={setReorderPointUom}>
                    <SelectTrigger>
                      <SelectValue placeholder="選擇單位" />
                    </SelectTrigger>
                    <SelectContent>
                      {UOM_OPTIONS.map(u => (
                        <SelectItem key={u.code} value={u.code}>{u.name}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 送出 */}
          <div className="flex gap-2">
            <Button type="submit" disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              儲存變更
            </Button>
            <Button type="button" variant="outline" onClick={() => navigate(`/products/${id}`)}>
              取消
            </Button>
          </div>
        </div>
      </form>
    </div>
  )
}
