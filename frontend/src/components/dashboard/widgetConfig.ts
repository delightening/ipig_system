// Dashboard Widget 配置和類型定義

import { ReactNode } from 'react'
import { LayoutItem } from 'react-grid-layout/legacy'

// Grid 配置常數
export const GRID_COLS = 12  // 12 列網格
export const GRID_ROW_HEIGHT = 80  // 每行高度 px

// R34-11：react-grid-layout responsive 斷點與每斷點欄數（從 DashboardPage 內聯遷出）
// 加密斷點：在既有 lg/md/sm/xs/xxs 之上「加掛」xl / xxl，讓大螢幕（1920 最大化、2K/4K、超寬）
// 各自落到獨立斷點、各存一份手動排版。既有門檻維持不動（零行為變更），僅向上新增。
//
// ⚠️ 門檻看的是「grid 容器寬度」而非螢幕解析度——容器 = 視窗寬度扣掉側邊欄 / padding，
// 且受視窗是否最大化、瀏覽器縮放、DPI 縮放影響。故採級距（落到最接近斷點），不用精確解析度。
// xl=1600 / xxl=2200 的取值：1920 螢幕的容器約 1680~1888、2560 約 2320~2528（視 sidebar overlay
// 與否），兩種情況都能分別落入 xl / xxl，而 1440 級（容器約 1200~1408）維持在 lg。數值可再微調。
export const GRID_BREAKPOINTS = { xxl: 2200, xl: 1600, lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 } as const
export const GRID_COLS_BY_BREAKPOINT = { xxl: 16, xl: 14, lg: 12, md: 9, sm: 6, xs: 4, xxs: 2 } as const

// 斷點鍵（寬→窄）。iteration / nearest 計算用。
export type BreakpointKey = keyof typeof GRID_BREAKPOINTS
export const BREAKPOINT_KEYS: BreakpointKey[] = ['xxl', 'xl', 'lg', 'md', 'sm', 'xs', 'xxs']

/**
 * 由容器寬度推導「作用中斷點」。
 *
 * 比較採嚴格大於，與 react-grid-layout 的 `getBreakpointFromWidth` 同語意
 * （寬度恰等於門檻時落在下一個較窄的斷點，例如 1200 → md）。兩者必須一致：
 * 我們據此記錄目前斷點，若與 grid 實際渲染的斷點不同，編輯模式拖曳會把座標
 * 存到錯的斷點（例如把 xl 的排版寫進 lg base）。
 *
 * @param width 容器寬度 px
 * @returns 該寬度對應的斷點鍵；寬度 0（尚未排版）回傳最窄的 `xxs`
 */
export function breakpointFromWidth(width: number): BreakpointKey {
    for (const key of BREAKPOINT_KEYS) {
        if (width > GRID_BREAKPOINTS[key]) return key
    }
    return 'xxs'
}

// 單一 widget 在某斷點的座標（不含身分 / 顯示狀態，那些屬 widget 層級、跨斷點共用）
export interface GridPos { x: number; y: number; w: number; h: number }

// 各斷點的座標 override：bp -> (widgetId -> 座標)。稀疏儲存——使用者實際拖過的斷點才有；
// 其餘斷點即時由 buildResponsiveLayouts 衍生（lg 為 base，不入此表）。
export type BreakpointLayouts = Partial<Record<BreakpointKey, Record<string, GridPos>>>

// Widget 選項類型（用於特定 widget 的額外配置）
export interface WidgetOptions {
    days?: number // 用於 weekly_trend 的天數設定
    shortcuts?: string[] // 用於 quick_actions：使用者選用的捷徑 action id（對應 QUICK_ACTION_CATALOG）
}

// 快捷按鈕（quick_actions）動作型錄：使用者從中勾選要顯示的捷徑。
// route 連到既有頁面；permission 未填＝不過濾（一律可見），頁面/後端仍自行把關。
export interface QuickActionDef {
    id: string
    label: string                       // zh 標籤（型錄為資料，直接用中文）
    route: string
    permission?: string
    category: 'erp' | 'aup' | 'animal_care'
}

