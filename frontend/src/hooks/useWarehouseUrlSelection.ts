import { useSearchParams } from 'react-router-dom'
import type { StorageLocationWithWarehouse, Warehouse } from '@/lib/api'

/**
 * 倉庫佈局頁的網址選取邏輯（`?warehouse=<倉庫代碼>&location=<儲位代碼>`）：
 * 分享連結 / 重整 / 上一頁都保留選擇。用代碼（短、可讀）而非 UUID；內部 API 仍以 id 呼叫。
 *
 * 解析規則抽成純函式（resolveWarehouseFromUrl / resolveLocationFromUrl）並以單元測試
 * 釘住行為——此邏輯曾兩度真實迴歸（#984 冷載入閃爍、#988 location 防呆取捨）。
 */

/** 由網址倉庫代碼解析選中倉庫；代碼無效/未指定 → 退回第一個倉；清單未載入 → undefined
 *  （相依查詢以 enabled 等待；warehouses 為小型且常快取的查詢，此等待極短）。 */
export function resolveWarehouseFromUrl(
    warehouses: Warehouse[] | undefined,
    urlWarehouseCode: string,
): Warehouse | undefined {
    return warehouses?.find((w) => w.code === urlWarehouseCode) ?? warehouses?.[0]
}

/** 由網址儲位代碼解析選中儲位（在本倉庫的儲位清單中比對）；無效/未指定/清單未載入 → null。
 *  註：畸形連結（無效倉庫碼 + location）僅在 fallback 倉庫剛好有同碼貨架時才會誤選，
 *  屬 benign 邊界情境；先前用 urlWarehouseInvalid 硬擋反而讓壞碼永久卡住選貨架，故不擋。 */
export function resolveLocationFromUrl(
    locations: StorageLocationWithWarehouse[] | undefined,
    urlLocationCode: string,
): StorageLocationWithWarehouse | null {
    return locations?.find((l) => l.code === urlLocationCode) ?? null
}

export function useWarehouseUrlSelection(warehouses: Warehouse[] | undefined) {
    const [searchParams, setSearchParams] = useSearchParams()

    const selectedWarehouse = resolveWarehouseFromUrl(
        warehouses,
        searchParams.get('warehouse') ?? '',
    )
    const urlLocationCode = searchParams.get('location') ?? ''

    const setSelectedWarehouseId = (id: string) => {
        const code = warehouses?.find((w) => w.id === id)?.code ?? id
        setSearchParams(
            (prev) => {
                const next = new URLSearchParams(prev)
                next.set('warehouse', code)
                next.delete('location') // 換倉庫 → 清掉舊倉庫的貨架選取
                return next
            },
            { replace: true },
        )
    }

    const setSelectedLocation = (loc: StorageLocationWithWarehouse | null) => {
        setSearchParams(
            (prev) => {
                const next = new URLSearchParams(prev)
                if (loc) next.set('location', loc.code)
                else next.delete('location')
                return next
            },
            { replace: true },
        )
    }

    return {
        selectedWarehouse,
        selectedWarehouseId: selectedWarehouse?.id ?? '',
        urlLocationCode,
        setSelectedWarehouseId,
        setSelectedLocation,
    }
}
