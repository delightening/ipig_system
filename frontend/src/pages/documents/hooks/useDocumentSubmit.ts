/**
 * 單據送審/儲存 Hook
 * 負責 payload 建構、驗證、save/submit mutations
 */
import { useCallback, type MutableRefObject } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Product } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { DocumentLine, DocumentFormData } from '../types'
import type { InputRefs } from './useDocumentLines'

interface UseDocumentSubmitOptions {
  id: string | undefined
  isEdit: boolean
  formData: DocumentFormData
  collectLineValues: (lineId: string) => Partial<DocumentLine>
  collectAllLineValues: () => void
  setUnsavedChanges: (v: boolean) => void
  products: Product[] | undefined
  isShelfRequired: boolean
  inputRefs: MutableRefObject<InputRefs>
}

export function useDocumentSubmit({
  id, isEdit, formData,
  collectLineValues, collectAllLineValues,
  setUnsavedChanges, products, isShelfRequired, inputRefs,
}: UseDocumentSubmitOptions) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const buildPayload = useCallback(
    (data: DocumentFormData) => {
      const mergedLines = data.lines.map((line) => {
        const lineId = line.id || `temp-${data.lines.indexOf(line)}`
        const values = collectLineValues(lineId)
        return Object.keys(values).length > 0 ? { ...line, ...values } : line
      })
      const mergedData = { ...data, lines: mergedLines }
      const needsSupplier = ['PO', 'GRN', 'PR'].includes(mergedData.doc_type)
      const needsProtocolRef = ['SO', 'DO'].includes(mergedData.doc_type)
      const isTransfer = mergedData.doc_type === 'TR'

      if (needsSupplier && !mergedData.partner_id?.trim()) throw new Error('請選擇供應商')
      if (needsProtocolRef && !mergedData.protocol_id?.trim()) throw new Error('請選擇銷貨計畫')
      if (!mergedData.warehouse_id?.trim() && !isTransfer) throw new Error('請選擇倉庫')
      if (isTransfer && (!mergedData.warehouse_from_id?.trim() || !mergedData.warehouse_to_id?.trim())) {
        throw new Error('調撥單需要選擇來源倉庫和目標倉庫')
      }

      const validLines = mergedData.lines.filter((line) => line.product_id && line.product_id.trim() !== '')
      if (mergedData.doc_type !== 'STK' && validLines.length === 0) throw new Error('請至少新增一項產品明細')

      for (let idx = 0; idx < validLines.length; idx++) {
        const line = validLines[idx]
        if (!line.product_id?.trim()) throw new Error(`第 ${idx + 1} 行：請選擇產品`)
        const qty = parseFloat(line.qty)
        if (isNaN(qty) || qty <= 0) throw new Error(`第 ${idx + 1} 行：數量必須大於 0`)
        if (!line.uom?.trim()) throw new Error(`第 ${idx + 1} 行：請輸入單位`)
        if (isShelfRequired && !line.storage_location_id?.trim()) {
          throw new Error(`第 ${idx + 1} 行：儲位/貨架為必填項`)
        }
        // GRN 單據的單價必填且不可為 0
        if (mergedData.doc_type === 'GRN') {
          const lineId = line.id || `temp-${idx}`
          const refs = inputRefs.current[lineId]
          const domPrice = refs?.unit_price?.value?.trim() || ''
          const priceStr = line.unit_price?.trim() || domPrice
          const price = priceStr ? parseFloat(priceStr) : 0
          if (!priceStr || isNaN(price) || price <= 0) {
            throw new Error(`第 ${idx + 1} 行：採購入庫的單價為必填，且不可為 0`)
          }
        }

        const requiresBatchExpiry = ['GRN', 'DO', 'SO', 'ADJ', 'STK'].includes(mergedData.doc_type)
        if (requiresBatchExpiry) {
          const product = products?.find((p) => p.id === line.product_id)
          const lineId = line.id || `temp-${idx}`
          const refs = inputRefs.current[lineId]

          // 直接從 DOM input 讀取當前值（最可靠的來源）
          const domExpiry = refs?.expiry_date?.dataset?.iso?.trim() || ''
          const domBatch = refs?.batch_no?.value?.trim() || ''

          // formData 值 OR DOM 值，任一有值即通過
          const hasExpiry = !!(line.expiry_date?.trim() || domExpiry)
          const hasBatch = !!(line.batch_no?.trim() || domBatch)

          if (product?.track_batch && !hasBatch) {
            throw new Error(`第 ${idx + 1} 行：該品項有管理批號，批號為必填項`)
          }
          if (product?.track_expiry && !hasExpiry) {
            throw new Error(`第 ${idx + 1} 行：該品項有管理效期，效期為必填項`)
          }

          // 確保 payload 帶上 DOM 讀到的值
          if (!line.expiry_date?.trim() && domExpiry) line.expiry_date = domExpiry
          if (!line.batch_no?.trim() && domBatch) line.batch_no = domBatch
        }
      }

      return {
        doc_type: mergedData.doc_type,
        doc_date: mergedData.doc_date,
        warehouse_id: mergedData.warehouse_id?.trim() ? mergedData.warehouse_id : null,
        warehouse_from_id: mergedData.warehouse_from_id?.trim() ? mergedData.warehouse_from_id : null,
        warehouse_to_id: mergedData.warehouse_to_id?.trim() ? mergedData.warehouse_to_id : null,
        partner_id: mergedData.partner_id?.trim() ? mergedData.partner_id : null,
        protocol_id: mergedData.protocol_id?.trim() ? mergedData.protocol_id : null,
        source_doc_id: mergedData.source_doc_id?.trim() ? mergedData.source_doc_id : null,
        remark: mergedData.remark?.trim() ? mergedData.remark : null,
        lines: validLines.map((line) => ({
          product_id: line.product_id,
          qty: parseFloat(line.qty) || 0,
          uom: line.uom?.trim() || 'pcs',
          unit_price: line.unit_price?.trim() ? parseFloat(line.unit_price) : null,
          batch_no: line.batch_no?.trim() ? line.batch_no : null,
          expiry_date: line.expiry_date?.trim() ? line.expiry_date : null,
          storage_location_id: line.storage_location_id?.trim() ? line.storage_location_id : null,
          storage_location_from_id: line.storage_location_from_id?.trim() ? line.storage_location_from_id : null,
          storage_location_to_id: line.storage_location_to_id?.trim() ? line.storage_location_to_id : null,
          remark: line.remark?.trim() ? line.remark : null,
        })),
      }
    },
    [collectLineValues, products, isShelfRequired, inputRefs]
  )

  const saveMutation = useMutation({
    mutationFn: async () => {
      collectAllLineValues()
      const payload = buildPayload(formData)
      if (isEdit) return api.put(`/documents/${id}`, payload)
      return api.post('/documents', payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      setUnsavedChanges(false)
      toast({ title: '成功', description: isEdit ? '單據已更新' : '單據已建立' })
      navigate(`/documents?type=${formData.doc_type}`)
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(error, '儲存失敗'), variant: 'destructive' })
    },
  })

  const submitMutation = useMutation({
    mutationFn: async () => {
      collectAllLineValues()
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
    onSuccess: async (response: { documentId: string | undefined }) => {
      await queryClient.invalidateQueries({ queryKey: ['documents'] })
      if (response.documentId) {
        await queryClient.invalidateQueries({ queryKey: ['document', response.documentId] })
      }
      setUnsavedChanges(false)
      toast({ title: '成功', description: '單據已送審' })
      navigate(`/documents/${response.documentId}`)
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(error, '送審失敗'), variant: 'destructive' })
    },
  })

  return { saveMutation, submitMutation }
}
