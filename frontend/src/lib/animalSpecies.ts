import { animalBreedNames, type AnimalBreed } from '@/types/animal'

interface SpeciesDisplayFields {
  species_name?: string | null
  breed: AnimalBreed
  breed_other?: string | null
}

/**
 * 動物「品種」欄的顯示文字。
 *
 * `species_id` 是動物種類的真相源，所以優先用後端 JOIN 出來的 `species_name`——
 * admin 自助新增的物種（例如「山羊」）只有這條路徑顯示得出來，走 breed enum 一律變成「其他」。
 * 尚未 backfill 或查詢沒 JOIN species 的舊資料才回退 breed enum；'other' 再退到自由文字。
 *
 * `breedLabel` 用來注入 i18n 翻譯函式；不給就用內建的繁中對照表。
 */
export function animalSpeciesLabel(
  animal: SpeciesDisplayFields,
  breedLabel: (breed: AnimalBreed) => string = (b) => animalBreedNames[b],
): string {
  // 自由文字優先於 species_name：選「其他」品種時表單會**強制**要求填寫品種名稱
  // （AnimalAddDialog 的 requiresBreedOther），那是使用者輸入的最具體資訊。
  // 若讓 species_name 先贏，畫面只會顯示物種主檔的「其他」，等於把使用者被迫
  // 填寫的內容整個蓋掉。
  //
  // 這個判斷不會誤傷 admin 自助新增的物種（例如山羊）：那些物種的 breed 雖然
  // 也被推導為 'other'，但表單只在選中 code='other' 的物種時才顯示自由文字欄，
  // 其餘情況 breed_other 為空 → 自然落到 species_name。
  if (animal.breed === 'other' && animal.breed_other) return animal.breed_other
  if (animal.species_name) return animal.species_name
  if (animal.breed === 'other') return breedLabel('other')
  return breedLabel(animal.breed)
}
