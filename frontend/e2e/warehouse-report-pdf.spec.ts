/**
 * R35-10: 倉庫報表 PDF 列印路徑 E2E。
 *
 * 涵蓋：列印按鈕點擊 → 後端回 PDF blob → 開新分頁。下載按鈕走相同 fetch
 * 但走 download attr，以同一個請求驗證 backend 路徑即可。
 *
 * 無倉庫種子資料時自動 skip（CI 種子不一定有 warehouse，避免硬性 fail）。
 */
import { test, expect } from './fixtures/admin-context'
import { ensureAdminOnPage } from './auth-helpers'

test.describe('R35-10 倉庫報表 PDF 列印', () => {
    test('列印按鈕應觸發 PDF 抓取並開新分頁', async ({ page, context }) => {
        // 取一個 warehouse id：先進列表頁建立 session，再從 API 撈
        await ensureAdminOnPage(page, '/warehouses')
        await expect(page).not.toHaveURL(/\/login/, { timeout: 15_000 })

        const warehousesResp = await page.request.get('/api/warehouses')
        if (!warehousesResp.ok()) {
            test.skip(true, `warehouses API ${warehousesResp.status()} — 無法取種子資料`)
        }
        const body = await warehousesResp.json()
        // API 形狀不固定（裸陣列 / { data } / { items }）— 任一陣列形狀都接受，
        // 否則（含 null / 非陣列）視為空清單以走 skip 路徑而非運行時 crash
        const items: Array<{ id: string }> =
            [body, body?.data, body?.items].find((v): v is Array<{ id: string }> => Array.isArray(v)) ?? []
        if (items.length === 0) {
            test.skip(true, '種子資料無 warehouse，略過 PDF 列印測試')
        }
        const warehouseId = items[0].id

        // 攔截 PDF 請求驗證 backend 真的回 PDF（限定本次 warehouseId，避免攔到其他不相關請求）
        const pdfResponsePromise = page.waitForResponse(
            (resp) =>
                resp.url().includes(`/warehouses/${warehouseId}/report/pdf`)
                && resp.request().method() === 'GET',
            { timeout: 30_000 },
        )

        await page.goto(`/inventory/warehouse-report/${warehouseId}`)
        await page.waitForLoadState('domcontentloaded')
        // 等報表資料 query 完成（顯示 "倉庫現況報表" 標題）
        await expect(page.getByRole('heading', { name: '倉庫現況報表' })).toBeVisible({
            timeout: 15_000,
        })

        const printButton = page.getByRole('button', { name: /列印/ })
        await expect(printButton).toBeEnabled({ timeout: 5_000 })

        // window.open(blob, _blank) 觸發 popup；同時驗 PDF API
        const [popup, pdfResponse] = await Promise.all([
            context.waitForEvent('page', { timeout: 30_000 }),
            pdfResponsePromise,
            printButton.click(),
        ])

        expect(pdfResponse.status()).toBe(200)
        expect(pdfResponse.headers()['content-type']).toMatch(/pdf/i)

        // 新分頁 URL 應為 blob:（前端用 createObjectURL）
        await popup.waitForLoadState('domcontentloaded').catch(() => {})
        expect(popup.url()).toMatch(/^blob:/)
        await popup.close()
    })
})
