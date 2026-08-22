/**
 * 「本場受僱人員 / 外部人員」的角色推導規則。
 *
 * 獨立成模組而非放在元件檔：從元件檔匯出非元件會破壞 React Fast Refresh，
 * 而這條規則需要被單元測試直接驗證——它決定新建帳號的預選值，選錯的後果
 * （無法加入部門、假單找不到主管簽核）是靜默的。
 */

/** 明確屬於外部的角色：這些人不受僱於本場 */
const DEFINITELY_EXTERNAL = new Set(['PI', 'CLIENT'])

/**
 * 明確屬於內部的角色：這些職能只有本場同仁擔任。
 *
 * ⚠️ 沒有列在這裡、也沒有列在 `DEFINITELY_EXTERNAL` 的角色是**刻意留白**：
 * `REVIEWER` / `VET` / `IACUC_CHAIR` / `QAU` 同一個角色可能是自己人也可能是
 * 外聘——IACUC 委員與獸醫目前都是外聘，但未來不排除內部化。
 *
 * 這種情況回 `null`（不預選、強迫使用者判斷）。預填的答案幾乎沒有人會回頭
 * 檢查，那只是把一個錯誤猜測換成另一個錯誤猜測。
 *
 * ⚠️ **不要改用 `roles.is_internal`**：那是角色層級的旗標，實測 REVIEWER /
 * VET / IACUC_CHAIR 在角色表都標為 `true`，與實際擔任者的身分不符。
 */
const DEFINITELY_INTERNAL = new Set([
    'EXPERIMENT_STAFF',
    'ADMIN_STAFF',
    'DIRECTOR',
    'PURCHASING',
    'WAREHOUSE_MANAGER',
    'EQUIPMENT_MAINTENANCE',
    'IACUC_STAFF',
    'INTERN',
])

/**
 * 依已選角色推導身分；無法判定時回 `null`（＝不預選，必須自己選）。
 *
 * 內外部混選一律回 `null`——那種組合本來就需要人來判斷。
 */
export function deriveAffiliation(selectedRoleCodes: string[]): boolean | null {
    if (selectedRoleCodes.length === 0) return null
    const hasInternal = selectedRoleCodes.some(c => DEFINITELY_INTERNAL.has(c))
    const hasExternal = selectedRoleCodes.some(c => DEFINITELY_EXTERNAL.has(c))
    if (hasInternal && !hasExternal) return true
    if (hasExternal && !hasInternal) return false
    return null
}
