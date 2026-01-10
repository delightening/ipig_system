import { useState, useEffect } from 'react'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Document, Product, Partner, Warehouse, DocType } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
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
  Save,
  Send,
  Plus,
  Trash2,
  Loader2,
  Search,
  AlertTriangle,
} from 'lucide-react'
import { formatNumber } from '@/lib/utils'

const docTypeNames: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷售單',
  DO: '銷售出庫',
  SR: '銷售退貨',
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
}

interface DocumentLine {
  id?: string
  line_no: number
  product_id: string
  product_name?: string
  product_sku?: string
  qty: string
  uom: string
  unit_price: string
  batch_no: string
  expiry_date: string
  remark: string
}

interface FormData {
  doc_type: DocType
  doc_date: string
  warehouse_id: string
  warehouse_from_id: string
  warehouse_to_id: string
  partner_id: string
  remark: string
  lines: DocumentLine[]
}

export function DocumentEditPage() {
  const { id } = useParams<{ id: string }>()
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const isEdit = !!id && id !== 'new'
  const defaultType = (searchParams.get('type') as DocType) || 'PO'

  const [formData, setFormData] = useState<FormData>({
    doc_type: defaultType,
    doc_date: new Date().toISOString().split('T')[0],
    warehouse_id: '',
    warehouse_from_id: '',
    warehouse_to_id: '',
    partner_id: '',
    remark: '',
    lines: [],
  })

  const [productSearchOpen, setProductSearchOpen] = useState(false)
  const [productSearch, setProductSearch] = useState('')
  const [selectedLineIndex, setSelectedLineIndex] = useState<number | null>(null)
  const [unsavedChanges, setUnsavedChanges] = useState(false)
  const [showUnsavedDialog, setShowUnsavedDialog] = useState(false)
  const [pendingNavigation, setPendingNavigation] = useState<string | null>(null)

  // 查詢現有單據（編輯模式）
  const { data: document, isLoading: loadingDocument } = useQuery({
    queryKey: ['document', id],
    queryFn: async () => {
      const response = await api.get<Document>(`/documents/${id}`)
      return response.data
    },
    enabled: isEdit,
  })

  // 查詢產品列表
  const { data: products } = useQuery({
    queryKey: ['products', productSearch],
    queryFn: async () => {
      const response = await api.get<Product[]>(
        `/products?keyword=${encodeURIComponent(productSearch)}&is_active=true`
      )
      return response.data
    },
  })

  // 查詢倉庫列表
  const { data: warehouses } = useQuery({
    queryKey: ['warehouses'],
    queryFn: async () => {
      const response = await api.get<Warehouse[]>('/warehouses')
      return response.data
    },
  })

  // 查詢供應商/客戶列表
  const { data: partners } = useQuery({
    queryKey: ['partners'],
    queryFn: async () => {
      const response = await api.get<Partner[]>('/partners')
      return response.data
    },
  })

  // 設定表單資料（編輯模式）
  useEffect(() => {
    if (document && isEdit) {
      setFormData({
        doc_type: document.doc_type,
        doc_date: document.doc_date,
        warehouse_id: document.warehouse_id || '',
        warehouse_from_id: document.warehouse_from_id || '',
        warehouse_to_id: document.warehouse_to_id || '',
        partner_id: document.partner_id || '',
        remark: document.remark || '',
        lines: document.lines.map((line) => ({
          id: line.id,
          line_no: line.line_no,
          product_id: line.product_id,
          product_name: line.product_name,
          product_sku: line.product_sku,
          qty: line.qty,
          uom: line.uom,
          unit_price: line.unit_price || '',
          batch_no: line.batch_no || '',
          expiry_date: line.expiry_date || '',
          remark: line.remark || '',
        })),
      })
    }
  }, [document, isEdit])

  // 監聽未儲存變更
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (unsavedChanges) {
        e.preventDefault()
        e.returnValue = ''
      }
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [unsavedChanges])

  // 建立/更新單據
  const saveMutation = useMutation({
    mutationFn: async (data: FormData) => {
      const payload = {
        ...data,
        lines: data.lines.map((line, index) => ({
          ...line,
          line_no: index + 1,
        })),
      }

      if (isEdit) {
        return api.put(`/documents/${id}`, payload)
      } else {
        return api.post('/documents', payload)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({ title: '成功', description: isEdit ? '單據已更新' : '單據已建立' })
      navigate('/documents')
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '儲存失敗',
        variant: 'destructive',
      })
    },
  })

  // 送審
  const submitMutation = useMutation({
    mutationFn: async () => {
      // 先儲存再送審
      const payload = {
        ...formData,
        lines: formData.lines.map((line, index) => ({
          ...line,
          line_no: index + 1,
        })),
      }

      if (isEdit) {
        await api.put(`/documents/${id}`, payload)
        return api.post(`/documents/${id}/submit`)
      } else {
        const createResponse = await api.post<{ id: string }>('/documents', payload)
        return api.post(`/documents/${createResponse.data.id}/submit`)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({ title: '成功', description: '單據已送審' })
      navigate('/documents')
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '送審失敗',
        variant: 'destructive',
      })
    },
  })

  // 更新表單欄位
  const updateField = <K extends keyof FormData>(field: K, value: FormData[K]) => {
    setFormData((prev) => ({ ...prev, [field]: value }))
    setUnsavedChanges(true)
  }

  // 新增明細行
  const addLine = () => {
    const newLine: DocumentLine = {
      line_no: formData.lines.length + 1,
      product_id: '',
      qty: '1',
      uom: '',
      unit_price: '',
      batch_no: '',
      expiry_date: '',
      remark: '',
    }
    setFormData((prev) => ({
      ...prev,
      lines: [...prev.lines, newLine],
    }))
    setUnsavedChanges(true)
  }

  // 更新明細行
  const updateLine = (index: number, field: keyof DocumentLine, value: string) => {
    setFormData((prev) => ({
      ...prev,
      lines: prev.lines.map((line, i) => (i === index ? { ...line, [field]: value } : line)),
    }))
    setUnsavedChanges(true)
  }

  // 刪除明細行
  const removeLine = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      lines: prev.lines.filter((_, i) => i !== index),
    }))
    setUnsavedChanges(true)
  }

  // 選擇產品
  const selectProduct = (product: Product) => {
    if (selectedLineIndex !== null) {
      setFormData((prev) => ({
        ...prev,
        lines: prev.lines.map((line, i) =>
          i === selectedLineIndex
            ? {
                ...line,
                product_id: product.id,
                product_name: product.name,
                product_sku: product.sku,
                uom: product.base_uom,
              }
            : line
        ),
      }))
      setUnsavedChanges(true)
    }
    setProductSearchOpen(false)
    setProductSearch('')
    setSelectedLineIndex(null)
  }

  // 開啟產品搜尋
  const openProductSearch = (lineIndex: number) => {
    setSelectedLineIndex(lineIndex)
    setProductSearchOpen(true)
  }

  // 處理返回導航
  const handleBack = () => {
    if (unsavedChanges) {
      setPendingNavigation('/documents')
      setShowUnsavedDialog(true)
    } else {
      navigate('/documents')
    }
  }

  // 確認離開
  const confirmNavigation = () => {
    setShowUnsavedDialog(false)
    if (pendingNavigation) {
      navigate(pendingNavigation)
    }
  }

  // 判斷是否需要供應商/客戶
  const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO', 'SR'].includes(formData.doc_type)
  const isTransfer = formData.doc_type === 'TR'
  const partnerType = ['PO', 'GRN', 'PR'].includes(formData.doc_type) ? 'supplier' : 'customer'
  const filteredPartners = partners?.filter((p) =>
    needsPartner ? p.partner_type === partnerType : true
  )

  // 計算總金額
  const totalAmount = formData.lines.reduce((sum, line) => {
    const qty = parseFloat(line.qty) || 0
    const price = parseFloat(line.unit_price) || 0
    return sum + qty * price
  }, 0)

  if (isEdit && loadingDocument) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="outline" size="icon" onClick={handleBack}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="text-3xl font-bold tracking-tight">
              {isEdit ? '編輯單據' : '新增單據'}
            </h1>
            <p className="text-muted-foreground">
              {isEdit
                ? `編輯 ${docTypeNames[formData.doc_type]}`
                : `建立新的${docTypeNames[formData.doc_type]}`}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => saveMutation.mutate(formData)}
            disabled={saveMutation.isPending || submitMutation.isPending}
          >
            {saveMutation.isPending ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            儲存草稿
          </Button>
          <Button
            onClick={() => submitMutation.mutate()}
            disabled={saveMutation.isPending || submitMutation.isPending || formData.lines.length === 0}
          >
            {submitMutation.isPending ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Send className="mr-2 h-4 w-4" />
            )}
            儲存並送審
          </Button>
        </div>
      </div>

      {/* 單據資訊 */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>單據資訊</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>單據類型</Label>
                <Select
                  value={formData.doc_type}
                  onValueChange={(v) => updateField('doc_type', v as DocType)}
                  disabled={isEdit}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(docTypeNames).map(([key, name]) => (
                      <SelectItem key={key} value={key}>
                        {name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>單據日期</Label>
                <Input
                  type="date"
                  value={formData.doc_date}
                  onChange={(e) => updateField('doc_date', e.target.value)}
                />
              </div>
            </div>

            {isTransfer ? (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>來源倉庫 *</Label>
                  <Select
                    value={formData.warehouse_from_id}
                    onValueChange={(v) => updateField('warehouse_from_id', v)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇來源倉庫" />
                    </SelectTrigger>
                    <SelectContent>
                      {warehouses?.map((wh) => (
                        <SelectItem key={wh.id} value={wh.id}>
                          {wh.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label>目標倉庫 *</Label>
                  <Select
                    value={formData.warehouse_to_id}
                    onValueChange={(v) => updateField('warehouse_to_id', v)}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇目標倉庫" />
                    </SelectTrigger>
                    <SelectContent>
                      {warehouses?.map((wh) => (
                        <SelectItem key={wh.id} value={wh.id}>
                          {wh.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            ) : (
              <div className="space-y-2">
                <Label>倉庫 *</Label>
                <Select
                  value={formData.warehouse_id}
                  onValueChange={(v) => updateField('warehouse_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="選擇倉庫" />
                  </SelectTrigger>
                  <SelectContent>
                    {warehouses?.map((wh) => (
                      <SelectItem key={wh.id} value={wh.id}>
                        {wh.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {needsPartner && (
              <div className="space-y-2">
                <Label>{partnerType === 'supplier' ? '供應商' : '客戶'} *</Label>
                <Select
                  value={formData.partner_id}
                  onValueChange={(v) => updateField('partner_id', v)}
                >
                  <SelectTrigger>
                    <SelectValue
                      placeholder={`選擇${partnerType === 'supplier' ? '供應商' : '客戶'}`}
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {filteredPartners?.map((partner) => (
                      <SelectItem key={partner.id} value={partner.id}>
                        {partner.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            <div className="space-y-2">
              <Label>備註</Label>
              <Input
                value={formData.remark}
                onChange={(e) => updateField('remark', e.target.value)}
                placeholder="輸入備註..."
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>單據摘要</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-muted-foreground">明細行數</span>
              <span className="font-medium">{formData.lines.length} 項</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">總數量</span>
              <span className="font-medium">
                {formatNumber(
                  formData.lines.reduce((sum, l) => sum + (parseFloat(l.qty) || 0), 0),
                  0
                )}
              </span>
            </div>
            {['PO', 'GRN', 'SO', 'DO'].includes(formData.doc_type) && (
              <div className="flex justify-between text-lg border-t pt-4">
                <span className="font-medium">總金額</span>
                <span className="font-bold">${formatNumber(totalAmount, 2)}</span>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* 明細行 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>單據明細</CardTitle>
          <Button onClick={addLine} size="sm">
            <Plus className="mr-2 h-4 w-4" />
            新增明細
          </Button>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[60px]">項次</TableHead>
                <TableHead className="w-[300px]">品項</TableHead>
                <TableHead className="w-[100px] text-right">數量</TableHead>
                <TableHead className="w-[80px]">單位</TableHead>
                {['PO', 'GRN', 'SO', 'DO'].includes(formData.doc_type) && (
                  <>
                    <TableHead className="w-[120px] text-right">單價</TableHead>
                    <TableHead className="w-[120px] text-right">金額</TableHead>
                  </>
                )}
                <TableHead className="w-[120px]">批號</TableHead>
                <TableHead className="w-[140px]">效期</TableHead>
                <TableHead className="w-[60px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {formData.lines.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={['PO', 'GRN', 'SO', 'DO'].includes(formData.doc_type) ? 9 : 7}
                    className="text-center py-8"
                  >
                    <p className="text-muted-foreground">尚無明細，請點擊「新增明細」</p>
                  </TableCell>
                </TableRow>
              ) : (
                formData.lines.map((line, index) => (
                  <TableRow key={index}>
                    <TableCell className="text-center">{index + 1}</TableCell>
                    <TableCell>
                      {line.product_id ? (
                        <div>
                          <div className="font-medium">{line.product_name}</div>
                          <div className="text-xs text-muted-foreground">{line.product_sku}</div>
                        </div>
                      ) : (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => openProductSearch(index)}
                          className="w-full justify-start"
                        >
                          <Search className="mr-2 h-4 w-4" />
                          選擇品項
                        </Button>
                      )}
                    </TableCell>
                    <TableCell>
                      <Input
                        type="number"
                        value={line.qty}
                        onChange={(e) => updateLine(index, 'qty', e.target.value)}
                        className="text-right"
                        min="0"
                      />
                    </TableCell>
                    <TableCell>
                      <span className="text-sm">{line.uom || '-'}</span>
                    </TableCell>
                    {['PO', 'GRN', 'SO', 'DO'].includes(formData.doc_type) && (
                      <>
                        <TableCell>
                          <Input
                            type="number"
                            value={line.unit_price}
                            onChange={(e) => updateLine(index, 'unit_price', e.target.value)}
                            className="text-right"
                            min="0"
                          />
                        </TableCell>
                        <TableCell className="text-right font-medium">
                          $
                          {formatNumber(
                            (parseFloat(line.qty) || 0) * (parseFloat(line.unit_price) || 0),
                            2
                          )}
                        </TableCell>
                      </>
                    )}
                    <TableCell>
                      <Input
                        value={line.batch_no}
                        onChange={(e) => updateLine(index, 'batch_no', e.target.value)}
                        placeholder="批號"
                      />
                    </TableCell>
                    <TableCell>
                      <Input
                        type="date"
                        value={line.expiry_date}
                        onChange={(e) => updateLine(index, 'expiry_date', e.target.value)}
                      />
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => removeLine(index)}
                        className="text-red-500 hover:text-red-700"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* 產品搜尋對話框 */}
      <Dialog open={productSearchOpen} onOpenChange={setProductSearchOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>選擇品項</DialogTitle>
            <DialogDescription>搜尋並選擇要新增的品項</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜尋品項..."
                value={productSearch}
                onChange={(e) => setProductSearch(e.target.value)}
                className="pl-9"
                autoFocus
              />
            </div>
            <div className="max-h-[400px] overflow-y-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>SKU</TableHead>
                    <TableHead>品項名稱</TableHead>
                    <TableHead>規格</TableHead>
                    <TableHead>單位</TableHead>
                    <TableHead></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {products?.slice(0, 20).map((product) => (
                    <TableRow
                      key={product.id}
                      className="cursor-pointer hover:bg-muted"
                      onClick={() => selectProduct(product)}
                    >
                      <TableCell className="font-mono text-xs">{product.sku}</TableCell>
                      <TableCell className="font-medium">{product.name}</TableCell>
                      <TableCell>{product.spec || '-'}</TableCell>
                      <TableCell>{product.base_uom}</TableCell>
                      <TableCell>
                        <Button size="sm" variant="outline">
                          選擇
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* 未儲存變更對話框 */}
      <Dialog open={showUnsavedDialog} onOpenChange={setShowUnsavedDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              尚有未儲存的變更
            </DialogTitle>
            <DialogDescription>
              您有尚未儲存的變更，確定要離開嗎？離開後變更將會遺失。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUnsavedDialog(false)}>
              繼續編輯
            </Button>
            <Button variant="destructive" onClick={confirmNavigation}>
              放棄變更
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
