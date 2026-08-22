import { describe, it, expect, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useState } from 'react'
import { useDocumentLines, generateLineId } from '@/pages/documents/hooks/useDocumentLines'
import type { DocumentFormData, DocumentLine } from '@/pages/documents/types'

function makeLine(overrides: Partial<DocumentLine> = {}): DocumentLine {
  return {
    id: generateLineId(),
    line_no: 1,
    product_id: '',
    qty: '1',
    uom: '',
    unit_price: '',
    batch_no: '',
    expiry_date: '',
    remark: '',
    ...overrides,
  }
}

function emptyForm(lines: DocumentLine[] = []): DocumentFormData {
  return {
    doc_type: 'GRN',
    doc_date: '2026-05-19',
    warehouse_id: '',
    warehouse_from_id: '',
    warehouse_to_id: '',
    partner_id: '',
    remark: '',
    lines,
  }
}

function useHarness(initialLines: DocumentLine[] = []) {
  const [formData, setFormData] = useState<DocumentFormData>(emptyForm(initialLines))
  const setUnsavedChanges = vi.fn()
  const lines = useDocumentLines(formData, setFormData, setUnsavedChanges)
  return { formData, setFormData, lines }
}

describe('useDocumentLines', () => {
  describe('updateLineField', () => {
    it('updates the targeted line matched by id', () => {
      const lineA = makeLine({ line_no: 1, product_id: 'p1' })
      const lineB = makeLine({ line_no: 2, product_id: 'p2' })
      const { result } = renderHook(() => useHarness([lineA, lineB]))

      act(() => {
        result.current.lines.updateLineField(lineA.id, 'storage_location_id', 'shelf-uuid-1')
      })

      expect(result.current.formData.lines[0].storage_location_id).toBe('shelf-uuid-1')
      // 第二行不應被誤動
      expect(result.current.formData.lines[1].storage_location_id).toBeUndefined()
    })

    it('does not touch unrelated lines', () => {
      const lineA = makeLine({ line_no: 1 })
      const lineB = makeLine({ line_no: 2 })
      const { result } = renderHook(() => useHarness([lineA, lineB]))

      act(() => {
        result.current.lines.updateLineField(lineB.id, 'storage_location_id', 'shelf-B')
      })

      expect(result.current.formData.lines[0].storage_location_id).toBeUndefined()
      expect(result.current.formData.lines[1].storage_location_id).toBe('shelf-B')
    })
  })

  describe('addLine', () => {
    it('always assigns a unique generated id to new lines', () => {
      const { result } = renderHook(() => useHarness())

      act(() => { result.current.lines.addLine() })
      act(() => { result.current.lines.addLine() })

      expect(result.current.formData.lines).toHaveLength(2)
      // 型別已收緊為必填 id；此處 assert 實際內容（非空、互不相同、temp- 前綴）
      expect(result.current.formData.lines[0].id).toMatch(/^temp-/)
      expect(result.current.formData.lines[1].id).toMatch(/^temp-/)
      expect(result.current.formData.lines[0].id).not.toBe(result.current.formData.lines[1].id)
    })
  })

  describe('generateLineId', () => {
    it('returns a unique-ish temp-prefixed string each call', () => {
      const a = generateLineId()
      const b = generateLineId()
      expect(a).toMatch(/^temp-/)
      expect(b).toMatch(/^temp-/)
      expect(a).not.toBe(b)
    })
  })
})
