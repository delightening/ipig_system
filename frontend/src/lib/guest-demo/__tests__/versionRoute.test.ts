import { describe, it, expect } from 'vitest'

import { getGuestDemoData } from '../routes'
import { formatBuildTime, shortSha } from '@/pages/admin/components/systemVersion'

/**
 * 訪客模式的 /admin/internal/version 必須回「虛構」示範值：
 * 沒有這筆路由時，請求會真的打到後端、無 session 得 401，頁首變成「載入失敗」。
 */
describe('guest-demo /admin/internal/version', () => {
    it('GET 回傳虛構的版本資訊（不打後端）', () => {
        const data = getGuestDemoData('/admin/internal/version', 'GET') as {
            git_sha: string
            build_time: string
            migration_max_version: number
        }
        expect(data).toBeDefined()
        expect(shortSha(data.git_sha)).toBe('0123456789ab')
        expect(formatBuildTime(data.build_time, 'zh-TW')).not.toBeNull()
        expect(data.migration_max_version).toBeGreaterThan(0)
    })
})