// label 為 i18n key（對應 dashboard.quickActions.actions.*），由元件以 t() 渲染。
export const QUICK_ACTION_CATALOG: QuickActionDef[] = [
    // ERP 單據類
    { id: 'new_document', label: 'dashboard.quickActions.actions.new_document', route: '/documents/new', permission: 'erp.document.create', category: 'erp' },
    // AUP 計畫類
    { id: 'new_protocol', label: 'dashboard.quickActions.actions.new_protocol', route: '/protocols/new', category: 'aup' },
    { id: 'import_protocol', label: 'dashboard.quickActions.actions.import_protocol', route: '/protocols/import-approved', permission: 'aup.protocol.import_approved', category: 'aup' },
    // 動物管理類
    { id: 'available_pigs', label: 'dashboard.quickActions.actions.available_pigs', route: '/animals/available', permission: 'animal.animal.view_all', category: 'animal_care' },
    { id: 'animal_list', label: 'dashboard.quickActions.actions.animal_list', route: '/animals', permission: 'animal.animal.view_all', category: 'animal_care' },
]

// Widget 佈局項目（react-grid-layout 相容格式）
export interface WidgetLayoutItem {
    i: string       // Widget ID
    x: number       // 起始列 (0-11)
    y: number       // 起始行
    w: number       // 寬度(列數)
    h: number       // 高度(行數)
    minW?: number   // 最小寬度
    minH?: number   // 最小高度
    maxW?: number   // 最大寬度
    maxH?: number   // 最大高度
    static?: boolean // 是否固定不可移動
    isDraggable?: boolean
    isResizable?: boolean
    // 自訂屬性
    visible?: boolean
    options?: WidgetOptions
}

// Dashboard 完整配置
export interface DashboardConfig {
    layout: WidgetLayoutItem[]
}

// 「重設佈局」最佳化重排（方法 A：保序 + 垂直壓實）。
// first-fit 貪婪打包：依傳入順序，把每個 widget 放進「放得下、y 最小、其次 x 最小」的位置，
// 不重疊、垂直無多餘空隙。寬度夾到欄數內。只處理傳入的 widget（通常為「有權限且可見」的子集），
// 故會自動適應每個使用者實際看得到的 widget 數量。
export function packLayoutCompact(items: WidgetLayoutItem[], cols: number = GRID_COLS): WidgetLayoutItem[] {
    const occupied: boolean[][] = []
    const fits = (x: number, y: number, w: number, h: number): boolean => {
        for (let yy = y; yy < y + h; yy++) {
            const row = occupied[yy]
            if (!row) continue
            for (let xx = x; xx < x + w; xx++) if (row[xx]) return false
        }
        return true
    }
    const mark = (x: number, y: number, w: number, h: number): void => {
        for (let yy = y; yy < y + h; yy++) {
            if (!occupied[yy]) occupied[yy] = new Array(cols).fill(false)
            for (let xx = x; xx < x + w; xx++) occupied[yy][xx] = true
        }
    }
    const placeOne = (item: WidgetLayoutItem): WidgetLayoutItem => {
        const w = Math.min(item.w, cols)
        for (let y = 0; ; y++) {
            for (let x = 0; x <= cols - w; x++) {
                if (fits(x, y, w, item.h)) {
                    mark(x, y, w, item.h)
                    return { ...item, x, y, w }
                }
            }
        }
    }
    return items.map(placeOne)
}

// 手機（xs / xxs）單欄堆疊的 widget 重要度順序：即時警示 / 可立即行動 的排前面。
// 涵蓋倉儲/ERP 與現場照護兩種現場使用者；實際顯示會再經權限過濾，故各角色各看到自己那串。
// 未列入此清單的 widget 一律排在最後（依原順序）。調整現場優先序只需改動本陣列。
export const MOBILE_WIDGET_PRIORITY: string[] = [
    'quick_actions',
    'low_stock_alert',
    'expiry_alert',
    'animals_on_medication',
    'vet_comments',
    'pending_documents',
    'today_inbound',
    'today_outbound',
    'calendar_widget',
    'my_projects',
    'equipment_status',
    'staff_attendance',
    'recent_documents',
    'recent_maintenance',
    'weekly_trend',
    'google_calendar_events',
    'leave_balance',
    'upcoming_leaves',
]

// 依 priority 將 widget 排成單欄（手機用）：每個 widget x=0、滿欄寬、y 依序累加，
// 保證零重疊。沿用各 widget 原本高度。未列入 priority 者排最後。
export function stackByPriority<T extends { i: string; h: number; minW?: number }>(
    items: T[],
    cols: number,
    priority: string[],
): (T & { x: number; y: number; w: number })[] {
    const rank = (id: string) => {
        const idx = priority.indexOf(id)
        return idx === -1 ? priority.length : idx
    }
    const ordered = [...items].sort((a, b) => rank(a.i) - rank(b.i))
    let y = 0
    return ordered.map((item) => {
        const placed = {
            ...item,
            x: 0,
            y,
            w: cols,
            minW: item.minW !== undefined ? Math.min(item.minW, cols) : undefined,
        }
        y += item.h
        return placed
    })
}

