// 疫苗/驅蟲結構化選項
//
// 主要來源：藥物選單主檔（treatment_drug_options，category=疫苗/驅蟲），管理員可在
// 藥物選單頁（/admin/treatment-drugs）直接增減品項，不必改程式碼部署（2026-08 使用者
// 裁定，取代原本寫死在本檔的清單）。
//
// 本檔僅保留「附註用」的靜態備援——委託人／計畫主持人／Study Director 這三個角色
// 只有 animal.animal.view_project，沒有讀藥物選單 API 所需的 animal.animal.view_all
// （2026-07-05 pentest F1 刻意收緊），拉不到即時清單時，這份備援讓原有 6 個常規疫苗
// 代碼＋ ivermectin 仍能正確顯示中文全稱，不會整批退化成裸代碼。新增的疫苗/驅蟲品項
// 只在藥物選單頁維護，不會同步進這份備援——對這三個唯讀角色而言，退化成顯示代碼是
// 可接受的已知限制（使用者裁定）。

export interface VaccinationOption {
    value: string
    label: string
}

/** 「其他」永遠是 UI 端的自由輸入出口，不是藥物庫的真實項目，不從 API 拉、由呼叫端手動附加 */
export const OTHER_OPTION: VaccinationOption = { value: 'OTHER', label: '其他' }

/** 疫苗代碼 → 中文全稱附註備援 */
export const FALLBACK_VACCINE_LABELS: Readonly<Record<string, string>> = {
    APP: 'APP 放線桿菌胸膜肺炎',
    SEP: 'SEP 黴漿菌',
    PCV2: 'PCV2 環狀病毒',
    PR: 'PR 假性狂犬病',
    AR: 'AR 萎縮性鼻炎',
    SE: 'SE 豬丹毒',
}

/** 驅蟲藥名 → 顯示全稱附註備援 */
export const FALLBACK_DEWORMER_LABELS: Readonly<Record<string, string>> = {
    ivermectin: 'Ivermectin',
}

/** 疫苗代碼 → 中文全稱：優先查即時清單，查無查附註備援，仍查無顯示原字串
 *（涵蓋「其他」自訂文字與既有舊資料） */
export function vaccineLabel(value: string | null | undefined, apiOptions: readonly VaccinationOption[]): string {
    if (!value) return ''
    return apiOptions.find((o) => o.value === value)?.label ?? FALLBACK_VACCINE_LABELS[value] ?? value
}

/** 驅蟲藥名 → 顯示全稱：優先查即時清單，查無查附註備援，仍查無顯示原字串 */
export function dewormerLabel(value: string | null | undefined, apiOptions: readonly VaccinationOption[]): string {
    if (!value) return ''
    return apiOptions.find((o) => o.value === value)?.label ?? FALLBACK_DEWORMER_LABELS[value] ?? value
}
