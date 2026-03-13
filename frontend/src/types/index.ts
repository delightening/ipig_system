/**
 * 型別統一匯出入口
 *
 * 提供所有型別的集中匯出點。
 *
 * 使用方式：
 *   import type { User, Protocol, AttendanceWithUser } from '@/types'
 */

// 純型別匯出
export type * from './common'
export type * from './protocol'
export type * from './hr'
export type * from './auth'
export type * from './erp'
export type * from './animal'
export type * from './aup'
export type * from './report'
export type * from './audit'
export type * from './notification'
export type * from './amendment'
export type * from './upload'
export type * from './treatment-drug'
export type * from './signature'

// 常值匯出（非純型別，需要 runtime 值）
export { LEAVE_TYPE_NAMES, LEAVE_STATUS_NAMES } from './hr'
export {
    animalStatusNames, allAnimalStatusNames, animalBreedNames, animalGenderNames, recordTypeNames,
} from './animal'
export { protocolStatusNames } from './aup'
export { notificationTypeNames } from './notification'
export {
    amendmentStatusNames, amendmentStatusColors, amendmentTypeNames,
    AMENDMENT_CHANGE_ITEM_OPTIONS,
} from './amendment'
export { storageLocationTypeNames } from './erp'
export { DRUG_CATEGORIES, DOSAGE_UNITS } from './treatment-drug'
