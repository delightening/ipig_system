import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Product } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Edit,
  MoreHorizontal,
  Loader2,
  Package,
  ClipboardCopy,
  Boxes,
  Calendar,
  User,
  FileText,
  History,
} from 'lucide-react'
import { formatDateTime, formatNumber } from '@/lib/utils'

// 品類定義
const CATEGORIES: Record<string, string> = {
  'DRG': '藥品',
  'MED': '醫材',
  'CON': '耗材',
  'CHM': '化學品',
  'EQP': '設備',
  // 向後兼容：LAB 映射為耗材
  'LAB': '耗材',
}

const SUBCATEGORIES: Record<string, Record<string, string>> = {
  'DRG': { 'ABX': '抗生素', 'ANL': '止痛藥', 'VIT': '維生素', 'OTH': '其他藥品' },
  'MED': { 'SYR': '注射器材', 'BND': '敷料繃帶', 'GLV': '手套', 'OTH': '其他醫材' },
  'CON': { 'GLV': '手套', 'GAU': '紗布敷料', 'CLN': '清潔消毒', 'TAG': '標示耗材', 'LAB': '實驗耗材', 'OTH': '其他耗材' },
  'CHM': { 'RGT': '試劑', 'SOL': '溶劑', 'STD': '標準品', 'OTH': '其他化學品' },
  'EQP': { 'INS': '儀器', 'TOL': '工具', 'PRT': '零件', 'OTH': '其他設備' },
  // 向後兼容：LAB 主分類的子分類映射到 CON
  'LAB': { 'TUB': '試管', 'PIP': '吸管', 'PLT': '培養皿', 'OTH': '其他耗材' },
}

const STORAGE_CONDITIONS: Record<string, string> = {
  'RT': '常溫 (15-25°C)',
  'RF': '冷藏 (2-8°C)',
  'FZ': '冷凍 (-20°C 以下)',
  'DK': '避光',
  'DY': '乾燥',
}

const UOM_MAP: Record<string, string> = {
  'EA': '個/支',
  'TB': '錠',
  'CP': '顆/膠囊',
  'BT': '瓶',
  'BX': '盒',
  'PK': '包',
  'RL': '卷',
  'SET': '組',
  'ML': '毫升',
  'L': '公升',
  'G': '公克',
  'KG': '公斤',
  'pcs': '個',
}

interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  status?: 'active' | 'inactive' | 'discontinued'
  storage_condition?: string
  barcode?: string
  license_no?: string
  default_expiry_days?: number
  tags?: string[]
  remark?: string
  pack_qty?: number
  image_url?: string
  created_by_name?: string
}

type TabType = 'basic' | 'inventory' | 'documents' | 'history'

