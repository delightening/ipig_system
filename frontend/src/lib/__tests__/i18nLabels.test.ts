import { afterAll, beforeAll, describe, expect, it } from 'vitest'

import i18n from '@/lib/i18n'
import { createLabelMap, createLabelOptions } from '@/lib/i18nLabels'
import {
  AMENDMENT_CHANGE_ITEM_OPTIONS,
  amendmentStatusNames,
  amendmentTypeNames,
} from '@/types/amendment'
import {
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
  animalStatusNames,
  recordTypeNames,
  transferStatusNames,
  transferTypeNames,
} from '@/types/animal'
import { storageLocationTypeNames } from '@/types/erp'
import { LEAVE_STATUS_NAMES, LEAVE_TYPE_NAMES } from '@/types/hr'

// 用既有語言包鍵（animals.genderLabels）驗 helper：male／female 在 zh-TW＝公／母、en＝Male／Female。
const GENDER_PREFIX = 'animals.genderLabels'

let previousLanguage: string

beforeAll(() => {
  previousLanguage = i18n.language
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

describe('createLabelMap', () => {
  it('同一個對照表物件：切語言後下一次讀取就是新語言（不必重建）', async () => {
    const map = createLabelMap(GENDER_PREFIX, ['male', 'female'] as const)

    await i18n.changeLanguage('zh-TW')
    expect(map.male).toBe('公')
    expect(map.female).toBe('母')

    await i18n.changeLanguage('en')
    expect(map.male).toBe('Male')
    expect(map.female).toBe('Female')

    await i18n.changeLanguage('zh-TW')
    expect(map.male).toBe('公')
  })

  it('Object.keys / entries / values / in 行為與一般物件相同', async () => {
    const map = createLabelMap(GENDER_PREFIX, ['male', 'female'] as const)

    await i18n.changeLanguage('zh-TW')
    expect(Object.keys(map)).toEqual(['male', 'female'])
    expect(Object.entries(map)).toEqual([
      ['male', '公'],
      ['female', '母'],
    ])
    expect(Object.values(map)).toEqual(['公', '母'])
    expect('male' in map).toBe(true)

    await i18n.changeLanguage('en')
    expect(Object.values(map)).toEqual(['Male', 'Female'])
  })

  it('未知代碼讀到 undefined（與原本的字面物件一致）', () => {
    const map: Record<string, string> = createLabelMap(GENDER_PREFIX, ['male', 'female'])

    expect(map.unknown).toBeUndefined()
    expect('unknown' in map).toBe(false)
  })

  it('語言包缺鍵時回傳 fallbacks 指定值；沒指定則回傳代碼本身', async () => {
    await i18n.changeLanguage('zh-TW')
    const map = createLabelMap('typesLabelsTest.doesNotExist', ['a', 'b'] as const, { a: 'fallback A' })

    expect(map.a).toBe('fallback A')
    expect(map.b).toBe('b')
  })
})

describe('createLabelOptions', () => {
  it('value 固定、label 每次讀取才翻譯；可直接 .map', async () => {
    const options = createLabelOptions(GENDER_PREFIX, ['male', 'female'] as const)

    await i18n.changeLanguage('zh-TW')
    expect(options.map((o) => [o.value, o.label])).toEqual([
      ['male', '公'],
      ['female', '母'],
    ])

    await i18n.changeLanguage('en')
    expect(options.map((o) => [o.value, o.label])).toEqual([
      ['male', 'Male'],
      ['female', 'Female'],
    ])
  })
})

/**
 * 既有語言包鍵的重用對照：這些對照表改為 getter 後，zh-TW 的輸出必須與改前的字面物件
 * **一字不差**（期望值是改前 `types/*.ts` 的原文），en 則必須有實際的英文（不是中文、不是鍵）。
 */
describe('types 對照表（沿用既有語言包鍵）', () => {
  const animalStatus = {
    unassigned: '未分配',
    in_experiment: '實驗中',
    completed: '實驗完成',
    euthanized: '已安樂死',
    sudden_death: '猝死',
    transferred: '已轉讓',
  }

  const zhCases: Array<[string, Record<string, string>, Record<string, string>]> = [
    ['animalStatusNames', animalStatusNames, animalStatus],
    ['allAnimalStatusNames', allAnimalStatusNames, animalStatus],
    ['animalBreedNames', animalBreedNames, { minipig: '迷你豬', white: '白豬', lyd: 'LYD', other: '其他' }],
    ['animalGenderNames', animalGenderNames, { male: '公', female: '母' }],
    [
      'recordTypeNames',
      recordTypeNames,
      { abnormal: '異常紀錄', experiment: '試驗紀錄', observation: '觀察紀錄' },
    ],
    [
      'transferStatusNames',
      transferStatusNames,
      {
        pending: '待審',
        vet_evaluated: '獸醫已評估',
        plan_assigned: '已指定新計劃',
        pi_approved: 'PI 已同意',
        completed: '轉讓完成',
        rejected: '已拒絕',
      },
    ],
    [
      'transferTypeNames',
      transferTypeNames,
      { external: '轉給其他機構', internal: '仍在機構內' },
    ],
    [
      'amendmentStatusNames',
      amendmentStatusNames,
      {
        DRAFT: '草稿',
        SUBMITTED: '已提交',
        CLASSIFIED: '已分類',
        UNDER_REVIEW: '審查中',
        REVISION_REQUIRED: '需修訂',
        RESUBMITTED: '已重送',
        APPROVED: '已核准',
        REJECTED: '已否決',
        ADMIN_APPROVED: '行政核准',
        EFFECTIVE: '已生效',
      },
    ],
    ['amendmentTypeNames', amendmentTypeNames, { MAJOR: '重大變更', MINOR: '小變更', PENDING: '待分類' }],
    [
      'LEAVE_TYPE_NAMES',
      LEAVE_TYPE_NAMES,
      {
        ANNUAL: '特休假',
        PERSONAL: '事假',
        SICK: '病假',
        COMPENSATORY: '補休假',
        MARRIAGE: '婚假',
        BEREAVEMENT: '喪假',
        MATERNITY: '產假',
        PATERNITY: '陪產假',
        MENSTRUAL: '生理假',
        OFFICIAL: '公假',
      },
    ],
    [
      'LEAVE_STATUS_NAMES',
      LEAVE_STATUS_NAMES,
      {
        DRAFT: '草稿',
        PENDING_PROXY: '待代理確認',
        PENDING_L1: '待單位主管審核',
        PENDING_L2: '待二級審核',
        PENDING_HR: '待行政審核',
        PENDING_GM: '待總經理核准',
        PENDING_DIRECTOR: '待負責人簽核',
        APPROVED: '已核准',
        REJECTED: '已駁回',
        CANCELLED: '已取消',
        REVOKED: '已銷假',
      },
    ],
    [
      'storageLocationTypeNames',
      storageLocationTypeNames,
      { shelf: '貨架', rack: '儲物架', zone: '區域', bin: '儲物格', wall: '牆壁', door: '門', window: '窗戶' },
    ],
  ]

  it.each(zhCases)('%s：zh-TW 輸出與改前的字面物件完全一致', async (_name, map, expected) => {
    await i18n.changeLanguage('zh-TW')
    expect({ ...map }).toEqual(expected)
  })

  it.each(zhCases)('%s：en 輸出每個代碼都有英文（非中文、非鍵路徑）', async (_name, map, expected) => {
    await i18n.changeLanguage('en')
    const snapshot = { ...map }

    expect(Object.keys(snapshot).sort()).toEqual(Object.keys(expected).sort())
    for (const [code, label] of Object.entries(snapshot)) {
      expect(label, code).toMatch(/\S/)
      expect(label, code).not.toMatch(/[一-鿿]/)
      // 缺鍵時 i18next 會回傳鍵路徑本身（例 animals.statusLabels.x），那不算有英文
      expect(label, code).not.toMatch(/^[A-Za-z]+(\.[A-Za-z_]+)+$/)
    }
  })

  it('AMENDMENT_CHANGE_ITEM_OPTIONS：value 不變、label 依語言', async () => {
    await i18n.changeLanguage('zh-TW')
    expect(AMENDMENT_CHANGE_ITEM_OPTIONS.map((o) => [o.value, o.label])).toEqual([
      ['ANIMAL_COUNT', '動物數量'],
      ['PROCEDURE', '實驗程序'],
      ['PERSONNEL', '試驗工作人員'],
      ['DURATION', '執行期間'],
      ['FUNDING', '經費來源'],
      ['FACILITY', '設施/場地'],
      ['SPECIES', '動物種類/品系'],
      ['ANESTHESIA', '麻醉方式'],
      ['EUTHANASIA', '安樂死方法'],
      ['OTHER', '其他'],
    ])

    await i18n.changeLanguage('en')
    const english = AMENDMENT_CHANGE_ITEM_OPTIONS.map((o) => o.label)
    expect(english).toHaveLength(10)
    for (const label of english) {
      expect(label).not.toMatch(/[一-鿿]/)
    }
  })
})
