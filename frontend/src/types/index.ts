/**
 * 型別統一匯出入口
 *
 * 提供所有型別的集中匯出點。
 *
 * 使用方式：
 *   import type { User, Protocol, AttendanceWithUser } from '@/types'
 */

export type * from './common'
export type * from './protocol'
export type * from './hr'

// 常值匯出（非純型別，需要 runtime 值）
export { LEAVE_TYPE_NAMES, LEAVE_STATUS_NAMES } from './hr'
