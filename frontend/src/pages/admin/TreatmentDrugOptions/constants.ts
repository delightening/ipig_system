import type { TFunction } from 'i18next'

/**
 * 藥物分類「值」→ 顯示用 i18n 鍵。
 *
 * 分類值（如 '麻醉'）是存進資料庫、送後端、與後端比對的資料，維持中文原值不可翻譯；
 * 這裡只負責在渲染時把它換成當前語言的標籤。對照表以外的分類（例如日後新增）原樣顯示。
 */
const DRUG_CATEGORY_LABEL_KEYS: Record<string, string> = {
    麻醉: 'adminOps.treatmentDrugs.categories.anesthesia',
    止痛: 'adminOps.treatmentDrugs.categories.analgesia',
    抗生素: 'adminOps.treatmentDrugs.categories.antibiotic',
    鎮靜: 'adminOps.treatmentDrugs.categories.sedation',
    驅蟲: 'adminOps.treatmentDrugs.categories.dewormer',
    疫苗: 'adminOps.treatmentDrugs.categories.vaccine',
    其他: 'adminOps.treatmentDrugs.categories.other',
}

export function drugCategoryLabel(category: string, t: TFunction): string {
    const key = DRUG_CATEGORY_LABEL_KEYS[category]
    return key ? t(key) : category
}
