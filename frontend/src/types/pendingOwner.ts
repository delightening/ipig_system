/**
 * 「這一關卡在誰手上」的統一契約。後端 `models/pending_owner.rs` 的鏡射。
 *
 * 全站 20 個以上的「待 XX」在途狀態共用這一個形狀，前端只要認得它，
 * 任何模組的徽章都能畫出同一種 hover 內容。
 */

/** 這一關的負責對象是怎麼決定的，決定文案形狀。 */
export type PendingOwnerKind =
    /** 綁角色 / 權限：任一符合者皆可處理 → 顯示「角色：人員」 */
    | 'role'
    /** 綁特定人員：只有這些人能處理 → 只顯示人員，不提角色 */
    | 'person'
    /** 球在申請人自己身上（需修正 / 補件） */
    | 'applicant'

export interface PendingOwner {
    /** i18n key 尾段，查 `pendingOwner.stage.<stage>`，例 `doc_wm_approve` */
    stage: string
    kind: PendingOwnerKind
    /** 角色代碼，查 `pendingOwner.role.<role_code>`，例 `WAREHOUSE_MANAGER` */
    role_code: string | null
    /** 已列出的人名，後端至多給 3 個 */
    candidates: string[]
    /** 未列出的人數。總人數 = candidates.length + overflow */
    overflow: number
    /** 進入本關的時間（ISO8601），用來算已等待幾天；取不到時為 null */
    since: string | null
}
