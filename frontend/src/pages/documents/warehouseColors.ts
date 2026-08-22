/**
 * 倉別穩定色盤（SO 跨倉明細的倉別 chip / 色點用）。
 * 以倉庫在 /warehouses/with-shelves 樹中的索引取色（穩定、不依名稱 hash）；
 * chip 一律帶倉名文字，顏色僅輔助（色覺無障礙）。
 */
const WAREHOUSE_PALETTE = [
    '#2563eb', // blue
    '#059669', // emerald
    '#d97706', // amber
    '#7c3aed', // violet
    '#0891b2', // cyan
    '#dc2626', // red
] as const

/** 倉庫索引 → 主色（超出色盤循環使用） */
export function warehouseColor(index: number): string {
    if (!Number.isFinite(index) || index < 0) return WAREHOUSE_PALETTE[0]
    return WAREHOUSE_PALETTE[index % WAREHOUSE_PALETTE.length]
}

/** 倉庫索引 → chip 樣式（淡底 + 主色文字）；hex 加 alpha 後綴產生淡背景 */
export function warehouseChipStyle(index: number): { backgroundColor: string; color: string } {
    const c = warehouseColor(index)
    return { backgroundColor: `${c}1f`, color: c }
}
