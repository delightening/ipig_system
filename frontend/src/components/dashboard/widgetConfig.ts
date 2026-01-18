// Dashboard Widget 配置和類型定義

import { ReactNode } from 'react'

// Widget 配置類型
export interface DashboardWidgetConfig {
    id: string
    visible: boolean
    order: number
}

// Widget 定義類型
export interface WidgetDefinition {
    id: string
    title: string
    description: string
    category: 'erp' | 'hr' | 'aup' | 'animal_care' | 'report'
    permission?: string // 需要的權限代碼
    component: ReactNode
    defaultVisible: boolean
    size?: 'small' | 'medium' | 'large' | 'full' // Widget 大小
}

// Widget 類別名稱對照
export const widgetCategoryNames: Record<string, string> = {
    erp: 'ERP 進銷存',
    hr: '人員管理',
    aup: '計畫管理',
    animal_care: '動物照護',
    report: '報表',
}

// 預設的 Widget 配置
export const DEFAULT_DASHBOARD_WIDGETS: DashboardWidgetConfig[] = [
    { id: 'calendar_widget', visible: true, order: 0 },
    { id: 'leave_balance', visible: true, order: 1 },
    { id: 'my_projects', visible: true, order: 2 },
    { id: 'animals_on_medication', visible: true, order: 3 },
    { id: 'vet_comments', visible: true, order: 4 },
    { id: 'low_stock_alert', visible: true, order: 5 },
    { id: 'pending_documents', visible: true, order: 6 },
    { id: 'today_inbound', visible: true, order: 7 },
    { id: 'today_outbound', visible: true, order: 8 },
    { id: 'weekly_trend', visible: true, order: 9 },
    { id: 'recent_documents', visible: true, order: 10 },
    { id: 'upcoming_leaves', visible: true, order: 11 },
    { id: 'staff_attendance', visible: true, order: 12 },
    { id: 'google_calendar_events', visible: true, order: 13 },
]

// Widget ID 對應中文名稱
export const widgetNames: Record<string, string> = {
    calendar_widget: '今日日曆',
    leave_balance: '請假餘額',
    my_projects: '我的計畫',
    animals_on_medication: '正在用藥動物',
    vet_comments: '獸醫師 Comment',
    low_stock_alert: '低庫存警示',
    pending_documents: '待處理單據',
    today_inbound: '今日入庫',
    today_outbound: '今日出庫',
    weekly_trend: '近7天趨勢',
    recent_documents: '最近單據',
    upcoming_leaves: '即將到期假期',
    staff_attendance: '工作人員出勤表',
    google_calendar_events: '日曆事件',
}

// Widget 描述
export const widgetDescriptions: Record<string, string> = {
    calendar_widget: '顯示今日日程和員工請假資訊',
    leave_balance: '顯示您的特休和補休餘額',
    my_projects: '顯示您參與的計畫及動物統計',
    animals_on_medication: '顯示目前正在用藥的動物清單',
    vet_comments: '顯示最近的獸醫師評論',
    low_stock_alert: '顯示低於安全庫存的品項',
    pending_documents: '顯示等待核准的單據數量',
    today_inbound: '顯示今日入庫單據數量',
    today_outbound: '顯示今日出庫單據數量',
    weekly_trend: '顯示近7天的出入庫趨勢',
    recent_documents: '顯示最近建立的單據',
    upcoming_leaves: '顯示即將到期的假期提醒',
    staff_attendance: '顯示工作人員出勤統計（管理職）',
    google_calendar_events: '顯示 Google Calendar 本週事件',
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
