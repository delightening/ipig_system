/**
 * 單據明細行管理 Hook
 * 負責新增、刪除、選取品項、批號處理、儲位批次套用
 */
import { useState, useCallback, useRef } from 'react'
import { Product, ProtocolListItem } from '@/lib/api'
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

const generateLineId = () =>
  `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

export function useDocumentLines(
  formData: DocumentFormData,
  setFormData: React.Dispatch<React.SetStateAction<DocumentFormData>>,
  setUnsavedChanges: (v: boolean) => void,
) {
  const [productSearchOpen, setProductSearchOpen] = useState(false)
  const [productSearch, setProductSearch] = useState('')
  const [categoryCode, setCategoryCode] = useState<string | undefined>()
  const [selectedLineId, setSelectedLineId] = useState<string | null>(null)
  const [lineAmounts, setLineAmounts] = useState<Record<string, number>>({})
  const [showIacucWarning, setShowIacucWarning] = useState(false)
  const [iacucWarningData, setIacucWarningData] = useState<{ batch_no: string; source_iacuc: string } | null>(null)
  const [batchStorageLocationId, setBatchStorageLocationId] = useState<string>('')
  const [batchStorageLocationFromId, setBatchStorageLocationFromId] = useState<string>('')
  const [batchStorageLocationToId, setBatchStorageLocationToId] = useState<string>('')

  const inputRefs = useRef<InputRefs>({})

  const updateLineField = useCallback(
    <K extends keyof DocumentLine>(lineId: string, field: K, value: DocumentLine[K]) => {
      setFormData((prev) => ({
        ...prev,
        lines: prev.lines.map((line) => {
          const curId = line.id || `temp-${prev.lines.indexOf(line)}`
          return curId === lineId ? { ...line, [field]: value } : line
        }),
      }))
      setUnsavedChanges(true)
    },
    [setFormData, setUnsavedChanges],
  )

  const collectLineValues = useCallback((lineId: string): Partial<DocumentLine> => {
    const refs = inputRefs.current[lineId]
    if (!refs) return {}
    const values: Partial<DocumentLine> = {}
    // qty 為 controlled input，不從 DOM ref 讀取，避免 stale ref / re-mount 資料遺失
    if (refs.unit_price) values.unit_price = refs.unit_price.value
    if (refs.expiry_date) {
      // DateTextInput stores ISO value in data-iso attribute (React-rendered, always in sync)
      const el = refs.expiry_date as HTMLInputElement
      const iso = el.dataset?.iso || ''
      if (iso) values.expiry_date = iso
    }
    if (refs.batch_no) {
      const batchVal = refs.batch_no.value
      if (batchVal) values.batch_no = batchVal
    }
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
  }, [collectLineValues, setFormData])

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
  }, [collectCurrentLineValues, formData.doc_type, batchStorageLocationId, batchStorageLocationFromId, batchStorageLocationToId, setFormData, setUnsavedChanges])

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
  }, [setFormData, setUnsavedChanges])

  const selectProduct = useCallback(
    (product: Product) => {
      if (selectedLineId) {
        const currentLines = collectCurrentLineValues()
        setFormData((prev) => ({
          ...prev,
          lines: currentLines.map((line) =>
            line.id === selectedLineId
              ? { ...line, product_id: product.id, product_name: product.name, product_sku: product.sku, uom: product.base_uom }
              : line
          ),
        }))
        setUnsavedChanges(true)
      }
      setProductSearchOpen(false)
      setProductSearch('')
      setSelectedLineId(null)
    },
    [selectedLineId, collectCurrentLineValues, setFormData, setUnsavedChanges]
  )

  const openProductSearch = useCallback((lineId: string) => {
    setSelectedLineId(lineId)
    setProductSearchOpen(true)
  }, [])

  const handleBatchChange = useCallback(
    (lineId: string, batchNo: string, expiryDate?: string, sourceIacuc?: string, activeProtocols?: ProtocolListItem[]) => {
      const refs = inputRefs.current[lineId]
      if (refs?.batch_no) refs.batch_no.value = batchNo
      if (refs?.expiry_date && expiryDate !== undefined) {
        const dateEl = refs.expiry_date as HTMLInputElement & { setIsoValue?: (v: string) => void }
        if (dateEl.setIsoValue) dateEl.setIsoValue(expiryDate || '')
      }

      if (sourceIacuc && sourceIacuc !== 'PUBLIC' && ['SO', 'DO'].includes(formData.doc_type)) {
        const currentProtocol = formData.protocol_id
          ? activeProtocols?.find((p) => p.id === formData.protocol_id)
          : null
        const currentIacucCode = currentProtocol?.iacuc_no
        if (currentIacucCode && currentIacucCode !== sourceIacuc) {
          setIacucWarningData({ batch_no: batchNo, source_iacuc: sourceIacuc })
          setShowIacucWarning(true)
        }
      }

      setFormData((prev) => {
        const updatedLines = prev.lines.map((line) => {
          const currentLineId = line.id || `temp-${prev.lines.indexOf(line)}`
          if (currentLineId === lineId) {
            return { ...line, batch_no: batchNo, expiry_date: expiryDate || '' }
          }
          const otherLineValues = collectLineValues(currentLineId)
          return { ...line, ...otherLineValues }
        })
        return { ...prev, lines: updatedLines }
      })
      setUnsavedChanges(true)
    },
    [formData.doc_type, formData.protocol_id, collectLineValues, setFormData, setUnsavedChanges]
  )

  const handleLineBlur = useCallback(() => {
    collectAllLineValues()
    setUnsavedChanges(true)
  }, [collectAllLineValues, setUnsavedChanges])

  const handleBatchShelfSelect = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({ ...prev, lines: prev.lines.map((l) => ({ ...l, storage_location_id: shelfId })) }))
        setUnsavedChanges(true)
      }
    },
    [setFormData, setUnsavedChanges]
  )

  const handleBatchShelfSelectFrom = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationFromId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({ ...prev, lines: prev.lines.map((l) => ({ ...l, storage_location_from_id: shelfId })) }))
        setUnsavedChanges(true)
      }
    },
    [setFormData, setUnsavedChanges]
  )

  const handleBatchShelfSelectTo = useCallback(
    (shelfId: string) => {
      setBatchStorageLocationToId(shelfId)
      if (shelfId) {
        setFormData((prev) => ({ ...prev, lines: prev.lines.map((l) => ({ ...l, storage_location_to_id: shelfId })) }))
        setUnsavedChanges(true)
      }
    },
    [setFormData, setUnsavedChanges]
  )

  return {
    inputRefs,
    productSearchOpen, setProductSearchOpen,
    productSearch, setProductSearch,
    categoryCode, setCategoryCode,
    selectedLineId,
    lineAmounts, setLineAmounts,
    showIacucWarning, setShowIacucWarning,
    iacucWarningData,
    batchStorageLocationId, batchStorageLocationFromId, batchStorageLocationToId,
    collectLineValues, collectAllLineValues,
    updateLineAmount, addLine, removeLine,
    selectProduct, openProductSearch,
    handleBatchChange, handleLineBlur,
    handleBatchShelfSelect, handleBatchShelfSelectFrom, handleBatchShelfSelectTo,
    updateLineField,
  }
}
