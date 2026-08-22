/**
 * 身分預選規則的回歸測試。
 *
 * 這條規則決定新建帳號時「本場受僱人員 / 外部人員」的預選值。選錯的後果是
 * **靜默**的：被誤標成外部的人無法加入任何部門，連帶讓他的假單找不到單位
 * 主管簽核；被誤標成內部的外聘人員則會出現在特休、加班、訓練等人事清單裡。
 *
 * 實際發生過：走邀請流程進來的人一律被硬編成外部，9 位在職同仁因此分類錯誤。
 */
import { describe, it, expect } from 'vitest'
import { deriveAffiliation } from '@/lib/staffAffiliation'

describe('deriveAffiliation', () => {
    it('沒選角色時不預選', () => {
        expect(deriveAffiliation([])).toBeNull()
    })

    it('明確內部角色 → 預選為受僱人員', () => {
        expect(deriveAffiliation(['EXPERIMENT_STAFF'])).toBe(true)
        expect(deriveAffiliation(['ADMIN_STAFF', 'PURCHASING'])).toBe(true)
        expect(deriveAffiliation(['DIRECTOR'])).toBe(true)
    })

    it('明確外部角色 → 預選為外部人員', () => {
        expect(deriveAffiliation(['PI'])).toBe(false)
        expect(deriveAffiliation(['CLIENT'])).toBe(false)
    })

    /**
     * 這幾個角色實測「同一角色兩種身分都有」——IACUC 委員與獸醫目前都是外聘，
     * 但未來不排除內部化。不預選才不會把錯的答案填給使用者。
     */
    it.each(['REVIEWER', 'VET', 'IACUC_CHAIR', 'QAU'])('曖昧角色 %s 不預選', code => {
        expect(deriveAffiliation([code])).toBeNull()
    })

    it('內外部混選時不預選——那種組合本來就需要人判斷', () => {
        expect(deriveAffiliation(['EXPERIMENT_STAFF', 'PI'])).toBeNull()
        expect(deriveAffiliation(['DIRECTOR', 'PI'])).toBeNull()
    })

    it('曖昧角色搭配明確角色時，以明確的那個為準', () => {
        expect(deriveAffiliation(['VET', 'EXPERIMENT_STAFF'])).toBe(true)
        expect(deriveAffiliation(['REVIEWER', 'PI'])).toBe(false)
    })

    it('未知角色碼不影響判定，也不會拋錯', () => {
        expect(deriveAffiliation(['SOMETHING_NEW'])).toBeNull()
        expect(deriveAffiliation(['SOMETHING_NEW', 'EXPERIMENT_STAFF'])).toBe(true)
    })
})
