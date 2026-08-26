import type { NotificationKind } from '@/types/notification'

/**
 * 通知 → 目的地路徑的**唯一**對照表。
 *
 * 這張表原本各自複製在 NotificationDropdown、NotificationsPage 兩處，
 * 分家後驚嘆號入口會是第三處。三份副本已經漂移：
 * 通知中心那份少了 `report`、且 `low_stock` 沒帶 `?filter=`（點進去看到的是未過濾的庫存頁）。
 * 抽成一處後這些差異一併收斂到較完整的那份。
 *
 * 回傳 `null` ＝ 這則通知沒有可跳轉的目的地（呼叫端據此決定不顯示外連圖示）。
 */
export function notificationTargetPath(notification: {
    related_entity_type?: string
    related_entity_id?: string
    kind?: NotificationKind
}): string | null {
    const { related_entity_type: type, related_entity_id: id, kind } = notification
    if (!type) return null

    switch (type) {
        // 以下四種都要 id 才有落點。`related_entity_id` 是選填，少了它直接內插
        // 會產出 `/protocols/undefined` —— 呼叫端只看 `if (path)`，於是會顯示外連
        // 圖示、點下去導到不存在的頁面。缺 id 一律回 null，與下面 euthanasia 一致。
        case 'protocol':
            return id ? `/protocols/${id}` : null
        case 'document':
            return id ? `/documents/${id}` : null
        case 'animal':
            return id ? `/animals/${id}` : null
        case 'amendment':
            return id ? `/protocols/amendments/${id}` : null
        // kind='action' 的請假/加班通知（代理確認、單位主管/負責人待審）都是「待我審核」
        // 分頁才看得到、按得下去的項目——落在預設的「我的請假/加班」分頁等於要使用者自己
        // 找路。kind='info'（如核准/駁回/取消通知）給申請人看的，維持落在預設分頁。
        case 'leave_request':
            return kind === 'action' ? '/hr/leaves?tab=approvals' : '/hr/leaves'
        case 'overtime_record':
        case 'overtime':
            return kind === 'action' ? '/hr/overtime?tab=approvals' : '/hr/overtime'
        case 'euthanasia_order':
        case 'euthanasia_appeal':
            // 這兩種以動物為落點，沒帶 id 就無處可去
            return id ? `/animals/${id}` : null
        case 'invitation':
            return '/hr/invitations'
        case 'expiry_warning':
            return '/inventory?filter=expiry_warning'
        case 'low_stock':
            return '/inventory?filter=low_stock'
        case 'equipment':
            return '/equipment'
        // 設備三種關卡待辦各自落在自己的分頁。落在 `/equipment` 預設分頁等於要使用者
        // 自己找路——那些紀錄在各自的分頁裡，預設分頁上看不到。
        case 'maintenance_record':
            return '/equipment?tab=maintenance'
        case 'equipment_disposal':
            // 分頁 id 是複數（`EquipmentPage.tsx` 的 `PageTabContent value="disposals"`），
            // 與 entity type 的單數不同，不要照抄。
            return '/equipment?tab=disposals'
        case 'equipment_idle_request':
            return '/equipment?tab=idle'
        case 'vet_patrol_reports':
            return '/vet-patrol-reports'
        case 'report':
            return '/admin/settings'
        default:
            return null
    }
}
