// AUP 第 8 節人員資料的角色分類（單一來源，供 ProfileSettingsPage 與歡迎指引共用）。
//
// 兩個獨立的閘門：
//  - 訓練/資格：依角色（審查相關＋staff）決定是否顯示，非 is_internal。
//  - 入職日期 / 入職前年數：依 is_internal 決定（僅內部人員），與角色無關。

/** 需填「訓練/資格」的角色：審查相關（主委/委員/IACUC 幹事）＋ 實驗/照護 staff。 */
export const TRAINING_ROLES = [
    'IACUC_CHAIR',
    'REVIEWER',
    'IACUC_STAFF',
    'EXPERIMENT_STAFF',
    'INTERN',
    'VET',
    'ANIMAL_CARE_STAFF',
    'admin',
    'SYSTEM_ADMIN',
] as const

/** 具「AUP 第 8 節人員資料」（至少職稱）的角色：訓練角色 ∪ PI。 */
export const AUP_PERSONNEL_ROLES = [...TRAINING_ROLES, 'PI'] as const

/** 使用者角色是否落在指定集合內。 */
export function hasAnyRole(
    userRoles: string[] | undefined,
    roleSet: readonly string[],
): boolean {
    return !!userRoles?.some(r => roleSet.includes(r))
}
