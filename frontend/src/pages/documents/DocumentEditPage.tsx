import React, { useState, useEffect } from 'react'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Document, Product, Partner, Warehouse, DocType, ProtocolListItem, StockLedgerDetail } from '@/lib/api'
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
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
  RM: '退料單',
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

  // 查詢進行中的協議（用於銷售單客戶選擇）
  const { data: activeProtocols } = useQuery({
    queryKey: ['active-protocols', partners],
    queryFn: async () => {
      // 查詢所有協議，然後過濾出進行中的
      const response = await api.get<ProtocolListItem[]>('/protocols')
      return response.data.filter(p => {
        // 基本狀態過濾：必須是已核准且未關閉的協議
        if (!((p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && 
              p.status !== 'CLOSED' && 
              p.iacuc_no)) {
          return false
        }
        
        // 檢查對應的客戶是否啟用（客戶代碼 = IACUC No.）
        // 如果partners已載入，則過濾掉停用的客戶
        if (partners && p.iacuc_no) {
          const customer = partners.find(
            partner => partner.partner_type === 'customer' && partner.code === p.iacuc_no
          )
          // 如果找到客戶且客戶已停用，則不顯示此協議
          if (customer && !customer.is_active) {
            return false
          }
        }
        
        return true
      })
    },
    enabled: formData.doc_type === 'SO' || formData.doc_type === 'DO', // 只在銷售單時查詢
  })

  // 批號選擇組件（內部組件）
  function BatchNumberSelect({
    lineIndex,
    productId,
    warehouseId,
    batchNo,
    docType,
    onBatchChange,
  }: {
    lineIndex: number
    productId: string
    warehouseId: string
    batchNo: string
    docType: DocType
    onBatchChange: (batchNo: string, expiryDate?: string) => void
  }) {
    // 判斷是否為採購單據（需要手動輸入）
    const isPurchaseDoc = ['PO', 'GRN', 'PR'].includes(docType)
    // 判斷是否為銷貨單據（使用下拉選單）
    const isSalesDoc = ['SO', 'DO'].includes(docType)

    // 查詢該產品在該倉庫的庫存流水以獲取可用批號（僅銷貨單據需要）
    const { data: stockLedger } = useQuery({
      queryKey: ['stock-ledger', productId, warehouseId],
      queryFn: async () => {
        if (!productId || !warehouseId) return []
        const response = await api.get<StockLedgerDetail[]>(
          `/inventory/ledger?product_id=${productId}&warehouse_id=${warehouseId}`
        )
        return response.data
      },
      enabled: !!productId && !!warehouseId && isSalesDoc, // 僅銷貨單據時查詢
    })

    // 從庫存流水中提取唯一的批號和對應的效期
    // 計算每個批號的當前庫存，只顯示庫存大於0的批號
    const batchOptions = React.useMemo(() => {
      if (!stockLedger || stockLedger.length === 0) return []
      
      // 計算每個批號的庫存量和對應的效期
      const batchMap = new Map<string, { qty: number; expiry: string }>()
      
      stockLedger.forEach((entry) => {
        if (entry.batch_no && entry.batch_no.trim() !== '') {
          const batch = entry.batch_no
          const qty = parseFloat(entry.qty_base) || 0
          
          // 判斷是入庫還是出庫
          const isIn = ['in', 'transfer_in', 'adjust_in'].includes(entry.direction)
          const qtyChange = isIn ? qty : -qty
          
          if (batchMap.has(batch)) {
            const existing = batchMap.get(batch)!
            existing.qty += qtyChange
            // 如果當前條目有效期且現有記錄沒有效期，則更新效期
            if (entry.expiry_date && !existing.expiry) {
              existing.expiry = entry.expiry_date
            }
          } else {
            batchMap.set(batch, {
              qty: qtyChange,
              expiry: entry.expiry_date || '',
            })
          }
        }
      })
      
      // 只返回庫存大於0的批號
      return Array.from(batchMap.entries())
        .filter(([, data]) => data.qty > 0)
        .map(([batch, data]) => ({ batch, expiry: data.expiry }))
        .sort((a, b) => a.batch.localeCompare(b.batch))
    }, [stockLedger])

    const handleBatchChange = (value: string) => {
      const selected = batchOptions.find((opt) => opt.batch === value)
      onBatchChange(value, selected?.expiry)
    }

    // 採購單據：始終顯示普通輸入框，允許手動輸入批號
    if (isPurchaseDoc) {
      return (
        <Input
          value={batchNo}
          onChange={(e) => onBatchChange(e.target.value)}
          placeholder="輸入批號"
        />
      )
    }

    // 銷貨單據：如果有可用批號，顯示下拉選單；否則顯示輸入框
    if (isSalesDoc) {
      // 如果沒有產品或倉庫，顯示普通輸入框
      if (!productId || !warehouseId) {
        return (
          <Input
            value={batchNo}
            onChange={(e) => onBatchChange(e.target.value)}
            placeholder="批號"
          />
        )
      }

      // 如果有可用批號，顯示下拉選單
      if (batchOptions.length > 0) {
        return (
          <Select value={batchNo} onValueChange={handleBatchChange}>
            <SelectTrigger>
              <SelectValue placeholder="選擇批號" />
            </SelectTrigger>
            <SelectContent>
              {batchOptions.map((option) => (
                <SelectItem key={option.batch} value={option.batch}>
                  {option.batch}
                  {option.expiry && ` (${option.expiry})`}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )
      }

      // 如果沒有可用批號，顯示普通輸入框
      return (
        <Input
          value={batchNo}
          onChange={(e) => onBatchChange(e.target.value)}
          placeholder="批號"
        />
      )
    }

    // 其他單據類型：顯示普通輸入框
    return (
      <Input
        value={batchNo}
        onChange={(e) => onBatchChange(e.target.value)}
        placeholder="批號"
      />
    )
  }

  // 根據IACUC No.創建或查找客戶
  const createOrFindCustomerMutation = useMutation({
    mutationFn: async (iacucNo: string) => {
      // 先查找是否已存在該客戶（客戶代碼 = IACUC No.）
      const existingPartnersResponse = await api.get<Partner[]>('/partners')
      const existingCustomer = existingPartnersResponse.data.find(
        p => p.partner_type === 'customer' && p.code === iacucNo
      )
      
      if (existingCustomer) {
        return existingCustomer
      }
      
      // 如果不存在，創建新客戶
      const newCustomerResponse = await api.post<Partner>('/partners', {
        partner_type: 'customer',
        code: iacucNo,  // 客戶代碼 = IACUC No.
        name: iacucNo,  // 客戶名稱 = IACUC No.
      })
      
      // 刷新客戶列表
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      
      return newCustomerResponse.data
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '創建客戶失敗',
        variant: 'destructive',
      })
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
      // 計算單據類型相關的驗證條件
      const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO'].includes(data.doc_type)
      const isTransfer = data.doc_type === 'TR'

      // 驗證必填欄位
      if (needsPartner && !data.partner_id?.trim()) {
        throw new Error('請選擇供應商/客戶')
      }
      if (!data.warehouse_id?.trim() && !isTransfer) {
        throw new Error('請選擇倉庫')
      }
      if (isTransfer && (!data.warehouse_from_id?.trim() || !data.warehouse_to_id?.trim())) {
        throw new Error('調撥單需要選擇來源倉庫和目標倉庫')
      }

      // 過濾並驗證明細行
      const validLines = data.lines.filter((line) => line.product_id && line.product_id.trim() !== '')
      
      // 盤點單可以沒有明細（會自動生成），其他單據必須有至少一行
      if (data.doc_type !== 'STK' && validLines.length === 0) {
        throw new Error('請至少新增一項產品明細')
      }

      // 驗證每行的必填欄位
      for (const line of validLines) {
        if (!line.product_id?.trim()) {
          throw new Error('請選擇產品')
        }
        const qty = parseFloat(line.qty)
        if (isNaN(qty) || qty <= 0) {
          throw new Error('數量必須大於 0')
        }
        if (!line.uom?.trim()) {
          throw new Error('請輸入單位')
        }
      }

      // 轉換資料格式以符合後端要求
      const payload: any = {
        doc_type: data.doc_type,
        doc_date: data.doc_date,
        warehouse_id: data.warehouse_id && data.warehouse_id.trim() !== '' ? data.warehouse_id : null,
        warehouse_from_id: data.warehouse_from_id && data.warehouse_from_id.trim() !== '' ? data.warehouse_from_id : null,
        warehouse_to_id: data.warehouse_to_id && data.warehouse_to_id.trim() !== '' ? data.warehouse_to_id : null,
        partner_id: data.partner_id && data.partner_id.trim() !== '' ? data.partner_id : null,
        remark: data.remark && data.remark.trim() !== '' ? data.remark : null,
        lines: validLines.map((line) => ({
          product_id: line.product_id,
          qty: parseFloat(line.qty) || 0,
          uom: line.uom && line.uom.trim() !== '' ? line.uom.trim() : 'pcs', // Default to 'pcs' if empty
          unit_price: line.unit_price && line.unit_price.trim() !== '' ? parseFloat(line.unit_price) : null,
          batch_no: line.batch_no && line.batch_no.trim() !== '' ? line.batch_no : null,
          expiry_date: line.expiry_date && line.expiry_date.trim() !== '' ? line.expiry_date : null,
          remark: line.remark && line.remark.trim() !== '' ? line.remark : null,
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
      // 根據單據類型跳轉到對應的列表頁面
      navigate(`/documents?type=${formData.doc_type}`)
    },
    onError: (error: any) => {
      const errorMessage = error?.message || error?.response?.data?.error?.message || '儲存失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  // 送審
  const submitMutation = useMutation({
    mutationFn: async () => {
      // 計算單據類型相關的驗證條件
      const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO'].includes(formData.doc_type)
      const isTransfer = formData.doc_type === 'TR'

      // 驗證必填欄位
      if (needsPartner && !formData.partner_id?.trim()) {
        throw new Error('請選擇供應商/客戶')
      }
      if (!formData.warehouse_id?.trim() && !isTransfer) {
        throw new Error('請選擇倉庫')
      }
      if (isTransfer && (!formData.warehouse_from_id?.trim() || !formData.warehouse_to_id?.trim())) {
        throw new Error('調撥單需要選擇來源倉庫和目標倉庫')
      }

      // 過濾並驗證明細行
      const validLines = formData.lines.filter((line) => line.product_id && line.product_id.trim() !== '')
      
      // 盤點單可以沒有明細（會自動生成），其他單據必須有至少一行
      if (formData.doc_type !== 'STK' && validLines.length === 0) {
        throw new Error('請至少新增一項產品明細')
      }

      // 驗證每行的必填欄位
      for (const line of validLines) {
        if (!line.product_id?.trim()) {
          throw new Error('請選擇產品')
        }
        const qty = parseFloat(line.qty)
        if (isNaN(qty) || qty <= 0) {
          throw new Error('數量必須大於 0')
        }
        if (!line.uom?.trim()) {
          throw new Error('請輸入單位')
        }
      }

      // 先儲存再送審
      // 轉換資料格式以符合後端要求
      const payload: any = {
        doc_type: formData.doc_type,
        doc_date: formData.doc_date,
        warehouse_id: formData.warehouse_id && formData.warehouse_id.trim() !== '' ? formData.warehouse_id : null,
        warehouse_from_id: formData.warehouse_from_id && formData.warehouse_from_id.trim() !== '' ? formData.warehouse_from_id : null,
        warehouse_to_id: formData.warehouse_to_id && formData.warehouse_to_id.trim() !== '' ? formData.warehouse_to_id : null,
        partner_id: formData.partner_id && formData.partner_id.trim() !== '' ? formData.partner_id : null,
        remark: formData.remark && formData.remark.trim() !== '' ? formData.remark : null,
        lines: validLines.map((line) => ({
          product_id: line.product_id,
          qty: parseFloat(line.qty) || 0,
          uom: line.uom && line.uom.trim() !== '' ? line.uom.trim() : 'pcs', // Default to 'pcs' if empty
          unit_price: line.unit_price && line.unit_price.trim() !== '' ? parseFloat(line.unit_price) : null,
          batch_no: line.batch_no && line.batch_no.trim() !== '' ? line.batch_no : null,
          expiry_date: line.expiry_date && line.expiry_date.trim() !== '' ? line.expiry_date : null,
          remark: line.remark && line.remark.trim() !== '' ? line.remark : null,
        })),
      }

      if (isEdit) {
        await api.put(`/documents/${id}`, payload)
        await api.post(`/documents/${id}/submit`)
        return { documentId: id }
      } else {
        const createResponse = await api.post<{ id: string }>('/documents', payload)
        const documentId = createResponse.data.id
        await api.post(`/documents/${documentId}/submit`)
        return { documentId }
      }
    },
    onSuccess: async (response: { documentId: string }) => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({ title: '成功', description: '單據已送審' })
      
      // 導航到詳情頁並自動重新整理以獲得更新後的資訊
      navigate(`/documents/${response.documentId}`)
      setTimeout(() => {
        window.location.reload()
      }, 500)
    },
    onError: (error: any) => {
      const errorMessage = error?.message || error?.response?.data?.error?.message || '送審失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
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
    const targetPath = `/documents${formData.doc_type ? `?type=${formData.doc_type}` : ''}`
    if (unsavedChanges) {
      setPendingNavigation(targetPath)
      setShowUnsavedDialog(true)
    } else {
      navigate(targetPath)
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
  const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO'].includes(formData.doc_type)
  const isTransfer = formData.doc_type === 'TR'
  const partnerType = ['PO', 'GRN', 'PR'].includes(formData.doc_type) ? 'supplier' : 'customer'
  
  // 處理IACUC No.選擇（僅銷售單）
  const handleIacucNoSelect = async (iacucNo: string) => {
    try {
      const customer = await createOrFindCustomerMutation.mutateAsync(iacucNo)
      updateField('partner_id', customer.id)
      toast({
        title: '成功',
        description: `已選擇客戶：${iacucNo}`,
      })
    } catch (error) {
      // 錯誤已在mutation中處理
    }
  }

  // 過濾客戶：只顯示進行中的客戶（客戶代碼 = IACUC No.）
  const filteredPartners = partners?.filter((p) => {
    if (!needsPartner) return true
    if (p.partner_type !== partnerType) return false
    
    // 如果是客戶，只顯示進行中的（客戶代碼對應的 IACUC No. 在進行中的協議中）
    if (partnerType === 'customer') {
      if (!activeProtocols || activeProtocols.length === 0) return false
      // 檢查客戶代碼是否對應進行中的 IACUC No.
      return activeProtocols.some(protocol => protocol.iacuc_no === p.code)
    }
    
    // 供應商不過濾
    return true
  })

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
                {formData.doc_type === 'SO' || formData.doc_type === 'DO' ? (
                  // 銷售單：使用IACUC No.選擇
                  <>
                    <Label>IACUC No. *</Label>
                    <Select
                      value={(() => {
                        // 根據當前選擇的partner_id找到對應的IACUC No.
                        if (!formData.partner_id) return ''
                        const selectedPartner = partners?.find(p => p.id === formData.partner_id)
                        return selectedPartner?.code || ''
                      })()}
                      onValueChange={handleIacucNoSelect}
                      disabled={createOrFindCustomerMutation.isPending}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="選擇IACUC No." />
                      </SelectTrigger>
                      <SelectContent>
                        {activeProtocols?.map((protocol) => (
                          <SelectItem key={protocol.iacuc_no} value={protocol.iacuc_no || ''}>
                            {protocol.iacuc_no} - {protocol.title}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    {createOrFindCustomerMutation.isPending && (
                      <p className="text-xs text-muted-foreground flex items-center gap-1">
                        <Loader2 className="h-3 w-3 animate-spin" />
                        正在創建客戶...
                      </p>
                    )}
                  </>
                ) : (
                  // 採購單：使用供應商選擇
                  <>
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
                  </>
                )}
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
            {['PO', 'GRN', 'DO'].includes(formData.doc_type) && (
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
                {['PO', 'GRN', 'DO'].includes(formData.doc_type) && (
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
                    colSpan={['PO', 'GRN', 'DO'].includes(formData.doc_type) ? 9 : 7}
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
                    {['PO', 'GRN', 'DO'].includes(formData.doc_type) && (
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
                      <BatchNumberSelect
                        lineIndex={index}
                        productId={line.product_id}
                        warehouseId={formData.warehouse_id}
                        batchNo={line.batch_no}
                        docType={formData.doc_type}
                        onBatchChange={(batchNo, expiryDate) => {
                          updateLine(index, 'batch_no', batchNo)
                          // 銷貨單據時，如果選擇了批號，自動更新效期
                          if (['SO', 'DO'].includes(formData.doc_type) && expiryDate) {
                            updateLine(index, 'expiry_date', expiryDate)
                          }
                        }}
                      />
                    </TableCell>
                    <TableCell>
                      <Input
                        type="date"
                        value={line.expiry_date}
                        onChange={(e) => updateLine(index, 'expiry_date', e.target.value)}
                        placeholder="效期"
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
