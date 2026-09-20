/**
 * 產品相關跨頁面共用常數
 */

/**
 * 儲存條件代碼 → i18n 鍵。
 *
 * 值是**翻譯鍵**而非顯示文字（模組頂層不可存翻譯後字串，語系切換後不會更新）；
 * 使用端在渲染時 `t(STORAGE_CONDITIONS[code])`。
 */
export const STORAGE_CONDITIONS: Record<string, string> = {
  'RT': 'erpMaster.storageConditions.RT',
  'RF': 'erpMaster.storageConditions.RF',
  'FZ': 'erpMaster.storageConditions.FZ',
  'DK': 'erpMaster.storageConditions.DK',
  'DY': 'erpMaster.storageConditions.DY',
}
