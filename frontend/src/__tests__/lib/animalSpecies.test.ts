import { animalSpeciesLabel } from '@/lib/animalSpecies'

/**
 * 品種顯示優先序。
 *
 * PR #1052 把顯示端改為優先讀後端 JOIN 出來的 `species_name`，但漏了一種情況：
 * 選「其他」品種時，新增動物表單會**強制**要求填寫品種名稱（如「藏香豬」），
 * 而該物種的 `species_name` 正是「其他」——`species_name` 先贏就把使用者
 * 被迫填寫的內容整個蓋掉，顯示成無資訊量的「其他」。
 */
describe('animalSpeciesLabel 顯示優先序', () => {
  it('「其他」品種：自由文字優先於 species_name（#1052 回歸）', () => {
    expect(
      animalSpeciesLabel({
        species_name: '其他',
        breed: 'other',
        breed_other: '藏香豬',
      }),
    ).toBe('藏香豬')
  })

  it('admin 自助新增的物種：顯示物種名，不受此規則影響', () => {
    // 山羊的 breed 同樣被後端推導為 'other'，但表單只在 code='other' 時才收
    // 自由文字，故 breed_other 為空 → 應落到 species_name
    expect(
      animalSpeciesLabel({ species_name: '山羊', breed: 'other', breed_other: '' }),
    ).toBe('山羊')
    expect(
      animalSpeciesLabel({ species_name: '山羊', breed: 'other', breed_other: null }),
    ).toBe('山羊')
  })

  it('一般品種：顯示 species_name', () => {
    expect(animalSpeciesLabel({ species_name: '迷你豬', breed: 'minipig' })).toBe('迷你豬')
  })

  it('沒有 species_name 時回退 breed 對照表', () => {
    expect(animalSpeciesLabel({ breed: 'white' })).toBe('白豬')
    expect(animalSpeciesLabel({ species_name: null, breed: 'minipig' })).toBe('迷你豬')
  })

  it('沒有 species_name 也沒有自由文字的「其他」→ 顯示「其他」', () => {
    expect(animalSpeciesLabel({ breed: 'other' })).toBe('其他')
    expect(animalSpeciesLabel({ breed: 'other', breed_other: '' })).toBe('其他')
  })

  it('可注入 i18n 翻譯函式', () => {
    const t = (b: string) => ({ minipig: 'Minipig', other: 'Other' }[b] ?? b)
    expect(animalSpeciesLabel({ breed: 'minipig' }, t as never)).toBe('Minipig')
    // 自由文字不經翻譯（它是使用者輸入的原文）
    expect(
      animalSpeciesLabel({ breed: 'other', breed_other: '藏香豬' }, t as never),
    ).toBe('藏香豬')
  })
})
