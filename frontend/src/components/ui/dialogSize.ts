/**
 * 全站對話框標準寬度（DESIGN.md §「Dialog 寬度標準」）。
 * 只用這 5 種，禁止各處硬編 `max-w-*` / `w-[…px]`。
 * 由 `DialogContent` / `AlertDialogContent` 的 `size` prop 取用。
 */
export type DialogSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl'

export const DIALOG_SIZE_CLASS: Record<DialogSize, string> = {
  sm: 'max-w-md', // 448px — 確認框 / 2FA / 刪除原因 / 單欄極簡
  md: 'max-w-lg', // 512px — 預設：標準 CRUD 表單
  lg: 'max-w-2xl', // 672px — 較大 / 雙欄表單
  xl: 'max-w-4xl', // 896px — 寬內容 / 多欄 / 含表格
  '2xl': 'max-w-6xl', // 1152px — 資料密集檢視
}
