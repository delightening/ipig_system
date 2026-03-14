import { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  Document,
  Product,
  Partner,
  Warehouse,
  DocType,
  ProtocolListItem,
} from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { STALE_TIME } from '@/lib/query'
import { formatQuantity, formatUnitPrice } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import type { DocumentLine, DocumentFormData } from '../types'

export type InputRefs = Record<
  string,
  {
    qty?: HTMLInputElement
    unit_price?: HTMLInputElement
    expiry_date?: HTMLInputElement
    batch_no?: HTMLInputElement
  }
>

export interface UseDocumentFormOptions {
  defaultType: DocType
}

export function useDocumentForm({ defaultType }: UseDocumentFormOptions) {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const isEdit = !!id && id !== 'new'

  const [formData, setFormData] = useState<DocumentFormData>({
    doc_type: defaultType,
    doc_date: new Date().toISOString().split('T')[0],
    warehouse_id: '',
    warehouse_from_id: '',
    warehouse_to_id: '',
    partner_id: '',
    protocol_no: '',
    source_doc_id: '',
    remark: '',
    lines: [],
  })

  const [productSearchOpen, setProductSearchOpen] = useState(false)
  const [productSearch, setProductSearch] = useState('')
  const [categoryId, setCategoryId] = useState<string | undefined>()
  const [selectedLineId, setSelectedLineId] = useState<string | null>(null)
  const [unsavedChanges, setUnsavedChanges] = useState(false)
  const [showUnsavedDialog, setShowUnsavedDialog] = useState(false)
  const [pendingNavigation, setPendingNavigation] = useState<string | null>(null)
  const [lineAmounts, setLineAmounts] = useState<Record<string, number>>({})
  const [showIacucWarning, setShowIacucWarning] = useState(false)
  const [iacucWarningData, setIacucWarningData] = useState<{ batch_no: string; source_iacuc: string } | null>(null)
  const [batchStorageLocationId, setBatchStorageLocationId] = useState<string>('')
  const [batchStorageLocationFromId, setBatchStorageLocationFromId] = useState<string>('')
  const [batchStorageLocationToId, setBatchStorageLocationToId] = useState<string>('')

  const inputRefs = useRef<InputRefs>({})

  const generateLineId = () =>
    `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

  const collectLineValues = useCallback((lineId: string): Partial<DocumentLine> => {
    const refs = inputRefs.current[lineId]
    if (!refs) return {}
    const values: Partial<DocumentLine> = {}
    if (refs.qty) values.qty = refs.qty.value
    if (refs.unit_price) values.unit_price = refs.unit_price.value
    if (refs.expiry_date) values.expiry_date = refs.expiry_date.value
    if (refs.batch_no) values.batch_no = refs.batch_no.value
    return values
  }, [])

  const collectCurrentLineValues = useCallback(() => {
    return formData.lines.map((line) => {
      const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
      const values = collectLineValues(lineId)
      return Object.keys(values).length > 0 ? { ...line, ...values } : line
    })
  }, [formData.lines, collectLineValues])

  const collectAllLineValues = useCallback(() => {
    setFormData((prev) => {
      const updatedLines = prev.lines.map((line) => {
        const lineId = line.id || `temp-${prev.lines.indexOf(line)}`
        const values = collectLineValues(lineId)
        return Object.keys(values).length > 0 ? { ...line, ...values } : line
      })
      return { ...prev, lines: updatedLines }
    })
  }, [collectLineValues])

  const { data: document, isLoading: loadingDocument } = useQuery({
    queryKey: ['document', id],
    queryFn: async () => {
      const response = await api.get<Document>(`/documents/${id}`)
      return response.data
    },
    enabled: isEdit,
    staleTime: STALE_TIME.LIST,
  })

  const { data: products } = useQuery({
    queryKey: ['products', { keyword: productSearch, category_id: categoryId }],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (productSearch) params.append('keyword', productSearch)
      if (categoryId) params.append('category_id', categoryId)
      params.append('is_active', 'true')
      const response = await api.get<Product[]>(`/products?${params.toString()}`)
      return response.data
    },
    enabled: productSearchOpen,
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: warehouses } = useQuery({
    queryKey: ['warehouses', 'active'],
    queryFn: async () => {
      const response = await api.get<Warehouse[]>('/warehouses?is_active=true')
      return response.data || []
    },
    staleTime: STALE_TIME.REFERENCE,
    refetchOnMount: true,
  })

  const { data: partners } = useQuery({
    queryKey: ['partners'],
    queryFn: async () => {
      const response = await api.get<Partner[]>('/partners')
      return response.data || []
    },
    staleTime: STALE_TIME.REFERENCE,
    refetchOnMount: true,
  })

  const { data: activeProtocols, isLoading: loadingProtocols } = useQuery({
    queryKey: ['active-protocols'],
    queryFn: async () => {
      const response = await api.get<ProtocolListItem[]>('/protocols')
      return response.data.filter((p) => {
        // 排除已關閉的計畫
        if (p.status === 'CLOSED') return false
        
        // 僅保留已批准且有 IACUC 編號的計畫
        const isApproved = p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS'
        if (!isApproved || !p.iacuc_no) return false
        
        return true
      })
    },
    // 納入採購與銷售相關單據
    enabled: ['PO', 'PR', 'SO', 'DO'].includes(formData.doc_type),
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: poReceiptStatus } = useQuery({
    queryKey: ['po-receipt-status', formData.source_doc_id],
    queryFn: async () => {
      const response = await api.get(`/documents/${formData.source_doc_id}/receipt-status`)
      return response.data
    },
    enabled: formData.doc_type === 'GRN' && !!formData.source_doc_id,
    staleTime: STALE_TIME.LIST,
  })

  const calculateLineAmount = useCallback((lineId: string) => {
    const refs = inputRefs.current[lineId]
    if (!refs) return 0
    const qty = refs.qty?.value ? parseFloat(refs.qty.value) : 0
    const price = refs.unit_price?.value ? parseFloat(refs.unit_price.value) : 0
    return qty * price
  }, [])

  const updateLineAmount = useCallback((lineId: string) => {
    const amount = calculateLineAmount(lineId)
    setLineAmounts((prev) => {
      if (prev[lineId] === amount) return prev
      return { ...prev, [lineId]: amount }
    })
  }, [calculateLineAmount])

  const createOrFindCustomerMutation = useMutation({
    mutationFn: async (iacucNo: string) => {
      const existingPartnersResponse = await api.get<Partner[]>('/partners')
      const existingCustomer = existingPartnersResponse.data.find(
        (p) => p.partner_type === 'customer' && p.code === iacucNo
      )
      if (existingCustomer) return existingCustomer
      const newCustomerResponse = await api.post<Partner>('/partners', {
        partner_type: 'customer',
        code: iacucNo,
        name: iacucNo,
      })
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      return newCustomerResponse.data
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '創建客戶失敗'),
        variant: 'destructive',
      })
    },
  })

  useEffect(() => {
    if (document && isEdit) {
      const lines = document.lines.map((line) => ({
        id: line.id,
        line_no: line.line_no,
        product_id: line.product_id,
        product_name: line.product_name,
        product_sku: line.product_sku,
        qty: formatQuantity(line.qty),
        uom: line.uom,
        unit_price: line.unit_price ? formatUnitPrice(line.unit_price) : '',
        batch_no: line.batch_no || '',
        expiry_date: line.expiry_date || '',
        storage_location_id: line.storage_location_id || '',
        storage_location_from_id: line.storage_location_from_id || '',
        storage_location_to_id: line.storage_location_to_id || '',
        remark: line.remark || '',
      }))
      setFormData({
        doc_type: document.doc_type,
        doc_date: document.doc_date,
        warehouse_id: document.warehouse_id || '',
        warehouse_from_id: document.warehouse_from_id || '',
        warehouse_to_id: document.warehouse_to_id || '',
        partner_id: document.partner_id || '',
        protocol_no: (document as any).protocol_no || '',
        source_doc_id: document.source_doc_id || '',
        remark: document.remark || '',
        lines,
      })
      lines.forEach((line) => {
        if (line.id && !inputRefs.current[line.id]) {
          inputRefs.current[line.id] = {}
        }
      })
      if (['PO', 'GRN', 'DO'].includes(document.doc_type)) {
        const initialAmounts: Record<string, number> = {}
        lines.forEach((line) => {
          if (line.id) {
            const qty = parseFloat(line.qty) || 0
            const price = parseFloat(line.unit_price) || 0
            initialAmounts[line.id] = qty * price
          }
        })
        setLineAmounts(initialAmounts)
      }
    }
  }, [document, isEdit])

  useEffect(() => {
    if (['PO', 'GRN', 'DO'].includes(formData.doc_type)) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        setTimeout(() => updateLineAmount(lineId), 0)
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- formData.lines causes ref loops
  }, [formData.doc_type, formData.lines.length])

  useEffect(() => {
    if (!isEdit) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        if (!inputRefs.current[lineId]) {
          inputRefs.current[lineId] = {}
        }
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- formData.lines causes ref loops
  }, [formData.lines.length, isEdit])

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

  const buildPayload = useCallback(
    (data: DocumentFormData) => {
      const mergedLines = data.lines.map((line) => {
        const lineId = line.id || `temp-${data.lines.indexOf(line)}`
        const values = collectLineValues(lineId)
        return Object.keys(values).length > 0 ? { ...line, ...values } : line
      })
      const mergedData = { ...data, lines: mergedLines }
      const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO'].includes(
        mergedData.doc_type
      )
      const isTransfer = mergedData.doc_type === 'TR'

      if (needsPartner && !mergedData.partner_id?.trim()) {
        throw new Error('請選擇供應商/客戶')
      }
      if (!mergedData.warehouse_id?.trim() && !isTransfer) {
        throw new Error('請選擇倉庫')
      }
      if (
        isTransfer &&
        (!mergedData.warehouse_from_id?.trim() ||
          !mergedData.warehouse_to_id?.trim())
      ) {
        throw new Error('調撥單需要選擇來源倉庫和目標倉庫')
      }

      const validLines = mergedData.lines.filter(
        (line) => line.product_id && line.product_id.trim() !== ''
      )
      if (mergedData.doc_type !== 'STK' && validLines.length === 0) {
        throw new Error('請至少新增一項產品明細')
      }
      for (let idx = 0; idx < validLines.length; idx++) {
        const line = validLines[idx]
        if (!line.product_id?.trim()) throw new Error(`第 ${idx + 1} 行：請選擇產品`)
        const qty = parseFloat(line.qty)
        if (isNaN(qty) || qty <= 0) throw new Error(`第 ${idx + 1} 行：數量必須大於 0`)
        if (!line.uom?.trim()) throw new Error(`第 ${idx + 1} 行：請輸入單位`)

        // 貨架/儲位檢查
        if (isShelfRequired && !line.storage_location_id?.trim()) {
          throw new Error(`第 ${idx + 1} 行：儲位/貨架為必填項`)
        }

        // 強制檢查批號與效期 (特定單據類型)
        const requiresBatchExpiry = ['GRN', 'DO', 'SO', 'ADJ', 'STK'].includes(mergedData.doc_type)
        if (requiresBatchExpiry) {
          if (!line.batch_no?.trim()) throw new Error(`第 ${idx + 1} 行：批號為必填項`)
          if (!line.expiry_date?.trim()) throw new Error(`第 ${idx + 1} 行：效期為必填項`)
        }
      }

      return {
        doc_type: mergedData.doc_type,
        doc_date: mergedData.doc_date,
        warehouse_id:
          mergedData.warehouse_id?.trim() ? mergedData.warehouse_id : null,
        warehouse_from_id:
          mergedData.warehouse_from_id?.trim()
            ? mergedData.warehouse_from_id
            : null,
        warehouse_to_id:
          mergedData.warehouse_to_id?.trim()
            ? mergedData.warehouse_to_id
            : null,
        partner_id:
          mergedData.partner_id?.trim() ? mergedData.partner_id : null,
        protocol_no:
          mergedData.protocol_no?.trim() ? mergedData.protocol_no : null,
        source_doc_id:
          mergedData.source_doc_id?.trim() ? mergedData.source_doc_id : null,
        remark:
          mergedData.remark?.trim() ? mergedData.remark : null,
        lines: validLines.map((line) => ({
          product_id: line.product_id,
          qty: parseFloat(line.qty) || 0,
          uom: line.uom?.trim() || 'pcs',
          unit_price:
            line.unit_price?.trim() ? parseFloat(line.unit_price) : null,
          batch_no: line.batch_no?.trim() ? line.batch_no : null,
          expiry_date: line.expiry_date?.trim() ? line.expiry_date : null,
          storage_location_id: line.storage_location_id?.trim() ? line.storage_location_id : null,
          storage_location_from_id: line.storage_location_from_id?.trim() ? line.storage_location_from_id : null,
          storage_location_to_id: line.storage_location_to_id?.trim() ? line.storage_location_to_id : null,
          remark: line.remark?.trim() ? line.remark : null,
        })),
      }
    },
    [collectLineValues]
  )

  const saveMutation = useMutation({
    mutationFn: async () => {
      collectAllLineValues()
      const payload = buildPayload(formData)
      if (isEdit) {
        return api.put(`/documents/${id}`, payload)
      }
      return api.post('/documents', payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({
        title: '成功',
        description: isEdit ? '單據已更新' : '單據已建立',
      })
      navigate(`/documents?type=${formData.doc_type}`)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '儲存失敗'),
        variant: 'destructive',
      })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => {
      const payload = buildPayload(formData)
      if (isEdit) {
        await api.put(`/documents/${id}`, payload)
        await api.post(`/documents/${id}/submit`)
        return { documentId: id }
      }
      const createResponse = await api.post<{ id: string }>('/documents', payload)
      const documentId = createResponse.data.id
      await api.post(`/documents/${documentId}/submit`)
      return { documentId }
    },
    onSuccess: async (response: { documentId: string }) => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({ title: '成功', description: '單據已送審' })
      navigate(`/documents/${response.documentId}`)
      setTimeout(() => window.location.reload(), 500)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '送審失敗'),
        variant: 'destructive',
      })
    },
  })

  const updateField = useCallback(
    <K extends keyof DocumentFormData>(
      field: K,
      value: DocumentFormData[K]
    ) => {
      setFormData((prev) => ({ ...prev, [field]: value }))
      setUnsavedChanges(true)
    },
    []
  )

  const addLine = useCallback(() => {
    const currentLines = collectCurrentLineValues()
    const lineId = generateLineId()
    const newLine: DocumentLine = {
      id: lineId,
      line_no: currentLines.length + 1,
      product_id: '',
      qty: '1',
      uom: '',
      unit_price: '',
      batch_no: '',
      expiry_date: '',
      storage_location_id: batchStorageLocationId || '',
      storage_location_from_id: batchStorageLocationFromId || '',
      storage_location_to_id: batchStorageLocationToId || '',
      remark: '',
    }
    setFormData((prev) => ({ ...prev, lines: [...currentLines, newLine] }))
    if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}
    if (['PO', 'GRN', 'DO'].includes(formData.doc_type)) {
      setLineAmounts((prev) => ({ ...prev, [lineId]: 0 }))
    }
    setUnsavedChanges(true)
  }, [collectCurrentLineValues, formData.doc_type])

  const handleBatchChange = useCallback(
    (lineId: string, batchNo: string, expiryDate?: string, sourceIacuc?: string) => {
      const refs = inputRefs.current[lineId]
      if (refs?.batch_no) refs.batch_no.value = batchNo
      if (['SO', 'DO'].includes(formData.doc_type)) {
        if (refs?.expiry_date) refs.expiry_date.value = expiryDate || ''

        // 比對 IACUC
        if (sourceIacuc && sourceIacuc !== 'PUBLIC') {
          const currentIacucCode = formData.partner_id
            ? partners?.find((p) => p.id === formData.partner_id)?.code
            : null

          if (currentIacucCode && currentIacucCode !== sourceIacuc) {
            setIacucWarningData({ batch_no: batchNo, source_iacuc: sourceIacuc })
            setShowIacucWarning(true)
          }
        }

        setFormData((prev) => {
          const updatedLines = prev.lines.map((line) => {
            const currentLineId =
              line.id || `temp-${prev.lines.indexOf(line)}`
            if (currentLineId === lineId) {
              return { ...line, batch_no: batchNo, expiry_date: expiryDate || '' }
            }
            const otherLineValues = collectLineValues(currentLineId)
            return { ...line, ...otherLineValues }
          })
          return { ...prev, lines: updatedLines }
        })
        setUnsavedChanges(true)
      }
    },
    [formData.doc_type, formData.partner_id, partners, collectLineValues]
  )

  const handleLineBlur = useCallback(() => {
    collectAllLineValues()
    setUnsavedChanges(true)
  }, [collectAllLineValues])

  const removeLine = useCallback((lineId: string) => {
    setFormData((prev) => ({
      ...prev,
      lines: prev.lines.filter((line) => line.id !== lineId),
    }))
    delete inputRefs.current[lineId]
    setLineAmounts((prev) => {
      const updated = { ...prev }
      delete updated[lineId]
      return updated
    })
    setUnsavedChanges(true)
  }, [])

  const selectProduct = useCallback(
    (product: Product) => {
      if (selectedLineId) {
        const currentLines = collectCurrentLineValues()
        setFormData((prev) => ({
          ...prev,
          lines: currentLines.map((line) =>
            line.id === selectedLineId
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
      setSelectedLineId(null)
    },
    [selectedLineId, collectCurrentLineValues]
  )

  const openProductSearch = useCallback((lineId: string) => {
    setSelectedLineId(lineId)
    setProductSearchOpen(true)
  }, [])

  const handleBack = useCallback(() => {
    const targetPath = `/documents${formData.doc_type ? `?type=${formData.doc_type}` : ''}`
    if (unsavedChanges) {
      setPendingNavigation(targetPath)
      setShowUnsavedDialog(true)
    } else {
      navigate(targetPath)
    }
  }, [formData.doc_type, unsavedChanges, navigate])

  const confirmNavigation = useCallback(() => {
    setShowUnsavedDialog(false)
    if (pendingNavigation) {
      navigate(pendingNavigation)
    }
  }, [pendingNavigation, navigate])

  const handleIacucNoSelect = useCallback(
    async (iacucNo: string) => {
      try {
        updateField('protocol_no', iacucNo)

        // 對於銷售單/出庫單，依然需要帶出客戶
        if (['SO', 'DO'].includes(formData.doc_type)) {
          const customer =
            await createOrFindCustomerMutation.mutateAsync(iacucNo)
          updateField('partner_id', customer.id)
          toast({ title: '成功', description: `已選擇客戶：${iacucNo}` })
        } else {
          toast({ title: '成功', description: `已選擇專屬計畫：${iacucNo}` })
        }
      } catch {
        // Error handled in mutation
      }
    },
    [createOrFindCustomerMutation, updateField, formData.doc_type]
  )

  const handleBatchShelfSelect = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({
          ...prev,
          lines: prev.lines.map((l) => ({ ...l, storage_location_id: shelfId })),
        }))
        setUnsavedChanges(true)
      }
    },
    [setFormData]
  )

  const handleBatchShelfSelectFrom = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationFromId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({
          ...prev,
          lines: prev.lines.map((l) => ({ ...l, storage_location_from_id: shelfId })),
        }))
        setUnsavedChanges(true)
      }
    },
    [setFormData]
  )

  const handleBatchShelfSelectTo = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationToId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({
          ...prev,
          lines: prev.lines.map((l) => ({ ...l, storage_location_to_id: shelfId })),
        }))
        setUnsavedChanges(true)
      }
    },
    [setFormData]
  )

  const needsPartner = ['PO', 'GRN', 'PR', 'SO', 'DO'].includes(
    formData.doc_type
  )
  const isTransfer = formData.doc_type === 'TR'
  const partnerType = ['PO', 'GRN', 'PR'].includes(formData.doc_type)
    ? 'supplier'
    : 'customer'

  const filteredPartners = useMemo(() => {
    if (!partners) return undefined
    return partners.filter((p) => {
      if (!needsPartner) return true
      if (p.partner_type !== partnerType) return false
      if (partnerType === 'customer') {
        if (!activeProtocols?.length) return false
        return activeProtocols.some((protocol) => protocol.iacuc_no === p.code)
      }
      return true
    })
  }, [partners, needsPartner, partnerType, activeProtocols])

  const totalAmount = useMemo(() => {
    return formData.lines.reduce((sum, line) => {
      const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
      const amount =
        lineAmounts[lineId] !== undefined
          ? lineAmounts[lineId]
          : (parseFloat(line.qty) || 0) * (parseFloat(line.unit_price) || 0)
      return sum + amount
    }, 0)
  }, [formData.lines, lineAmounts])

  const needsShelf = !['PO'].includes(formData.doc_type)
  const isShelfRequired = !['PO'].includes(formData.doc_type)
  const isIacucRequired = ['SO', 'DO'].includes(formData.doc_type)
  const iacucDisabled = ['GRN', 'STK', 'ADJ'].includes(formData.doc_type)

  return {
    id,
    isEdit,
    formData,
    setFormData,
    updateField,
    productSearchOpen,
    setProductSearchOpen,
    productSearch,
    setProductSearch,
    selectedLineId,
    showUnsavedDialog,
    setShowUnsavedDialog,
    confirmNavigation,
    lineAmounts,
    inputRefs,
    loadingDocument,
    products,
    warehouses,
    partners,
    activeProtocols,
    loadingProtocols,
    filteredPartners,
    needsPartner,
    isTransfer,
    partnerType,
    totalAmount,
    needsShelf,
    isShelfRequired,
    isIacucRequired,
    iacucDisabled,
    collectLineValues,
    collectAllLineValues,
    addLine,
    removeLine,
    selectProduct,
    openProductSearch,
    handleBatchChange,
    handleLineBlur,
    handleBack,
    handleIacucNoSelect,
    updateLineAmount,
    createOrFindCustomerMutation,
    saveMutation,
    submitMutation,
    showIacucWarning,
    setShowIacucWarning,
    iacucWarningData,
    batchStorageLocationId,
    batchStorageLocationFromId,
    batchStorageLocationToId,
    handleBatchShelfSelect,
    handleBatchShelfSelectFrom,
    handleBatchShelfSelectTo,
    poReceiptStatus,
    source_doc_id: formData.source_doc_id,
    categoryId,
    setCategoryId,
  }
}
