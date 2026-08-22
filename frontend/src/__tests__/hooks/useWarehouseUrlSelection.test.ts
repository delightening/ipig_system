import { describe, it, expect } from 'vitest'
import {
    resolveLocationFromUrl,
    resolveWarehouseFromUrl,
} from '@/hooks/useWarehouseUrlSelection'
import type { StorageLocationWithWarehouse, Warehouse } from '@/lib/api'

// 只造測試需要的欄位；其餘以 cast 補足（純函式僅讀 id/code）。
const wh = (id: string, code: string) => ({ id, code }) as Warehouse
const loc = (id: string, code: string) => ({ id, code }) as StorageLocationWithWarehouse

const WAREHOUSES = [wh('uuid-a', '4'), wh('uuid-b', '7')]
const LOCATIONS = [loc('loc-1', 'A11'), loc('loc-2', 'B02')]

describe('resolveWarehouseFromUrl', () => {
    // 迴歸 #984：冷載入（warehouses 尚未載入）不得閃爍選到任何倉——必須回 undefined，
    // 由相依查詢以 enabled 等待，而非先 fallback 再跳回。
    it('清單未載入（冷載入）→ undefined，不預選', () => {
        expect(resolveWarehouseFromUrl(undefined, '4')).toBeUndefined()
        expect(resolveWarehouseFromUrl(undefined, '')).toBeUndefined()
    })

    it('網址代碼有效 → 選中對應倉庫', () => {
        expect(resolveWarehouseFromUrl(WAREHOUSES, '7')).toEqual(wh('uuid-b', '7'))
    })

    it('網址代碼無效或未指定 → 退回第一個啟用倉', () => {
        expect(resolveWarehouseFromUrl(WAREHOUSES, 'nope')).toEqual(wh('uuid-a', '4'))
        expect(resolveWarehouseFromUrl(WAREHOUSES, '')).toEqual(wh('uuid-a', '4'))
    })

    it('空倉庫清單 → undefined', () => {
        expect(resolveWarehouseFromUrl([], '4')).toBeUndefined()
    })
})

describe('resolveLocationFromUrl', () => {
    // 迴歸 #988：無效 location 代碼回 null 即可（不硬擋、不卡死後續選取）。
    it('代碼無效或未指定 → null（不誤選、不擋後續操作）', () => {
        expect(resolveLocationFromUrl(LOCATIONS, 'ZZ99')).toBeNull()
        expect(resolveLocationFromUrl(LOCATIONS, '')).toBeNull()
    })

    it('清單未載入 → null', () => {
        expect(resolveLocationFromUrl(undefined, 'A11')).toBeNull()
    })

    it('代碼有效 → 選中對應儲位', () => {
        expect(resolveLocationFromUrl(LOCATIONS, 'A11')).toEqual(loc('loc-1', 'A11'))
    })
})