// dashboard_widgets 偏好的儲存格式（v2）。後端把整包當 opaque JSON 存。
// widgets：標準 widget 集（身分 + 顯示/隱藏 + 選項 + lg 基準座標），沿用所有既有邏輯。
// byBreakpoint：各斷點的座標 override（lg 不入此表，以 widgets 為準）。
export interface DashboardWidgetsPref {
    v: 2
    widgets: WidgetLayoutItem[]
    byBreakpoint: BreakpointLayouts
}

// 將後端回傳的偏好值正規化為 v2。相容舊格式（v1：純 WidgetLayoutItem[] 陣列＝僅 lg 座標）。
// 不認得的值（null / 非預期物件）退回預設佈局。
export function normalizeDashboardPref(value: unknown): DashboardWidgetsPref {
    if (Array.isArray(value)) {
        return { v: 2, widgets: value as WidgetLayoutItem[], byBreakpoint: {} }
    }
    if (value && typeof value === 'object' && Array.isArray((value as DashboardWidgetsPref).widgets)) {
        const pref = value as DashboardWidgetsPref
        // 防禦：byBreakpoint 必須是物件（非 primitive / 非陣列），否則退回空物件，
        // 避免 DB 偏好 JSON 損壞時後續存取造成執行期錯誤。
        const byBreakpoint =
            pref.byBreakpoint && typeof pref.byBreakpoint === 'object' && !Array.isArray(pref.byBreakpoint)
                ? pref.byBreakpoint
                : {}
        return { v: 2, widgets: pref.widgets, byBreakpoint }
    }
    return { v: 2, widgets: DEFAULT_DASHBOARD_LAYOUT, byBreakpoint: {} }
}

// 將座標夾到指定欄數內（寬度、minW、起始列）。
function clampToCols(item: LayoutItem, cols: number): LayoutItem {
    const w = Math.min(item.w, cols)
    const minW = item.minW !== undefined ? Math.min(item.minW, cols) : undefined
    return { ...item, w, minW, x: Math.min(item.x, cols - w) }
}

// 將 override 座標套回 base 佈局（取得身分 / 高度 / 約束）；override 沒涵蓋的 widget 沿用 base 座標。
function applyPositions(baseLayout: LayoutItem[], positions: Record<string, GridPos>): LayoutItem[] {
    return baseLayout.map((item) => {
        const pos = positions[item.i]
        return pos ? { ...item, x: pos.x, y: pos.y, w: pos.w, h: pos.h } : item
    })
}

// 找出距離目標斷點「最近的已存佈局」作為 seed（新斷點首次進入時的預設排版）。
// 候選 = lg（base，恆存在）+ 有 override 的斷點；以斷點門檻寬度差最小者勝。
function nearestSavedLayout(
    bp: BreakpointKey,
    baseLayout: LayoutItem[],
    byBreakpoint: BreakpointLayouts,
): LayoutItem[] {
    const target = GRID_BREAKPOINTS[bp]
    let best = { dist: Math.abs(GRID_BREAKPOINTS.lg - target), layout: baseLayout }
    for (const key of BREAKPOINT_KEYS) {
        const override = byBreakpoint[key]
        if (!override || key === bp || key === 'lg') continue
        const dist = Math.abs(GRID_BREAKPOINTS[key] - target)
        if (dist < best.dist) best = { dist, layout: applyPositions(baseLayout, override) }
    }
    return best.layout
}

// 由桌機正規佈局（lg base）＋各斷點 override 建構所有斷點佈局：
// - lg：base 本身。
// - 有 override 的斷點：套用存檔座標（夾到欄數內）。
// - xs / xxs（手機）：依重要度單欄堆疊（stackByPriority），避免窄螢幕重疊。
// - 其餘無 override 的斷點（xxl / xl / md / sm）：seed 自最近的已存佈局並夾到欄數內。
export function buildResponsiveLayouts(
    baseLayout: LayoutItem[],
    byBreakpoint: BreakpointLayouts = {},
): Record<BreakpointKey, LayoutItem[]> {
    const result = {} as Record<BreakpointKey, LayoutItem[]>
    for (const bp of BREAKPOINT_KEYS) {
        const cols = GRID_COLS_BY_BREAKPOINT[bp]
        const override = byBreakpoint[bp]
        if (bp === 'lg') {
            result[bp] = baseLayout
        } else if (override) {
            result[bp] = applyPositions(baseLayout, override).map((item) => clampToCols(item, cols))
        } else if (bp === 'xs' || bp === 'xxs') {
            result[bp] = stackByPriority(baseLayout, cols, MOBILE_WIDGET_PRIORITY)
        } else {
            result[bp] = nearestSavedLayout(bp, baseLayout, byBreakpoint).map((item) => clampToCols(item, cols))
        }
    }
    return result
}

