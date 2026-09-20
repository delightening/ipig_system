/**
 * 密碼複雜度驗證模組
 *
 * 統一前後端密碼規則：
 * - 至少 10 字元
 * - 必須包含大寫字母、小寫字母、數字
 * - 不得使用常見弱密碼
 */

import i18n from '@/lib/i18n'

/** 常見弱密碼黑名單（小寫比較） */
const COMMON_WEAK_PASSWORDS: ReadonlySet<string> = new Set([
  '123456',
  'password',
  'qwerty',
  '12345678',
  'abc123',
  'password1',
  'admin',
  'letmein',
  'welcome',
  'monkey',
  '1234567890',
  'qwerty123',
  'iloveyou',
  'admin123',
  'password123',
  'changeme',
  'changeme123',
  'p@ssw0rd',
  'passw0rd',
  '123456789',
  '111111',
  'sunshine',
  'princess',
  'football',
  'shadow',
  'master',
  'dragon',
  'login',
  'baseball',
  'trustno1',
])

/** 密碼最低長度 */
export const PASSWORD_MIN_LENGTH = 10

export interface PasswordCheckResult {
  /** 是否至少 10 字元 */
  length: boolean
  /** 是否包含大寫字母 */
  uppercase: boolean
  /** 是否包含小寫字母 */
  lowercase: boolean
  /** 是否包含數字 */
  number: boolean
  /** 是否不在弱密碼黑名單中 */
  notCommon: boolean
}

/** 逐項檢查密碼複雜度 */
export function checkPasswordComplexity(password: string): PasswordCheckResult {
  return {
    length: password.length >= PASSWORD_MIN_LENGTH,
    uppercase: /[A-Z]/.test(password),
    lowercase: /[a-z]/.test(password),
    number: /[0-9]/.test(password),
    notCommon: !COMMON_WEAK_PASSWORDS.has(password.toLowerCase()),
  }
}

/** 密碼是否符合所有規則 */
export function isPasswordValid(password: string): boolean {
  const checks = checkPasswordComplexity(password)
  return checks.length && checks.uppercase && checks.lowercase && checks.number && checks.notCommon
}

/** 取得密碼第一個不通過的中文錯誤訊息，全部通過回傳 null */
export function getPasswordError(password: string): string | null {
  const checks = checkPasswordComplexity(password)
  if (!checks.length) return i18n.t('auth.passwordRules.minLength', { min: PASSWORD_MIN_LENGTH })
  if (!checks.uppercase) return i18n.t('auth.passwordRules.needUppercase')
  if (!checks.lowercase) return i18n.t('auth.passwordRules.needLowercase')
  if (!checks.number) return i18n.t('auth.passwordRules.needNumber')
  if (!checks.notCommon) return i18n.t('auth.passwordRules.tooCommon')
  return null
}

/** 計算密碼強度等級 (0-5)，供強度指示器使用 */
export function getPasswordStrength(password: string): number {
  if (!password) return 0
  const checks = checkPasswordComplexity(password)
  return [checks.length, checks.uppercase, checks.lowercase, checks.number, checks.notCommon].filter(
    Boolean,
  ).length
}

/** 密碼強度等級標籤 */
export function getStrengthLabel(strength: number): string {
  if (strength <= 1) return i18n.t('auth.passwordStrength.veryWeak')
  if (strength <= 2) return i18n.t('auth.passwordStrength.weak')
  if (strength <= 3) return i18n.t('auth.passwordStrength.medium')
  if (strength <= 4) return i18n.t('auth.passwordStrength.strong')
  return i18n.t('auth.passwordStrength.veryStrong')
}

/** 密碼強度對應顏色 class（Tailwind） */
export function getStrengthColor(strength: number): string {
  if (strength <= 2) return 'bg-destructive'
  if (strength <= 3) return 'bg-status-warning-text'
  return 'bg-status-success-text'
}
