/**
 * Document form side effects: load existing document data, sync line amounts, init refs
 */
import { useEffect } from 'react'
import { Document } from '@/lib/api'
import { formatQuantity, formatUnitPrice } from '@/lib/utils'
import type { DocumentFormData } from '../types'
import type { InputRefs } from './useDocumentLines'

interface UseDocumentFormEffectsParams {
  document: Document | undefined
  isEdit: boolean
  formData: DocumentFormData
  setFormData: React.Dispatch<React.SetStateAction<DocumentFormData>>
  inputRefs: React.MutableRefObject<InputRefs>
  setLineAmounts: React.Dispatch<React.SetStateAction<Record<string, number>>>
  updateLineAmount: (lineId: string) => void
  unsavedChanges: boolean
}

export function useDocumentFormEffects({
  document,
  isEdit,
  formData,
  setFormData,
  inputRefs,
  setLineAmounts,
  updateLineAmount,
  unsavedChanges,
}: UseDocumentFormEffectsParams) {
  // Load existing document into form
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
      if (line.id && !inputRefs.current[line.id]) inputRefs.current[line.id] = {}
    })
    if (['PO', 'GRN', 'DO'].includes(document.doc_type)) {
      const initialAmounts: Record<string, number> = {}
      docLines.forEach((line) => {
        if (line.id) {
          initialAmounts[line.id] = (parseFloat(line.qty) || 0) * (parseFloat(line.unit_price) || 0)
        }
      })
      setLineAmounts(initialAmounts)
    }
  }, [document, isEdit])

  // Recalculate line amounts when doc_type or line count changes
  useEffect(() => {
    if (['PO', 'GRN', 'DO'].includes(formData.doc_type)) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        setTimeout(() => updateLineAmount(lineId), 0)
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [formData.doc_type, formData.lines.length])

  // Init input refs for new lines
  useEffect(() => {
    if (!isEdit) {
      formData.lines.forEach((line) => {
        const lineId = line.id || `temp-${formData.lines.indexOf(line)}`
        if (!inputRefs.current[lineId]) inputRefs.current[lineId] = {}
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [formData.lines.length, isEdit])

  // Warn before unload if unsaved
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (unsavedChanges) { e.preventDefault(); e.returnValue = '' }
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [unsavedChanges])
}