// Widget 定義類型（靜態定義）
export interface WidgetDefinition {
    id: string
    title: string
    description: string
    category: 'erp' | 'hr' | 'aup' | 'animal_care' | 'report'
    permission?: string
    component: ReactNode
    defaultVisible: boolean
    minW?: number  // 最小寬度
    minH?: number  // 最小高度
    maxW?: number  // 最大寬度
    maxH?: number  // 最大高度
}

// Widget 類別名稱對照
export const widgetCategoryNames: Record<string, { label: string; translate: boolean }> = {
    general: { label: 'dashboard.widgets.categories.general', translate: true },
    erp: { label: 'dashboard.widgets.categories.erp', translate: true },
    hr: { label: 'dashboard.widgets.categories.hr', translate: true },
    aup: { label: 'dashboard.widgets.categories.aup', translate: true },
    animal_care: { label: 'dashboard.widgets.categories.animal_care', translate: true },
    report: { label: 'dashboard.widgets.categories.report', translate: true },
}

// 各 Widget 的限制條件
export const widgetConstraints: Record<string, { minW: number; minH: number; maxW?: number; maxH?: number }> = {
    quick_actions: { minW: 2, minH: 1, maxW: 12 },
    calendar_widget: { minW: 2, minH: 2, maxW: 12 },
    leave_balance: { minW: 2, minH: 2, maxW: 4 },
    my_projects: { minW: 2, minH: 2, maxW: 8 },
    animals_on_medication: { minW: 2, minH: 2, maxW: 8 },
    vet_comments: { minW: 2, minH: 2, maxW: 8 },
    low_stock_alert: { minW: 2, minH: 2, maxW: 4 },
    expiry_alert: { minW: 2, minH: 2, maxW: 4 },
    pending_documents: { minW: 2, minH: 2, maxW: 4 },
    today_inbound: { minW: 2, minH: 2, maxW: 4 },
    today_outbound: { minW: 2, minH: 2, maxW: 4 },
    weekly_trend: { minW: 2, minH: 2, maxW: 12 },
    recent_documents: { minW: 2, minH: 2, maxW: 12 },
    recent_maintenance: { minW: 2, minH: 2, maxW: 12 },
    equipment_status: { minW: 2, minH: 3, maxW: 6 },
    upcoming_leaves: { minW: 2, minH: 2, maxW: 4 },
    staff_attendance: { minW: 2, minH: 2, maxW: 12 },
    google_calendar_events: { minW: 2, minH: 2, maxW: 8 },
}

// Widget 選項配置（哪些 widget 有額外選項）
export interface WidgetOptionDefinition {
    type: 'number'
    label: string
    key: keyof WidgetOptions
    min?: number
    max?: number
    default: number
}

export const widgetOptionsConfig: Record<string, WidgetOptionDefinition[]> = {
    weekly_trend: [
        { type: 'number', label: 'dashboard.settings.days', key: 'days', min: 3, max: 7, default: 7 }
    ]
}