export function ProductDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [activeTab, setActiveTab] = useState<TabType>('basic')
  const [statusDialogOpen, setStatusDialogOpen] = useState(false)
  const [statusAction, setStatusAction] = useState<'activate' | 'deactivate' | 'discontinue'>('activate')

  const { data: product, isLoading, error } = useQuery({
    queryKey: ['product', id],
    queryFn: async () => {
      const response = await api.get<ExtendedProduct>(`/products/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const statusMutation = useMutation({
    mutationFn: async (status: string) => {
      return api.patch(`/products/${id}/status`, { status })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['product', id] })
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: '產品狀態已更新' })
      setStatusDialogOpen(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '狀態更新失敗',
        variant: 'destructive',
      })
    },
  })

  const handleCopySku = async () => {
    if (product) {
      await navigator.clipboard.writeText(product.sku)
      toast({ title: '已複製', description: `SKU: ${product.sku}` })
    }
  }

  const getStatusBadge = () => {
    if (!product) return null
    const status = product.status || (product.is_active ? 'active' : 'inactive')
    switch (status) {
      case 'active':
        return <Badge variant="success" className="text-sm">● 啟用</Badge>
      case 'inactive':
        return <Badge variant="warning" className="text-sm">● 停用</Badge>
      case 'discontinued':
        return <Badge variant="destructive" className="text-sm">● 停產</Badge>
      default:
        return <Badge variant="secondary" className="text-sm">● 未知</Badge>
    }
  }

  const getCategoryName = () => {
    if (!product) return '-'
    if (product.category_name) return product.category_name
    if (product.category_code) {
      // LAB 主分類映射為耗材
      const categoryCode = product.category_code === 'LAB' ? 'CON' : product.category_code
      return CATEGORIES[categoryCode] || product.category_code
    }
    return '-'
  }

  const getSubcategoryName = () => {
    if (!product) return '-'
    if (product.subcategory_name) return product.subcategory_name
    if (product.category_code && product.subcategory_code) {
      // LAB 主分類映射為 CON，子分類保持不變
      const categoryCode = product.category_code === 'LAB' ? 'CON' : product.category_code
      return SUBCATEGORIES[categoryCode]?.[product.subcategory_code] || SUBCATEGORIES[product.category_code]?.[product.subcategory_code] || product.subcategory_code
    }
    return '-'
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
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
        <Button variant="outline" onClick={() => navigate('/products')}>
          返回產品列表
        </Button>
      </div>
    )
  }

  const tabs: { id: TabType; label: string; icon: React.ReactNode }[] = [
    { id: 'basic', label: '基本資訊', icon: <Package className="h-4 w-4" /> },
    { id: 'inventory', label: '庫存設定', icon: <Boxes className="h-4 w-4" /> },
    { id: 'documents', label: '相關單據', icon: <FileText className="h-4 w-4" /> },
    { id: 'history', label: '異動紀錄', icon: <History className="h-4 w-4" /> },
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/products')}
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-start gap-6">
            {/* Product Image Placeholder */}
            <div className="w-24 h-24 rounded-lg bg-muted flex items-center justify-center shrink-0">
              {product.image_url ? (
                <img
                  src={product.image_url}
                  alt={product.name}
                  className="w-full h-full object-cover rounded-lg"
                />
              ) : (
                <Package className="h-10 w-10 text-muted-foreground" />
              )}
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight mb-1">{product.name}</h1>
              <div className="flex items-center gap-2 mb-2">
                <code className="font-mono text-sm bg-muted px-2 py-0.5 rounded">
                  {product.sku}
                </code>
                <button
                  onClick={handleCopySku}
                  className="text-muted-foreground hover:text-foreground transition-colors"
                  title="複製 SKU"
                >
                  <ClipboardCopy className="h-4 w-4" />
                </button>
              </div>
              <div className="flex items-center gap-3 text-sm text-muted-foreground">
                <span>品類: {getCategoryName()}</span>
                <span>{'>'}</span>
                <span>{getSubcategoryName()}</span>
                <span className="mx-2">│</span>
                {getStatusBadge()}
              </div>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={() => navigate(`/products/${id}/edit`)}>
            <Edit className="mr-2 h-4 w-4" />
            編輯
          </Button>
          <div className="relative">
            <select
              className="appearance-none bg-background border rounded-md px-3 py-2 pr-8 cursor-pointer hover:bg-muted focus:outline-none focus:ring-2 focus:ring-ring text-sm"
              onChange={(e) => {
                const action = e.target.value
                e.target.value = ''
                switch (action) {
                  case 'copy':
                    navigate(`/products/new?copy=${id}`)
                    break
                  case 'activate':
                    setStatusAction('activate')
                    setStatusDialogOpen(true)
                    break
                  case 'deactivate':
                    setStatusAction('deactivate')
                    setStatusDialogOpen(true)
                    break
                  case 'discontinue':
                    setStatusAction('discontinue')
                    setStatusDialogOpen(true)
                    break
                }
              }}
              defaultValue=""
            >
              <option value="" disabled>更多操作...</option>
              <option value="copy">複製產品</option>
              <option disabled>───</option>
              {product.is_active ? (
                <option value="deactivate">停用</option>
              ) : (
                <option value="activate">啟用</option>
              )}
              <option value="discontinue">標記停產</option>
            </select>
            <MoreHorizontal className="absolute right-2 top-1/2 -translate-y-1/2 h-4 w-4 pointer-events-none text-muted-foreground" />
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b">
        <div className="flex gap-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-primary text-primary'
                  : 'border-transparent text-muted-foreground hover:text-foreground hover:border-muted-foreground/30'
              }`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Tab Content */}
      {activeTab === 'basic' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle>基本資訊</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <InfoRow label="產品名稱" value={product.name} />
              <InfoRow label="規格描述" value={product.spec || '-'} />
              <InfoRow label="品類" value={getCategoryName()} />
              <InfoRow label="子類" value={getSubcategoryName()} />
              <InfoRow
                label="庫存單位"
                value={`${product.base_uom} (${UOM_MAP[product.base_uom] || product.base_uom})`}
              />
              <InfoRow label="包裝量" value={product.pack_qty?.toString() || '-'} />
              <InfoRow label="原廠條碼" value={product.barcode || '-'} />
              <InfoRow
                label="保存條件"
                value={product.storage_condition ? STORAGE_CONDITIONS[product.storage_condition] || product.storage_condition : '-'}
              />
              <InfoRow label="許可證號" value={product.license_no || '-'} />
              {product.tags && product.tags.length > 0 && (
                <div className="flex justify-between py-2 border-b">
                  <span className="text-muted-foreground">搜尋標籤</span>
                  <div className="flex gap-1 flex-wrap justify-end">
                    {product.tags.map((tag, idx) => (
                      <Badge key={idx} variant="secondary" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
              <InfoRow label="備註" value={product.remark || '-'} multiline />
            </CardContent>
          </Card>

          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>追蹤設定</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <InfoRow
                  label="追蹤批號"
                  value={product.track_batch ? '是' : '否'}
                  badge={product.track_batch}
                />
                <InfoRow
                  label="追蹤效期"
                  value={product.track_expiry
                    ? `是 (預設有效天數: ${product.default_expiry_days || '-'} 天)`
                    : '否'
                  }
                  badge={product.track_expiry}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>系統資訊</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-2 py-2 border-b">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">建立時間</span>
                  <span className="ml-auto">{formatDateTime(product.created_at)}</span>
                </div>
                <div className="flex items-center gap-2 py-2 border-b">
                  <User className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">建立者</span>
                  <span className="ml-auto">{product.created_by_name || '-'}</span>
                </div>
                <div className="flex items-center gap-2 py-2 border-b">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">更新時間</span>
                  <span className="ml-auto">{formatDateTime(product.updated_at)}</span>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {activeTab === 'inventory' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle>庫存管理設定</CardTitle>
              <CardDescription>安全庫存與補貨點設定</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <InfoRow
                label="安全庫存"
                value={product.safety_stock
                  ? `${formatNumber(product.safety_stock, 0)} ${UOM_MAP[product.base_uom] || product.base_uom}`
                  : '未設定'
                }
              />
              <InfoRow
                label="補貨點"
                value={product.reorder_point
                  ? `${formatNumber(product.reorder_point, 0)} ${UOM_MAP[product.base_uom] || product.base_uom}`
                  : '未設定'
                }
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>各倉庫庫存快照</CardTitle>
              <CardDescription>目前各倉庫的庫存狀態</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-center py-8 text-muted-foreground">
                <Boxes className="h-10 w-10 mx-auto mb-2 opacity-50" />
                <p>庫存資料載入中或尚無資料</p>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'documents' && (
        <Card>
          <CardHeader>
            <CardTitle>相關單據</CardTitle>
            <CardDescription>最近的採購單、銷售單、庫存異動</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12 text-muted-foreground">
              <FileText className="h-10 w-10 mx-auto mb-2 opacity-50" />
              <p>尚無相關單據</p>
            </div>
          </CardContent>
        </Card>
      )}

      {activeTab === 'history' && (
        <Card>
          <CardHeader>
            <CardTitle>異動紀錄</CardTitle>
            <CardDescription>產品資料的變更歷史 (Audit Log)</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12 text-muted-foreground">
              <History className="h-10 w-10 mx-auto mb-2 opacity-50" />
              <p>尚無異動紀錄</p>
            </div>
          </CardContent>
        </Card>
      )}

      {/* 狀態變更對話框 */}
      <Dialog open={statusDialogOpen} onOpenChange={setStatusDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {statusAction === 'activate' && '啟用產品'}
              {statusAction === 'deactivate' && '停用產品'}
              {statusAction === 'discontinue' && '標記停產'}
            </DialogTitle>
            <DialogDescription>
              {statusAction === 'activate' && '確定要啟用此產品嗎？啟用後可在採購、銷售等模組中使用。'}
              {statusAction === 'deactivate' && '確定要停用此產品嗎？停用後將無法在新單據中選擇此產品。'}
              {statusAction === 'discontinue' && '確定要將此產品標記為停產嗎？停產後僅供歷史查詢，無法恢復為啟用狀態。'}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setStatusDialogOpen(false)}
              disabled={statusMutation.isPending}
            >
              取消
            </Button>
            <Button
              variant={statusAction === 'discontinue' ? 'destructive' : 'default'}
              onClick={() => {
                const status = statusAction === 'activate' ? 'active'
                  : statusAction === 'deactivate' ? 'inactive'
                  : 'discontinued'
                statusMutation.mutate(status)
              }}
              disabled={statusMutation.isPending}
            >
              {statusMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              確認
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function InfoRow({
  label,
  value,
  badge,
  multiline,
}: {
  label: string
  value: string
  badge?: boolean
  multiline?: boolean
}) {
  return (
    <div className={`flex ${multiline ? 'flex-col gap-1' : 'justify-between'} py-2 border-b`}>
      <span className="text-muted-foreground">{label}</span>
      <span className={multiline ? 'text-sm' : ''}>
        {badge !== undefined ? (
          <Badge variant={badge ? 'success' : 'secondary'} className="text-xs">
            {value}
          </Badge>
        ) : (
          value
        )}
      </span>
    </div>
  )
}
