export const EXPERIMENT_STAFF = 'EXPERIMENT_STAFF'
/** 補登審查者角色：執秘 = IACUC_STAFF、委員 = REVIEWER、獸醫 = VET */
export const IACUC_STAFF = 'IACUC_STAFF'
export const REVIEWER = 'REVIEWER'
export const VET = 'VET'
/** 下拉中代表「其他（院外 / 外部，非系統使用者）」的 sentinel 值 */
export const PI_OTHER = '__OTHER__'

export interface UserOption {
  id: string
  display_name?: string
  username?: string
  email?: string
  roles?: string[]
}

export function userLabel(u: UserOption): string {
  return u.display_name || u.username || u.email || u.id
}

/** 是否為試驗工作人員（EXPERIMENT_STAFF）；PI 下拉排除、SD 下拉僅保留。 */
export const isStaff = (u: UserOption) => (u.roles ?? []).includes(EXPERIMENT_STAFF)