// 預設的 Dashboard 佈局配置
// x: 起始列 (0-11), y: 起始行, w: 寬度(列數), h: 高度(行數)
export const DEFAULT_DASHBOARD_LAYOUT: WidgetLayoutItem[] = [
    // 頂部: 快捷按鈕（全寬一列；預設帶幾個常用捷徑，使用者可於設定中增減）
    { i: 'quick_actions', x: 0, y: 0, w: 12, h: 2, visible: true, minW: 2, minH: 1, options: { shortcuts: ['new_document', 'new_protocol', 'available_pigs'] } },

    // 第一行: 今日日曆(4列) + 獸醫師評論(3列) + 正在用藥動物(2列)
    { i: 'calendar_widget', x: 0, y: 0, w: 4, h: 3, visible: true, minW: 2, minH: 2 },
    { i: 'vet_comments', x: 4, y: 0, w: 3, h: 3, visible: true, minW: 2, minH: 2 },
    { i: 'animals_on_medication', x: 7, y: 0, w: 2, h: 6, visible: true, minW: 2, minH: 2 },

    // 第二行: 我的計畫(4列) + 請假餘額(3列)
    { i: 'my_projects', x: 0, y: 3, w: 4, h: 3, visible: true, minW: 2, minH: 2 },
    { i: 'leave_balance', x: 4, y: 3, w: 3, h: 3, visible: true, minW: 2, minH: 2 },

    // 第三行: 低庫存警示(2列) + 過期警示(2列) + 最近單據(5列)
    // R35-17: 7 天內到期 widget 與 low_stock_alert 並排
    { i: 'low_stock_alert', x: 0, y: 6, w: 2, h: 2, visible: true, minW: 2, minH: 2 },
    { i: 'expiry_alert', x: 2, y: 6, w: 2, h: 2, visible: true, minW: 2, minH: 2 },
    { i: 'today_inbound', x: 0, y: 8, w: 2, h: 2, visible: true, minW: 2, minH: 2 },
    { i: 'recent_documents', x: 4, y: 6, w: 5, h: 4, visible: true, minW: 2, minH: 2 },

    // 第四行右側: 今日出庫(2列) + 待處理單據(2列，避開 staff_attendance 的 cols 4-8 row 10-12)
    { i: 'today_outbound', x: 2, y: 8, w: 2, h: 2, visible: true, minW: 2, minH: 2 },
    { i: 'pending_documents', x: 9, y: 10, w: 2, h: 2, visible: true, minW: 2, minH: 2 },

    // 維修/保養紀錄 (與最近單據同列右側)
    { i: 'recent_maintenance', x: 9, y: 6, w: 3, h: 4, visible: true, minW: 2, minH: 2 },

    // 設備狀態總覽 (右欄頂部，與前兩行並排)
    { i: 'equipment_status', x: 9, y: 0, w: 3, h: 6, visible: true, minW: 2, minH: 3 },

    // 第五行: 近7天趨勢(4列) + 員工出勤表(5列)
    { i: 'weekly_trend', x: 0, y: 10, w: 4, h: 3, visible: true, minW: 2, minH: 2, options: { days: 7 } },
    { i: 'staff_attendance', x: 4, y: 10, w: 5, h: 3, visible: true, minW: 2, minH: 2 },

    // 第六行: Google 日曆事件(9列)
    { i: 'google_calendar_events', x: 0, y: 13, w: 9, h: 5, visible: true, minW: 2, minH: 2 },

    // 即將到期假期 (預設隱藏)
    { i: 'upcoming_leaves', x: 4, y: 7, w: 2, h: 2, visible: false, minW: 2, minH: 2 },
]


// Widget ID 對應名稱
export const widgetNames: Record<string, { label: string; translate: boolean }> = {
    quick_actions: { label: 'dashboard.widgets.names.quick_actions', translate: true },
    calendar_widget: { label: 'dashboard.widgets.names.calendar_widget', translate: true },
    leave_balance: { label: 'dashboard.widgets.names.leave_balance', translate: true },
    my_projects: { label: 'dashboard.widgets.names.my_projects', translate: true },
    animals_on_medication: { label: 'dashboard.widgets.names.animals_on_medication', translate: true },
    vet_comments: { label: 'dashboard.widgets.names.vet_comments', translate: true },
    low_stock_alert: { label: 'dashboard.widgets.names.low_stock_alert', translate: true },
    expiry_alert: { label: 'dashboard.widgets.names.expiry_alert', translate: true },
    pending_documents: { label: 'dashboard.widgets.names.pending_documents', translate: true },
    today_inbound: { label: 'dashboard.widgets.names.today_inbound', translate: true },
    today_outbound: { label: 'dashboard.widgets.names.today_outbound', translate: true },
    weekly_trend: { label: 'dashboard.widgets.names.weekly_trend', translate: true },
    recent_documents: { label: 'dashboard.widgets.names.recent_documents', translate: true },
    recent_maintenance: { label: 'dashboard.widgets.names.recent_maintenance', translate: true },
    equipment_status: { label: 'dashboard.widgets.names.equipment_status', translate: true },
    upcoming_leaves: { label: 'dashboard.widgets.names.upcoming_leaves', translate: true },
    staff_attendance: { label: 'dashboard.widgets.names.staff_attendance', translate: true },
    google_calendar_events: { label: 'dashboard.widgets.names.google_calendar_events', translate: true },
}

