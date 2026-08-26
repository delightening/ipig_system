import { describe, it, expect } from 'vitest'

import { getMaintenanceBadge } from '@/pages/admin/types'

/**
 * 2026-08-26：使用者反映設備「維修/保養」有 6 筆待驗收沒人處理，其中 **5 筆是保養類**，
 * 最久一筆從完修（2026-07-13）到驗收擱了 44 天。
 *
 * 直接原因之一就在這支函式：原本 `type === 'maintenance'` 直接早退回傳藍色「保養」、
 * 完全不看 `status`，於是待驗收的保養紀錄與**已完修**的長得一模一樣，要點進去才知道。
 *
 * 本測試鎖住修正後的規則，重點是**兩個方向都測**：
 * - 待驗收必須蓋掉類型徽章（否則回歸就是那 44 天重演）
 * - 其餘狀態的保養必須維持藍色（否則會變成「一律顯示狀態」，類型資訊消失）
 *
 * 只寫前者的話，把整個 `pending_review` 判斷拔掉會紅；只寫後者的話拔掉不會紅——
 * 判準是「把這個機制整個拔掉，哪一支會紅」。
 */
describe('getMaintenanceBadge', () => {
    it('待驗收不分類型一律顯示紫色「待驗收」', () => {
        expect(getMaintenanceBadge('maintenance', 'pending_review')).toEqual({
            variant: 'purple',
            labelKey: 'admin.maintenanceLabels.status.pending_review',
        })
        expect(getMaintenanceBadge('repair', 'pending_review')).toEqual({
            variant: 'purple',
            labelKey: 'admin.maintenanceLabels.status.pending_review',
        })
    })

    it('保養的其餘狀態維持藍色「保養」，完成與否由完修日期欄判讀', () => {
        for (const status of ['pending', 'in_progress', 'completed', 'unrepairable'] as const) {
            expect(getMaintenanceBadge('maintenance', status)).toEqual({
                variant: 'info',
                labelKey: 'admin.maintenanceLabels.type.maintenance',
            })
        }
    })

    it('維修依狀態上色', () => {
        expect(getMaintenanceBadge('repair', 'completed').variant).toBe('success')
        expect(getMaintenanceBadge('repair', 'unrepairable').variant).toBe('error')
        expect(getMaintenanceBadge('repair', 'in_progress').variant).toBe('neutral')
        expect(getMaintenanceBadge('repair', 'pending').variant).toBe('warning')
    })
})
