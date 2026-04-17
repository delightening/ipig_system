/**
 * 單據表單主 Hook
 * 組合 useDocumentLines + useDocumentSubmit，管理表單資料與查詢
 */
import { useState, useEffect, useCallback, useMemo } from 'react'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import api, {
  Document,
  Product,
  Partner,
  Warehouse,
  DocType,
  ProtocolListItem,
} from '@/lib/api'
import { STALE_TIME } from '@/lib/query'
import { formatQuantity, formatUnitPrice } from '@/lib/utils'
import type { DocumentFormData } from '../types'
import { useDocumentLines } from './useDocumentLines'
import { useDocumentSubmit } from './useDocumentSubmit'

export type { InputRefs } from './useDocumentLines'

export interface UseDocumentFormOptions {
  defaultType: DocType
}

export function useDocumentForm({ defaultType }: UseDocumentFormOptions) {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const isCopy = searchParams.get('copy') === 'true'
  const isEdit = !!id && id !== 'new'

  const [formData, setFormData] = useState<DocumentFormData>({
    doc_type: (defaultType as string) === 'new' ? ('' as DocType) : defaultType,
    doc_date: new Date().toISOString().split('T')[0],
    warehouse_id: '',
    warehouse_from_id: '',
    warehouse_to_id: '',
    partner_id: '',
    protocol_id: '',
    protocol_no: '',
    source_doc_id: '',
    remark: '',
    lines: [],
  })

  const [unsavedChanges, setUnsavedChanges] = useState(false)
  const [showUnsavedDialog, setShowUnsavedDialog] = useState(false)
  const [pendingNavigation, setPendingNavigation] = useState<string | null>(null)

  // --- Sub-hooks ---
  const lines = useDocumentLines(formData, setFormData, setUnsavedChanges)

  // --- Queries ---
  const { data: document, isLoading: loadingDocument } = useQuery({
    queryKey: ['document', id],
    queryFn: async () => (await api.get<Document>(`/documents/${id}`)).data,
    enabled: isEdit,
    staleTime: STALE_TIME.LIST,
  })

  const { data: products } = useQuery({
    queryKey: ['products', { keyword: lines.productSearch, category_code: lines.categoryCode }],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (lines.productSearch) params.append('keyword', lines.productSearch)
      if (lines.categoryCode) params.append('category_code', lines.categoryCode)
      params.append('is_active', 'true')
      return (await api.get<Product[]>(`/products?${params.toString()}`)).data
    },
    enabled: lines.productSearchOpen,
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: warehouses } = useQuery({
    queryKey: ['warehouses', 'active'],
    queryFn: async () => (await api.get<Warehouse[]>('/warehouses?is_active=true')).data || [],
    staleTime: STALE_TIME.REFERENCE,
    refetchOnMount: true,
  })

  const { data: partners } = useQuery({
    queryKey: ['partners'],
    queryFn: async () => (await api.get<Partner[]>('/partners')).data || [],
    staleTime: STALE_TIME.REFERENCE,
    refetchOnMount: true,
  })

  const { data: activeProtocols, isLoading: loadingProtocols } = useQuery({
    queryKey: ['active-protocols'],
    queryFn: async () => {
      const response = await api.get<ProtocolListItem[]>('/protocols')
      return response.data.filter((p) => {
        if (p.status === 'CLOSED') return false
        const isApproved = p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS'
        return isApproved && !!p.iacuc_no
      })
    },
    enabled: ['PO', 'PR', 'SO', 'DO'].includes(formData.doc_type),
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: poReceiptStatus } = useQuery({
    queryKey: ['po-receipt-status', formData.source_doc_id],
    queryFn: async () => (await api.get(`/documents/${formData.source_doc_id}/receipt-status`)).data,
    enabled: formData.doc_type === 'GRN' && !!formData.source_doc_id,
    staleTime: STALE_TIME.LIST,
  })

  // --- Derived state ---
  const needsPartner = ['PO', 'GRN', 'PR'].includes(formData.doc_type)
  const needsProtocol = ['SO', 'DO'].includes(formData.doc_type)
  const isTransfer = formData.doc_type === 'TR'
  const needsShelf = !['PO', 'PR'].includes(formData.doc_type)
  const isShelfRequired = !['PO', 'PR'].includes(formData.doc_type)
  const iacucDisabled = ['GRN', 'STK', 'ADJ'].includes(formData.doc_type)

  const filteredPartners = useMemo(() => {
    if (!partners) return undefined
    return partners.filter((p) => p.partner_type === 'supplier')
  }, [partners])

  const totalAmount = useMemo(() => {
    return formData.lines.reduce((sum, line) => {
      const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
      const amount = lines.lineAmounts[lineId] !== undefined
        ? lines.lineAmounts[lineId]
        : (parseFloat(line.qty) || 0) * (parseFloat(line.unit_price) || 0)
      return sum + amount
    }, 0)
  }, [formData.lines, lines.lineAmounts])

  // --- Submit hook ---
  const { saveMutation, submitMutation } = useDocumentSubmit({
    id, isEdit, formData,
    collectLineValues: lines.collectLineValues,
    collectAllLineValues: lines.collectAllLineValues,
    setUnsavedChanges,
    products,
    isShelfRequired,
    inputRefs: lines.inputRefs,
  })

  // --- Field update ---
  const updateField = useCallback(
    <K extends keyof DocumentFormData>(field: K, value: DocumentFormData[K]) => {
      setFormData((prev) => ({ ...prev, [field]: value }))
      setUnsavedChanges(true)
    },
    []
  )

  // --- Effects ---
  useEffect(() => {
    if (!document || !isEdit) return
    const docLines = document.lines.map((line) => ({
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
      protocol_id: document.protocol_id || '',
      protocol_no: document.protocol_no || '',
      source_doc_id: document.source_doc_id || '',
      remark: document.remark || '',
      lines: docLines,
    })
    docLines.forEach((line) => {
      if (line.id && !lines.inputRefs.current[line.id]) lines.inputRefs.current[line.id] = {}
    })
    if (['PO', 'GRN', 'DO'].includes(document.doc_type)) {
      const initialAmounts: Record<string, number> = {}
      docLines.forEach((line) => {
        if (line.id) {
          initialAmounts[line.id] = (parseFloat(line.qty) || 0) * (parseFloat(line.unit_price) || 0)
        }
      })
      lines.setLineAmounts(initialAmounts)
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [document, isEdit, lines.inputRefs, lines.setLineAmounts])

  // 複製單據：從 sessionStorage 載入資料
  useEffect(() => {
    if (isEdit || !isCopy) return
    const raw = sessionStorage.getItem('document_copy_data')
    if (!raw) return
    sessionStorage.removeItem('document_copy_data')
    try {
      const parsed = JSON.parse(raw)
      // M-02: 確認 parsed 為物件且 lines 為陣列，防止 sessionStorage 被竄改後注入非預期資料
      if (typeof parsed !== 'object' || parsed === null) return
      const copyData = parsed as DocumentFormData
      setFormData({
        doc_type: copyData.doc_type || defaultType,
        doc_date: new Date().toISOString().split('T')[0],
        warehouse_id: copyData.warehouse_id || '',
        warehouse_from_id: copyData.warehouse_from_id || '',
        warehouse_to_id: copyData.warehouse_to_id || '',
        partner_id: copyData.partner_id || '',
        protocol_id: copyData.protocol_id || '',
        protocol_no: copyData.protocol_no || '',
        source_doc_id: '',
        remark: copyData.remark || '',
        lines: (Array.isArray(copyData.lines) ? copyData.lines : []).map((line, idx) => ({
          ...line,
          id: undefined,
          line_no: idx + 1,
        })),
      })
    } catch {
      // ignore invalid JSON
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isCopy, isEdit])

  useEffect(() => {
    if (['PO', 'GRN', 'DO'].includes(formData.doc_type)) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        setTimeout(() => lines.updateLineAmount(lineId), 0)
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [formData.doc_type, formData.lines.length])

  useEffect(() => {
    if (!isEdit) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        if (!lines.inputRefs.current[lineId]) lines.inputRefs.current[lineId] = {}
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [formData.lines.length, isEdit])

  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (unsavedChanges) { e.preventDefault(); e.returnValue = '' }
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [unsavedChanges])

  // --- Navigation ---
  const handleBack = useCallback(() => {
    const targetPath = `/documents${formData.doc_type && formData.doc_type !== ('' as DocType) ? `?type=${formData.doc_type}` : ''}`
    if (unsavedChanges) {
      setPendingNavigation(targetPath)
      setShowUnsavedDialog(true)
    } else {
      navigate(targetPath)
    }
  }, [formData.doc_type, unsavedChanges, navigate])

  const confirmNavigation = useCallback(() => {
    setShowUnsavedDialog(false)
    if (pendingNavigation) navigate(pendingNavigation)
  }, [pendingNavigation, navigate])

  const handleProtocolSelect = useCallback(
    (protocolId: string) => {
      updateField('protocol_id', protocolId)
      const protocol = activeProtocols?.find((p) => p.id === protocolId)
      updateField('protocol_no', protocol?.iacuc_no || protocol?.protocol_no || '')
    },
    [activeProtocols, updateField]
  )

  const handleIacucNoSelect = useCallback(
    (iacucNo: string) => updateField('protocol_no', iacucNo),
    [updateField]
  )

  // Wrap handleBatchChange to pass activeProtocols
  const handleBatchChangeWrapped = useCallback(
    (lineId: string, batchNo: string, expiryDate?: string, sourceIacuc?: string) => {
      lines.handleBatchChange(lineId, batchNo, expiryDate, sourceIacuc, activeProtocols)
    },
    [lines, activeProtocols]
  )

  return {
    id, isEdit, formData, setFormData, updateField,
    productSearchOpen: lines.productSearchOpen,
    setProductSearchOpen: lines.setProductSearchOpen,
    productSearch: lines.productSearch,
    setProductSearch: lines.setProductSearch,
    selectedLineId: lines.selectedLineId,
    showUnsavedDialog, setShowUnsavedDialog, confirmNavigation,
    lineAmounts: lines.lineAmounts,
    inputRefs: lines.inputRefs,
    loadingDocument, products, warehouses, partners,
    activeProtocols, loadingProtocols,
    filteredPartners, needsPartner, needsProtocol, isTransfer,
    totalAmount, needsShelf, isShelfRequired, iacucDisabled,
    collectLineValues: lines.collectLineValues,
    collectAllLineValues: lines.collectAllLineValues,
    addLine: lines.addLine,
    removeLine: lines.removeLine,
    selectProduct: lines.selectProduct,
    openProductSearch: lines.openProductSearch,
    handleBatchChange: handleBatchChangeWrapped,
    handleLineBlur: lines.handleLineBlur,
    handleBack, handleProtocolSelect, handleIacucNoSelect,
    updateLineAmount: lines.updateLineAmount,
    saveMutation, submitMutation,
    showIacucWarning: lines.showIacucWarning,
    setShowIacucWarning: lines.setShowIacucWarning,
    iacucWarningData: lines.iacucWarningData,
    batchStorageLocationId: lines.batchStorageLocationId,
    batchStorageLocationFromId: lines.batchStorageLocationFromId,
    batchStorageLocationToId: lines.batchStorageLocationToId,
    handleBatchShelfSelect: lines.handleBatchShelfSelect,
    handleBatchShelfSelectFrom: lines.handleBatchShelfSelectFrom,
    handleBatchShelfSelectTo: lines.handleBatchShelfSelectTo,
    poReceiptStatus,
    source_doc_id: formData.source_doc_id,
    categoryCode: lines.categoryCode,
    setCategoryCode: lines.setCategoryCode,
  }
}
