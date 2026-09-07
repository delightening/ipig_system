import { useQuery } from '@tanstack/react-query'
import { Info } from 'lucide-react'
import api from '@/lib/api'
import type { Warehouse } from '@/types/erp'

interface Props {
    /** 目前選定的倉庫；未選時不顯示任何東西 */
    warehouseId?: string
}

/**
 * 開盤點單時，若選到「不排例行盤點」的倉庫就說明一句。
 *
 * ## 為什麼需要它
 *
 * 2026-09-07 使用者裁定：儲藏室每天領用，盤了也馬上變，因此不排例行盤點，
 * 改成缺貨或有異狀時才盤；廢棄物處理區裡的東西不是庫存資產，同樣不盤。
 * 其餘五個地點維持月盤。
 *
 * 這條規則存在 `warehouses.skip_routine_stocktake`（migration 014），但
 * **後端沒有任何邏輯讀它**——本系統沒有盤點排程器，月盤是人工開單。
 * 少了這個提示，那個旗標就只是一個沒有人會看到的欄位。
 *
 * ## 為什麼是提示而不是擋下來
 *
 * 「不排例行盤點」不等於「不准盤」。儲藏室正是要在缺貨或異狀時單獨盤——
 * 那是整套規則唯一的修正機制。擋下來會把校正路徑一起封死。
 *
 * 所以這裡只說明「這個倉庫不在月盤名單內」，讓開單的人確認自己不是照月盤
 * 慣例誤開，而不是阻止他。
 */
export function StocktakeExemptNotice({ warehouseId }: Props) {
    const { data: warehouses } = useQuery({
        queryKey: ['warehouses'],
        queryFn: async () => (await api.get<Warehouse[]>('/warehouses')).data,
        staleTime: 5 * 60 * 1000,
        enabled: !!warehouseId,
    })

    if (!warehouseId) return null
    const wh = warehouses?.find(w => w.id === warehouseId)
    if (!wh?.skip_routine_stocktake) return null

    return (
        <div className="col-span-2 flex items-start gap-2 rounded-lg border border-sky-300 bg-sky-50 p-3 text-sm dark:border-sky-800 dark:bg-sky-950/30">
            <Info className="mt-0.5 h-4 w-4 shrink-0 text-sky-600 dark:text-sky-400" aria-hidden="true" />
            <span>
                「{wh.name}」<strong>不在例行盤點名單內</strong>
                ——這裡的東西流動快，帳面不維護準確度。
                若你是照每月盤點的慣例開這張單，請確認是否選錯倉庫；
                若是因為<strong>領不出來或發現帳實不符</strong>而要校正，那就對了，請繼續。
            </span>
        </div>
    )
}

export default StocktakeExemptNotice
