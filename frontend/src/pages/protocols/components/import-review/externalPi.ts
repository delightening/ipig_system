/** 外部 PI（無系統帳號）姓名 / 聯絡資訊。
 * 委託單位（sponsor）改由研究資料（ResearchBasicFields）統一捕捉，不再於此重複（R64-5c C2 / D5）。 */
export interface ExternalPiData {
  piName: string
  piEmail: string
  piPhone: string
}

export const EMPTY_EXTERNAL_PI: ExternalPiData = {
  piName: '',
  piEmail: '',
  piPhone: '',
}