// Widget 描述
export const widgetDescriptions: Record<string, { label: string; translate: boolean }> = {
    quick_actions: { label: 'dashboard.widgets.descriptions.quick_actions', translate: true },
    calendar_widget: { label: 'dashboard.widgets.descriptions.calendar_widget', translate: true },
    leave_balance: { label: 'dashboard.widgets.descriptions.leave_balance', translate: true },
    my_projects: { label: 'dashboard.widgets.descriptions.my_projects', translate: true },
    animals_on_medication: { label: 'dashboard.widgets.descriptions.animals_on_medication', translate: true },
    vet_comments: { label: 'dashboard.widgets.descriptions.vet_comments', translate: true },
    low_stock_alert: { label: 'dashboard.widgets.descriptions.low_stock_alert', translate: true },
    expiry_alert: { label: 'dashboard.widgets.descriptions.expiry_alert', translate: true },
    pending_documents: { label: 'dashboard.widgets.descriptions.pending_documents', translate: true },
    today_inbound: { label: 'dashboard.widgets.descriptions.today_inbound', translate: true },
    today_outbound: { label: 'dashboard.widgets.descriptions.today_outbound', translate: true },
    weekly_trend: { label: 'dashboard.widgets.descriptions.weekly_trend', translate: true },
    recent_documents: { label: 'dashboard.widgets.descriptions.recent_documents', translate: true },
    recent_maintenance: { label: 'dashboard.widgets.descriptions.recent_maintenance', translate: true },
    equipment_status: { label: 'dashboard.widgets.descriptions.equipment_status', translate: true },
    upcoming_leaves: { label: 'dashboard.widgets.descriptions.upcoming_leaves', translate: true },
    staff_attendance: { label: 'dashboard.widgets.descriptions.staff_attendance', translate: true },
    google_calendar_events: { label: 'dashboard.widgets.descriptions.google_calendar_events', translate: true },
}

// Widget 權限要求
// 閘門須對齊「該 widget 呼叫的後端端點所要求的權限」，否則使用者看得到 widget
// 卻收到 403 → 顯示「載入失敗」。值為陣列時採 OR 語意（有其一即可）。
export const widgetPermissions: Record<string, string | string[] | undefined> = {
    quick_actions: undefined,
    // 後端 GET /hr/dashboard/calendar 要求 hr.attendance.view_all（回傳全體請假/日程），
    // 故 widget 閘門須對齊；否則僅有 hr.attendance.view 的 EXPERIMENT_STAFF 會看到 widget
    // 卻收到 403 → 顯示「載入失敗」。
    calendar_widget: 'hr.attendance.view_all',
    leave_balance: 'hr.balance.view',
    my_projects: undefined,
    // GET /animals 需 animal.animal.view_all 或 view_project（OR）
    animals_on_medication: ['animal.animal.view_all', 'animal.animal.view_project'],
    // GET /animals/vet-comments 需 animal.record.view
    vet_comments: 'animal.record.view',
    low_stock_alert: 'erp',
    expiry_alert: 'erp',
    pending_documents: 'erp',
    today_inbound: 'erp',
    today_outbound: 'erp',
    weekly_trend: 'erp',
    recent_documents: 'erp',
    recent_maintenance: 'erp',
    equipment_status: 'erp',
    upcoming_leaves: 'hr.leave.view',
    staff_attendance: 'admin',
    // GET /hr/calendar/events 需 hr.calendar.view（非 hr.attendance.view；
    // admin 持 attendance.view 但無 calendar.view，舊閘門會讓 admin 見 widget 卻 403）
    google_calendar_events: 'hr.calendar.view',
}

// Widget 類別
export const widgetCategories: Record<string, string> = {
    quick_actions: 'general',
    calendar_widget: 'hr',
    leave_balance: 'hr',
    my_projects: 'aup',
    animals_on_medication: 'animal_care',
    vet_comments: 'animal_care',
    low_stock_alert: 'erp',
    expiry_alert: 'erp',
    pending_documents: 'erp',
    today_inbound: 'erp',
    today_outbound: 'erp',
    weekly_trend: 'erp',
    recent_documents: 'erp',
    recent_maintenance: 'erp',
    equipment_status: 'erp',
    upcoming_leaves: 'hr',
    staff_attendance: 'report',
    google_calendar_events: 'hr',
}
