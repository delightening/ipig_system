import { useState } from 'react'

/**
 * Tab 切換狀態管理。
 * 適用於 HrLeavePage、AdminAuditPage、BloodTestCostReportPage、AccountingReportPage 等有 Tabs 的頁面。
 * setActiveTab 接受 string 以相容 Radix UI Tabs 的 onValueChange。
 *
 * @param initial 初始選中的 tab 值
 * @returns activeTab、setActiveTab
 */
export function useTabState<T extends string>(initial: T) {
  const [activeTab, setActiveTabState] = useState<T>(initial)
  const setActiveTab = (value: string) => setActiveTabState(value as T)
  return { activeTab, setActiveTab }
}
