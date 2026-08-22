/**
 * 計畫書「版本名冊（manifest）」— 補登依版本過濾欄位渲染的單一功能真相源。
 *
 * 設計依據：docs/design/protocol-import/version-manifest-backfill-plan.md（§9 各版名冊、§10.2）。
 *
 * 原則：
 * - 未列於 VERSION_GATED_FIELDS 的欄位 = 共同核心，所有版本都顯示（下游會計/豬計數依賴）。
 * - 系統機制欄位（分機、附件上傳、電子簽名）= 系統級、與版本無關，不在此 gate。
 * - F = 最新版；新案（source_form_version 為 null）視為最新版。
 * - 版本可持續擴充：未來 G/H/I/J/K 追加於 PROTOCOL_FORM_VERSIONS 並在各 gated 欄位補上即可，
 *   不得把 F 當終態寫死。
 */

export type ProtocolFormVersion = 'C' | 'D' | 'E' | 'F'

/** 版本序（舊 → 新）。C 與 D 欄位結構相同（皆印 AD-04-01-01C），共用同一名冊。 */
export const PROTOCOL_FORM_VERSIONS: readonly ProtocolFormVersion[] = ['C', 'D', 'E', 'F'] as const

/** 目前最新版（動態語意：新增版本時改此值）。 */
export const LATEST_PROTOCOL_FORM_VERSION: ProtocolFormVersion = 'F'

/** 「先選版本」下拉標籤（範本頁登記簿對齊；此處為顯示用途）。 */
export const PROTOCOL_FORM_VERSION_LABELS: Record<ProtocolFormVersion, string> = {
  C: 'C 版（AD-04-01-01C）',
  D: 'D 版（AD-04-01-01C，欄位同 C）',
  E: 'E 版（AD-04-01-01E）',
  F: 'F 版（AD-04-01-01F，現行）',
}

/**
 * 版本相依欄位鍵 → 該欄位存在於哪些版本。
 * 元件以 `fieldVisibleForVersion(key, version)` gate 顯示與必填。
 */
export const VERSION_GATED_FIELDS = {
  // ── C/D 專有（E/F 已移除的孤兒欄位）──
  'basic.testUnit': ['C', 'D'], // 試驗單位 Test Unit 多站塊
  'basic.sponsorAddress': ['C', 'D'], // 委託單位地址
  resultsAnalysis: ['C', 'D'], // GLP 結果分析（允收/統計/判定）
  documentArchiving: ['C', 'D'], // GLP 研究文件與歸檔
  'design.housingEnvironmentSop': ['C', 'D'], // 飼養環境 SOP 塊
  'items.testItemExtra': ['C', 'D'], // 試驗物質額外 6 欄（成份/來源/特徵/留樣/留樣地點/COA）

  // ── C/D/E（F 已移除）──
  'basic.testItemType': ['C', 'D', 'E'], // 試驗物質類型 9 項
  'basic.techCategories': ['C', 'D', 'E'], // 技術類別 9 項
  'basic.registrationAuthorities': ['C', 'D', 'E'], // 預定註冊權責機關 5 項（GLP）

  // ── E 起才有（C/D 無）──
  'basic.sdSplit': ['E', 'F'], // SD/PI 角色分離（+SD email）
  'basic.sponsorContactPerson': ['E', 'F'], // 委託單位聯絡人
  'purpose.replacementPlatforms': ['E', 'F'], // 替代方案搜尋平台勾選（E 3 / F 8）

  // ── F 專有（C/D/E 無）──
  'basic.piPosition': ['F'], // PI 職稱
  'basic.projectTypeOther': ['F'], // 計畫類型第 6 項「其他」
  'purpose.duplicateFourOptions': ['F'], // 是否重複 4 選項（C/D/E 為 2 選項）
  'purpose.reductionSpecialCare': ['F'], // 減量—特殊照護
  'purpose.reductionSingleHousing': ['F'], // 減量—單獨飼養（9 原因）
  'purpose.reductionAnimalReuse': ['F'], // 減量—動物再應用（5 計畫）
  'design.painDistressSigns': ['F'], // 痛苦症狀 17 項勾選
  'design.painReliefMeasures': ['F'], // 緩解措施 4 項勾選
  'design.painCategoryItems': ['F'], // 疼痛等級各級細項勾選（C/D/E 僅級別+描述）
  'guidelines.databases': ['F'], // 資料庫搜尋 A~L（12 項勾選）
} as const satisfies Record<string, readonly ProtocolFormVersion[]>

export type VersionGatedFieldKey = keyof typeof VERSION_GATED_FIELDS

/**
 * 某版本是否顯示指定的版本相依欄位。
 * @param key 版本相依欄位鍵
 * @param version 該計畫的 source_form_version；null/undefined（新案）視為最新版
 */
export function fieldVisibleForVersion(
  key: VersionGatedFieldKey,
  version: ProtocolFormVersion | null | undefined,
): boolean {
  const effective = normalizeFormVersion(version)
  return (VERSION_GATED_FIELDS[key] as readonly ProtocolFormVersion[]).includes(effective)
}

/** 將任意字串正規化為已知版本；未知/空 → 最新版。 */
export function normalizeFormVersion(
  version: string | null | undefined,
): ProtocolFormVersion {
  const upper = version?.trim().toUpperCase()
  return (PROTOCOL_FORM_VERSIONS as readonly string[]).includes(upper ?? '')
    ? (upper as ProtocolFormVersion)
    : LATEST_PROTOCOL_FORM_VERSION
}
