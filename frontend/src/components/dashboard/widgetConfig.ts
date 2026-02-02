// Dashboard Widget 配置和類型定義

import { ReactNode } from 'react'

// Grid 配置常數
export const GRID_COLS = 12  // 12 列網格
export const GRID_ROW_HEIGHT = 80  // 每行高度 px

// Widget 選項類型（用於特定 widget 的額外配置）
export interface WidgetOptions {
    days?: number // 用於 weekly_trend 的天數設定
}

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
export const widgetCategoryNames: Record<string, string> = {
    erp: 'dashboard.widgets.categories.erp',
    hr: 'dashboard.widgets.categories.hr',
    aup: 'dashboard.widgets.categories.aup',
    animal_care: 'dashboard.widgets.categories.animal_care',
    report: 'dashboard.widgets.categories.report',
}

// 各 Widget 的限制條件
export const widgetConstraints: Record<string, { minW: number; minH: number; maxW?: number; maxH?: number }> = {
    calendar_widget: { minW: 4, minH: 2, maxW: 12 },
    leave_balance: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    my_projects: { minW: 3, minH: 2, maxW: 8 },
    animals_on_medication: { minW: 3, minH: 2, maxW: 8 },
    vet_comments: { minW: 3, minH: 2, maxW: 8 },
    low_stock_alert: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    pending_documents: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    today_inbound: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    today_outbound: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    weekly_trend: { minW: 4, minH: 2, maxW: 12 },
    recent_documents: { minW: 4, minH: 2, maxW: 12 },
    upcoming_leaves: { minW: 2, minH: 1, maxW: 4, maxH: 2 },
    staff_attendance: { minW: 6, minH: 2, maxW: 12 },
    google_calendar_events: { minW: 3, minH: 2, maxW: 8 },
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
    // 第一行: 今日日曆(4列) + 請假餘額(2列) + 我的計畫(3列) + 獸醫師評論(3列)
    { i: 'calendar_widget', x: 0, y: 0, w: 4, h: 3, visible: true, minW: 4, minH: 2 },
    { i: 'leave_balance', x: 4, y: 0, w: 2, h: 1, visible: true, minW: 2, minH: 1 },
    { i: 'my_projects', x: 6, y: 0, w: 3, h: 2, visible: true, minW: 3, minH: 2 },
    { i: 'vet_comments', x: 9, y: 0, w: 3, h: 2, visible: true, minW: 3, minH: 2 },

    // 第二行: 正在用藥動物 + 最近單據
    { i: 'animals_on_medication', x: 4, y: 1, w: 2, h: 2, visible: true, minW: 2, minH: 2 },
    { i: 'recent_documents', x: 6, y: 2, w: 6, h: 3, visible: true, minW: 4, minH: 2 },

    // 第三行: ERP 小 widgets (低庫存, 待處理, 今日入庫, 今日出庫)
    { i: 'low_stock_alert', x: 0, y: 3, w: 2, h: 1, visible: true, minW: 2, minH: 1 },
    { i: 'pending_documents', x: 2, y: 3, w: 2, h: 1, visible: true, minW: 2, minH: 1 },
    { i: 'today_inbound', x: 4, y: 3, w: 2, h: 1, visible: true, minW: 2, minH: 1 },
    { i: 'today_outbound', x: 0, y: 4, w: 2, h: 1, visible: true, minW: 2, minH: 1 },

    // 第四行: 近7天趨勢
    { i: 'weekly_trend', x: 0, y: 5, w: 6, h: 3, visible: true, minW: 4, minH: 2, options: { days: 7 } },

    // 第五行: 日曆事件 + 工作人員出勤表
    { i: 'google_calendar_events', x: 0, y: 8, w: 4, h: 3, visible: true, minW: 3, minH: 2 },
    { i: 'staff_attendance', x: 4, y: 8, w: 8, h: 3, visible: true, minW: 6, minH: 2 },

    // 即將到期假期
    { i: 'upcoming_leaves', x: 4, y: 5, w: 2, h: 1, visible: false, minW: 2, minH: 1 },
]


// Widget ID 對應名稱 Key
export const widgetNames: Record<string, string> = {
    calendar_widget: 'dashboard.widgets.names.calendar_widget',
    leave_balance: 'dashboard.widgets.names.leave_balance',
    my_projects: 'dashboard.widgets.names.my_projects',
    animals_on_medication: 'dashboard.widgets.names.animals_on_medication',
    vet_comments: 'dashboard.widgets.names.vet_comments',
    low_stock_alert: 'dashboard.widgets.names.low_stock_alert',
    pending_documents: 'dashboard.widgets.names.pending_documents',
    today_inbound: 'dashboard.widgets.names.today_inbound',
    today_outbound: 'dashboard.widgets.names.today_outbound',
    weekly_trend: 'dashboard.widgets.names.weekly_trend',
    recent_documents: 'dashboard.widgets.names.recent_documents',
    upcoming_leaves: 'dashboard.widgets.names.upcoming_leaves',
    staff_attendance: 'dashboard.widgets.names.staff_attendance',
    google_calendar_events: 'dashboard.widgets.names.google_calendar_events',
}

// Widget 描述 Key
export const widgetDescriptions: Record<string, string> = {
    calendar_widget: 'dashboard.widgets.descriptions.calendar_widget',
    leave_balance: 'dashboard.widgets.descriptions.leave_balance',
    my_projects: 'dashboard.widgets.descriptions.my_projects',
    animals_on_medication: 'dashboard.widgets.descriptions.animals_on_medication',
    vet_comments: 'dashboard.widgets.descriptions.vet_comments',
    low_stock_alert: 'dashboard.widgets.descriptions.low_stock_alert',
    pending_documents: 'dashboard.widgets.descriptions.pending_documents',
    today_inbound: 'dashboard.widgets.descriptions.today_inbound',
    today_outbound: 'dashboard.widgets.descriptions.today_outbound',
    weekly_trend: 'dashboard.widgets.descriptions.weekly_trend',
    recent_documents: 'dashboard.widgets.descriptions.recent_documents',
    upcoming_leaves: 'dashboard.widgets.descriptions.upcoming_leaves',
    staff_attendance: 'dashboard.widgets.descriptions.staff_attendance',
    google_calendar_events: 'dashboard.widgets.descriptions.google_calendar_events',
}

// Widget 權限要求
export const widgetPermissions: Record<string, string | undefined> = {
    calendar_widget: undefined,
    leave_balance: undefined,
    my_projects: undefined,
    animals_on_medication: undefined,
    vet_comments: undefined,
    low_stock_alert: 'erp',
    pending_documents: 'erp',
    today_inbound: 'erp',
    today_outbound: 'erp',
    weekly_trend: 'erp',
    recent_documents: 'erp',
    upcoming_leaves: undefined,
    staff_attendance: 'admin',
    google_calendar_events: undefined,
}

// Widget 類別
export const widgetCategories: Record<string, string> = {
    calendar_widget: 'hr',
    leave_balance: 'hr',
    my_projects: 'aup',
    animals_on_medication: 'animal_care',
    vet_comments: 'animal_care',
    low_stock_alert: 'erp',
    pending_documents: 'erp',
    today_inbound: 'erp',
    today_outbound: 'erp',
    weekly_trend: 'erp',
    recent_documents: 'erp',
    upcoming_leaves: 'hr',
    staff_attendance: 'report',
    google_calendar_events: 'hr',
}
