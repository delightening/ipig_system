/**
 * 單據送審/儲存 Hook
 * 負責 payload 建構、驗證、save/submit mutations
 */
import { useCallback, type MutableRefObject } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { Product } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { DocumentLine, DocumentFormData } from '../types'
import { scopeForPayload } from '../stocktakeScope'
import { lineQtyError } from '../lineQtyRules'
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
  const { t } = useTranslation()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const buildPayload = useCallback(
    (data: DocumentFormData) => {
      // 明細層級的錯誤訊息＝「第 N 行：」前綴＋規則本體；前綴永遠在句首，兩種語言語序一致。
      const lineMessage = (lineNo: number, message: string) =>
        `${t('erpDocs.documents.validation.linePrefix', { lineNo })}${message}`
      const mergedLines = data.lines.map((line) => {
        const values = collectLineValues(line.id)
        return Object.keys(values).length > 0 ? { ...line, ...values } : line
      })
      const mergedData = { ...data, lines: mergedLines }
      const needsSupplier = ['PO', 'GRN', 'PR'].includes(mergedData.doc_type)
      const needsProtocolRef = mergedData.doc_type === 'SO'
      const isTransfer = mergedData.doc_type === 'TR'

      if (needsSupplier && !mergedData.partner_id?.trim()) {
        throw new Error(t('erpDocs.documents.validation.selectSupplier'))
      }
      if (needsProtocolRef && !mergedData.protocol_id?.trim()) {
        throw new Error(t('erpDocs.documents.validation.selectSalesProtocol'))
      }
      // SO 跨倉（#1004）：表頭倉庫改選填（每行倉庫＝儲位所屬倉），不再強制。
      if (!mergedData.warehouse_id?.trim() && !isTransfer && mergedData.doc_type !== 'SO') {
        throw new Error(t('validation.selectWarehouse'))
      }
      if (isTransfer) {
        if (!mergedData.warehouse_from_id?.trim() || !mergedData.warehouse_to_id?.trim()) {
          throw new Error(t('validation.transferWarehouseRequired'))
        }
        // 2026-06-09: 開放同倉庫調撥（A 倉儲位1 → A 倉儲位2）。
        // 原 H3 規則「同倉庫即擋」立於 migration 069 之前（當時 TR 僅倉庫層級，
        // 同倉=無效操作）；069 加入 per-line 來源/目標儲位後，同倉不同儲位已是有意義的
        // 搬移，且 TR 單據本身帶送審＋stock_ledger 軌跡。改為逐行檢查「來源儲位 ≠ 目標儲位」
        // （見下方明細迴圈），避免真正的無效搬移。
      }

      const validLines = mergedData.lines.filter((line) => line.product_id && line.product_id.trim() !== '')
      if (mergedData.doc_type !== 'STK' && validLines.length === 0) throw new Error(t('validation.atLeastOneItem'))

      for (let idx = 0; idx < validLines.length; idx++) {
        const line = validLines[idx]
        if (!line.product_id?.trim()) throw new Error(lineMessage(idx + 1, t('validation.selectProduct')))
        // 數量規則依單據類型分流（STK 收 0 拒負／ADJ 收正負拒 0／其餘須 > 0），
        // 規則本體在 lineQtyRules.ts，與後端 crud.rs::validate_line_qty_price 對齊。
        // 2026-09-14：原本這裡一律「qty <= 0 即擋」，沒有任何 doc_type 分支，比後端嚴——
        //   · STK 盤到 0 送不出去，而 0 正是盤點最該登記的結果（系統有、現場沒有＝全數短少）；
        //   · ADJ 的調減（負數）一併被擋，而 payload 全程不轉正負號，等於調減開不出單。
        const qtyError = lineQtyError(mergedData.doc_type, line.qty, idx + 1)
        if (qtyError) throw new Error(qtyError)
        if (!line.uom?.trim()) throw new Error(lineMessage(idx + 1, t('validation.unitRequired')))
        if (isShelfRequired) {
          // 調撥單 (TR) 用的是 from/to 兩欄；其他單據用單一 storage_location_id。
          // 2026-05-20 fix: 原本所有 doc_type 都只檢查 storage_location_id，導致
          // TR row 即使 from/to 都填了仍報「儲位/貨架為必填項」，調撥單完全送不出。
          if (isTransfer) {
            // 拆兩條檢查 → 錯誤訊息明確指出哪個欄位缺
            if (!line.storage_location_from_id?.trim()) {
              throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.sourceLocationRequired')))
            }
            if (!line.storage_location_to_id?.trim()) {
              throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.targetLocationRequired')))
            }
            // 2026-06-09: 同倉庫調撥時，來源/目標儲位必須不同，否則為無效搬移
            if (
              mergedData.warehouse_from_id === mergedData.warehouse_to_id &&
              line.storage_location_from_id === line.storage_location_to_id
            ) {
              throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.sameLocationTransfer')))
            }
          } else if (!line.storage_location_id?.trim()) {
            throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.storageLocationRequired')))
          }
        }
        // GRN 單價必填且必須大於 0（與後端 validate_line_qty_price 一致）。
        // SO 為內部耗材領用不記金額（2026-07-21 裁定），無單價驗證。
        if (mergedData.doc_type === 'GRN') {
          const refs = inputRefs.current[line.id]
          const domPrice = refs?.unit_price?.value?.trim() || ''
          const priceStr = line.unit_price?.trim() || domPrice
          const price = priceStr ? parseFloat(priceStr) : 0
          if (!priceStr || isNaN(price) || price <= 0) {
            throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.grnPriceRequired')))
          }
        }

        // 2026-05-20 (H2): TR 加入 — TR 移動庫存，若品項 track_batch 必須指定批號
        // 否則破壞批號可追溯性（GLP §11 要求）
        // R84-13（2026-08-13）：SR 已從 DocType 移除（業務上不存在銷貨退貨），從此清單移除，
        // 與後端 requires_batch_expiry() 對齊（現涵蓋 GRN/SO/ADJ/STK/PR/TR）。
        const requiresBatchExpiry = ['GRN', 'SO', 'ADJ', 'STK', 'TR', 'PR'].includes(mergedData.doc_type)
        if (requiresBatchExpiry) {
          const product = products?.find((p) => p.id === line.product_id)
          const refs = inputRefs.current[line.id]

          // 直接從 DOM input 讀取當前值（最可靠的來源）
          const domExpiry = refs?.expiry_date?.dataset?.iso?.trim() || ''
          const domBatch = refs?.batch_no?.value?.trim() || ''

          // formData 值 OR DOM 值，任一有值即通過
          const hasExpiry = !!(line.expiry_date?.trim() || domExpiry)
          const hasBatch = !!(line.batch_no?.trim() || domBatch)

          if (product?.track_batch && !hasBatch) {
            throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.batchRequired')))
          }
          if (product?.track_expiry && !hasExpiry) {
            throw new Error(lineMessage(idx + 1, t('erpDocs.documents.validation.expiryRequired')))
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
        stocktake_scope: scopeForPayload(mergedData.doc_type, mergedData.stocktake_scope, isEdit),
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
    [collectLineValues, products, isShelfRequired, inputRefs, isEdit, t]
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
      toast({
        title: t('common.success'),
        description: isEdit ? t('erpDocs.documents.save.updated') : t('erpDocs.documents.save.created'),
      })
      navigate(`/documents?type=${formData.doc_type}`)
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('erpDocs.documents.save.saveFailed')), variant: 'destructive' })
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
      toast({ title: t('common.success'), description: t('erpDocs.documents.toast.submitted') })
      navigate(`/documents/${response.documentId}`)
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('erpDocs.documents.toast.submitFailed')), variant: 'destructive' })
    },
  })

  return { saveMutation, submitMutation }
}
