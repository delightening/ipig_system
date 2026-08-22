import { describe, it, expect } from 'vitest'
import { warehouseColor, warehouseChipStyle } from '@/pages/documents/warehouseColors'

describe('warehouseColor', () => {
    it('同索引恆定同色（跨 render 穩定）', () => {
        expect(warehouseColor(0)).toBe(warehouseColor(0))
        expect(warehouseColor(3)).toBe(warehouseColor(3))
    })

    it('相鄰索引不同色', () => {
        expect(warehouseColor(0)).not.toBe(warehouseColor(1))
        expect(warehouseColor(1)).not.toBe(warehouseColor(2))
    })

    it('超出色盤長度循環而不爆炸', () => {
        expect(warehouseColor(6)).toBe(warehouseColor(0))
        expect(warehouseColor(100)).toMatch(/^#[0-9a-f]{6}$/)
    })

    it('異常索引回退第一色（不 throw）', () => {
        expect(warehouseColor(-1)).toBe(warehouseColor(0))
        expect(warehouseColor(NaN)).toBe(warehouseColor(0))
    })
})

describe('warehouseChipStyle', () => {
    it('淡背景 = 主色 + alpha 後綴，文字 = 主色', () => {
        const { backgroundColor, color } = warehouseChipStyle(1)
        expect(color).toBe(warehouseColor(1))
        expect(backgroundColor).toBe(`${warehouseColor(1)}1f`)
    })
})
